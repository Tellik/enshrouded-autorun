use crate::lua::values as runtime_values;
use kfc::{
    guid::ResourceId,
    reflection::{PrimitiveType, TypeIndex},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
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
pub(crate) struct Selector {
    pub(crate) root: TypeIndex,
    pub(crate) path: String,
}

#[derive(Clone, Copy)]
pub(crate) struct Projection {
    pub(crate) type_hash: u32,
    pub(crate) root_size: usize,
    pub(crate) offset: usize,
    pub(crate) kind: PrimitiveType,
}

pub(crate) struct Index {
    selector: Selector,
    projection: Projection,
    owners: HashSet<String>,
    resources: Vec<ResourceId>,
    listed: bool,
    pub(crate) scanned: usize,
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
    pub(crate) payload_reads: u64,
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
pub(crate) struct Progress {
    pub(crate) session_id: u64,
    pub(crate) scanned: usize,
    pub(crate) total: usize,
    pub(crate) payload_reads: usize,
    pub(crate) complete: bool,
    pub(crate) phase: &'static str,
}

#[derive(Debug)]
pub(crate) struct Lookup {
    pub(crate) resource: Option<(ResourceId, Vec<u8>)>,
    pub(crate) reason: Option<String>,
    pub(crate) progress: Progress,
}

impl Indexes {
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub(crate) fn pending_reason(&self, position: usize) -> Option<String> {
        self.entries[position].failure.clone()
    }

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

    pub(crate) fn observe_session(&mut self, session: u64, now: Instant) {
        if self.session != Some(session) {
            for index in &mut self.entries {
                index.reset(now, None);
                self.metrics.resets += 1;
            }
            self.session = Some(session);
        }
    }

    pub(crate) fn register(
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

    pub(crate) fn progress(&self, position: usize) -> Progress {
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
    pub(crate) fn lookup(
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

    pub(crate) fn advance(
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

#[cfg(test)]
#[path = "loaded_resource_indexes_tests.rs"]
mod tests;
