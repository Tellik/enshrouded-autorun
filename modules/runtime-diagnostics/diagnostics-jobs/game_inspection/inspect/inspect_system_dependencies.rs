//! Read-only native job using the production ECS dependency resolver.
use crate::reflection::{functions, types};
use kfc::reflection::{TypeIndex, TypeRegistry};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

type InspectionError = Box<dyn std::error::Error + Send + Sync>;

fn read_json(path: &Path) -> Result<Value, InspectionError> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn captured_type_name(value: &Value, registry: &TypeRegistry) -> Result<String, InspectionError> {
    let index = match value {
        Value::Null => return Ok(String::new()),
        Value::Number(number) => number.as_u64().ok_or("captured type index is invalid")?,
        _ => return Err("captured type index must be an integer or null".into()),
    };
    let index = TypeIndex::new(usize::try_from(index)?);
    Ok(registry
        .get(index)
        .ok_or("captured registration type is absent from executable reflection")?
        .qualified_name
        .to_owned())
}

fn runtime_capture_adapter(
    capture: &Value,
    registry: &TypeRegistry,
) -> Result<Value, InspectionError> {
    if capture["profileIndependent"] != true {
        return Err("component capture is not a profile-independent live registration".into());
    }
    let registries = capture["registries"]
        .as_array()
        .ok_or("component capture has no registry list")?;
    if registries.len() != 1 {
        return Err(format!(
            "expected one captured ECS registry, found {}",
            registries.len()
        )
        .into());
    }
    let captured_registry = &registries[0];
    let rows = captured_registry["rows"]
        .as_array()
        .ok_or("component capture has no registration rows")?;
    if captured_registry["layoutVersion"] != 1
        || captured_registry["stableReadback"] != true
        || captured_registry["count"].as_u64() != Some(rows.len() as u64)
    {
        return Err("captured ECS registry layout or stable-readback evidence is invalid".into());
    }
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let runtime_type = captured_type_name(&row["runtimeType"], registry)?;
        let template_type = captured_type_name(&row["templateType"], registry)?;
        if runtime_type.is_empty() && template_type.is_empty() {
            return Err("captured registration has neither runtime nor template type".into());
        }
        entries.push(json!({
            "index": row["index"],
            "qualified_name": row["qualifiedName"],
            "qualified_hash": row["qualifiedHash"],
            "size": row["runtimeSize"],
            "runtimeType": runtime_type,
            "templateType": template_type
        }));
    }
    Ok(json!({"available":true,"entries":entries}))
}

fn game_executable(root: &Path) -> Result<PathBuf, InspectionError> {
    let current = std::env::current_exe()?;
    if current
        .file_name()
        .is_some_and(|name| name == "enshrouded.exe" || name == "enshrouded_server.exe")
    {
        return Ok(current);
    }
    let client = root.join("enshrouded.exe");
    let server = root.join("enshrouded_server.exe");
    let executable = if server.is_file() && !client.is_file() {
        server
    } else {
        client
    };
    if !executable.is_file() {
        return Err(format!("game executable was not found under {}", root.display()).into());
    }
    Ok(executable)
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), InspectionError> {
    if cancelled.load(Ordering::Acquire) {
        return Err("diagnostic job was stopped".into());
    }
    Ok(())
}

pub(crate) fn inspect_system_dependencies(
    root: &Path,
    cancelled: &AtomicBool,
) -> Result<Value, InspectionError> {
    check_cancelled(cancelled)?;
    let executable = game_executable(root)?;
    let capture =
        shroudforge_loader_config::paths::export_dir(root).join("diagnostics/capture.runtime");
    let function_path = capture.join("functions.json");
    let registration_path = capture.join("components.json");
    if !function_path.is_file() || !registration_path.is_file() {
        return Err(format!(
            "run capture.runtime first. Expected {} and {}",
            function_path.display(),
            registration_path.display()
        )
        .into());
    }
    let bytes = std::fs::read(&executable)?;
    check_cancelled(cancelled)?;
    let u32_at = |offset: usize| -> Result<u32, InspectionError> {
        Ok(u32::from_le_bytes(
            bytes
                .get(offset..offset.checked_add(4).ok_or("PE offset overflow")?)
                .ok_or("truncated PE metadata")?
                .try_into()?,
        ))
    };
    let nt = usize::try_from(u32_at(0x3c)?)?;
    if bytes.get(..2) != Some(b"MZ")
        || bytes.get(nt..nt.checked_add(4).ok_or("PE offset overflow")?) != Some(b"PE\0\0")
    {
        return Err("invalid game PE headers".into());
    }
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let timestamp = u32_at(nt.checked_add(8).ok_or("PE offset overflow")?)?;
    let image_size = u32_at(nt.checked_add(24 + 56).ok_or("PE offset overflow")?)?;
    let inventory = read_json(&function_path)?;
    let captured_components = read_json(&registration_path)?;
    if inventory["image"]["timestamp"].as_u64() != Some(u64::from(timestamp))
        || inventory["image"]["size"].as_u64() != Some(u64::from(image_size))
    {
        return Err("function snapshot timestamp or image size differs from the executable".into());
    }
    if inventory["image"]["sha256"].as_str() != Some(sha256.as_str())
        || captured_components["image"] != inventory["image"]
    {
        return Err("function and component captures must match the exact game executable".into());
    }
    check_cancelled(cancelled)?;
    let registry = Rc::new(TypeRegistry::load_from_executable(&executable)?);
    let registrations = runtime_capture_adapter(&captured_components, &registry)?;
    let mut resolver = functions::Resolver::new(registry.clone());
    resolver.set_inventory(&inventory);
    resolver.set_registry(&registrations);
    let entries = inventory["entries"]
        .as_array()
        .ok_or("function snapshot has no entries")?;
    let coverage = resolver.coverage(entries);
    let mut systems = Vec::new();
    let mut dependency_indices = BTreeSet::new();
    for entry in entries
        .iter()
        .filter(|entry| entry["engine_descriptors"].is_array())
    {
        check_cancelled(cancelled)?;
        let mut resolved = entry.clone();
        resolver.enrich(&mut resolved);
        for descriptor in resolved["engine_descriptors"]
            .as_array()
            .into_iter()
            .flatten()
        {
            for slot in descriptor["dependency_slots"]
                .as_array()
                .into_iter()
                .flatten()
            {
                for dependency in slot["resolved_entries"].as_array().into_iter().flatten() {
                    if let Some(index) = dependency["type"]["index"].as_u64() {
                        dependency_indices.insert(usize::try_from(index)?);
                    }
                }
            }
        }
        systems.push(resolved);
    }
    let mut layouts = Vec::new();
    for index in dependency_indices {
        check_cancelled(cancelled)?;
        let index = TypeIndex::new(index);
        let ty = registry
            .get(index)
            .ok_or("dependency index is absent from reflection")?;
        let inheritance = types::inheritance(&registry, index).map_err(std::io::Error::other)?;
        let fields = types::fields(&registry, index).map_err(std::io::Error::other)?;
        layouts.push(json!({"metadata":ty,"inheritance":inheritance,
            "effective_fields":fields.into_iter().map(|(owner,field)| json!({"declaring_type_index":owner,"field":field})).collect::<Vec<_>>()}));
    }
    let limitations = vec![
        "Access roles require exact-image original scheduler evidence. Descriptor optionality, native ABIs, engine execution contexts and gameplay effects remain unproven",
        "The live registration capture validates identities and storage layout, but does not prove callback ABIs or native call safety",
    ];
    Ok(
        json!({"schema_version":1,"kind":"runtime-system-dependency-inspection",
        "executable":executable,"image":{"sha256":sha256,"timestamp":timestamp,"size":image_size},
        "sources":{"functions":function_path,"registrations":registration_path,"registry_format":"capture.runtime live registration",
            "function_snapshot_identity":"sha256","types":"fresh executable extraction; no type cache","resolver":"modules/mod-runtime/reflection/resolve_system_dependencies.rs"},
        "coverage":coverage,"system_dependency_layout":inventory["system_dependency_layout"],
        "native_inventory_count":entries.len(),"systems":systems,"dependency_type_layouts":layouts,
        "complete_engine_api":false,"limitations":limitations}),
    )
}
