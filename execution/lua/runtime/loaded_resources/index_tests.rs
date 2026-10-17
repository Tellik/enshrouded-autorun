use super::*;
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct Backend {
    ids: RefCell<Vec<ResourceId>>,
    keys: RefCell<HashMap<ResourceId, [u8; 4]>>,
    lists: Cell<usize>,
    reads: Cell<usize>,
    scalar_reads: Cell<usize>,
}

impl Backend {
    fn add(&self, id: u32, key: u32) -> ResourceId {
        let id = ResourceId::new(Guid::new([1; 16]), 99, id);
        self.ids.borrow_mut().push(id);
        self.keys.borrow_mut().insert(id, key.to_le_bytes());
        id
    }

    fn prepare(
        &self,
        cache: &mut Indexes,
        session: u64,
        field: usize,
        owner: &str,
        now: Instant,
    ) -> Result<usize, String> {
        cache.register(
            session,
            owner,
            Selector {
                root: TypeIndex::new(field),
                path: "key".into(),
            },
            Projection {
                type_hash: 99,
                root_size: field * 4 + 4,
                offset: field * 4,
                kind: PrimitiveType::UInt32,
            },
            now,
        )
    }

    fn lookup(
        &self,
        cache: &mut Indexes,
        session: u64,
        field: usize,
        wanted: u32,
        now: Instant,
    ) -> Result<Lookup, String> {
        let position = self.prepare(cache, session, field, "mod", now)?;
        cache.lookup(
            position,
            wanted.to_le_bytes(),
            now,
            |id| {
                self.reads.set(self.reads.get() + 1);
                self.keys
                    .borrow()
                    .get(id)
                    .map(|key| key.to_vec())
                    .ok_or("synthetic resource is unreadable".into())
            },
            |bytes| {
                bytes
                    .try_into()
                    .map_err(|_| "invalid synthetic scalar width".into())
            },
        )
    }

    fn tick(&self, cache: &mut Indexes, session: u64, now: Instant) {
        cache.advance(
            session,
            now,
            |_| {
                self.lists.set(self.lists.get() + 1);
                Ok(self.ids.borrow().clone())
            },
            |ids, _| {
                self.scalar_reads.set(self.scalar_reads.get() + ids.len());
                ids.iter()
                    .map(|id| {
                        self.keys
                            .borrow()
                            .get(id)
                            .copied()
                            .ok_or_else(|| "synthetic resource is unreadable".into())
                    })
                    .collect()
            },
        );
    }

    fn drain(&self, cache: &mut Indexes, session: u64, now: Instant) {
        for _ in 0..256 {
            self.tick(cache, session, now);
            if cache
                .entries
                .iter()
                .all(|index| index.completed_at.is_some())
            {
                return;
            }
        }
        panic!("synthetic index jobs did not complete");
    }
}

#[test]
fn one_preparation_finishes_without_more_lua_lookups_or_an_active_action() {
    let backend = Backend::default();
    let expected = backend.add(0, 5000);
    for i in 1..4262 {
        backend.add(i, i);
    }
    let mut cache = Indexes::default();
    let now = Instant::now();
    let first = backend.lookup(&mut cache, 7, 0, 5000, now).unwrap();
    assert!(!first.progress.complete && first.resource.is_none());
    assert_eq!(
        backend.lists.get(),
        0,
        "Lua lookup performs no resource listing"
    );
    for _ in 0..5 {
        backend.lookup(&mut cache, 7, 0, 5000, now).unwrap();
    }
    assert_eq!(
        backend.scalar_reads.get(),
        0,
        "repeated Lua calls do not advance discovery"
    );
    backend.tick(&mut cache, 7, now);
    assert!(cache.entries[0].scanned <= MAX_ROUND_KEYS);
    backend.drain(&mut cache, 7, now);
    assert_eq!(backend.lists.get(), 1);
    assert_eq!(backend.scalar_reads.get(), 4262);
    assert_eq!(
        backend.reads.get(),
        0,
        "discovery reads scalar batches, not full payloads"
    );
    let result = backend.lookup(&mut cache, 7, 0, 5000, now).unwrap();
    assert_eq!(result.resource.unwrap().0, expected);
    assert_eq!(result.progress.payload_reads, 1);
    assert_eq!(
        backend.reads.get(),
        1,
        "only the final candidate needs a complete snapshot"
    );
}

#[test]
fn no_partial_unique_result_escapes_and_late_duplicate_remains_ambiguous() {
    let backend = Backend::default();
    for i in 0..(MAX_ROUND_KEYS as u32 + 2) {
        backend.add(i, if i == MAX_ROUND_KEYS as u32 + 1 { 0 } else { i });
    }
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.lookup(&mut cache, 7, 0, 0, now).unwrap();
    backend.tick(&mut cache, 7, now);
    let partial = backend.lookup(&mut cache, 7, 0, 0, now).unwrap();
    assert!(!partial.progress.complete && partial.resource.is_none());
    backend.drain(&mut cache, 7, now);
    let result = backend.lookup(&mut cache, 7, 0, 0, now).unwrap();
    assert!(result.progress.complete && result.resource.is_none());
    assert!(result.reason.unwrap().contains("ambiguous"));
    assert_eq!(backend.reads.get(), 0);
}

#[test]
fn jobs_are_shared_and_declarations_survive_world_and_menu_changes() {
    let backend = Backend::default();
    backend.add(0, 42);
    backend.add(1, 43);
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 0, 0, "first", now).unwrap();
    backend.prepare(&mut cache, 0, 0, "second", now).unwrap();
    assert_eq!(cache.entries.len(), 1);
    assert_eq!(cache.entries[0].owners.len(), 2);
    backend.drain(&mut cache, 9, now);
    assert_eq!(cache.session, Some(9));
    assert_eq!(cache.entries[0].owners.len(), 2);
    for key in [42, 43] {
        assert!(
            backend
                .lookup(&mut cache, 9, 0, key, now)
                .unwrap()
                .resource
                .is_some()
        );
    }
    assert_eq!(backend.lists.get(), 1);
    backend.drain(&mut cache, 0, now);
    assert_eq!(backend.lists.get(), 2);
    assert_eq!(cache.entries.len(), 1);
}

#[test]
fn negative_cache_requeues_and_discovers_later_loaded_resources() {
    let backend = Backend::default();
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.lookup(&mut cache, 1, 0, 42, now).unwrap();
    backend.drain(&mut cache, 1, now);
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .unwrap()
            .reason
            .unwrap()
            .contains("not found")
    );
    let expected = backend.add(0, 42);
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now + Duration::from_secs(1))
            .unwrap()
            .resource
            .is_none()
    );
    let pending = backend
        .lookup(&mut cache, 1, 0, 42, now + NEGATIVE_TTL)
        .unwrap();
    assert!(!pending.progress.complete);
    assert_eq!(backend.lists.get(), 1);
    backend.drain(&mut cache, 1, now + NEGATIVE_TTL);
    assert_eq!(
        backend
            .lookup(&mut cache, 1, 0, 42, now + NEGATIVE_TTL)
            .unwrap()
            .resource
            .unwrap()
            .0,
        expected
    );
    assert_eq!(backend.lists.get(), 2);
}

#[test]
fn selective_mutations_preserve_unrelated_indexes_and_requeue_overlapping_keys() {
    let backend = Backend::default();
    let id = backend.add(0, 42);
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 1, 0, "mod", now).unwrap();
    backend.prepare(&mut cache, 1, 1, "mod", now).unwrap();
    backend.drain(&mut cache, 1, now);
    cache.invalidate_resource(&id, 8, 4);
    assert!(
        cache
            .entries
            .iter()
            .all(|index| index.completed_at.is_some())
    );
    let other_type = ResourceId::new(Guid::new([1; 16]), 100, 0);
    cache.invalidate_resource(&other_type, 0, 4);
    assert!(
        cache
            .entries
            .iter()
            .all(|index| index.completed_at.is_some())
    );
    cache.invalidate_resource(&id, 0, 4);
    assert!(
        cache
            .entries
            .iter()
            .find(|index| index.selector.root == TypeIndex::new(0))
            .unwrap()
            .completed_at
            .is_none()
    );
    assert!(
        cache
            .entries
            .iter()
            .find(|index| index.selector.root == TypeIndex::new(1))
            .unwrap()
            .completed_at
            .is_some()
    );
    backend.drain(&mut cache, 1, Instant::now());
    assert_eq!(backend.lists.get(), 3);
    cache.invalidate();
    assert_eq!(
        cache.entries.len(),
        2,
        "invalidation retains preparation declarations"
    );
    assert!(
        cache
            .entries
            .iter()
            .all(|index| index.completed_at.is_none())
    );
}

#[test]
fn unreadable_late_batch_discards_every_partial_value_and_retries_with_backoff() {
    let backend = Backend::default();
    backend.add(0, 42);
    for i in 1..70 {
        backend.add(i, 100 + i);
    }
    let late = backend.add(70, 999);
    backend.keys.borrow_mut().remove(&late);
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 1, 0, "mod", now).unwrap();
    for _ in 0..10 {
        backend.tick(&mut cache, 1, now);
    }
    let index = &cache.entries[0];
    assert!(index.failure.as_ref().unwrap().contains("unreadable"));
    assert_eq!(index.scanned, 0);
    assert!(index.values.is_empty());
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .unwrap()
            .resource
            .is_none()
    );
    assert_eq!(
        backend.lists.get(),
        1,
        "failure does not spin or relist before retry time"
    );
    backend.keys.borrow_mut().insert(late, 999u32.to_le_bytes());
    backend.drain(&mut cache, 1, now + RETRY_DELAY);
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now + RETRY_DELAY)
            .unwrap()
            .resource
            .is_some()
    );
}

#[test]
fn duplicate_ids_and_incomplete_native_batches_cannot_finish_an_index() {
    let backend = Backend::default();
    let id = backend.add(0, 42);
    backend.ids.borrow_mut().push(id);
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 1, 0, "mod", now).unwrap();
    backend.tick(&mut cache, 1, now);
    assert!(
        cache.entries[0]
            .failure
            .as_ref()
            .unwrap()
            .contains("duplicate ResourceIds")
    );
    backend.ids.borrow_mut().pop();
    cache.advance(
        1,
        now + RETRY_DELAY,
        |_| Ok(vec![id]),
        |_, _| Ok(Vec::new()),
    );
    assert!(!cache.entries[0].completed_at.is_some());
    assert!(
        cache.entries[0]
            .failure
            .as_ref()
            .unwrap()
            .contains("incomplete")
    );
    assert!(cache.entries[0].values.is_empty());
}

#[test]
fn active_capacity_is_not_evicted_and_owner_cleanup_preserves_other_interests() {
    let backend = Backend::default();
    let mut cache = Indexes::default();
    let now = Instant::now();
    for field in 0..MAX_INDEXES {
        backend.prepare(&mut cache, 1, field, "first", now).unwrap();
    }
    backend.prepare(&mut cache, 1, 0, "second", now).unwrap();
    assert!(
        backend
            .prepare(&mut cache, 1, MAX_INDEXES, "third", now)
            .unwrap_err()
            .contains("capacity")
    );
    assert_eq!(cache.entries.len(), MAX_INDEXES);
    cache.remove_owner("first");
    assert_eq!(cache.entries.len(), 1);
    assert!(cache.entries[0].owners.contains("second"));
    backend
        .prepare(&mut cache, 1, MAX_INDEXES, "third", now)
        .unwrap();
    assert_eq!(cache.entries.len(), 2);
}

#[test]
fn memory_limit_and_nonfinite_scalar_batches_never_publish_partial_indexes() {
    let backend = Backend::default();
    let id = backend.add(0, f32::NAN.to_bits());
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 1, 0, "mod", now).unwrap();
    cache.entries[0].projection.kind = PrimitiveType::Float32;
    backend.tick(&mut cache, 1, now);
    assert!(cache.entries[0].values.is_empty());
    assert!(cache.entries[0].completed_at.is_none());
    assert!(
        cache.entries[0]
            .failure
            .as_ref()
            .unwrap()
            .contains("finite")
    );
    cache.advance(
        1,
        now + RETRY_DELAY,
        |_| Ok(vec![id; MAX_INDEX_BYTES / RESOURCE_INDEX_BYTES + 1]),
        |_, _| panic!("an oversized listing must not reach native scalar reads"),
    );
    assert!(cache.entries[0].resources.is_empty());
    assert!(
        cache.entries[0]
            .failure
            .as_ref()
            .unwrap()
            .contains("memory safety limit")
    );
}

#[test]
fn failed_candidate_confirmation_requeues_without_publishing_a_stale_resource() {
    let backend = Backend::default();
    let id = backend.add(0, 42);
    let mut cache = Indexes::default();
    let now = Instant::now();
    backend.prepare(&mut cache, 1, 0, "mod", now).unwrap();
    backend.drain(&mut cache, 1, now);
    backend.keys.borrow_mut().insert(id, 43u32.to_le_bytes());
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .unwrap_err()
            .contains("candidate changed")
    );
    assert!(cache.entries[0].values.is_empty());
    assert!(!cache.entries[0].listed);
    backend.drain(&mut cache, 1, now + RETRY_DELAY);
    assert_eq!(
        backend
            .lookup(&mut cache, 1, 0, 43, now + RETRY_DELAY)
            .unwrap()
            .resource
            .unwrap()
            .0,
        id
    );
}

#[test]
fn indexed_hash_identifiers_select_the_reflected_leaf_through_typedefs() {
    let lua = mlua::Lua::new();
    let mut data = serde_json::to_value(crate::env::registry::tests::fixture()).unwrap();
    let types = data["types"].as_array_mut().unwrap();
    for (index, name, kind) in [
        (6, "HashKey", "STRUCT"),
        (7, "ItemId", "TYPEDEF"),
        (8, "Item", "STRUCT"),
    ] {
        types.push(serde_json::json!({"index":index,"name":name,"qualifiedName":format!("test::{name}"),"impactName":name,
            "size":4,"alignment":4,"elementAlignment":4,"fieldCount":0,"primitiveType":kind,"flags":"",
            "nameHash":index,"impactHash":index,"qualifiedHash":index,"internalHash":index}));
    }
    types[6]["structFields"] =
        serde_json::json!({"value":{"name":"value","type":0,"dataOffset":0}});
    types[7]["innerType"] = serde_json::json!(6);
    types[8]["structFields"] =
        serde_json::json!({"itemId":{"name":"itemId","type":7,"dataOffset":0}});
    let registry = Rc::new(serde_json::from_value::<TypeRegistry>(data).unwrap());
    // Preserve the strong type. Index its explicit leaf rather than assuming
    // every four-byte wrapper is itself an integer with identical semantics.
    assert!(ScalarField::resolve(&registry, TypeIndex::new(8), "itemId").is_err());
    let leaf = ScalarField::resolve(&registry, TypeIndex::new(8), "itemId.value").unwrap();
    assert_eq!(
        leaf.wanted(&lua, &registry, &LuaValue::Integer(3357517884))
            .unwrap(),
        3357517884u32.to_le_bytes()
    );
    assert_eq!(
        leaf.key(&3357517884u32.to_le_bytes()).unwrap(),
        3357517884u32.to_le_bytes()
    );
}

#[test]
fn fixed_scalar_codec_rejects_dynamic_paths_structs_unaligned_and_nonfinite_values() {
    let lua = mlua::Lua::new();
    let mut data = serde_json::to_value(crate::env::registry::tests::fixture()).unwrap();
    let types = data["types"].as_array_mut().unwrap();
    for (index, name, kind, size) in [
        (6, "Float", "FLOAT32", 4),
        (7, "Mode", "ENUM", 4),
        (8, "Record", "STRUCT", 16),
    ] {
        types.push(serde_json::json!({"index":index,"name":name,"qualifiedName":format!("test::{name}"),"impactName":name,
            "size":size,"alignment":4,"elementAlignment":4,"fieldCount":0,"primitiveType":kind,"flags":"",
            "nameHash":index,"impactHash":index,"qualifiedHash":index,"internalHash":index}));
    }
    types[7]["innerType"] = serde_json::json!(0);
    types[7]["enumFields"] = serde_json::json!({"First":{"name":"First","value":1},"Second":{"name":"Second","value":2}});
    types[8]["structFields"] = serde_json::json!({
        "number":{"name":"number","type":6,"dataOffset":0},
        "mode":{"name":"mode","type":7,"dataOffset":4},
        "nested":{"name":"nested","type":1,"dataOffset":8},
        "unaligned":{"name":"unaligned","type":0,"dataOffset":11}});
    let registry = Rc::new(serde_json::from_value::<TypeRegistry>(data).unwrap());
    let field = ScalarField::resolve(&registry, TypeIndex::new(2), "x").unwrap();
    assert_eq!(
        field
            .wanted(&lua, &registry, &LuaValue::Integer(42))
            .unwrap(),
        42u32.to_le_bytes()
    );
    assert!(
        field
            .wanted(&lua, &registry, &LuaValue::Integer(-1))
            .is_err()
    );
    assert!(ScalarField::resolve(&registry, TypeIndex::new(5), "items[1]").is_err());
    assert!(ScalarField::resolve(&registry, TypeIndex::new(8), "nested").is_err());
    assert!(ScalarField::resolve(&registry, TypeIndex::new(8), "unaligned").is_err());
    let float = ScalarField::resolve(&registry, TypeIndex::new(8), "number").unwrap();
    assert!(
        float
            .wanted(&lua, &registry, &LuaValue::Number(f64::INFINITY))
            .is_err()
    );
    assert!(float.key(&f32::NAN.to_le_bytes()).is_err());
    assert_eq!(
        float
            .wanted(&lua, &registry, &LuaValue::Number(-0.0))
            .unwrap(),
        (-0.0f32).to_le_bytes()
    );
    let mode = ScalarField::resolve(&registry, TypeIndex::new(8), "mode").unwrap();
    assert_eq!(
        mode.wanted(
            &lua,
            &registry,
            &LuaValue::String(lua.create_string("Second").unwrap())
        )
        .unwrap(),
        2u32.to_le_bytes()
    );
    assert!(
        mode.wanted(
            &lua,
            &registry,
            &LuaValue::String(lua.create_string("Invented").unwrap())
        )
        .is_err()
    );
}
