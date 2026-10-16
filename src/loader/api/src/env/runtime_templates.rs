//! Checked views into a current entity's shared template configuration.
use super::{AppState, loader, registry, runtime_values, util::add_function_with_mod};
use crate::{
    alias::MappedValue,
    lua::{FunctionArgs, LuaValue},
};
use kfc::{
    guid::ResourceId,
    reflection::{PrimitiveType, TypeIndex, TypeRegistry},
};
use mlua::{MultiValue, Table, UserData};
use mod_loader::Mod;
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

const TEMPLATE_TYPE: &str = "keen::ecs::TemplateResource";

/// This deliberately owns the resource identity, not the entity handle. A mod
/// can restore its shared-template change after that particular entity leaves.
pub(crate) struct RuntimeTemplateComponent {
    owner_mod: Mod,
    resource: ResourceId,
    root_type: TypeIndex,
    component_type: TypeIndex,
    key: String,
    session: u64,
    write_fault: RefCell<Option<String>>,
}

pub(crate) fn attach(lua: &mlua::Lua, ecs: &Table, owner_mod: &Mod) -> mlua::Result<()> {
    add_function_with_mod(lua, ecs, "get_template_component", owner_mod, get_component)
}

fn require_access(state: &AppState, owner_mod: &Mod, operation: &str) -> Result<(), String> {
    if let Some(reason) = loader::runtime_denial_reason(state, owner_mod, operation) {
        return Err(reason);
    }
    if !loader::available(state, owner_mod, operation) {
        return Err(format!("{operation} is currently unavailable"));
    }
    Ok(())
}

fn require_session(session: u64) -> Result<(), String> {
    if session == 0 || loader::runtime_provider::world_session_id() != session {
        return Err("template view belongs to another or an inactive world session".into());
    }
    Ok(())
}

/// Select the actual configuration role, never a similarly named runtime layout.
fn template_registration(
    registry: &TypeRegistry,
    report: &Value,
    selector: TypeIndex,
) -> Result<TypeIndex, String> {
    let requested = registry
        .get(selector)
        .ok_or("unresolved template selector")?;
    if report["available"] != true {
        return Err("live template registration is unavailable".into());
    }
    let rows = report["entries"]
        .as_array()
        .ok_or("live registry has no entries")?;
    let mut matches = rows.iter().filter(|row| {
        row["qualified_name"] == requested.qualified_name
            || row["template_type"]["qualified_name"] == requested.qualified_name
    });
    let row = matches.next().ok_or_else(|| {
        format!(
            "no live template registration for {}",
            requested.qualified_name
        )
    })?;
    if matches.next().is_some() {
        return Err(format!(
            "ambiguous live template registration for {}",
            requested.qualified_name
        ));
    }
    let owner_name = row["qualified_name"]
        .as_str()
        .ok_or("registration has no name")?;
    let owner = registry
        .iter()
        .find(|ty| ty.qualified_name == owner_name)
        .ok_or("registration owner is absent from reflection")?;
    if row["qualified_hash"].as_u64() != Some(u64::from(owner.qualified_hash))
        || row["index"]
            .as_u64()
            .is_none_or(|index| index > u64::from(u16::MAX))
    {
        return Err("template registration owner identity disagrees with reflection".into());
    }
    let template = &row["template_type"];
    let name = template["qualified_name"]
        .as_str()
        .ok_or("registration has no template configuration layout")?;
    let ty = registry
        .iter()
        .find(|ty| ty.qualified_name == name)
        .ok_or("template layout is absent from reflection")?;
    if ty.primitive_type != PrimitiveType::Struct
        || ty.size == 0
        || template["qualified_hash"].as_u64() != Some(u64::from(ty.qualified_hash))
        || template["size"].as_u64() != Some(u64::from(ty.size))
    {
        return Err("template registration layout disagrees with reflection".into());
    }
    Ok(ty.index)
}

/// Use the production mapper for container and concrete-variant interpretation.
/// Only the one-based position leaves this function. No Lua type-shape guess or
/// manually copied variant hash/offset table participates in selection.
fn select_variant(
    registry: &Rc<TypeRegistry>,
    root: TypeIndex,
    component: TypeIndex,
    bytes: &[u8],
) -> Result<usize, String> {
    let root = registry
        .get(root)
        .ok_or("unresolved template resource layout")?;
    let expected = registry
        .get(component)
        .ok_or("unresolved template component layout")?;
    if root.primitive_type != PrimitiveType::Struct || root.size as usize > bytes.len() {
        return Err("TemplateResource is not a complete reflected struct".into());
    }
    // Reject unsupported native container codecs before invoking the upstream
    // mapper, whose DS decoding paths are intentionally not implemented.
    let fields = registry::fields(registry, root.index)?;
    let (_, components_field) = fields
        .iter()
        .find(|(_, field)| field.name == "components")
        .ok_or("TemplateResource has no reflected components field")?;
    let array_layout = registry
        .get(components_field.r#type)
        .ok_or("unresolved components array layout")?;
    let variant_layout = array_layout
        .inner_type
        .and_then(|index| registry.get(index))
        .ok_or("components array has no reflected element layout")?;
    if array_layout.primitive_type != PrimitiveType::BlobArray
        || array_layout.size != 8
        || variant_layout.primitive_type != PrimitiveType::BlobVariant
        || variant_layout.size != 12
    {
        return Err("template components do not use the supported owned BlobVariant array".into());
    }
    let owned: Rc<[u8]> = bytes.into();
    let mapped = MappedValue::from_bytes(registry, root, &owned).map_err(|e| e.to_string())?;
    let root = mapped
        .as_struct()
        .ok_or("TemplateResource is not a reflected struct")?;
    let components = root
        .get("components")
        .map_err(|e| e.to_string())?
        .ok_or("TemplateResource has no components field")?;
    let components = components
        .as_array()
        .ok_or("template components are not an array")?;
    if components.r#type().primitive_type != PrimitiveType::BlobArray
        || components.element_type().primitive_type != PrimitiveType::BlobVariant
        || components.element_type().size != 12
        || components.len() > bytes.len() / 12
    {
        return Err("template components do not use the supported owned BlobVariant array".into());
    }
    let base = components
        .element_type()
        .inner_type()
        .ok_or("variant has no base layout")?;
    if !registry.is_sub_type(&base, expected) {
        return Err("template component is not a subtype of its reflected variant base".into());
    }
    let mut found = None;
    for index in 0..components.len() {
        let value = components
            .get(index)
            .map_err(|e| e.to_string())?
            .ok_or("template component array has a missing element")?;
        let variant = value
            .as_variant()
            .ok_or("template component is not a BlobVariant")?;
        let ty = variant.variant_type();
        if ty.index != component {
            continue;
        }
        if ty.qualified_name != expected.qualified_name
            || ty.qualified_hash != expected.qualified_hash
        {
            return Err("template variant identity disagrees with reflection".into());
        }
        if found.replace(index + 1).is_some() {
            return Err(format!(
                "template has multiple {} variants",
                expected.qualified_name
            ));
        }
    }
    found.ok_or_else(|| format!("template has no {} configuration", expected.qualified_name))
}

fn get_component(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner_mod: &Mod,
) -> mlua::Result<(Option<RuntimeTemplateComponent>, Option<String>)> {
    let get = || -> Result<RuntimeTemplateComponent, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        require_access(&state, owner_mod, "runtime.ecs.read")?;
        if args.len() != 2 {
            return Err("get_template_component expects entity and component type".into());
        }
        let session = loader::runtime_provider::world_session_id();
        require_session(session)?;
        let entity = args.get::<u32>(0).map_err(|e| e.to_string())?;
        let types = state.type_registry();
        let selector =
            registry::resolve(types, args.get::<LuaValue>(1).map_err(|e| e.to_string())?)?;
        let report = loader::runtime_provider::registry()?;
        let component = template_registration(types, &report, selector)?;
        let guid = loader::runtime_provider::entity_template_uuid(entity)
            .ok_or("entity template UUID is unavailable or unstable")?;
        let resource = shroudforge_parser::kfc_format::resource_id(types, TEMPLATE_TYPE, guid, 0)
            .ok_or("TemplateResource is absent from active reflection")?;
        let root = shroudforge_parser::kfc_format::type_for_resource(types, &resource)
            .filter(|ty| ty.qualified_name == TEMPLATE_TYPE)
            .ok_or("resource identity does not resolve to TemplateResource")?;
        let bytes = loader::runtime_provider::resource_read(&resource)?;
        select_variant(types, root.index, component, &bytes)?;
        if loader::runtime_provider::entity_template_uuid(entity) != Some(guid) {
            return Err("entity template changed during configuration resolution".into());
        }
        require_session(session)?;
        let component_name = &types.get(component).unwrap().qualified_name;
        Ok(RuntimeTemplateComponent {
            owner_mod: owner_mod.clone(),
            resource,
            root_type: root.index,
            component_type: component,
            key: format!(
                "{}/{}/{}/{}",
                resource.guid(),
                TEMPLATE_TYPE,
                resource.part_index(),
                component_name
            ),
            session,
            write_fault: RefCell::new(None),
        })
    };
    Ok(match get() {
        Ok(view) => (Some(view), None),
        Err(reason) => (None, Some(reason)),
    })
}

impl RuntimeTemplateComponent {
    pub(crate) fn ownership_identity(&self, owner: &Mod) -> Result<(ResourceId, u64), String> {
        if self.owner_mod.info().id != owner.info().id {
            return Err("template view belongs to a different runtime mod".into());
        }
        Ok((self.resource, self.session))
    }

    fn selected_field(
        &self,
        types: &Rc<TypeRegistry>,
        position: usize,
        path: &str,
        bytes: &[u8],
    ) -> Result<runtime_values::InlineField, String> {
        let full_path = format!("components[{position}].value.{path}");
        runtime_values::resource_field(types, self.root_type, &full_path, bytes)
            .map_err(|reason| format!("{path}: {reason}"))
    }

    fn expect_fields(
        &self,
        lua: &mlua::Lua,
        types: &Rc<TypeRegistry>,
        position: usize,
        before: &[u8],
        expected_fields: &Table,
    ) -> Result<(), String> {
        let mut candidate = before.to_vec();
        let mut mask = vec![0; before.len()];
        let mut count = 0;
        for pair in expected_fields.clone().pairs::<LuaValue, LuaValue>() {
            let (path, value) = pair.map_err(|reason| reason.to_string())?;
            let LuaValue::String(path) = path else {
                return Err(
                    "expected_fields keys must be component-relative reflected field paths".into(),
                );
            };
            count += 1;
            if count > 1024 {
                return Err("at most 1024 expected field paths may be selected".into());
            }
            let path = path.to_str().map_err(|reason| reason.to_string())?;
            self.selected_field(types, position, &path, before)?
                .write_into(lua, &value, &mut candidate, &mut mask)
                .map_err(|reason| format!("expected {path}: {reason}"))?;
        }
        if count == 0 {
            return Err("expected_fields must be nonempty when provided".into());
        }
        if candidate.as_slice() != before {
            return Err(
                "Template field snapshot changed before update. Read current fields and retry"
                    .into(),
            );
        }
        Ok(())
    }

    fn snapshot(&self, state: &AppState) -> Result<(Vec<u8>, usize), String> {
        require_access(state, &self.owner_mod, "runtime.ecs.read")?;
        require_session(self.session)?;
        let types = state.type_registry();
        let report = loader::runtime_provider::registry()?;
        if template_registration(types, &report, self.component_type)? != self.component_type {
            return Err("template registration changed since view creation".into());
        }
        if shroudforge_parser::kfc_format::type_for_resource(types, &self.resource)
            .is_none_or(|ty| ty.index != self.root_type || ty.qualified_name != TEMPLATE_TYPE)
        {
            return Err("template resource layout changed since view creation".into());
        }
        let bytes = loader::runtime_provider::resource_read(&self.resource)?;
        let position = select_variant(types, self.root_type, self.component_type, &bytes)?;
        require_session(self.session)?;
        Ok((bytes, position))
    }

    fn read_fields(&self, lua: &mlua::Lua, args: MultiValue) -> Result<Table, String> {
        if args.len() != 1 {
            return Err("read_fields expects one dense array of field paths".into());
        }
        let LuaValue::Table(paths) = &args[0] else {
            return Err("read_fields expects a dense array of field paths".into());
        };
        let mut selected = BTreeSet::new();
        for pair in paths.clone().pairs::<LuaValue, LuaValue>() {
            let (key, path) = pair.map_err(|e| e.to_string())?;
            let LuaValue::Integer(index) = key else {
                return Err("field paths must be a dense array".into());
            };
            if index <= 0 || index as usize > paths.raw_len() || selected.len() >= 1024 {
                return Err("select a dense array of at most 1024 field paths".into());
            }
            let LuaValue::String(path) = path else {
                return Err("field paths must be strings".into());
            };
            if !selected.insert(path.to_str().map_err(|e| e.to_string())?.to_owned()) {
                return Err("duplicate field path".into());
            }
        }
        if selected.is_empty() || selected.len() != paths.raw_len() {
            return Err("select a nonempty dense array of field paths".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let (bytes, position) = self.snapshot(&state)?;
        let result = lua
            .create_table_with_capacity(0, selected.len())
            .map_err(|e| e.to_string())?;
        for path in selected {
            let field = self.selected_field(state.type_registry(), position, &path, &bytes)?;
            result
                .raw_set(path.as_str(), field.read(lua, &bytes)?)
                .map_err(|e| e.to_string())?;
        }
        require_session(self.session)?;
        Ok(result)
    }

    fn write_field(&self, lua: &mlua::Lua, args: MultiValue) -> Result<(), String> {
        if let Some(reason) = self.write_fault.borrow().as_ref() {
            return Err(reason.clone());
        }
        if args.len() != 3 && args.len() != 4 {
            return Err("write_field expects path, value, expected current value and optional expected_fields".into());
        }
        let LuaValue::String(path) = &args[0] else {
            return Err("write_field expects a reflected field path".into());
        };
        let path = path.to_str().map_err(|e| e.to_string())?;
        let state = lua.app_data_ref::<AppState>().unwrap();
        require_access(&state, &self.owner_mod, "runtime.ecs.write")?;
        require_access(&state, &self.owner_mod, "runtime.assets.write")?;
        let (before, position) = self.snapshot(&state)?;
        let field = self.selected_field(state.type_registry(), position, &path, &before)?;
        let range = field.byte_range();
        let mut expected = before.clone();
        field.write_into(lua, &args[2], &mut expected, &mut vec![0; before.len()])?;
        if expected[range.clone()] != before[range.clone()] {
            return Err(
                "template scalar changed before update. Read current fields and retry".into(),
            );
        }
        if args.len() == 4 {
            let LuaValue::Table(expected_fields) = &args[3] else {
                return Err("expected_fields must be a nonempty field-path/value map".into());
            };
            self.expect_fields(
                lua,
                state.type_registry(),
                position,
                &before,
                expected_fields,
            )?;
        }
        let mut after = before.clone();
        field.write_into(lua, &args[1], &mut after, &mut vec![0; before.len()])?;
        require_session(self.session)?;
        super::runtime_ownership::check_write(&state, &self.owner_mod,
            &super::runtime_ownership::Target::Resource(self.resource))?;
        if let Err(reason) = state.compare_exchange_loaded_resource_scalar(
            &self.resource,
            before.len(),
            range.start,
            &before,
            &after[range.clone()],
            self.session,
        ) {
            if reason.to_lowercase().contains("uncertain") {
                state.record_uncertain_runtime_write(self.session, &reason);
                *self.write_fault.borrow_mut() = Some(reason.clone());
            }
            return Err(reason);
        }
        // Full-snapshot comparison, mutation and independent resulting-payload
        // readback were confirmed under both original native gates. A later
        // observation or session change cannot turn that success into failure.
        Ok(())
    }
}

impl UserData for RuntimeTemplateComponent {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("key", |_, this| Ok(this.key.clone()));
        fields.add_field_method_get("scope", |_, _| Ok("shared-template"));
        fields.add_field_method_get("session_id", |_, this| Ok(this.session));
        fields.add_field_method_get("type", |lua, this| {
            lua.app_data_ref::<AppState>()
                .unwrap()
                .get_type(lua, this.component_type)
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("read_fields", |lua, this, args: MultiValue| {
            Ok(match this.read_fields(lua, args) {
                Ok(values) => (Some(values), None),
                Err(reason) => (None, Some(reason)),
            })
        });
        methods.add_method("write_field", |lua, this, args: MultiValue| {
            Ok(match this.write_field(lua, args) {
                Ok(()) => (true, None),
                Err(reason) => (false, Some(reason)),
            })
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Rc<TypeRegistry> {
        let mut types = Vec::new();
        for (index, (name, primitive, size, inner, flags, count)) in [
            ("F32", "FLOAT32", 4, None, "", 0),
            ("Component", "STRUCT", 0, None, "", 0),
            ("FallDamage", "STRUCT", 8, Some(1), "", 2),
            ("Other", "STRUCT", 8, Some(1), "", 0),
            (
                "Variant",
                "BLOB_VARIANT",
                12,
                Some(1),
                "HAS_BLOB_VARIANT",
                1,
            ),
            ("Components", "BLOB_ARRAY", 8, Some(4), "HAS_BLOB_ARRAY", 0),
            (
                "TemplateResource",
                "STRUCT",
                8,
                None,
                "HAS_BLOB_ARRAY | HAS_BLOB_VARIANT",
                1,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let qualified = format!("keen::ecs::{name}");
            types.push(
                json!({"index":index,"name":name,"qualifiedName":qualified,"impactName":name,
                "size":size,"alignment":4,"elementAlignment":4,"fieldCount":count,
                "primitiveType":primitive,"innerType":inner,"flags":flags,
                "nameHash":kfc::hash::fnv(name),"qualifiedHash":kfc::hash::fnv(&qualified),
                "impactHash":kfc::hash::fnv(name),"internalHash":index}),
            );
        }
        types[2]["structFields"] = json!({"start":{"name":"start","type":0,"dataOffset":0},
            "lethal":{"name":"lethal","type":0,"dataOffset":4}});
        types[6]["structFields"] =
            json!({"components":{"name":"components","type":5,"dataOffset":0}});
        Rc::new(
            serde_json::from_value(json!({"version":"template-view-fixture","types":types}))
                .unwrap(),
        )
    }

    fn snapshot(names: &[&str]) -> Vec<u8> {
        let payload = 8 + names.len() * 12;
        let mut bytes = vec![0; payload + names.len() * 8];
        bytes[0..4].copy_from_slice(&8u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&(names.len() as u32).to_le_bytes());
        for (index, name) in names.iter().enumerate() {
            let descriptor = 8 + index * 12;
            let start = payload + index * 8;
            bytes[descriptor..descriptor + 4].copy_from_slice(&kfc::hash::fnv(*name).to_le_bytes());
            bytes[descriptor + 4..descriptor + 8]
                .copy_from_slice(&((start - descriptor - 4) as u32).to_le_bytes());
            bytes[descriptor + 8..descriptor + 12].copy_from_slice(&8u32.to_le_bytes());
            bytes[start..start + 4].copy_from_slice(&8f32.to_le_bytes());
            bytes[start + 4..start + 8].copy_from_slice(&16f32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn concrete_variant_selection_uses_the_mapper_and_rejects_missing_or_duplicate_types() {
        let types = fixture();
        let root = TypeIndex::new(6);
        let component = TypeIndex::new(2);
        let bytes = snapshot(&["keen::ecs::Other", "keen::ecs::FallDamage"]);
        assert_eq!(select_variant(&types, root, component, &bytes).unwrap(), 2);
        assert!(
            select_variant(&types, root, component, &snapshot(&["keen::ecs::Other"]))
                .unwrap_err()
                .contains("has no")
        );
        assert!(
            select_variant(
                &types,
                root,
                component,
                &snapshot(&["keen::ecs::FallDamage", "keen::ecs::FallDamage"])
            )
            .unwrap_err()
            .contains("multiple")
        );
        assert!(select_variant(&types, root, component, &bytes[..8]).is_err());
        assert!(
            runtime_values::resource_field(
                &types,
                root,
                "components[2].value.start",
                &bytes[..bytes.len() - 7]
            )
            .is_err()
        );
        let mut unsupported = serde_json::to_value(types.as_ref()).unwrap();
        unsupported["types"][5]["primitiveType"] = json!("DS_ARRAY");
        let unsupported = Rc::new(serde_json::from_value::<TypeRegistry>(unsupported).unwrap());
        assert!(select_variant(&unsupported, root, component, &bytes).is_err());
        let field =
            runtime_values::resource_field(&types, root, "components[2].value.start", &bytes)
                .unwrap();
        let lua = mlua::Lua::new();
        assert!(
            matches!(field.read(&lua, &bytes).unwrap(), LuaValue::Number(value) if value == 8.0)
        );
        let owned: Rc<[u8]> = bytes.into();
        let mapped = MappedValue::from_bytes(&types, types.get(root).unwrap(), &owned).unwrap();
        lua.globals()
            .set(
                "template",
                super::super::game::value::convert_value_to_lua(&mapped, &lua).unwrap(),
            )
            .unwrap();
        lua.load("assert(type(template.components[2].type)=='string' and template.components[2].type=='keen::ecs::FallDamage')").exec().unwrap();
    }

    #[test]
    fn a_template_role_requires_one_exact_live_registration() {
        let types = fixture();
        let row = json!({"index":210,"qualified_name":"keen::ecs::FallDamage",
            "qualified_hash":kfc::hash::fnv("keen::ecs::FallDamage"),"runtime_type":null,
            "template_type":{"qualified_name":"keen::ecs::FallDamage","qualified_hash":kfc::hash::fnv("keen::ecs::FallDamage"),"size":8}});
        let selector = TypeIndex::new(2);
        assert_eq!(
            template_registration(
                &types,
                &json!({"available":true,"entries":[row.clone()]}),
                selector
            )
            .unwrap(),
            selector
        );
        let mut separate_owner = row.clone();
        separate_owner["qualified_name"] = json!("keen::ecs::Other");
        separate_owner["qualified_hash"] = json!(kfc::hash::fnv("keen::ecs::Other"));
        assert_eq!(
            template_registration(
                &types,
                &json!({"available":true,"entries":[separate_owner]}),
                TypeIndex::new(3)
            )
            .unwrap(),
            selector
        );
        for report in [
            json!({"available":false,"entries":[row.clone()]}),
            json!({"available":true,"entries":[]}),
            json!({"available":true,"entries":[row.clone(),row.clone()]}),
        ] {
            assert!(template_registration(&types, &report, selector).is_err());
        }
        let mut wrong = row.clone();
        wrong["template_type"]["size"] = json!(16);
        assert!(
            template_registration(
                &types,
                &json!({"available":true,"entries":[wrong]}),
                selector
            )
            .is_err()
        );
        let mut missing = row;
        missing["template_type"] = Value::Null;
        assert!(
            template_registration(
                &types,
                &json!({"available":true,"entries":[missing]}),
                selector
            )
            .is_err()
        );
    }
}
