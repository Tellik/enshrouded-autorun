use super::*;
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
