//! Incremental scalar indexes over loaded ResourceIds, never process pointers.
use super::Resource;
use crate::{
    alias::TypeHandle,
    env::{AppState, loader::runtime_provider, runtime_values, util::get_type},
    lua::{FunctionArgs, LuaValue},
};
use kfc::{
    guid::{Guid, ResourceId},
    reflection::{PrimitiveType, TypeIndex, TypeRegistry},
};
use mlua::Table;
use shroudforge_parser::kfc_format;
use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
    rc::Rc,
    time::{Duration, Instant},
};

const MAX_INDEXES: usize = 16;
const MAX_RESOURCES: usize = 1 << 20;
const PAYLOAD_BUDGET: usize = 64;
const NEGATIVE_TTL: Duration = Duration::from_secs(2);

#[derive(Clone, PartialEq, Eq)]
struct Selector {
    root: TypeIndex,
    path: String,
}

struct Index {
    selector: Selector,
    resources: Vec<ResourceId>,
    scanned: usize,
    // None denotes two or more different resources with the same scalar bits.
    values: HashMap<[u8; 4], Option<ResourceId>>,
    completed_at: Option<Instant>,
}

#[derive(Default)]
pub(crate) struct Indexes {
    session: Option<u64>,
    entries: VecDeque<Index>,
}

struct Progress {
    session_id: u64,
    scanned: usize,
    total: usize,
    payload_reads: usize,
    complete: bool,
    phase: &'static str,
}

struct Lookup {
    resource: Option<(ResourceId, Vec<u8>)>,
    reason: Option<String>,
    progress: Progress,
}

impl Indexes {
    pub(crate) fn invalidate(&mut self) {
        self.entries.clear();
    }

    fn lookup(
        &mut self,
        session: u64,
        selector: Selector,
        wanted: [u8; 4],
        now: Instant,
        mut list: impl FnMut() -> Result<Vec<ResourceId>, String>,
        mut read: impl FnMut(&ResourceId) -> Result<Vec<u8>, String>,
        mut key: impl FnMut(&[u8]) -> Result<[u8; 4], String>,
    ) -> Result<Lookup, String> {
        if self.session != Some(session) {
            self.invalidate();
            self.session = Some(session);
        }
        let cached = self
            .entries
            .iter()
            .position(|entry| entry.selector == selector)
            .and_then(|position| self.entries.remove(position));
        // A negative snapshot is short-lived. This also retries an ambiguous
        // snapshot after a later resource unload or external scalar change.
        let mut index = match cached {
            Some(index)
                if !index.completed_at.is_some_and(|completed| {
                    index.values.get(&wanted).copied().flatten().is_none()
                        && now.saturating_duration_since(completed) >= NEGATIVE_TTL
                }) =>
            {
                index
            }
            _ => {
                let resources = list()?;
                if resources.len() > MAX_RESOURCES {
                    return Err("loaded resource index exceeds the resource safety limit".into());
                }
                let identities: HashSet<_> = resources.iter().copied().collect();
                if identities.len() != resources.len() {
                    return Err("loaded resource listing contains duplicate ResourceIds".into());
                }
                Index {
                    selector,
                    resources,
                    scanned: 0,
                    values: HashMap::new(),
                    completed_at: None,
                }
            }
        };
        // Removed indexes are published again only after all reads succeed.
        // An unreadable payload, session change or stale candidate discards
        // the entire partial index rather than asserting a unique match.
        let mut reads = 0;
        while index.scanned < index.resources.len() && reads < PAYLOAD_BUDGET {
            let id = index.resources[index.scanned];
            let bytes = read(&id)?;
            reads += 1;
            let scalar = key(&bytes)?;
            match index.values.entry(scalar) {
                Entry::Vacant(entry) => {
                    entry.insert(Some(id));
                }
                Entry::Occupied(mut entry) => {
                    entry.insert(None);
                }
            }
            index.scanned += 1;
        }
        let complete = index.scanned == index.resources.len();
        if complete && index.completed_at.is_none() {
            index.completed_at = Some(now);
        }
        let mut result = Lookup {
            resource: None,
            reason: None,
            progress: Progress {
                session_id: session,
                scanned: index.scanned,
                total: index.resources.len(),
                payload_reads: reads,
                complete,
                phase: if complete { "complete" } else { "indexing" },
            },
        };
        if !complete {
            result.reason = Some(format!(
                "loaded resource index is still scanning ({}/{})",
                index.scanned,
                index.resources.len()
            ));
        } else {
            match index.values.get(&wanted) {
                None => result.reason = Some("loaded resource not found for indexed scalar".into()),
                Some(None) => {
                    result.reason =
                        Some("loaded resource scalar is ambiguous in the completed index".into())
                }
                Some(Some(_)) if reads == PAYLOAD_BUDGET => {
                    result.progress.phase = "confirming";
                    result.reason = Some("loaded resource lookup is still scanning: fresh candidate confirmation pending".into());
                }
                Some(Some(id)) => {
                    let bytes = read(id)?;
                    if key(&bytes)? != wanted {
                        return Err("loaded resource index candidate changed. Index discarded, retry lookup".into());
                    }
                    result.progress.payload_reads += 1;
                    result.resource = Some((*id, bytes));
                }
            }
        }
        if self.entries.len() == MAX_INDEXES {
            self.entries.pop_front();
        }
        self.entries.push_back(index);
        Ok(result)
    }
}

struct ScalarField {
    field: runtime_values::InlineField,
    kind: PrimitiveType,
    root_size: usize,
}

impl ScalarField {
    fn resolve(registry: &Rc<TypeRegistry>, root: TypeIndex, path: &str) -> Result<Self, String> {
        if path.is_empty() || path.len() > 1024 {
            return Err("indexed field path must contain 1 to 1024 bytes".into());
        }
        // inline_field rejects DS pointers, blob traversal and overlapping
        // descriptors. The source offset is fixed for every resource of this type.
        let field = runtime_values::inline_field(registry, root, path)?;
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
    // Like get_loaded_resource, zero is a valid read-only menu snapshot.
    if runtime_provider::world_session_id() != expected {
        return Err("world session changed during loaded resource indexing".into());
    }
    Ok(())
}

pub(super) fn get(
    lua: &mlua::Lua,
    args: FunctionArgs,
) -> mlua::Result<(Option<Resource>, Option<String>, Option<Table>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let lookup = || -> Result<Lookup, String> {
        if state.phase() != crate::RuntimePhase::Ingame {
            return Err(
                "loaded ResourceService access is available only in the ingame phase".into(),
            );
        }
        if args.len() != 3 {
            return Err("get_loaded_resource_by_field expects a type, fixed inline scalar field path and value".into());
        }
        let registry = state.type_registry();
        let metadata = get_type(&args, 0, registry.as_ref()).map_err(|e| e.to_string())?;
        let path = args.get::<String>(1).map_err(|e| e.to_string())?;
        let scalar = ScalarField::resolve(registry, metadata.index, &path)?;
        let wanted = scalar.wanted(
            lua,
            registry,
            args.get::<LuaValue>(2).map_err(|e| e.to_string())?,
        )?;
        let type_id = kfc_format::resource_id(registry, &metadata.qualified_name, Guid::NONE, 0)
            .ok_or("indexed resource type is absent from the active reflection")?;
        let session = runtime_provider::world_session_id();
        let result = state.loaded_resource_indexes().lookup(
            session,
            Selector {
                root: metadata.index,
                path,
            },
            wanted,
            Instant::now(),
            || {
                require_session(session)?;
                let ids = runtime_provider::resources_by_type(type_id.type_hash())?;
                require_session(session)?;
                Ok(ids)
            },
            |id| {
                require_session(session)?;
                let bytes = runtime_provider::resource_read(id)?;
                require_session(session)?;
                Ok(bytes)
            },
            |bytes| scalar.key(bytes),
        );
        if let Err(reason) = require_session(session) {
            state.invalidate_loaded_resource_indexes();
            return Err(reason);
        }
        result
    };
    Ok(match lookup() {
        Ok(result) => {
            let progress = lua.create_table()?;
            progress.raw_set("session_id", result.progress.session_id)?;
            progress.raw_set("scanned", result.progress.scanned)?;
            progress.raw_set("total", result.progress.total)?;
            progress.raw_set("payload_reads", result.progress.payload_reads)?;
            progress.raw_set("complete", result.progress.complete)?;
            progress.raw_set("phase", result.progress.phase)?;
            let resource = result.resource.map(|(id, bytes)| {
                Resource::new(state.get_loaded_resource_info_from_snapshot(
                    &id,
                    result.progress.session_id,
                    bytes,
                ))
            });
            (resource, result.reason, Some(progress))
        }
        Err(reason) => (None, Some(reason), None),
    })
}

#[cfg(test)]
#[path = "loaded_index_tests.rs"]
mod tests;
