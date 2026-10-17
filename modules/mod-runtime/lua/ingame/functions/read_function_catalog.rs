//! Build-scoped provisional bindings for every evidenced native entry point.
use crate::runtime_resolution::functions::Resolver;
use crate::{
    lua::convert_json::json_to_lua,
    lua::ingame::{
        prove_buffer_transform::{self as buffer_transform, Plan},
        runtime_provider,
    },
    session::AppState,
};
use mlua::{Lua, Table, Value as LuaValue};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::Rc,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct Catalog {
    pub(super) base: Option<&'static Value>,
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
    pub(super) fn refresh(&mut self) -> Result<(), String> {
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

    pub(super) fn describe(&mut self, rva: u32, analyze: bool) -> Value {
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

pub(super) fn descriptor_to_lua(lua: &Lua, descriptor: &Value) -> mlua::Result<Table> {
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
    super::bind_profile_modifier::attach(lua, table, patch, catalog.clone())?;
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
#[path = "function_catalog_tests.rs"]
mod tests;
