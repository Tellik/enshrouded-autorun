use super::*;
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct Backend {
    ids: RefCell<Vec<ResourceId>>,
    keys: RefCell<HashMap<ResourceId, [u8; 4]>>,
    lists: Cell<usize>,
    reads: Cell<usize>,
}

impl Backend {
    fn add(&self, id: u32, key: u32) -> ResourceId {
        let id = ResourceId::new(Guid::new([1; 16]), 99, id);
        self.ids.borrow_mut().push(id);
        self.keys.borrow_mut().insert(id, key.to_le_bytes());
        id
    }

    fn lookup(
        &self,
        cache: &mut Indexes,
        session: u64,
        field: usize,
        wanted: u32,
        now: Instant,
    ) -> Result<Lookup, String> {
        cache.lookup(
            session,
            Selector {
                root: TypeIndex::new(field),
                path: "key".into(),
            },
            wanted.to_le_bytes(),
            now,
            || {
                self.lists.set(self.lists.get() + 1);
                Ok(self.ids.borrow().clone())
            },
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
}

#[test]
fn budget_includes_candidate_confirmation_and_no_partial_match_escapes() {
    let backend = Backend::default();
    let first = backend.add(0, 1000);
    for i in 1..64 {
        backend.add(i, i);
    }
    let mut cache = Indexes::default();
    let now = Instant::now();
    let pending = backend.lookup(&mut cache, 7, 0, 1000, now).unwrap();
    assert!(pending.resource.is_none());
    assert!(pending.progress.complete);
    assert_eq!(pending.progress.phase, "confirming");
    assert_eq!(pending.progress.payload_reads, 64);
    assert_eq!(backend.reads.get(), 64);
    let confirmed = backend.lookup(&mut cache, 7, 0, 1000, now).unwrap();
    assert_eq!(confirmed.resource.unwrap().0, first);
    assert_eq!(confirmed.progress.payload_reads, 1);
    assert_eq!(backend.lists.get(), 1);
}

#[test]
fn ambiguity_is_detected_after_the_entire_index_has_been_read() {
    let backend = Backend::default();
    for i in 0..130 {
        backend.add(i, if i == 129 { 0 } else { i });
    }
    let mut cache = Indexes::default();
    let now = Instant::now();
    for scanned in [64, 128] {
        let partial = backend.lookup(&mut cache, 7, 0, 0, now).unwrap();
        assert_eq!(partial.progress.scanned, scanned);
        assert!(!partial.progress.complete);
        assert!(partial.resource.is_none());
        assert!(partial.reason.unwrap().contains("still scanning"));
    }
    let result = backend.lookup(&mut cache, 7, 0, 0, now).unwrap();
    assert!(result.progress.complete);
    assert_eq!(result.progress.payload_reads, 2);
    assert!(result.resource.is_none());
    assert!(result.reason.unwrap().contains("ambiguous"));
}

#[test]
fn indexes_are_shared_by_field_and_reset_between_world_and_menu_sessions() {
    let backend = Backend::default();
    backend.add(0, 42);
    backend.add(1, 43);
    let mut cache = Indexes::default();
    let now = Instant::now();
    for key in [42, 43] {
        assert!(
            backend
                .lookup(&mut cache, 0, 0, key, now)
                .unwrap()
                .resource
                .is_some()
        );
    }
    assert_eq!(backend.lists.get(), 1);
    assert!(
        backend
            .lookup(&mut cache, 9, 0, 42, now)
            .unwrap()
            .resource
            .is_some()
    );
    assert!(
        backend
            .lookup(&mut cache, 0, 0, 42, now)
            .unwrap()
            .resource
            .is_some()
    );
    assert_eq!(backend.lists.get(), 3);
    assert_eq!(cache.session, Some(0));
    assert_eq!(cache.entries.len(), 1);
}

#[test]
fn negative_cache_expires_and_discovers_later_loaded_resources() {
    let backend = Backend::default();
    let mut cache = Indexes::default();
    let now = Instant::now();
    let missing = backend.lookup(&mut cache, 1, 0, 42, now).unwrap();
    assert!(missing.progress.complete);
    assert_eq!(missing.progress.total, 0);
    assert!(missing.reason.unwrap().contains("not found"));
    let expected = backend.add(0, 42);
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now + Duration::from_secs(1))
            .unwrap()
            .resource
            .is_none()
    );
    assert_eq!(backend.lists.get(), 1);
    let fresh = backend
        .lookup(&mut cache, 1, 0, 42, now + NEGATIVE_TTL)
        .unwrap();
    assert_eq!(fresh.resource.unwrap().0, expected);
    assert_eq!(backend.lists.get(), 2);
}

#[test]
fn stale_candidates_and_explicit_write_invalidation_discard_the_index() {
    let backend = Backend::default();
    let id = backend.add(0, 42);
    let mut cache = Indexes::default();
    let now = Instant::now();
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .unwrap()
            .resource
            .is_some()
    );
    backend.keys.borrow_mut().insert(id, 43u32.to_le_bytes());
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .err()
            .unwrap()
            .contains("candidate changed")
    );
    assert!(cache.entries.is_empty());
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 43, now)
            .unwrap()
            .resource
            .is_some()
    );
    cache.invalidate();
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 43, now)
            .unwrap()
            .resource
            .is_some()
    );
    assert_eq!(backend.lists.get(), 3);
}

#[test]
fn unreadable_late_resources_and_duplicate_ids_cannot_establish_uniqueness() {
    let backend = Backend::default();
    backend.add(0, 42);
    let late = backend.add(1, 43);
    backend.keys.borrow_mut().remove(&late);
    let mut cache = Indexes::default();
    let now = Instant::now();
    assert!(backend.lookup(&mut cache, 1, 0, 42, now).is_err());
    assert!(cache.entries.is_empty());
    backend.ids.borrow_mut().push(late);
    assert!(
        backend
            .lookup(&mut cache, 1, 0, 42, now)
            .err()
            .unwrap()
            .contains("duplicate ResourceIds")
    );
}

#[test]
fn cache_is_limited_to_sixteen_least_recently_used_indexes() {
    let backend = Backend::default();
    backend.add(0, 42);
    let mut cache = Indexes::default();
    let now = Instant::now();
    for field in 0..MAX_INDEXES {
        backend.lookup(&mut cache, 1, field, 42, now).unwrap();
    }
    backend.lookup(&mut cache, 1, 0, 42, now).unwrap();
    backend.lookup(&mut cache, 1, MAX_INDEXES, 42, now).unwrap();
    assert_eq!(cache.entries.len(), MAX_INDEXES);
    assert!(
        cache
            .entries
            .iter()
            .any(|entry| entry.selector.root == TypeIndex::new(0))
    );
    assert!(
        !cache
            .entries
            .iter()
            .any(|entry| entry.selector.root == TypeIndex::new(1))
    );
    backend.lookup(&mut cache, 1, 1, 42, now).unwrap();
    assert_eq!(backend.lists.get(), MAX_INDEXES + 2);
}

#[test]
fn session_failure_during_fresh_confirmation_drops_the_completed_index() {
    let mut cache = Indexes::default();
    let id = ResourceId::new(Guid::new([1; 16]), 99, 0);
    let calls = Cell::new(0);
    let result = cache.lookup(
        1,
        Selector {
            root: TypeIndex::new(0),
            path: "key".into(),
        },
        [42, 0, 0, 0],
        Instant::now(),
        || Ok(vec![id]),
        |_| {
            calls.set(calls.get() + 1);
            if calls.get() == 2 {
                Err("world session changed".into())
            } else {
                Ok(vec![42, 0, 0, 0])
            }
        },
        |bytes| Ok(bytes.try_into().unwrap()),
    );
    assert!(result.err().unwrap().contains("session changed"));
    assert!(cache.entries.is_empty());
}

#[test]
fn indexed_hash_identifiers_select_the_reflected_leaf_through_typedefs() {
    let lua = mlua::Lua::new();
    let mut data = serde_json::to_value(crate::env::registry::tests::fixture()).unwrap();
    let types = data["types"].as_array_mut().unwrap();
    for (index, name, kind) in [(6, "HashKey", "STRUCT"), (7, "ItemId", "TYPEDEF"), (8, "Item", "STRUCT")] {
        types.push(serde_json::json!({"index":index,"name":name,"qualifiedName":format!("test::{name}"),"impactName":name,
            "size":4,"alignment":4,"elementAlignment":4,"fieldCount":0,"primitiveType":kind,"flags":"",
            "nameHash":index,"impactHash":index,"qualifiedHash":index,"internalHash":index}));
    }
    types[6]["structFields"] = serde_json::json!({"value":{"name":"value","type":0,"dataOffset":0}});
    types[7]["innerType"] = serde_json::json!(6);
    types[8]["structFields"] = serde_json::json!({"itemId":{"name":"itemId","type":7,"dataOffset":0}});
    let registry = Rc::new(serde_json::from_value::<TypeRegistry>(data).unwrap());
    // Preserve the strong type. Index its explicit leaf rather than assuming
    // every four-byte wrapper is itself an integer with identical semantics.
    assert!(ScalarField::resolve(&registry, TypeIndex::new(8), "itemId").is_err());
    let leaf = ScalarField::resolve(&registry, TypeIndex::new(8), "itemId.value").unwrap();
    assert_eq!(leaf.wanted(&lua, &registry, &LuaValue::Integer(3357517884)).unwrap(),
        3357517884u32.to_le_bytes());
    assert_eq!(leaf.key(&3357517884u32.to_le_bytes()).unwrap(), 3357517884u32.to_le_bytes());
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
