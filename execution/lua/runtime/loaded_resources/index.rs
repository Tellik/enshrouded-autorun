//! Shared, bounded resource-index jobs over ResourceIds, never process pointers.
use super::Resource;
use crate::{
    type_aliases::TypeHandle,
    env::{AppState, loader::runtime_provider, runtime_values, type_lookup::get_type},
    lua::{FunctionArgs, LuaValue},
};
use kfc::kfc_format;
use kfc::{
    guid::{Guid, ResourceId},
    reflection::{PrimitiveType, TypeIndex, TypeRegistry},
};
use mlua::Table;
use mod_loader::Mod;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
    rc::Rc,
    time::{Duration, Instant},
};

const MAX_INDEXES: usize = 16;
const MAX_RESOURCES: usize = 1 << 20;
const KEY_BATCH: usize = 64;
const MAX_ROUND_KEYS: usize = 1024;
const MAX_INDEX_BYTES: usize = 16 * 1024 * 1024;
// Conservative admission estimate for the ID list, hash table and duplicate keys.
const RESOURCE_INDEX_BYTES: usize = 128;
const MAX_OWNERS_PER_INDEX: usize = 256;
const NEGATIVE_TTL: Duration = Duration::from_secs(2);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const ROUND_BUDGET: Duration = Duration::from_millis(1);

#[derive(Clone, PartialEq, Eq)]
struct Selector {
    root: TypeIndex,
    path: String,
}

#[derive(Clone, Copy)]
struct Projection {
    type_hash: u32,
    root_size: usize,
    offset: usize,
    kind: PrimitiveType,
}

struct Index {
    selector: Selector,
    projection: Projection,
    owners: HashSet<String>,
    resources: Vec<ResourceId>,
    listed: bool,
    scanned: usize,
    // None denotes two or more different resources with the same scalar bits.
    values: HashMap<[u8; 4], Option<ResourceId>>,
    completed_at: Option<Instant>,
    retry_at: Instant,
    failure: Option<String>,
}

impl Index {
    fn reset(&mut self, now: Instant, reason: Option<String>) {
        self.resources = Vec::new();
        self.values = HashMap::new();
        self.listed = false;
        self.scanned = 0;
        self.completed_at = None;
        self.retry_at = if reason.is_some() {
            now + RETRY_DELAY
        } else {
            now
        };
        self.failure = reason;
    }

    fn estimated_bytes(&self) -> usize {
        self.resources.len().saturating_mul(RESOURCE_INDEX_BYTES)
    }
}

#[derive(Default)]
struct Metrics {
    rounds: u64,
    batches: u64,
    scalar_reads: u64,
    payload_reads: u64,
    failures: u64,
    resets: u64,
    budget_overruns: u64,
    last_round_us: u64,
    maximum_round_us: u64,
}

#[derive(Default)]
pub(crate) struct Indexes {
    session: Option<u64>,
    entries: VecDeque<Index>,
    metrics: Metrics,
}

#[derive(Debug)]
struct Progress {
    session_id: u64,
    scanned: usize,
    total: usize,
    payload_reads: usize,
    complete: bool,
    phase: &'static str,
}

#[derive(Debug)]
struct Lookup {
    resource: Option<(ResourceId, Vec<u8>)>,
    reason: Option<String>,
    progress: Progress,
}

impl Indexes {
    /// Keep declarations so a confirmed world change or mutation restarts work
    /// without requiring the mod to perform its gameplay action again.
    pub(crate) fn invalidate(&mut self) {
        let now = Instant::now();
        for index in &mut self.entries {
            index.reset(now, None);
            self.metrics.resets += 1;
        }
    }

    pub(crate) fn invalidate_resource(
        &mut self,
        resource: &ResourceId,
        offset: usize,
        width: usize,
    ) {
        let Some(end) = offset.checked_add(width) else {
            self.invalidate();
            return;
        };
        let now = Instant::now();
        for index in &mut self.entries {
            let key = index.projection.offset;
            if index.projection.type_hash == resource.type_hash() && offset < key + 4 && key < end {
                index.reset(now, None);
                self.metrics.resets += 1;
            }
        }
    }

    pub(crate) fn remove_owner(&mut self, owner: &str) {
        self.entries.retain_mut(|index| {
            index.owners.remove(owner);
            !index.owners.is_empty()
        });
    }

    fn observe_session(&mut self, session: u64, now: Instant) {
        if self.session != Some(session) {
            for index in &mut self.entries {
                index.reset(now, None);
                self.metrics.resets += 1;
            }
            self.session = Some(session);
        }
    }

    fn register(
        &mut self,
        session: u64,
        owner: &str,
        selector: Selector,
        projection: Projection,
        now: Instant,
    ) -> Result<usize, String> {
        self.observe_session(session, now);
        if let Some(position) = self
            .entries
            .iter()
            .position(|index| index.selector == selector)
        {
            let index = &mut self.entries[position];
            if !index.owners.contains(owner) && index.owners.len() >= MAX_OWNERS_PER_INDEX {
                return Err("loaded resource index owner limit reached".into());
            }
            index.owners.insert(owner.to_owned());
            return Ok(position);
        }
        if self.entries.len() >= MAX_INDEXES {
            return Err(
                "loaded resource index capacity reached. Active jobs were preserved".into(),
            );
        }
        let position = self.entries.len();
        self.entries.push_back(Index {
            selector,
            projection,
            owners: HashSet::from([owner.to_owned()]),
            resources: Vec::new(),
            listed: false,
            scanned: 0,
            values: HashMap::new(),
            completed_at: None,
            retry_at: now,
            failure: None,
        });
        Ok(position)
    }

    fn progress(&self, position: usize) -> Progress {
        let index = &self.entries[position];
        Progress {
            session_id: self.session.unwrap_or_default(),
            scanned: index.scanned,
            total: index.resources.len(),
            payload_reads: 0,
            complete: index.completed_at.is_some(),
            phase: if index.completed_at.is_some() {
                "complete"
            } else {
                "indexing"
            },
        }
    }

    /// Queries register and inspect work. Only the runner advances discovery.
    fn lookup(
        &mut self,
        position: usize,
        wanted: [u8; 4],
        now: Instant,
        mut read: impl FnMut(&ResourceId) -> Result<Vec<u8>, String>,
        mut key: impl FnMut(&[u8]) -> Result<[u8; 4], String>,
    ) -> Result<Lookup, String> {
        let index = &mut self.entries[position];
        if index.completed_at.is_some_and(|completed| {
            index.values.get(&wanted).copied().flatten().is_none()
                && now.saturating_duration_since(completed) >= NEGATIVE_TTL
        }) {
            index.reset(now, None);
            self.metrics.resets += 1;
        }
        let mut result = Lookup {
            resource: None,
            reason: None,
            progress: self.progress(position),
        };
        let index = &mut self.entries[position];
        if index.completed_at.is_none() {
            result.reason = Some(index.failure.clone().unwrap_or_else(|| {
                format!(
                    "loaded resource index is still scanning ({}/{})",
                    index.scanned,
                    index.resources.len()
                )
            }));
            return Ok(result);
        }
        match index.values.get(&wanted).copied() {
            None => result.reason = Some("loaded resource not found for indexed scalar".into()),
            Some(None) => {
                result.reason =
                    Some("loaded resource scalar is ambiguous in the completed index".into())
            }
            Some(Some(id)) => {
                self.metrics.payload_reads += 1;
                let confirmed = read(&id).and_then(|bytes| {
                    if key(&bytes)? != wanted {
                        return Err("loaded resource index candidate changed. Index discarded, retry lookup".into());
                    }
                    Ok(bytes)
                });
                match confirmed {
                    Ok(bytes) => {
                        result.progress.payload_reads = 1;
                        result.resource = Some((id, bytes));
                    }
                    Err(reason) => {
                        index.reset(now, Some(reason.clone()));
                        self.metrics.failures += 1;
                        self.metrics.resets += 1;
                        return Err(reason);
                    }
                }
            }
        }
        Ok(result)
    }

    fn advance(
        &mut self,
        session: u64,
        now: Instant,
        mut list: impl FnMut(u32) -> Result<Vec<ResourceId>, String>,
        mut batch: impl FnMut(&[ResourceId], Projection) -> Result<Vec<[u8; 4]>, String>,
    ) {
        self.observe_session(session, now);
        let started = Instant::now();
        let deadline = started + ROUND_BUDGET;
        let mut round_keys = 0;
        let mut idle = 0;
        while !self.entries.is_empty() && Instant::now() < deadline && round_keys < MAX_ROUND_KEYS {
            let mut index = self.entries.pop_front().unwrap();
            if index.completed_at.is_some() || now < index.retry_at {
                self.entries.push_back(index);
                idle += 1;
                if idle >= self.entries.len() {
                    break;
                }
                continue;
            }
            idle = 0;
            let others_bytes: usize = self.entries.iter().map(Index::estimated_bytes).sum();
            let work = (|| -> Result<(), String> {
                if !index.listed {
                    let resources = list(index.projection.type_hash)?;
                    if resources.len() > MAX_RESOURCES
                        || others_bytes
                            .saturating_add(resources.len().saturating_mul(RESOURCE_INDEX_BYTES))
                            > MAX_INDEX_BYTES
                    {
                        return Err(
                            "loaded resource index exceeds the shared memory safety limit".into(),
                        );
                    }
                    let unique: HashSet<_> = resources.iter().copied().collect();
                    if unique.len() != resources.len() {
                        return Err("loaded resource listing contains duplicate ResourceIds".into());
                    }
                    if resources
                        .iter()
                        .any(|id| id.type_hash() != index.projection.type_hash)
                    {
                        return Err(
                            "loaded resource listing contains an unexpected resource type".into(),
                        );
                    }
                    index.resources = resources;
                    index.listed = true;
                }
                // The listing itself consumes this round's cooperative budget.
                if Instant::now() >= deadline {
                    return Ok(());
                }
                let end = (index.scanned + KEY_BATCH).min(index.resources.len());
                if end > index.scanned {
                    let ids = &index.resources[index.scanned..end];
                    self.metrics.batches += 1;
                    let keys = batch(ids, index.projection)?;
                    if keys.len() != ids.len() {
                        return Err("native scalar batch returned an incomplete result".into());
                    }
                    for key in &keys {
                        runtime_values::validate_runtime_scalar(index.projection.kind, key)?;
                    }
                    for (id, key) in ids.iter().copied().zip(keys) {
                        match index.values.entry(key) {
                            Entry::Vacant(entry) => {
                                entry.insert(Some(id));
                            }
                            Entry::Occupied(mut entry) => {
                                entry.insert(None);
                            }
                        }
                    }
                    let count = end - index.scanned;
                    index.scanned = end;
                    round_keys += count;
                    self.metrics.scalar_reads += count as u64;
                }
                if index.scanned == index.resources.len() {
                    index.completed_at = Some(now);
                    index.failure = None;
                }
                Ok(())
            })();
            if let Err(reason) = work {
                index.reset(now, Some(reason));
                self.metrics.failures += 1;
                self.metrics.resets += 1;
            }
            self.entries.push_back(index);
        }
        let elapsed = started.elapsed();
        self.metrics.rounds += 1;
        self.metrics.last_round_us = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
        self.metrics.maximum_round_us = self
            .metrics
            .maximum_round_us
            .max(self.metrics.last_round_us);
        self.metrics.budget_overruns += u64::from(elapsed > ROUND_BUDGET);
    }

    pub(crate) fn diagnostics(&self) -> Value {
        let ready = self
            .entries
            .iter()
            .filter(|index| index.completed_at.is_some())
            .count();
        json!({"jobs":self.entries.len(),"queued":self.entries.len()-ready,"ready":ready,
            "estimatedBytes":self.entries.iter().map(Index::estimated_bytes).sum::<usize>(),
            "maximumBytes":MAX_INDEX_BYTES,"roundBudgetMicroseconds":ROUND_BUDGET.as_micros(),
            "rounds":self.metrics.rounds,"scalarBatches":self.metrics.batches,
            "scalarReads":self.metrics.scalar_reads,"candidatePayloadReads":self.metrics.payload_reads,
            "failures":self.metrics.failures,"resets":self.metrics.resets,
            "lastRoundMicroseconds":self.metrics.last_round_us,"maximumRoundMicroseconds":self.metrics.maximum_round_us,
            "budgetOverruns":self.metrics.budget_overruns})
    }
}

struct ScalarField {
    field: Rc<runtime_values::InlineField>,
    kind: PrimitiveType,
    root_size: usize,
}

impl ScalarField {
    #[cfg(test)]
    fn resolve(registry: &Rc<TypeRegistry>, root: TypeIndex, path: &str) -> Result<Self, String> {
        if path.is_empty() || path.len() > 1024 {
            return Err("indexed field path must contain 1 to 1024 bytes".into());
        }
        // inline_field rejects DS pointers, blob traversal and overlapping
        // descriptors. The source offset is fixed for every resource of this type.
        let field = Rc::new(runtime_values::inline_field(registry, root, path)?);
        Self::from_field(registry, root, field)
    }

    fn from_field(
        registry: &Rc<TypeRegistry>,
        root: TypeIndex,
        field: Rc<runtime_values::InlineField>,
    ) -> Result<Self, String> {
        let range = field.byte_range();
        if range.len() != 4 || range.start % 4 != 0 {
            return Err("loaded resource indexes require an aligned 32-bit scalar field".into());
        }
        let mut index = field.type_index();
        let mut seen = HashSet::new();
        let kind = loop {
            if !seen.insert(index) || seen.len() > 64 {
                return Err("cyclic indexed scalar typedef".into());
            }
            let ty = registry
                .get(index)
                .ok_or("unresolved indexed scalar type")?;
            if ty.size != 4 {
                return Err("indexed scalar typedef has an inconsistent storage size".into());
            }
            if ty.primitive_type != PrimitiveType::Typedef {
                break ty.primitive_type;
            }
            index = ty.inner_type.ok_or("indexed typedef has no value type")?;
        };
        if !matches!(
            kind,
            PrimitiveType::UInt32
                | PrimitiveType::SInt32
                | PrimitiveType::Float32
                | PrimitiveType::Enum
        ) {
            return Err(
                "loaded resource indexes support 32-bit integers, finite floats and enums".into(),
            );
        }
        Ok(Self {
            field,
            kind,
            root_size: registry.get(root).ok_or("unresolved resource type")?.size as usize,
        })
    }

    fn wanted(
        &self,
        lua: &mlua::Lua,
        registry: &Rc<TypeRegistry>,
        value: &LuaValue,
    ) -> Result<[u8; 4], String> {
        let ty = TypeHandle::new(registry.clone(), self.field.type_index());
        let checked = super::value::validate_and_clone_lua_value(value, &ty, lua)
            .map_err(|e| e.to_string())?;
        let encoded = super::value::convert_lua_to_value(&checked, &ty)
            .map_err(|e| e.to_string())?
            .to_bytes(registry, &ty)
            .map_err(|e| e.to_string())?;
        runtime_values::validate_runtime_scalar(self.kind, &encoded)?;
        encoded
            .try_into()
            .map_err(|_| "indexed scalar codec did not produce four bytes".into())
    }

    fn key(&self, bytes: &[u8]) -> Result<[u8; 4], String> {
        if bytes.len() < self.root_size {
            return Err("indexed resource snapshot is smaller than its reflected root".into());
        }
        let scalar = bytes
            .get(self.field.byte_range())
            .ok_or("indexed scalar is outside resource snapshot")?;
        runtime_values::validate_runtime_scalar(self.kind, scalar)?;
        scalar
            .try_into()
            .map_err(|_| "indexed scalar has an invalid width".into())
    }
}

fn require_session(expected: u64) -> Result<(), String> {
    // Session zero remains a supported read-only menu snapshot.
    if runtime_provider::world_session_id() != expected {
        return Err("world session changed during loaded resource indexing".into());
    }
    Ok(())
}

fn request(
    state: &AppState,
    args: &FunctionArgs,
) -> Result<(Selector, ScalarField, Projection), String> {
    if state.phase() != crate::RuntimePhase::Ingame {
        return Err("loaded ResourceService access is available only in the ingame phase".into());
    }
    let registry = state.type_registry();
    let metadata = get_type(args, 0, registry.as_ref()).map_err(|error| error.to_string())?;
    let path = args.get::<String>(1).map_err(|error| error.to_string())?;
    if path.is_empty() || path.len() > 1024 {
        return Err("indexed field path must contain 1 to 1024 bytes".into());
    }
    let scalar = ScalarField::from_field(
        registry,
        metadata.index,
        state.inline_field(metadata.index, &path)?,
    )?;
    let type_id = kfc_format::resource_id(registry, &metadata.qualified_name, Guid::NONE, 0)
        .ok_or("indexed resource type is absent from the active reflection")?;
    let projection = Projection {
        type_hash: type_id.type_hash(),
        root_size: scalar.root_size,
        offset: scalar.field.byte_range().start,
        kind: scalar.kind,
    };
    Ok((
        Selector {
            root: metadata.index,
            path,
        },
        scalar,
        projection,
    ))
}

fn progress_table(lua: &mlua::Lua, progress: Progress) -> mlua::Result<Table> {
    let result = lua.create_table()?;
    result.raw_set("session_id", progress.session_id)?;
    result.raw_set("scanned", progress.scanned)?;
    result.raw_set("total", progress.total)?;
    result.raw_set("payload_reads", progress.payload_reads)?;
    result.raw_set("complete", progress.complete)?;
    result.raw_set("phase", progress.phase)?;
    Ok(result)
}

/// Idempotent preparation declaration. It survives a later nonzero world
/// session and is removed with its owner, not after a particular gameplay action.
pub(crate) fn prepare(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(bool, Option<String>, Option<Table>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let prepare = || -> Result<(Progress, Option<String>), String> {
        if let Some(reason) =
            crate::env::loader::runtime_denial_reason(&state, owner, "runtime.ecs.read")
        {
            return Err(reason);
        }
        if args.len() != 2 {
            return Err(
                "prepare_loaded_index expects a type and fixed inline scalar field path".into(),
            );
        }
        let (selector, _, projection) = request(&state, &args)?;
        let mut indexes = state.loaded_resource_indexes();
        let position = indexes.register(
            runtime_provider::world_session_id(),
            &owner.info().id,
            selector,
            projection,
            Instant::now(),
        )?;
        let progress = indexes.progress(position);
        let reason = if progress.complete {
            None
        } else {
            Some(
                indexes.entries[position]
                    .failure
                    .clone()
                    .unwrap_or_else(|| "loaded resource index preparation is queued".into()),
            )
        };
        Ok((progress, reason))
    };
    Ok(match prepare() {
        Ok((progress, reason)) => (
            progress.complete,
            reason,
            Some(progress_table(lua, progress)?),
        ),
        Err(reason) => (false, Some(reason), None),
    })
}

pub(super) fn get(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<Resource>, Option<String>, Option<Table>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let lookup = || -> Result<Lookup, String> {
        if let Some(reason) =
            crate::env::loader::runtime_denial_reason(&state, owner, "runtime.ecs.read")
        {
            return Err(reason);
        }
        if args.len() != 3 {
            return Err("get_loaded_resource_by_field expects a type, fixed inline scalar field path and value".into());
        }
        let (selector, scalar, projection) = request(&state, &args)?;
        let wanted = scalar.wanted(
            lua,
            state.type_registry(),
            args.get::<LuaValue>(2).map_err(|error| error.to_string())?,
        )?;
        let session = runtime_provider::world_session_id();
        let mut indexes = state.loaded_resource_indexes();
        let position = indexes.register(
            session,
            &owner.info().id,
            selector,
            projection,
            Instant::now(),
        )?;
        let result = indexes.lookup(
            position,
            wanted,
            Instant::now(),
            |id| {
                require_session(session)?;
                let bytes = runtime_provider::resource_read(id)?;
                require_session(session)?;
                Ok(bytes)
            },
            |bytes| scalar.key(bytes),
        );
        if let Err(reason) = require_session(session) {
            indexes.observe_session(runtime_provider::world_session_id(), Instant::now());
            return Err(reason);
        }
        result
    };
    Ok(match lookup() {
        Ok(result) => {
            let resource = result.resource.map(|(id, bytes)| {
                Resource::new(state.get_loaded_resource_info_from_snapshot(
                    &id,
                    result.progress.session_id,
                    bytes,
                ))
            });
            (
                resource,
                result.reason,
                Some(progress_table(lua, result.progress)?),
            )
        }
        Err(reason) => (None, Some(reason), None),
    })
}

/// The single runtime owner advances every registered job. No Lua callback or
/// repeated mod lookup is needed to finish its initial index.
pub(crate) fn tick(state: &AppState) {
    if state.phase() != crate::RuntimePhase::Ingame {
        return;
    }
    let mut indexes = state.loaded_resource_indexes();
    if indexes.entries.is_empty() {
        return;
    }
    let session = runtime_provider::world_session_id();
    indexes.advance(
        session,
        Instant::now(),
        |type_hash| {
            require_session(session)?;
            let resources = runtime_provider::resources_by_type(type_hash)?;
            require_session(session)?;
            Ok(resources)
        },
        |ids, projection| {
            require_session(session)?;
            let keys = runtime_provider::resource_scalar_batch(
                ids,
                projection.type_hash,
                projection.root_size,
                projection.offset,
                session,
            )?;
            require_session(session)?;
            Ok(keys)
        },
    );
    if runtime_provider::world_session_id() != session {
        indexes.observe_session(runtime_provider::world_session_id(), Instant::now());
    }
}

#[cfg(test)]
#[path = "index_tests.rs"]
mod tests;
