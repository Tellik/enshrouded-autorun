use super::*;

fn resource_fixture() -> Rc<TypeRegistry> {
    let mut types = Vec::new();
    for (i, (name, kind, size, inner, flags, count)) in [
        ("F32", "FLOAT32", 4, None, "", 0),
        ("Component", "STRUCT", 0, None, "", 0),
        ("FallDamage", "STRUCT", 8, Some(1), "", 2),
        (
            "Variant",
            "BLOB_VARIANT",
            12,
            Some(1),
            "HAS_BLOB_VARIANT",
            1,
        ),
        ("Components", "BLOB_ARRAY", 8, Some(3), "HAS_BLOB_ARRAY", 0),
        (
            "Template",
            "STRUCT",
            8,
            None,
            "HAS_BLOB_ARRAY | HAS_BLOB_VARIANT",
            1,
        ),
        ("DsArray", "DS_ARRAY", 24, Some(0), "HAS_DS", 0),
        ("DsRoot", "STRUCT", 24, None, "HAS_DS", 1),
        ("Values", "BLOB_ARRAY", 8, Some(0), "HAS_BLOB_ARRAY", 0),
        ("ValueRoot", "STRUCT", 8, None, "HAS_BLOB_ARRAY", 1),
        ("Bool", "BOOL", 1, None, "", 0),
        ("BoolValues", "BLOB_ARRAY", 8, Some(10), "HAS_BLOB_ARRAY", 0),
        ("BoolRoot", "STRUCT", 8, None, "HAS_BLOB_ARRAY", 1),
        ("String", "BLOB_STRING", 8, None, "HAS_BLOB_STRING", 0),
        ("StringRoot", "STRUCT", 8, None, "HAS_BLOB_STRING", 1),
    ]
    .into_iter()
    .enumerate()
    {
        let qualified = format!("resource_test::{name}");
        types.push(
            serde_json::json!({"index":i,"name":name,"qualifiedName":qualified,
                "impactName":name,"size":size,"alignment":4,"elementAlignment":4,
                "fieldCount":count,"primitiveType":kind,"innerType":inner,"flags":flags,
                "nameHash":kfc::hash::fnv(name),"qualifiedHash":kfc::hash::fnv(&qualified),
                "impactHash":kfc::hash::fnv(name),"internalHash":i}),
        );
    }
    types[2]["structFields"] = serde_json::json!({
            "start":{"name":"start","type":0,"dataOffset":0},
            "lethal":{"name":"lethal","type":0,"dataOffset":4}});
    for (index, field, child) in [
        (5, "components", 4),
        (7, "items", 6),
        (9, "items", 8),
        (12, "items", 11),
        (14, "text", 13),
    ] {
        types[index]["structFields"] =
            serde_json::json!({field:{"name":field,"type":child,"dataOffset":0}});
    }
    Rc::new(
        serde_json::from_value(serde_json::json!({"version":"resource-fixture","types":types}))
            .unwrap(),
    )
}

fn resource_bytes(payload: usize) -> Vec<u8> {
    let mut bytes = vec![0; payload + 16];
    bytes[0..4].copy_from_slice(&8u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    for (descriptor, start) in [(8, payload), (20, payload + 8)] {
        bytes[descriptor..descriptor + 4]
            .copy_from_slice(&kfc::hash::fnv("resource_test::FallDamage").to_le_bytes());
        bytes[descriptor + 4..descriptor + 8]
            .copy_from_slice(&((start - descriptor - 4) as u32).to_le_bytes());
        bytes[descriptor + 8..descriptor + 12].copy_from_slice(&8u32.to_le_bytes());
        bytes[start..start + 4].copy_from_slice(&8f32.to_le_bytes());
        bytes[start + 4..start + 8].copy_from_slice(&16f32.to_le_bytes());
    }
    bytes
}

#[test]
fn field_normalization_uses_the_live_writer_codec_and_float_domain() {
    let lua = mlua::Lua::new();
    lua.globals()
        .set("values", create(&lua, resource_fixture()).unwrap())
        .unwrap();
    lua.load(r#"
            local canonical, reason = values.normalize_field('resource_test::FallDamage', 'start', 0.1)
            assert(reason == nil and canonical == 0.10000000149011612)
            local invalid, why = values.normalize_field('resource_test::FallDamage', 'start', math.huge)
            assert(invalid == nil and why)
            local dynamic, failure = values.normalize_field('resource_test::DsRoot', 'items[1]', 1)
            assert(dynamic == nil and failure)
        "#).exec().unwrap();
}

#[test]
fn resource_blob_paths_match_the_pinned_mapper_and_change_only_the_selected_word() {
    let registry = resource_fixture();
    let lua = mlua::Lua::new();
    for payload in (32..=256).step_by(4) {
        let mut bytes = resource_bytes(payload);
        let owned: Rc<[u8]> = bytes.clone().into();
        let mapped =
            MappedValue::from_bytes(&registry, registry.get(TypeIndex::new(5)).unwrap(), &owned)
                .unwrap();
        let components = mapped
            .as_struct()
            .unwrap()
            .get("components")
            .unwrap()
            .unwrap();
        for (position, name, expected) in [(1, "start", 8.0), (2, "lethal", 16.0)] {
            let path = format!("components[{position}].value.{name}");
            let field = resource_field(&registry, TypeIndex::new(5), &path, &bytes).unwrap();
            let variant = components
                .as_array()
                .unwrap()
                .get(position - 1)
                .unwrap()
                .unwrap();
            let actual = variant
                .as_variant()
                .unwrap()
                .value()
                .get(name)
                .unwrap()
                .unwrap()
                .as_f32()
                .unwrap();
            let decoded = field.read(&lua, &bytes).unwrap();
            assert_eq!(actual, expected);
            assert!(matches!(decoded, LuaValue::Number(value) if value == actual as f64));
        }
        let field = resource_field(
            &registry,
            TypeIndex::new(5),
            "components[1].value.start",
            &bytes,
        )
        .unwrap();
        assert_eq!(field.byte_range(), payload..payload + 4);
        let before = bytes.clone();
        let mut mask = vec![0; bytes.len()];
        field
            .write_into(&lua, &LuaValue::Number(16.0), &mut bytes, &mut mask)
            .unwrap();
        assert_eq!(&bytes[payload..payload + 4], &16f32.to_le_bytes());
        for i in 0..bytes.len() {
            assert_eq!(mask[i], u8::from((payload..payload + 4).contains(&i)));
            if mask[i] == 0 {
                assert_eq!(bytes[i], before[i]);
            }
        }
        assert!(
            field
                .write_into(
                    &lua,
                    &LuaValue::Number(f64::INFINITY),
                    &mut bytes,
                    &mut vec![0; before.len()]
                )
                .is_err()
        );
    }
    assert!(inline_field(&registry, TypeIndex::new(5), "components[1].value.start").is_err());
}

#[test]
fn resource_paths_reject_bad_indices_descriptors_truncation_and_nonfinite_values() {
    let registry = resource_fixture();
    let bytes = resource_bytes(32);
    let resolve =
        |path: &str, bytes: &[u8]| resource_field(&registry, TypeIndex::new(5), path, bytes);
    for path in [
        "",
        "components[0].value.start",
        "components[3].value.start",
        "components[-1].value.start",
        "components[+1].value.start",
        "components[1]value.start",
        "components[1].value.",
        "components[1].start",
        "components[1].type",
        "components[1].value",
        "components[1].value.missing",
    ] {
        assert!(resolve(path, &bytes).is_err(), "accepted {path}");
    }
    for length in 0..40 {
        assert!(resolve("components[1].value.start", &bytes[..length]).is_err());
    }
    for (offset, value) in [
        (0, u32::MAX),
        (0, 4),
        (4, u32::MAX),
        (8, 0),
        (12, u32::MAX),
        (12, 0),
        (16, 4),
    ] {
        let mut corrupt = bytes.clone();
        corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(
            resolve("components[1].value.start", &corrupt).is_err(),
            "accepted corruption {offset}={value}"
        );
    }
    let mut corrupt = bytes.clone();
    corrupt[32..36].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(resolve("components[1].value.start", &corrupt).is_err());
}

#[test]
fn resource_scalar_arrays_are_bounded_and_ds_strings_and_bool_words_are_rejected() {
    let registry = resource_fixture();
    let mut bytes = vec![0; 16];
    bytes[0..4].copy_from_slice(&8u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&3f32.to_le_bytes());
    bytes[12..16].copy_from_slice(&7f32.to_le_bytes());
    assert_eq!(
        resource_field(&registry, TypeIndex::new(9), "items[2]", &bytes)
            .unwrap()
            .byte_range(),
        12..16
    );
    for (index, path) in [
        (7, "items[1]"),
        (12, "items[1]"),
        (14, "text"),
        (14, "text[1]"),
    ] {
        assert!(resource_field(&registry, TypeIndex::new(index), path, &[0; 24]).is_err());
    }
    let mut unaligned = bytes.clone();
    unaligned[0..4].copy_from_slice(&9u32.to_le_bytes());
    unaligned.push(0);
    assert!(resource_field(&registry, TypeIndex::new(9), "items[1]", &unaligned).is_err());
}

#[test]
#[ignore = "set SHROUDFORGE_TEST_REGISTRY and SHROUDFORGE_TEST_RESOURCE_CAPTURE to owned exports"]
fn installed_template_snapshot_resolves_resource_fields_when_explicitly_requested() {
    let registry: Rc<TypeRegistry> = Rc::new(
        serde_json::from_slice(
            &std::fs::read(std::env::var("SHROUDFORGE_TEST_REGISTRY").unwrap()).unwrap(),
        )
        .unwrap(),
    );
    let capture: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("SHROUDFORGE_TEST_RESOURCE_CAPTURE").unwrap()).unwrap(),
    )
    .unwrap();
    let lua = mlua::Lua::new();
    let root = registry
        .iter()
        .find(|ty| ty.qualified_name == "keen::ecs::TemplateResource")
        .unwrap();
    let config = registry
        .iter()
        .find(|ty| ty.qualified_name == "keen::ecs::FallDamage")
        .unwrap();
    let mut checked = 0;
    for service in capture["player"]["services"].as_array().unwrap() {
        for resource in service["templates"].as_array().unwrap() {
            let bytes: Vec<u8> = resource["resourceBytes"]
                .as_str()
                .unwrap()
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect();
            assert_eq!(bytes.len(), resource["size"].as_u64().unwrap() as usize);
            for variant in resource["fallDamage"].as_array().unwrap() {
                for (name, value_position) in [
                    ("fallDamageStartDistance", 0),
                    ("fallDamageLethalDistance", 1),
                ] {
                    let path = format!(
                        "components[{}].value.{name}",
                        variant["ordinal"].as_u64().unwrap()
                    );
                    let field = resource_field(&registry, root.index, &path, &bytes).unwrap();
                    let expected_offset = variant["payloadOffset"].as_u64().unwrap() as usize
                        + config.struct_fields[name].data_offset as usize;
                    assert_eq!(field.byte_range(), expected_offset..expected_offset + 4);
                    let expected = variant["values"][value_position].as_f64().unwrap();
                    assert!(
                        matches!(field.read(&lua, &bytes).unwrap(), LuaValue::Number(value) if value == expected)
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0);
    eprintln!(
        "Production resource walker resolved {checked} original fall-damage fields through the pinned KFC mapper"
    );
}

#[test]
fn lua_values_round_trip_inherited_fields_defaults_and_validate_errors() {
    let lua = mlua::Lua::new();
    let registry = Rc::new(super::super::registry::tests::fixture());
    lua.globals()
        .set("values", create(&lua, registry).unwrap())
        .unwrap();
    lua.load(
        r#"
            local a, reason = values.new('test::Child')
            assert(a and not reason and a.x == 17 and a.y == 23)
            local b = assert(values.new('test::Child'))
            a.x = 1234
            assert(b.x == 17)
            local bytes = assert(values.encode('test::Child', a))
            assert(#bytes == 8)
            local decoded = assert(values.decode('test::Child', bytes))
            assert(decoded.x == 1234 and decoded.y == 23)
            local missing, why = values.decode('test::Child', 'short')
            assert(missing == nil and why:find('truncated'))
            local invalid, error = values.encode('test::U32', -1)
            assert(invalid == nil and error)
            local ds, ds_error = values.new('test::HasDs')
            assert(ds == nil and ds_error:find('engine allocation'))
        "#,
    )
    .exec()
    .unwrap();
}

#[test]
fn named_byte_enum_expected_fields_use_the_production_codec_and_preserve_siblings() {
    let lua = mlua::Lua::new();
    let mut fixture = serde_json::to_value(super::super::registry::tests::fixture()).unwrap();
    let types = fixture["types"].as_array_mut().unwrap();
    types.push(serde_json::json!({"index":6,"name":"State","qualifiedName":"test::State","impactName":"State",
            "size":1,"alignment":1,"elementAlignment":1,"fieldCount":2,"primitiveType":"ENUM","flags":"","innerType":8,
            "nameHash":1,"impactHash":2,"qualifiedHash":3,"internalHash":4,
            "enumFields":{"Walking":{"name":"Walking","value":2},"BuildingFlying":{"name":"BuildingFlying","value":11}}}));
    types.push(serde_json::json!({"index":7,"name":"Locomotion","qualifiedName":"test::Locomotion","impactName":"Locomotion",
            "size":8,"alignment":4,"elementAlignment":4,"fieldCount":2,"primitiveType":"STRUCT","flags":"",
            "nameHash":5,"impactHash":6,"qualifiedHash":7,"internalHash":8,
            "structFields":{"counter":{"name":"counter","type":0,"dataOffset":0},"state":{"name":"state","type":6,"dataOffset":5}}}));
    types.push(serde_json::json!({"index":8,"name":"U8","qualifiedName":"test::U8","impactName":"U8",
            "size":1,"alignment":1,"elementAlignment":1,"fieldCount":0,"primitiveType":"UINT8","flags":"",
            "nameHash":9,"impactHash":10,"qualifiedHash":11,"internalHash":12}));
    let registry = Rc::new(serde_json::from_value::<TypeRegistry>(fixture).unwrap());
    let field = inline_field(&registry, TypeIndex::new(7), "state").unwrap();
    let before = [99, 0, 0, 0, 0xa5, 2, 0x5a, 0xff];
    assert_eq!(
        field
            .read(&lua, &before)
            .unwrap()
            .as_string()
            .unwrap()
            .to_str()
            .unwrap(),
        "Walking"
    );
    let expected = lua.create_table().unwrap();
    expected.set("state", "Walking").unwrap();
    expect_inline_fields(&lua, &registry, TypeIndex::new(7), &before, &expected).unwrap();
    let mut after = before;
    let mut mask = [0; 8];
    field
        .write_into(
            &lua,
            &LuaValue::String(lua.create_string("BuildingFlying").unwrap()),
            &mut after,
            &mut mask,
        )
        .unwrap();
    assert_eq!(after, [99, 0, 0, 0, 0xa5, 11, 0x5a, 0xff]);
    assert_eq!(mask, [0, 0, 0, 0, 0, 1, 0, 0]);
    assert!(
        expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected)
            .unwrap_err()
            .contains("snapshot changed")
    );
    expected.set("state", "BuildingFlying").unwrap();
    expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected).unwrap();
    expected.set("state", "InventedState").unwrap();
    assert!(expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected).is_err());
    assert!(
        expect_inline_fields(
            &lua,
            &registry,
            TypeIndex::new(7),
            &after,
            &lua.create_table().unwrap()
        )
        .is_err()
    );
}

#[test]
#[ignore = "set SHROUDFORGE_TEST_REGISTRY to an installed parser cache"]
fn installed_registries_resolve_all_indices_when_explicitly_requested() {
    use mlua::ObjectLike;
    let path = std::env::var("SHROUDFORGE_TEST_REGISTRY").expect("registry cache path required");
    let registry: Rc<TypeRegistry> =
        Rc::new(serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap());
    let lua = mlua::Lua::new();
    let definitions = crate::definition::generator::generate(&registry);
    for ty in registry.iter() {
        assert_eq!(
            registry::resolve(&registry, &LuaValue::Integer(ty.qualified_hash as i64)).unwrap(),
            ty.index
        );
        registry::inheritance(&registry, ty.index).unwrap();
        registry::fields(&registry, ty.index).unwrap();
        let value = lua
            .create_userdata(super::super::Type::new(TypeHandle::new(
                registry.clone(),
                ty.index,
            )))
            .unwrap();
        assert_eq!(value.get::<u8>("flags_bits").unwrap(), ty.flags.bits());
        assert_eq!(value.get::<usize>("index").unwrap(), ty.index.as_usize());
        assert_eq!(
            value.get::<u32>("qualified_hash").unwrap(),
            ty.qualified_hash
        );
        if ty.primitive_type == PrimitiveType::Struct {
            assert!(
                definitions.contains(&format!(
                    "---@class {}",
                    ty.qualified_name.replace("::", ".")
                )),
                "missing generated type: {}",
                ty.qualified_name
            );
        }
    }
    eprintln!(
        "Lua metadata verified for all {} types ({})",
        registry.len(),
        registry.version
    );
    if let Ok(path) = std::env::var("SHROUDFORGE_TEST_ECS_SNAPSHOTS") {
        let capture: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let snapshots = capture["snapshots"]
            .as_array()
            .expect("capture snapshots required");
        let values = create(&lua, registry.clone()).unwrap();
        let owned_decode = values.get::<mlua::Function>("decode").unwrap();
        let mut results = Vec::new();
        let mut whole_count = 0;
        let mut field_count = 0;
        let mut scalar_count = 0;
        let mut partially_decoded = 0;
        for snapshot in snapshots {
            let name = snapshot["component"].as_str().unwrap();
            let index = registry::resolve(
                &registry,
                &LuaValue::String(lua.create_string(name).unwrap()),
            )
            .unwrap();
            let ty = registry.get(index).unwrap();
            assert_eq!(
                snapshot["qualifiedHash"].as_u64(),
                Some(u64::from(ty.qualified_hash))
            );
            let hex = snapshot["bytes"].as_str().unwrap();
            assert_eq!(hex.len(), ty.size as usize * 2);
            let bytes: Vec<_> = hex
                .as_bytes()
                .chunks_exact(2)
                .map(|pair| {
                    ((pair[0] as char).to_digit(16).unwrap() * 16
                        + (pair[1] as char).to_digit(16).unwrap()) as u8
                })
                .collect();
            assert_eq!(bytes.len(), ty.size as usize);
            let (whole, whole_reason): (LuaValue, Option<String>) = owned_decode
                .call((name, lua.create_string(&bytes).unwrap()))
                .unwrap();
            let whole_ok = whole_reason.is_none() && !whole.is_nil();
            whole_count += usize::from(whole_ok);
            let decoded = lua.create_table().unwrap();
            let mut fields_ok = 0;
            let mut field_errors = serde_json::Map::new();
            for (_, field) in registry::fields(&registry, index).unwrap() {
                match inline_field(&registry, index, &field.name)
                    .and_then(|field| field.read(&lua, &bytes))
                {
                    Ok(value) => {
                        decoded.raw_set(field.name.as_str(), value).unwrap();
                        fields_ok += 1;
                    }
                    Err(reason) => {
                        field_errors.insert(field.name.clone(), serde_json::json!(reason));
                    }
                }
            }
            for (field, expected) in snapshot["inlineScalarFields"]
                .as_object()
                .into_iter()
                .flatten()
            {
                let value = decoded.raw_get::<LuaValue>(field.as_str()).unwrap();
                match value {
                    LuaValue::Boolean(value) => {
                        assert_eq!(expected.as_bool(), Some(value), "{name}.{field}")
                    }
                    LuaValue::Integer(value) => {
                        assert_eq!(expected.as_f64(), Some(value as f64), "{name}.{field}")
                    }
                    LuaValue::Number(value) => {
                        let field_type = registry::fields(&registry, index)
                            .unwrap()
                            .iter()
                            .find(|(_, metadata)| metadata.name == *field)
                            .and_then(|(_, metadata)| registry.get(metadata.r#type))
                            .unwrap();
                        if field_type.primitive_type == PrimitiveType::Float32 {
                            // JSON parsing can shift the f64 representation of
                            // f32::MAX by one f64 ULP. Compare the actual declared
                            // f32 representation, without an epsilon tolerance.
                            assert_eq!(
                                (expected.as_f64().unwrap() as f32).to_bits(),
                                (value as f32).to_bits(),
                                "{name}.{field}"
                            );
                        } else {
                            assert_eq!(expected.as_f64(), Some(value), "{name}.{field}");
                        }
                    }
                    _ => panic!(
                        "capture scalar did not decode through production field codec: {name}.{field}"
                    ),
                }
                scalar_count += 1;
            }
            if name == "keen::ecs::NetworkPlayerAttributes" {
                lua.globals()
                    .set("captured_player_fields", decoded.clone())
                    .unwrap();
                lua.load("assert(type(captured_player_fields.health) == 'number'); assert(type(captured_player_fields.stamina) == 'number'); assert(type(captured_player_fields.movementSprint) == 'number')").exec().unwrap();
            }
            field_count += fields_ok;
            partially_decoded += usize::from(!whole_ok && fields_ok > 0);
            results.push(serde_json::json!({"component":name,"whole_value_decoded":whole_ok,"whole_value_reason":whole_reason,
                    "inline_fields_decoded":fields_ok,"inline_field_reasons":field_errors,
                    "identity_only":!whole_ok && fields_ok == 0}));
        }
        let report = serde_json::json!({"source_capture":path,"capture_execution":"external-read-only-process-snapshot",
                "new_dll_or_live_lua_verified":false,"type_count":registry.len(),"snapshot_count":snapshots.len(),
                "whole_values_decoded":whole_count,"partial_values_decoded":partially_decoded,"inline_fields_decoded":field_count,
                "capture_scalars_compared":scalar_count,"entries":results});
        std::fs::write(
            format!("{path}.lua-codec.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        eprintln!(
            "Production Lua codecs checked {} external snapshots: {whole_count} whole values, {partially_decoded} partial values, {field_count} inline fields, {scalar_count} captured scalars compared",
            snapshots.len()
        );
    }
}
