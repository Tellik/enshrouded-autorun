use super::*;
#[test]
fn original_engine_names_are_resolved_without_inventing_helper_names() {
    let base = Box::leak(Box::new(json!({"image":{"sha256":"fixture"},"entries":[
        {"rva":16,"engine_descriptors":[{"name":"network_player_attributes"}]},
        {"rva":32,"engine_descriptors":[{"name":"shared_name"}]},
        {"rva":48,"engine_descriptors":[{"name":"shared_name"}]}
    ]})));
    let mut catalog = Catalog {
        base: Some(base),
        ..Default::default()
    };
    for (i, rva) in [16, 32, 48].into_iter().enumerate() {
        catalog.indices.insert(rva, i);
    }
    let lua = Lua::new();
    assert_eq!(
        catalog
            .selector(LuaValue::String(
                lua.create_string("network_player_attributes").unwrap()
            ))
            .unwrap(),
        16
    );
    assert!(
        catalog
            .selector(LuaValue::String(lua.create_string("shared_name").unwrap()))
            .is_err()
    );
    let descriptor = catalog.describe(16, false);
    assert_eq!(descriptor["name"], "network_player_attributes");
    assert_eq!(descriptor["name_provisional"], false);
    assert_eq!(descriptor["native_callable"], false);
}

#[test]
#[ignore = "set SHROUDFORGE_TEST_FUNCTIONS to a production-reader verification JSON"]
fn production_callback_binding_corpus() {
    let path = std::env::var("SHROUDFORGE_TEST_FUNCTIONS").unwrap();
    let registry: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let base: Value =
        serde_json::from_slice(&std::fs::read(format!("{path}.functions.json")).unwrap()).unwrap();
    let base = Box::leak(Box::new(base));
    let mut catalog = Catalog {
        base: Some(base),
        ..Default::default()
    };
    for (i, entry) in base["entries"].as_array().unwrap().iter().enumerate() {
        catalog
            .indices
            .insert(entry["rva"].as_u64().unwrap() as u32, i);
    }
    catalog.set_registry(registry.clone());
    let mut adapters = Vec::new();
    let lua = Lua::new();
    for rva in catalog.owners.keys().copied().collect::<Vec<_>>() {
        let descriptor = catalog.describe(rva, false);
        let plan = catalog.plans.get(&rva).cloned().flatten();
        if let Some(plan) = plan {
            let bound = binding(&lua, &descriptor, Some(plan.clone())).unwrap();
            let source: Vec<u8> = (0..plan.source_minimum).map(|i| (i * 37) as u8).collect();
            let destination = vec![0xa5; plan.destination_minimum + 16];
            let expected = plan.apply(&source, &destination).unwrap();
            let (actual, reason, result): (mlua::String, Option<String>, Option<u32>) = bound
                .get::<mlua::Function>("call")
                .unwrap()
                .call((
                    lua.create_string(&source).unwrap(),
                    lua.create_string(&destination).unwrap(),
                ))
                .unwrap();
            assert!(reason.is_none());
            assert_eq!(actual.as_bytes().as_ref(), expected);
            assert_eq!(result, plan.native_constant_result);
            adapters.push(descriptor);
        }
    }
    assert!(
        !adapters.is_empty(),
        "no independently observed callback adapter accepted"
    );
    let artifact = json!({"schema_version":1,"candidate_count":catalog.keys().len(),"callback_count":catalog.owners.len(),"adapters":adapters});
    std::fs::write(
        format!("{path}.adapters.json"),
        serde_json::to_vec_pretty(&artifact).unwrap(),
    )
    .unwrap();
    println!(
        "{} candidates, {} distinct registered callbacks, {} proven owned-buffer adapters",
        catalog.keys().len(),
        catalog.owners.len(),
        adapters.len()
    );
    for adapter in &adapters {
        println!(
            "{}: {}",
            adapter["name"], adapter["owners"][0]["qualified_name"]
        );
    }
}
#[test]
fn includes_leaf_callbacks_merges_owners_and_returns_partial_bindings() {
    let base = Box::leak(Box::new(
        json!({"image":{"sha256":"fixture"},"entries":[{"rva":16,"id":"native:16"}]}),
    ));
    let mut catalog = Catalog {
        base: Some(base),
        ..Default::default()
    };
    catalog.indices.insert(16, 0);
    catalog.set_registry(
        json!({"entries":[{"qualified_name":"keen::ecs::Example", "callbacks":[
            {"function_rva":16,"slot_offset":0}, {"function_rva":32,"slot_offset":8}]}]}),
    );
    assert_eq!(catalog.keys(), vec![16, 32]);
    let lua = Lua::new();
    assert_eq!(
        catalog
            .selector(LuaValue::String(
                lua.create_string("fixture/unclear_00000020").unwrap()
            ))
            .unwrap(),
        32
    );
    assert!(
        catalog
            .selector(LuaValue::String(
                lua.create_string("other/unclear_00000020").unwrap()
            ))
            .is_err()
    );
    assert!(catalog.selector(LuaValue::Integer(33)).is_err());
    let descriptor = catalog.describe(32, false);
    assert_eq!(
        descriptor["owners"][0]["qualified_name"],
        "keen::ecs::Example"
    );
    let bound = binding(&lua, &descriptor, None).unwrap();
    let result: (Option<String>, String, Option<u32>) = bound
        .get::<mlua::Function>("call")
        .unwrap()
        .call(())
        .unwrap();
    assert!(result.0.is_none());
    assert!(result.1.contains("unresolved"));
    bound.set("callable", true).unwrap(); // Descriptor edits cannot change the captured dispatcher.
    let result: (Option<String>, String, Option<u32>) = bound
        .get::<mlua::Function>("call")
        .unwrap()
        .call(())
        .unwrap();
    assert!(result.0.is_none());
}
