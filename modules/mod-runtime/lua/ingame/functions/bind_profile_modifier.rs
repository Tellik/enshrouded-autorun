use super::*;

pub(super) fn attach(
    lua: &Lua,
    table: &Table,
    patch: &Table,
    catalog: Rc<RefCell<Catalog>>,
) -> mlua::Result<()> {
    let modifiers_catalog = catalog;
    let checked_available = patch.raw_get::<mlua::Function>("available")?;
    let checked_set = patch.raw_get::<mlua::Function>("set_enabled")?;
    table.raw_set(
        "bind_modifier",
        lua.create_function(move |lua, id: String| {
            let mut catalog = modifiers_catalog.borrow_mut();
            if let Err(reason) = catalog.refresh() {
                return Ok((None, Some(reason)));
            }
            let matches: Vec<_> = catalog.base.unwrap()["entries"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|entry| {
                    entry["modifiers"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|m| m["id"] == id)
                        .map(|m| (entry["rva"].as_u64().unwrap() as u32, m.clone()))
                })
                .collect();
            let [(rva, modifier)] = matches.as_slice() else {
                let detail = if matches.is_empty() {
                    "no exact-build guarded profile patch has a verified owning function"
                } else {
                    "multiple guarded profile patches claim this modifier ID"
                };
                return Ok((
                    None,
                    Some(format!(
                        "modifier {id} is unavailable: {detail}. This binding requires a guarded profile patch"
                    )),
                ));
            };
            if let Some(requirements) = modifier["attribute_requirements"].as_array() {
                let state = lua.app_data_ref::<crate::env::AppState>().unwrap();
                let attributes = match state.attribute_catalog() {
                    Ok(c) => c,
                    Err(reason) => return Ok((None, Some(reason))),
                };
                for required in requirements {
                    let selector = LuaValue::Integer(required["hash"].as_i64().unwrap());
                    let valid = attributes.get(&selector).is_ok_and(|actual| {
                        ["name", "index", "root_hash"]
                            .iter()
                            .all(|key| actual[*key] == required[*key])
                            && actual["storage_writable"] == true
                    });
                    if !valid {
                        return Ok((
                            None,
                            Some(format!(
                                "KFC attribute definition no longer proves modifier {id}: {}",
                                required["name"]
                            )),
                        ));
                    }
                }
            }
            let operation = modifier["backend_operation"].as_str().unwrap().to_owned();
            let descriptor = catalog.describe(*rva, false);
            let LuaValue::Table(bound) = json_to_lua(lua, modifier)? else {
                unreachable!()
            };
            bound.raw_set("backend_kind", "guarded-profile-patch")?;
            bound.raw_set("execution", "guarded-profile-patch")?;
            bound.raw_set("profile_required", true)?;
            bound.raw_set("native_callable", false)?;
            bound.raw_set("owner", descriptor_to_lua(lua, &descriptor)?)?;
            let available = checked_available.clone();
            let available_operation = operation.clone();
            bound.raw_set(
                "available",
                lua.create_function(move |_, ()| {
                    available.call::<bool>(available_operation.clone())
                })?,
            )?;
            let set = checked_set.clone();
            bound.raw_set(
                "set_enabled",
                lua.create_function(move |_, enabled: bool| {
                    set.call::<(bool, Option<String>)>((operation.clone(), enabled))
                })?,
            )?;
            Ok((Some(bound), None))
        })?,
    )?;
    Ok(())
}
