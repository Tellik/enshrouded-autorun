//! Build-scoped provisional bindings for every evidenced native entry point.
use super::{
    AppState,
    buffer_transform::{self, Plan},
    loader::runtime_provider,
    registry::json_to_lua,
};
use crate::runtime_resolution::functions::Resolver;
use mlua::{Lua, Table, Value as LuaValue};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Default)]
struct Catalog {
    base: Option<&'static Value>,
    indices: BTreeMap<u32, usize>,
    owners: BTreeMap<u32, Vec<Value>>,
    plans: BTreeMap<u32, Option<Plan>>,
    registry: Value,
    last_refresh: Option<Instant>,
    registry_reason: Option<String>,
    resolution: Option<Resolver>,
    dependency_coverage: Value,
}

impl Catalog {
    fn refresh(&mut self) -> Result<(), String> {
        if self.base.is_none() {
            let base = runtime_provider::functions()?;
            let entries = base["entries"]
                .as_array()
                .ok_or("invalid function inventory")?;
            for (i, entry) in entries.iter().enumerate() {
                let rva = entry["rva"]
                    .as_u64()
                    .and_then(|v| u32::try_from(v).ok())
                    .ok_or("invalid function RVA")?;
                if self.indices.insert(rva, i).is_some() {
                    return Err("duplicate function RVA".into());
                }
            }
            self.base = Some(base);
            if let Some(resolution) = &mut self.resolution {
                resolution.set_inventory(base);
            }
            self.update_coverage();
        }
        if self
            .last_refresh
            .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
        {
            return Ok(());
        }
        self.last_refresh = Some(Instant::now());
        match runtime_provider::registry() {
            Ok(registry) if registry["available"] == true => {
                self.registry_reason = None;
                if registry != self.registry {
                    self.set_registry(registry);
                }
            }
            other => {
                self.registry_reason = Some(match other {
                    Err(reason) => reason,
                    _ => "live component registration is not ready".into(),
                });
                let unavailable = json!({"available":false,"reason":self.registry_reason});
                if self.registry != unavailable {
                    self.set_registry(unavailable);
                }
            }
        }
        Ok(())
    }

    fn set_registry(&mut self, registry: Value) {
        self.owners.clear();
        self.plans.clear();
        if let Some(entries) = registry["entries"].as_array() {
            for entry in entries {
                if let Some(callbacks) = entry["callbacks"].as_array() {
                    for callback in callbacks {
                        let Some(rva) = callback["function_rva"]
                            .as_u64()
                            .and_then(|v| u32::try_from(v).ok())
                        else {
                            continue;
                        };
                        self.owners.entry(rva).or_default().push(json!({
                            "qualified_name":entry["qualified_name"], "qualified_hash":entry["qualified_hash"],
                            "registration_index":entry["index"], "runtime_type":entry["runtime_type"],
                            "template_type":entry["template_type"], "origin":callback["origin"], "slot_offset":callback["slot_offset"]
                        }));
                        let plan = callback["code_hex"]
                            .as_str()
                            .and_then(buffer_transform::unhex)
                            .and_then(|code| buffer_transform::prove(&code));
                        self.plans.insert(rva, plan);
                    }
                }
            }
        }
        if let Some(resolution) = &mut self.resolution {
            resolution.set_registry(&registry);
        }
        self.registry = registry;
        self.update_coverage();
    }

    fn update_coverage(&mut self) {
        if let (Some(resolution), Some(base)) = (&self.resolution, self.base) {
            self.dependency_coverage = resolution.coverage(base["entries"].as_array().unwrap());
        }
    }

    fn keys(&self) -> Vec<u32> {
        self.indices
            .keys()
            .chain(self.owners.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn selector(&self, selector: LuaValue) -> Result<u32, String> {
        let rva = match selector {
            LuaValue::Integer(value) => {
                u32::try_from(value).map_err(|_| "RVA outside u32".to_owned())?
            }
            LuaValue::String(value) => {
                let text = value.to_str().map_err(|e| e.to_string())?;
                let text = if let Some((sha, name)) = text.split_once('/') {
                    if Some(sha) != self.base.unwrap()["image"]["sha256"].as_str() {
                        return Err("binding belongs to a different executable".into());
                    }
                    name
                } else {
                    text.as_ref()
                };
                if let Some(rva) = text.strip_prefix("unclear_") {
                    if rva.len() != 8 {
                        return Err("expected unclear_<eight hexadecimal RVA digits>".into());
                    }
                    u32::from_str_radix(rva, 16)
                        .map_err(|_| "invalid provisional binding name".to_owned())?
                } else if let Some(rva) = text.strip_prefix("native:") {
                    rva.parse::<u32>()
                        .map_err(|_| "invalid native binding ID".to_owned())?
                } else {
                    let matches: Vec<u32> = self
                        .indices
                        .iter()
                        .filter_map(|(rva, index)| {
                            (self.base.unwrap()["entries"][*index]["engine_descriptors"]
                                .as_array()
                                .is_some_and(|descriptors| {
                                    descriptors.iter().any(|v| v["name"] == text)
                                })
                                || self.base.unwrap()["entries"][*index]["operations"]
                                    .as_array()
                                    .is_some_and(|operations| {
                                        operations.iter().any(|v| v["id"] == text)
                                    }))
                            .then_some(*rva)
                        })
                        .collect();
                    match matches.as_slice() {
                        [rva] => *rva,
                        [] => return Err(format!("engine function name not found: {text}")),
                        _ => {
                            return Err(format!(
                                "Ambiguous engine function name: {text}. Select a build-scoped RVA"
                            ));
                        }
                    }
                }
            }
            _ => return Err("expected binding name or integer RVA".into()),
        };
        if !self.indices.contains_key(&rva) && !self.owners.contains_key(&rva) {
            return Err("no evidenced entry point at this RVA".into());
        }
        Ok(rva)
    }

    fn describe(&mut self, rva: u32, analyze: bool) -> Value {
        if analyze && !self.plans.contains_key(&rva) {
            self.plans.insert(
                rva,
                runtime_provider::function_code(rva)
                    .and_then(|code| buffer_transform::prove(&code)),
            );
        }
        let mut entry = self.indices.get(&rva).map(|i| self.base.unwrap()["entries"][*i].clone())
            .unwrap_or_else(|| json!({"id":format!("native:{rva}"),"rva":rva,"ranges":[],"code_bytes":null,"chain_resolved":null}));
        if let Some(resolution) = &self.resolution {
            resolution.enrich(&mut entry);
        }
        let names: std::collections::BTreeSet<_> = entry["engine_descriptors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v["name"].as_str().map(str::to_owned))
            .collect();
        let name = if names.len() == 1 {
            names.first().unwrap().clone()
        } else {
            format!("unclear_{rva:08x}")
        };
        entry["name_provisional"] = json!(names.len() != 1);
        entry["name"] = json!(name);
        entry["key"] = json!(format!(
            "{}/{}",
            self.base.unwrap()["image"]["sha256"]
                .as_str()
                .unwrap_or("unknown-image"),
            name
        ));
        entry["owners"] = json!(self.owners.get(&rva).cloned().unwrap_or_default());
        entry["provisional"] = json!(true);
        entry["native_callable"] = json!(false);
        entry["signature"] = json!({"native_arguments":null,"native_return":null,"calling_convention":"windows-x64","complete":false});
        entry["validation"] = json!({"address":if self.owners.contains_key(&rva) { "live-engine-registration" } else { entry["address_evidence"].as_str().unwrap_or("unwind-entry") },
            "type_ownership":self.owners.contains_key(&rva),"native_signature":false,"engine_context":false,
            "gameplay_effects":false,"buffer_adapter_checked":self.plans.contains_key(&rva)});
        if let Some(Some(plan)) = self.plans.get(&rva) {
            entry["callable"] = json!(true);
            entry["execution"] = json!("owned-buffer-copy-interpreter");
            entry["buffer_transform"] = serde_json::to_value(plan).unwrap();
            entry["signature"]["adapter_arguments"] = json!(["source_bytes", "destination_bytes"]);
            entry["signature"]["adapter_return"] =
                json!("destination_bytes, reason, native_constant_result");
            entry["validation"]["owned_buffer_effects"] = json!(true);
            entry["reason"] = Value::Null;
        } else {
            entry["callable"] = json!(false);
            entry["execution"] = Value::Null;
            entry["reason"] = json!(
                "Native arguments, engine context and effects are unresolved. No verified adapter"
            );
        }
        entry
    }

    fn page(
        &mut self,
        lua: &Lua,
        keys: &[u32],
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> mlua::Result<Table> {
        let offset = offset.unwrap_or(0).min(keys.len());
        let end = offset
            .saturating_add(limit.unwrap_or(256).clamp(1, 4096))
            .min(keys.len());
        let page = json!({"schema_version":1,"image":self.base.unwrap()["image"],
            "count":keys.len(),"inventory_count":self.keys().len(),"unwind_count":self.base.unwrap()["unwind_count"],
            "registered_callback_count":self.owners.len(),"dependency_resolution":self.dependency_coverage,
            "system_dependency_layout":self.base.unwrap()["system_dependency_layout"],
            "pointer_target_count":self.base.unwrap()["pointer_target_count"],"unreadable_pointer_pages":self.base.unwrap()["unreadable_pointer_pages"],
            "range_count":self.base.unwrap()["range_count"],"source":"unwind-code-pointers-and-live-registration",
            "complete_engine_api":false,"registry_reason":self.registry_reason,
            "limitations":["unreferenced leaf functions and inlined code can be absent", "code pointers can target internal labels. A function boundary is not implied", "provisional names are scoped to the executable SHA256", "native ABI and gameplay effects remain unresolved unless explicitly validated"],
            "offset":offset,"next_offset":(end < keys.len()).then_some(end)});
        let LuaValue::Table(page) = json_to_lua(lua, &page)? else {
            unreachable!()
        };
        let entries = lua.create_table_with_capacity(end - offset, 0)?;
        for rva in &keys[offset..end] {
            entries.push(descriptor_to_lua(lua, &self.describe(*rva, false))?)?;
        }
        page.raw_set("entries", entries)?;
        Ok(page)
    }
}

fn descriptor_to_lua(lua: &Lua, descriptor: &Value) -> mlua::Result<Table> {
    let LuaValue::Table(table) = json_to_lua(lua, descriptor)? else {
        unreachable!()
    };
    let Some(descriptors) = descriptor["engine_descriptors"].as_array() else {
        return Ok(table);
    };
    if !descriptors.iter().any(|descriptor| {
        descriptor["dependency_slots"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|slot| slot["resolved_entries"].is_array())
    }) {
        return Ok(table);
    }
    let state = lua
        .app_data_ref::<AppState>()
        .ok_or_else(|| mlua::Error::runtime("parser registry is unavailable"))?;
    let lua_descriptors = table.raw_get::<Table>("engine_descriptors")?;
    let attach_type = |table: &Table, key: &str, identity: &Value| -> mlua::Result<()> {
        if let Some(index) = identity["index"]
            .as_u64()
            .and_then(|index| usize::try_from(index).ok())
        {
            table.raw_set(
                key,
                state.get_type(lua, kfc::reflection::TypeIndex::new(index))?,
            )?;
        }
        Ok(())
    };
    for (descriptor_index, descriptor) in descriptors.iter().enumerate() {
        let lua_descriptor = lua_descriptors.raw_get::<Table>(descriptor_index + 1)?;
        let Some(slots) = descriptor["dependency_slots"].as_array() else {
            continue;
        };
        let lua_slots = lua_descriptor.raw_get::<Table>("dependency_slots")?;
        for (slot_index, slot) in slots.iter().enumerate() {
            let Some(dependencies) = slot["resolved_entries"].as_array() else {
                continue;
            };
            let lua_dependencies = lua_slots
                .raw_get::<Table>(slot_index + 1)?
                .raw_get::<Table>("resolved_entries")?;
            for (dependency_index, dependency) in dependencies.iter().enumerate() {
                let lua_dependency = lua_dependencies.raw_get::<Table>(dependency_index + 1)?;
                attach_type(&lua_dependency, "type", &dependency["type"])?;
                let lua_registrations = lua_dependency.raw_get::<Table>("registrations")?;
                for (registration_index, registration) in dependency["registrations"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    let lua_registration =
                        lua_registrations.raw_get::<Table>(registration_index + 1)?;
                    for key in ["runtime_type", "template_type"] {
                        attach_type(&lua_registration, key, &registration[key])?;
                    }
                }
            }
        }
    }
    Ok(table)
}

fn binding(lua: &Lua, descriptor: &Value, plan: Option<Plan>) -> mlua::Result<Table> {
    let table = descriptor_to_lua(lua, descriptor)?;
    let reason = descriptor["reason"]
        .as_str()
        .unwrap_or("no verified adapter")
        .to_owned();
    table.raw_set(
        "call",
        lua.create_function(move |lua, args: mlua::MultiValue| {
            let Some(plan) = &plan else {
                return Ok((None, Some(reason.clone()), None));
            };
            if args.len() != 2 {
                return Ok((
                    None,
                    Some("call expects source_bytes and destination_bytes".into()),
                    None,
                ));
            }
            let (Some(LuaValue::String(source)), Some(LuaValue::String(destination))) =
                (args.front(), args.get(1))
            else {
                return Ok((
                    None,
                    Some("call expects two owned Lua byte strings".into()),
                    None,
                ));
            };
            match plan.apply(source.as_bytes().as_ref(), destination.as_bytes().as_ref()) {
                Ok(bytes) => Ok((
                    Some(lua.create_string(bytes)?),
                    None,
                    plan.native_constant_result,
                )),
                Err(reason) => Ok((None, Some(reason), None)),
            }
        })?,
    )?;
    Ok(table)
}

pub(crate) fn attach(lua: &Lua, table: &Table, patch: &Table) -> mlua::Result<()> {
    let registry = lua
        .app_data_ref::<AppState>()
        .unwrap()
        .type_registry()
        .clone();
    let catalog = Rc::new(RefCell::new(Catalog {
        resolution: Some(Resolver::new(registry)),
        ..Default::default()
    }));
    let list_catalog = catalog.clone();
    let list = lua.create_function(
        move |lua, (offset, limit): (Option<usize>, Option<usize>)| {
            let mut catalog = list_catalog.borrow_mut();
            if let Err(reason) = catalog.refresh() {
                return Ok((None, Some(reason)));
            }
            let keys = catalog.keys();
            Ok((Some(catalog.page(lua, &keys, offset, limit)?), None))
        },
    )?;
    table.raw_set("list", list.clone())?;
    table.raw_set("list_native", list)?;
    let find_catalog = catalog.clone();
    table.raw_set(
        "find",
        lua.create_function(
            move |lua, (options, offset, limit): (Option<Table>, Option<usize>, Option<usize>)| {
                let name = options
                    .as_ref()
                    .map(|options| options.get::<Option<String>>("name"))
                    .transpose()?
                    .flatten();
                let component = options
                    .as_ref()
                    .map(|options| options.get::<LuaValue>("component"))
                    .transpose()?
                    .unwrap_or(LuaValue::Nil);
                let component = if matches!(component, LuaValue::Nil) {
                    None
                } else {
                    let state = lua.app_data_ref::<AppState>().unwrap();
                    match super::registry::resolve(state.type_registry(), &component) {
                        Ok(index) => Some(
                            state
                                .type_registry()
                                .get(index)
                                .unwrap()
                                .qualified_name
                                .clone(),
                        ),
                        Err(reason) => return Ok((None, Some(reason))),
                    }
                };
                let mut catalog = find_catalog.borrow_mut();
                if let Err(reason) = catalog.refresh() {
                    return Ok((None, Some(reason)));
                }
                let keys = catalog
                    .keys()
                    .into_iter()
                    .filter(|rva| {
                        let entry = catalog
                            .indices
                            .get(rva)
                            .map(|index| &catalog.base.unwrap()["entries"][*index]);
                        let name_matches = name.as_ref().is_none_or(|name| {
                            format!("unclear_{rva:08x}").contains(name)
                                || entry.is_some_and(|entry| {
                                    entry["engine_descriptors"]
                                        .as_array()
                                        .into_iter()
                                        .flatten()
                                        .any(|descriptor| {
                                            descriptor["name"]
                                                .as_str()
                                                .is_some_and(|actual| actual.contains(name))
                                        })
                                        || entry["operations"].as_array().into_iter().flatten().any(
                                            |operation| {
                                                operation["id"]
                                                    .as_str()
                                                    .is_some_and(|actual| actual.contains(name))
                                            },
                                        )
                                })
                        });
                        name_matches
                            && component.as_ref().is_none_or(|component| {
                                catalog.owners.get(rva).is_some_and(|owners| {
                                    owners.iter().any(|owner| {
                                        owner["qualified_name"] == *component
                                            || owner["runtime_type"]["qualified_name"] == *component
                                            || owner["template_type"]["qualified_name"]
                                                == *component
                                    })
                                }) || entry.is_some_and(|entry| {
                                    catalog.resolution.as_ref().is_some_and(|resolution| {
                                        resolution.matches_component(entry, component)
                                    })
                                })
                            })
                    })
                    .collect::<Vec<_>>();
                Ok((Some(catalog.page(lua, &keys, offset, limit)?), None))
            },
        )?,
    )?;
    let modifiers_catalog = catalog.clone();
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
                let state = lua.app_data_ref::<super::AppState>().unwrap();
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
    let get = lua.create_function(move |lua, selector: LuaValue| {
        let mut catalog = catalog.borrow_mut();
        if let Err(reason) = catalog.refresh() {
            return Ok((None, Some(reason)));
        }
        let rva = match catalog.selector(selector) {
            Ok(rva) => rva,
            Err(reason) => return Ok((None, Some(reason))),
        };
        let descriptor = catalog.describe(rva, true);
        let plan = catalog.plans.get(&rva).cloned().flatten();
        Ok((Some(binding(lua, &descriptor, plan)?), None))
    })?;
    table.raw_set("get", get.clone())?;
    table.raw_set("get_native", get)?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
    fn bundled_mods_bind_profile_modifiers_and_restore_on_unload() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        for target in ["client", "server"] {
            let directory = root.join(format!("src/loader/runtime/profiles/enshrouded/{target}"));
            let profile = std::fs::read_dir(directory)
                .unwrap()
                .find_map(|entry| {
                    let path = entry.ok()?.path();
                    if path.extension()?.to_str()? != "json" {
                        return None;
                    }
                    let profile = serde_json::from_slice::<Value>(&std::fs::read(path).unwrap()).unwrap();
                    profile["runtimePatches"].is_object().then_some(profile)
                })
                .unwrap();
            for patch in profile["runtimePatches"].as_object().unwrap().values() {
                // These mods now use typed ECS or resource APIs. Legacy
                // profile entries remain available to older callers.
                if matches!(patch["modifier"]["id"].as_str(), Some("refill_stamina" | "preserve_health_on_fall" | "override_rotation_constant")) {
                    continue;
                }
                let lua = Lua::new();
                lua.globals()
                    .set(
                        "expected_modifier",
                        patch["modifier"]["id"].as_str().unwrap(),
                    )
                    .unwrap();
                lua.load(r#"
                    calls = {}; state = false
                    runtime = {require=function() end, report_effect=function() end,
                        functions={bind_modifier=function(id)
                            assert(id == expected_modifier)
                            return {available=function() return true end,
                                set_enabled=function(value) state=value; calls[#calls+1]=value; return true end}
                        end}}
                    shroudforge={settings={get=function() return true end},
                        log={error=function(message) error(message) end,info=function() end},
                        ui={on_action=function() end}}
                "#).exec().unwrap();
                let alias = profile["runtimePatches"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .find(|(_, v)| *v == patch)
                    .unwrap()
                    .0;
                let slug = format!(
                    "sf-{}",
                    alias
                        .strip_prefix("runtime.patch.")
                        .unwrap()
                        .replace('_', "-")
                );
                let package_root = if slug == "sf-unlimited-flight" {
                    "mods"
                } else {
                    "reference-mods-0.9"
                };
                let source = std::fs::read_to_string(
                    root.join(format!("{package_root}/{slug}/src/mod.lua")),
                )
                .unwrap();
                let module: Table = lua.load(source).eval().unwrap();
                module
                    .get::<mlua::Function>("on_load")
                    .unwrap()
                    .call::<()>(())
                    .unwrap();
                assert!(
                    lua.globals().get::<bool>("state").unwrap(),
                    "{target}/{slug}"
                );
                if let Some(update) = module.get::<Option<mlua::Function>>("on_update").unwrap() {
                    update.call::<()>(()).unwrap();
                }
                module
                    .get::<mlua::Function>("on_unload")
                    .unwrap()
                    .call::<()>(())
                    .unwrap();
                assert!(
                    !lua.globals().get::<bool>("state").unwrap(),
                    "{target}/{slug}"
                );
                assert!(lua.globals().get::<Table>("calls").unwrap().len().unwrap() >= 2);
            }
        }
    }

    #[test]
    fn stamina_mod_uses_typed_roots_and_retries_without_a_modifier_patch() {
        let lua = Lua::new();
        lua.load(r#"
            reads_ready=false; writes_ready=false; setting=true
            queries=0; writes=0; current=76; maximum=113; busy=true
            effects={}
            shroudforge={settings={get=function(name) assert(name=='autoRefill'); return setting end}}
            runtime={
                has=function(name)
                    if name=='runtime.ecs.read' then return reads_ready end
                    if name=='runtime.ecs.write' then return writes_ready end
                    error('unexpected feature '..name)
                end,
                require=function() error('startup must wait for the world') end,
                functions={bind_modifier=function() error('native modifier must not be used') end},
                report_effect=function(state,detail) effects[#effects+1]={state=state,detail=detail} end,
                ecs={
                    query=function(a,b)
                        assert(a=='keen::ecs::PlayerInput' and b=='keen::ecs::Stamina')
                        queries=queries+1
                        if queries==1 then return nil,'Live ECS query is still scanning' end
                        return {77}
                    end,
                    read_attributes=function(entity,name)
                        assert(entity==77 and name=='Stamina')
                        return {values={Stamina=current,Stamina_Max=maximum}}
                    end,
                    update_attribute=function(entity,name,value)
                        assert(entity==77 and name=='Stamina' and value==maximum)
                        writes=writes+1
                        if busy then busy=false; return nil,'ecs-scene-frame-busy' end
                        current=value
                        return {applied=true,values={Stamina=current,Stamina_Max=maximum}}
                    end
                }
            }
        "#).exec().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../reference-mods-0.9/sf-auto-stamina-refill/src/mod.lua");
        let module: Table = lua.load(std::fs::read_to_string(path).unwrap()).eval().unwrap();
        module.get::<mlua::Function>("on_load").unwrap().call::<()>(()).unwrap();
        let update = module.get::<mlua::Function>("on_update").unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==0 and writes==0); reads_ready=true").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==0 and writes==0); writes_ready=true").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==1 and writes==0)").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(writes==1 and current==76 and effects[#effects].state=='waiting')").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(writes==2 and current==113 and effects[#effects].state=='write-confirmed')").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(writes==2 and effects[#effects].state=='no-change'); setting=false; current=70").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(writes==2 and current==70)").exec().unwrap();
        module.get::<mlua::Function>("on_unload").unwrap().call::<()>(()).unwrap();
        lua.load("assert(writes==2)").exec().unwrap();
    }

    #[test]
    fn no_fall_mod_uses_actual_template_scalars_and_scoped_restoration() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../reference-mods-0.9/sf-no-fall-damage/src/mod.lua");
        let source = std::fs::read_to_string(path).unwrap();
        assert!(!source.contains("bind_modifier") && !source.contains("runtime.gameplay.patch"));
        // Exercise the production mapper's Variant.type contract rather than
        // supplying a hand-written Type-shaped fixture again.
        let mut types = Vec::new();
        for (index, (name, primitive, size, inner, count)) in [
            ("Float", "FLOAT32", 4, None, 0),
            ("Component", "STRUCT", 0, None, 0),
            ("FallDamage", "STRUCT", 8, Some(1), 2),
            ("Variant", "BLOB_VARIANT", 12, Some(1), 1),
        ]
        .into_iter()
        .enumerate()
        {
            let qualified = format!("keen::ecs::{name}");
            types.push(json!({"index":index,"name":name,"qualifiedName":qualified,
                "impactName":name,"size":size,"alignment":4,"elementAlignment":4,
                "fieldCount":count,"primitiveType":primitive,"innerType":inner,"flags":"",
                "nameHash":kfc::hash::fnv(name),"qualifiedHash":kfc::hash::fnv(&qualified),
                "impactHash":kfc::hash::fnv(name),"internalHash":index}));
        }
        types[2]["structFields"] = json!({
            "fallDamageStartDistance":{"name":"fallDamageStartDistance","type":0,"dataOffset":0},
            "fallDamageLethalDistance":{"name":"fallDamageLethalDistance","type":0,"dataOffset":4}});
        let registry = Rc::new(serde_json::from_value::<kfc::reflection::TypeRegistry>(
            json!({"version":"no-fall-variant-contract","types":types}),
        ).unwrap());
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&kfc::hash::fnv("keen::ecs::FallDamage").to_le_bytes());
        bytes.extend_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(&8u32.to_le_bytes());
        bytes.extend_from_slice(&8f32.to_le_bytes());
        bytes.extend_from_slice(&16f32.to_le_bytes());
        let owned: Rc<[u8]> = bytes.into();
        let mapped_component = crate::alias::MappedValue::from_bytes(
            &registry,
            registry.get(kfc::reflection::TypeIndex::new(3)).unwrap(),
            &owned,
        ).unwrap();
        let fixture = r#"
            assert(type(mapped_component.type)=='string' and mapped_component.type=='keen::ecs::FallDamage')
            assert(mapped_component.value.fallDamageStartDistance==8 and mapped_component.value.fallDamageLethalDistance==16)
            session=0; reads_ready=false; writes_ready=false; query_pending=false
            queries=0; writes={}; reads=0; start=8; lethal=16; mode='ok'; effects={}
            local template_type={qualified_name='keen::ecs::TemplateResource'}
            function snapshot()
                return {guid='fixture-template',type=template_type,part=7,loaded=true,
                    source='loaded-runtime',data={components={
                        {type='keen::ecs::Other',value={}},
                        {type=mapped_component.type,value={
                            fallDamageStartDistance=start,fallDamageLethalDistance=lethal}}}}}
            end
            runtime={
                has=function(name)
                    if name=='runtime.ecs.read' then return reads_ready end
                    if name=='runtime.assets.write' then return writes_ready end
                    error('unexpected feature '..name)
                end,
                functions={bind_modifier=function() error('no modifier allowed') end},
                require=function() error('startup must wait') end,
                report_effect=function(state,detail) effects[#effects+1]={state=state,detail=detail} end,
                world={session_id=function() return session end},
                ecs={query=function(a,b)
                    assert(a=='keen::ecs::PlayerInput' and b=='keen::ecs::DynamicFallDamage')
                    queries=queries+1
                    if query_pending then query_pending=false; return nil,'query pending' end
                    return {101,102} -- Both players really use the same template.
                end,get_template=function(entity) assert(entity==101 or entity==102); return snapshot() end}}
            game={assets={
                get_loaded_resource=function(guid,ty,part)
                    assert(guid=='fixture-template' and ty.qualified_name==template_type.qualified_name and part==7)
                    reads=reads+1
                    if mode=='read-failure' then error('resource temporarily unavailable') end
                    return snapshot()
                end,
                write_loaded_field=function(resource,path,value,expected)
                    assert(resource.guid=='fixture-template' and path=='components[2].value.fallDamageStartDistance')
                    writes[#writes+1]={value=value,expected=expected}
                    if mode=='busy' then mode='ok'; return nil,'resource gate busy' end
                    assert(start==expected)
                    if mode~='false-success' then start=value end
                    if mode=='uncertain' then return nil,'Resource write outcome is uncertain. Do not retry blindly' end
                    return snapshot()
                end}}
            shroudforge={ui={on_action=function(id,callback)
                assert(id=='toggleFallDamagePatch'); toggle=callback
            end}}
        "#;
        let load = || {
            let lua = Lua::new();
            lua.globals().set("mapped_component",
                crate::env::game::value::convert_value_to_lua(&mapped_component, &lua).unwrap(),
            ).unwrap();
            lua.load(fixture).exec().unwrap();
            let module: Table = lua.load(&source).eval().unwrap();
            module.get::<mlua::Function>("on_load").unwrap().call::<()>(()).unwrap();
            (lua, module)
        };
        let (lua, module) = load();
        let update = module.get::<mlua::Function>("on_update").unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==0 and #writes==0); session=11; reads_ready=true").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==0 and #writes==0); writes_ready=true; query_pending=true; mode='busy'").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(queries==1 and #writes==0)").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(#writes==1 and start==8 and effects[#effects].state=='waiting')").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(#writes==2 and start==16 and reads>=4 and effects[#effects].state=='write-confirmed')").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(#writes==2); writes_ready=false; toggle(); assert(start==16 and #writes==2)").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(#writes==2); writes_ready=true").exec().unwrap();
        update.call::<()>(()).unwrap();
        lua.load("assert(#writes==3 and start==8 and writes[3].expected==16); toggle(); assert(#writes==4 and start==16)").exec().unwrap();
        module.get::<mlua::Function>("on_unload").unwrap().call::<()>(()).unwrap();
        lua.load("assert(#writes==5 and start==8 and effects[#effects].state=='write-confirmed')").exec().unwrap();

        // An uncertain return, false success, foreign change or session switch
        // must never be retried, restored blindly or bypassed with a UI toggle.
        for scenario in ["uncertain", "false-success", "foreign-change", "session-change"] {
            let (lua, module) = load();
            lua.load("session=11; reads_ready=true; writes_ready=true").exec().unwrap();
            if matches!(scenario, "uncertain" | "false-success") {
                lua.globals().set("mode", scenario).unwrap();
            }
            let update = module.get::<mlua::Function>("on_update").unwrap();
            update.call::<()>(()).unwrap();
            match scenario {
                "foreign-change" => lua.load("start=12").exec().unwrap(),
                "session-change" => lua.load("session=12").exec().unwrap(),
                _ => (),
            }
            update.call::<()>(()).unwrap();
            lua.load("toggle(); toggle(); assert(#writes==1 and effects[#effects].state=='write-failed')").exec().unwrap();
            module.get::<mlua::Function>("on_unload").unwrap().call::<()>(()).unwrap();
            lua.load("assert(#writes==1)").exec().unwrap();
            if scenario == "foreign-change" { lua.load("assert(start==12)").exec().unwrap(); }
        }
        let (lua, module) = load();
        lua.load("session=11; reads_ready=true; writes_ready=true; start=20").exec().unwrap();
        module.get::<mlua::Function>("on_update").unwrap().call::<()>(()).unwrap();
        module.get::<mlua::Function>("on_unload").unwrap().call::<()>(()).unwrap();
        lua.load("assert(#writes==0 and start==20)").exec().unwrap();
    }

    #[test]
    fn blueprint_mod_resolves_original_query_action_instead_of_fixed_id() {
        let lua = Lua::new();
        lua.load(r#"
            loader={features={patch=true}}
            shroudforge={log={info=function() end}}
            recipes={data={recipes={{knowledgeRequirement={knowledgeOrQueryId={value=999}}}}}}
            game={types={get=function(name) return name end},assets={get_resources_by_type=function(name)
                if name=='keen::GameKnowledgeQueryResourceDb' then
                    return {{data={queries={{name='Unlock_Flame_Altar_PK',actions={{name='NPC_Flame_Hint01',query={knowledgeOrQueryId={value=123456}}}}}}}}}
                end
                return {recipes}
            end}}
        "#).exec().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../reference-mods-0.9/sf-unlock-blueprints/src/mod.lua");
        lua.load(std::fs::read_to_string(path).unwrap())
            .exec()
            .unwrap();
        lua.load("assert(recipes.data.recipes[1].knowledgeRequirement.knowledgeOrQueryId.value == 123456)").exec().unwrap();
    }
    #[test]
    #[ignore = "set SHROUDFORGE_TEST_FUNCTIONS to a production-reader verification JSON"]
    fn production_callback_binding_corpus() {
        let path = std::env::var("SHROUDFORGE_TEST_FUNCTIONS").unwrap();
        let registry: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let base: Value =
            serde_json::from_slice(&std::fs::read(format!("{path}.functions.json")).unwrap())
                .unwrap();
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
}
