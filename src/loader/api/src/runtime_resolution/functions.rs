//! Join original engine system dependencies with parser types and live registrations.
//! Scheduler evidence can establish access roles for one exact executable image.
//! Dependencies and access roles never establish native arguments or execution permission.
use kfc::reflection::{TypeIndex, TypeMetadata, TypeRegistry};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

pub(crate) struct Resolver {
    registry: Rc<TypeRegistry>,
    types: BTreeMap<String, TypeIndex>,
    registrations: BTreeMap<String, Vec<Value>>,
    registry_reason: Option<String>,
    dependency_layout: BTreeMap<u64, Value>,
    dependency_layout_source: Option<String>,
    dependency_layout_reason: Option<String>,
}

impl Resolver {
    pub fn new(registry: Rc<TypeRegistry>) -> Self {
        let types = registry
            .iter()
            .map(|ty| (ty.qualified_name.clone(), ty.index))
            .collect();
        Self {
            registry,
            types,
            registrations: BTreeMap::new(),
            registry_reason: Some("live component registration is not ready".into()),
            dependency_layout: BTreeMap::new(),
            dependency_layout_source: None,
            dependency_layout_reason: Some("no exact-image scheduler dependency evidence".into()),
        }
    }

    pub fn set_inventory(&mut self, inventory: &Value) {
        self.dependency_layout.clear();
        self.dependency_layout_source = None;
        let checked = (|| -> Result<(BTreeMap<u64, Value>, String), &'static str> {
            let model = &inventory["system_dependency_layout"];
            let image = &inventory["image"];
            let hash = image["sha256"].as_str().unwrap_or_default();
            if hash.len() != 64
                || !hash.bytes().all(|c| c.is_ascii_hexdigit())
                || !["target", "sha256", "timestamp", "size"]
                    .into_iter()
                    .all(|key| !image[key].is_null() && model["image"][key] == image[key])
                || model["validated"] != true
                || model["id"] != "ecs-system-dependencies-v1"
            {
                return Err("no exact-image scheduler dependency evidence");
            }
            let source = model["source"]
                .as_str()
                .filter(|source| !source.is_empty())
                .ok_or("scheduler dependency evidence has no source")?;
            let slots = model["slots"]
                .as_array()
                .filter(|slots| slots.len() == 6)
                .ok_or("scheduler dependency evidence must describe six slots")?;
            let mut result = BTreeMap::new();
            for slot in slots {
                let offset = slot["offset"]
                    .as_u64()
                    .ok_or("invalid scheduler slot offset")?;
                if !(24..=104).contains(&offset) || (offset - 24) % 16 != 0 {
                    return Err("invalid scheduler slot offset");
                }
                match (slot["storage_role"].as_str(), slot["access_mode"].as_str()) {
                    (Some("template"), Some("configuration-read"))
                    | (Some("runtime"), Some("read" | "write")) => {}
                    _ => return Err("invalid scheduler slot access role"),
                }
                if result.insert(offset, slot.clone()).is_some() {
                    return Err("duplicate scheduler dependency slot");
                }
            }
            Ok((result, source.into()))
        })();
        self.dependency_layout_reason = match checked {
            Ok((slots, source)) => {
                self.dependency_layout = slots;
                self.dependency_layout_source = Some(source);
                None
            }
            Err(reason) => Some(reason.into()),
        };
    }

    fn slot_evidence(&self, slot: &Value) -> Option<&Value> {
        self.dependency_layout.get(&slot["offset"].as_u64()?)
    }

    fn slots_complete(&self, descriptor: &Value) -> bool {
        descriptor["dependency_slots"]
            .as_array()
            .is_some_and(|slots| {
                slots.len() == 6
                    && slots
                        .iter()
                        .filter_map(|slot| slot["offset"].as_u64())
                        .collect::<BTreeSet<_>>()
                        == BTreeSet::from([24, 40, 56, 72, 88, 104])
            })
    }

    fn metadata(&self, name: &str) -> Option<&TypeMetadata> {
        self.types
            .get(name)
            .and_then(|index| self.registry.get(*index))
    }

    fn identity(&self, name: &str) -> Option<Value> {
        self.metadata(name).map(|ty| {
            json!({"index":ty.index.as_usize(),
            "qualified_name":ty.qualified_name,"qualified_hash":ty.qualified_hash,
            "size":ty.size,"alignment":ty.alignment})
        })
    }

    fn checked_live_type(&self, value: &Value) -> Result<Option<Value>, String> {
        if value.is_null() {
            return Ok(None);
        }
        let name = value["qualified_name"]
            .as_str()
            .ok_or("live type has no qualified name")?;
        let ty = self
            .metadata(name)
            .ok_or_else(|| format!("live type is absent from reflection: {name}"))?;
        if value["qualified_hash"].as_u64() != Some(u64::from(ty.qualified_hash))
            || value["size"].as_u64() != Some(u64::from(ty.size))
        {
            return Err(format!(
                "live type hash or size disagrees with reflection: {name}"
            ));
        }
        Ok(self.identity(name))
    }

    pub fn set_registry(&mut self, registry: &Value) {
        self.registrations.clear();
        self.registry_reason = if registry["available"] == true {
            None
        } else {
            Some(
                registry["reason"]
                    .as_str()
                    .unwrap_or("live component registration is not ready")
                    .into(),
            )
        };
        if self.registry_reason.is_some() {
            return;
        }
        let Some(entries) = registry["entries"].as_array() else {
            self.registry_reason = Some("live component registry has no entries".into());
            return;
        };
        for row in entries {
            let Some(name) = row["qualified_name"].as_str() else {
                continue;
            };
            let mut names: BTreeMap<String, Vec<&str>> = BTreeMap::new();
            names.entry(name.into()).or_default().push("registration");
            for (key, role) in [("runtime_type", "runtime"), ("template_type", "template")] {
                if let Some(type_name) = row[key]["qualified_name"].as_str() {
                    names.entry(type_name.into()).or_default().push(role);
                }
            }
            let checked = (|| -> Result<Value, String> {
                let ty = self
                    .metadata(name)
                    .ok_or_else(|| format!("registration is absent from reflection: {name}"))?;
                if row["qualified_hash"].as_u64() != Some(u64::from(ty.qualified_hash))
                    || row["index"]
                        .as_u64()
                        .is_none_or(|index| index > u64::from(u16::MAX))
                {
                    return Err(format!(
                        "registration identity disagrees with reflection: {name}"
                    ));
                }
                let runtime = self.checked_live_type(&row["runtime_type"])?;
                let template = self.checked_live_type(&row["template_type"])?;
                if runtime.is_none() && template.is_none() {
                    return Err(format!(
                        "registration has neither runtime nor template type: {name}"
                    ));
                }
                let runtime_size = runtime
                    .as_ref()
                    .and_then(|ty| ty["size"].as_u64())
                    .unwrap_or(0);
                if row["runtime_size"].as_u64() != Some(runtime_size) {
                    return Err(format!(
                        "registration storage size disagrees with reflection: {name}"
                    ));
                }
                Ok(
                    json!({"index":row["index"],"qualified_name":name,"qualified_hash":ty.qualified_hash,
                    "runtime_type":runtime,"template_type":template,"runtime_size":runtime_size,
                    "validated":true,"reason":null}),
                )
            })();
            for (type_name, roles) in names {
                let mut registration = checked.clone().unwrap_or_else(|reason| {
                    json!({
                    "index":row["index"],"qualified_name":name,"validated":false,"reason":reason})
                });
                registration["roles"] = json!(roles);
                self.registrations
                    .entry(type_name)
                    .or_default()
                    .push(registration);
            }
        }
    }

    fn dependency(&self, name: &str) -> Value {
        let qualified = if name.contains("::") {
            name.into()
        } else {
            format!("keen::ecs::{name}")
        };
        let ty = self.identity(&qualified);
        let layout_reason = self.metadata(&qualified).and_then(|ty| {
            super::types::inheritance(&self.registry, ty.index)
                .and_then(|_| super::types::fields(&self.registry, ty.index))
                .err()
        });
        let reflected = ty.is_some() && layout_reason.is_none();
        let registrations = self
            .registrations
            .get(&qualified)
            .cloned()
            .unwrap_or_default();
        let registered = registrations.len() == 1 && registrations[0]["validated"] == true;
        let reason = if !reflected {
            Some(layout_reason.unwrap_or_else(|| {
                format!("dependency type is absent from reflection: {qualified}")
            }))
        } else if let Some(reason) = &self.registry_reason {
            Some(reason.clone())
        } else if registrations.is_empty() {
            Some("dependency has no live component registration".into())
        } else if registrations.len() != 1 {
            Some("dependency belongs to multiple live registrations".into())
        } else if !registered {
            Some(
                registrations[0]["reason"]
                    .as_str()
                    .unwrap_or("registration is not validated")
                    .into(),
            )
        } else {
            None
        };
        json!({"name":name,"qualified_name":qualified,"type":ty,
            "reflection_resolved":reflected,"registration_resolved":registered,
            "registrations":registrations,"reason":reason})
    }

    pub fn enrich(&self, entry: &mut Value) {
        let Some(descriptors) = entry["engine_descriptors"].as_array_mut() else {
            return;
        };
        for descriptor in descriptors {
            let slots_complete = self.slots_complete(descriptor);
            let mut total = 0;
            let mut reflected = 0;
            let mut registered = 0;
            let mut access_resolved = 0;
            if let Some(slots) = descriptor["dependency_slots"].as_array_mut() {
                for slot in slots {
                    let dependencies: Vec<_> = slot["entries"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|name| self.dependency(name))
                        .collect();
                    total += dependencies.len();
                    reflected += dependencies
                        .iter()
                        .filter(|dependency| dependency["reflection_resolved"] == true)
                        .count();
                    registered += dependencies
                        .iter()
                        .filter(|dependency| dependency["registration_resolved"] == true)
                        .count();
                    slot["resolved_entries"] = json!(dependencies);
                    let evidence = self.slot_evidence(slot);
                    access_resolved += usize::from(evidence.is_some());
                    slot["access_mode"] = evidence
                        .map(|value| value["access_mode"].clone())
                        .unwrap_or(Value::Null);
                    slot["storage_role"] = evidence
                        .map(|value| value["storage_role"].clone())
                        .unwrap_or(Value::Null);
                    slot["system_record_mask_offset"] = evidence
                        .map(|value| value["system_record_mask_offset"].clone())
                        .unwrap_or(Value::Null);
                    slot["access_evidence"] = if evidence.is_some() {
                        json!(self.dependency_layout_source)
                    } else {
                        Value::Null
                    };
                    slot["optionality"] = Value::Null;
                }
            }
            descriptor["dependency_resolution"] = json!({"count":total,"reflection_resolved":reflected,
                "registration_resolved":registered,"complete":total == reflected && total == registered,
                "dependency_slots_complete":slots_complete,"access_modes_resolved":slots_complete && access_resolved == 6,
                "access_mode_resolved_slots":access_resolved,"access_mode_reason":self.dependency_layout_reason,
                "optionalities_resolved":false,"native_signature_resolved":false});
        }
    }

    pub fn coverage(&self, entries: &[Value]) -> Value {
        let mut descriptors = 0;
        let mut total = 0;
        let mut reflected = 0;
        let mut registered = 0;
        let mut names = BTreeSet::new();
        let mut reflected_names = BTreeSet::new();
        let mut registered_names = BTreeSet::new();
        let mut slots_count = 0;
        let mut access_resolved = 0;
        let mut complete_slots = true;
        for entry in entries {
            for descriptor in entry["engine_descriptors"].as_array().into_iter().flatten() {
                descriptors += 1;
                complete_slots &= self.slots_complete(descriptor);
                for slot in descriptor["dependency_slots"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    slots_count += 1;
                    access_resolved += usize::from(self.slot_evidence(slot).is_some());
                    for name in slot["entries"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                    {
                        let dependency = self.dependency(name);
                        total += 1;
                        reflected += usize::from(dependency["reflection_resolved"] == true);
                        registered += usize::from(dependency["registration_resolved"] == true);
                        let qualified = dependency["qualified_name"].as_str().unwrap();
                        names.insert(qualified.to_owned());
                        if dependency["reflection_resolved"] == true {
                            reflected_names.insert(qualified.to_owned());
                        }
                        if dependency["registration_resolved"] == true {
                            registered_names.insert(qualified.to_owned());
                        }
                    }
                }
            }
        }
        json!({"engine_descriptors":descriptors,"dependency_references":total,"distinct_dependency_types":names.len(),
            "reflection_resolved":reflected,"registration_resolved":registered,
            "distinct_reflection_resolved":reflected_names.len(),"distinct_registration_resolved":registered_names.len(),
            "complete":total == reflected && total == registered,"registry_reason":self.registry_reason,
            "dependency_slots":slots_count,"dependency_slots_complete":descriptors > 0 && complete_slots,
            "access_mode_resolved_slots":access_resolved,
            "access_modes_resolved":descriptors > 0 && complete_slots && slots_count == access_resolved,
            "access_mode_reason":self.dependency_layout_reason,"optionalities_resolved":false,
            "native_signatures_resolved":false})
    }

    pub fn matches_component(&self, entry: &Value, qualified_name: &str) -> bool {
        entry["engine_descriptors"].as_array().into_iter().flatten().any(|descriptor| {
            descriptor["dependency_slots"].as_array().into_iter().flatten().any(|slot| {
                slot["entries"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|name| {
                    let dependency_name = if name.contains("::") { name.into() } else { format!("keen::ecs::{name}") };
                    dependency_name == qualified_name ||
                        self.registrations.get(&dependency_name).into_iter().flatten().any(|registration| {
                            registration["validated"] == true && (registration["qualified_name"] == qualified_name ||
                                registration["runtime_type"]["qualified_name"] == qualified_name ||
                                registration["template_type"]["qualified_name"] == qualified_name)
                        })
                })
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inventory() -> Value {
        let profile: Value = serde_json::from_str(include_str!(
            "../../../runtime/profiles/enshrouded/client/25966634-resource-read.json"
        ))
        .unwrap();
        let model = &profile["systemDependencyLayout"];
        let slots: Vec<_> = model["slots"].as_array().unwrap().iter().map(|slot| json!({
            "offset":slot["offset"],"storage_role":slot["storageRole"],
            "access_mode":slot["accessMode"],"system_record_mask_offset":slot["systemRecordMaskOffset"]
        })).collect();
        let mut image = profile["image"].clone();
        image["target"] = profile["target"].clone();
        json!({"image":image,"system_dependency_layout":{
            "id":model["id"],"source":model["source"],"validated":true,"image":image,"slots":slots}})
    }

    fn entry() -> Value {
        let slots =
            [24, 40, 56, 72, 88, 104].map(|offset| json!({"offset":offset,"entries":["Fixture"]}));
        json!({"engine_descriptors":[{"name":"fixture","dependency_slots":slots}]})
    }

    fn resolver() -> Resolver {
        Resolver::new(Rc::new(TypeRegistry::default()))
    }

    #[test]
    fn exact_image_scheduler_evidence_resolves_six_roles_but_no_native_signature() {
        let mut resolver = resolver();
        resolver.set_inventory(&inventory());
        let mut entry = entry();
        resolver.enrich(&mut entry);
        let descriptor = &entry["engine_descriptors"][0];
        for (index, (storage, access, mask)) in [
            ("template", "configuration-read", 0xd0),
            ("template", "configuration-read", 0x120),
            ("runtime", "read", 0x30),
            ("runtime", "read", 0x80),
            ("runtime", "write", 0x2e0),
            ("runtime", "write", 0x330),
        ]
        .into_iter()
        .enumerate()
        {
            let slot = &descriptor["dependency_slots"][index];
            assert_eq!(slot["storage_role"], storage);
            assert_eq!(slot["access_mode"], access);
            assert_eq!(slot["system_record_mask_offset"], mask);
            assert!(slot["optionality"].is_null());
        }
        assert_eq!(
            descriptor["dependency_resolution"]["access_modes_resolved"],
            true
        );
        assert_eq!(
            descriptor["dependency_resolution"]["native_signature_resolved"],
            false
        );
        let coverage = resolver.coverage(&[entry]);
        assert_eq!(coverage["dependency_references"], 6);
        assert_eq!(coverage["dependency_slots"], 6);
        assert_eq!(coverage["access_mode_resolved_slots"], 6);
        assert_eq!(coverage["access_modes_resolved"], true);
        assert_eq!(coverage["optionalities_resolved"], false);
        assert_eq!(coverage["native_signatures_resolved"], false);
    }

    #[test]
    fn different_image_or_missing_validation_never_inherits_roles() {
        for key in ["sha256", "target", "timestamp", "size", "validated"] {
            let mut inventory = inventory();
            if key == "validated" {
                inventory["system_dependency_layout"][key] = json!(false);
            } else {
                inventory["system_dependency_layout"]["image"][key] = Value::Null;
            }
            let mut resolver = resolver();
            resolver.set_inventory(&inventory);
            let mut entry = entry();
            resolver.enrich(&mut entry);
            assert!(entry["engine_descriptors"][0]["dependency_slots"][5]["access_mode"].is_null());
            assert_eq!(resolver.coverage(&[entry])["access_modes_resolved"], false);
        }
    }

    #[test]
    fn incomplete_legacy_snapshot_keeps_roles_separate_from_complete_coverage() {
        let mut resolver = resolver();
        resolver.set_inventory(&inventory());
        let mut entry = entry();
        entry["engine_descriptors"][0]["dependency_slots"]
            .as_array_mut()
            .unwrap()
            .pop();
        resolver.enrich(&mut entry);
        assert_eq!(
            entry["engine_descriptors"][0]["dependency_resolution"]["dependency_slots_complete"],
            false
        );
        assert_eq!(resolver.coverage(&[entry])["access_modes_resolved"], false);
    }

    #[test]
    fn duplicate_model_slot_is_rejected_and_previous_roles_are_cleared() {
        let mut resolver = resolver();
        let mut inventory = inventory();
        resolver.set_inventory(&inventory);
        inventory["system_dependency_layout"]["slots"][5]["offset"] = json!(24);
        resolver.set_inventory(&inventory);
        let mut entry = entry();
        resolver.enrich(&mut entry);
        assert!(entry["engine_descriptors"][0]["dependency_slots"][0]["storage_role"].is_null());
        assert_eq!(resolver.coverage(&[entry])["access_modes_resolved"], false);
    }
}
