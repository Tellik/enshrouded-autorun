//! Read-only inspector using the actual pure runtime dependency resolver.
//! Usage: inspect_system_dependencies EXE FUNCTIONS.json REGISTRY.json OUTPUT.json
//!        [--registry-format native|provider-verification]
//! The older provider-verification format records type names and runtime sizes.
//! Its explicit dev adapter cannot supply an independent live template-size proof.
use shroudforge_api::reflection::{functions, types};

use kfc::reflection::{TypeMetadata, TypeRegistry};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    rc::Rc,
};

fn read_json(path: &PathBuf) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

fn type_identity(ty: &TypeMetadata) -> Value {
    json!({"qualified_name":ty.qualified_name,"qualified_hash":ty.qualified_hash,"size":ty.size})
}

fn provider_capture_adapter(
    capture: &Value,
    registry: &TypeRegistry,
) -> Result<Value, Box<dyn std::error::Error>> {
    if capture["available"] != true {
        return Err("provider capture is unavailable".into());
    }
    let by_name: BTreeMap<_, _> = registry
        .iter()
        .map(|ty| (ty.qualified_name.as_str(), ty))
        .collect();
    let mut entries = Vec::new();
    for row in capture["entries"]
        .as_array()
        .ok_or("provider capture has no entries")?
    {
        let mut entry = json!({"index":row["index"],"qualified_name":row["qualified_name"],
            "qualified_hash":row["qualified_hash"],"runtime_size":row["size"],
            "runtime_type":null,"template_type":null});
        for (source, destination) in [
            ("runtimeType", "runtime_type"),
            ("templateType", "template_type"),
        ] {
            let name = row[source]
                .as_str()
                .ok_or("provider capture type name is missing")?;
            if name.is_empty() {
                continue;
            }
            let ty = by_name
                .get(name)
                .ok_or_else(|| format!("captured type not present in the executable: {name}"))?;
            entry[destination] = type_identity(ty);
        }
        entries.push(entry);
    }
    Ok(
        json!({"available":true,"source":"explicit-provider-verification-dev-adapter","entries":entries}),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(args.len() == 4 || (args.len() == 6 && args[4] == "--registry-format")) {
        return Err("expected EXE FUNCTIONS.json REGISTRY.json OUTPUT.json [--registry-format native|provider-verification]".into());
    }
    let registry_format = if args.len() == 6 {
        args[5].to_str().ok_or("invalid registry format")?
    } else {
        "native"
    };
    if !matches!(registry_format, "native" | "provider-verification") {
        return Err("unknown registry format".into());
    }
    let executable = PathBuf::from(&args[0]);
    let function_path = PathBuf::from(&args[1]);
    let registration_path = PathBuf::from(&args[2]);
    let output_path = PathBuf::from(&args[3]);
    let bytes = std::fs::read(&executable)?;
    let u32_at = |offset: usize| -> Result<u32, Box<dyn std::error::Error>> {
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
        return Err("invalid PE headers".into());
    }
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let timestamp = u32_at(nt.checked_add(8).ok_or("PE offset overflow")?)?;
    let image_size = u32_at(nt.checked_add(24 + 56).ok_or("PE offset overflow")?)?;
    let inventory = read_json(&function_path)?;
    if inventory["image"]["timestamp"].as_u64() != Some(u64::from(timestamp))
        || inventory["image"]["size"].as_u64() != Some(u64::from(image_size))
    {
        return Err("function snapshot timestamp or image size differs from the executable".into());
    }
    let captured_hash = inventory["image"]["sha256"]
        .as_str()
        .filter(|hash| !hash.is_empty());
    if captured_hash.is_some_and(|hash| hash != sha256) {
        return Err("function snapshot SHA256 differs from the executable".into());
    }
    let registry = Rc::new(TypeRegistry::load_from_executable(&executable)?);
    let captured_registry = read_json(&registration_path)?;
    let registrations = match registry_format {
        "provider-verification" => provider_capture_adapter(&captured_registry, &registry)?,
        _ => captured_registry,
    };
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
        let index = kfc::reflection::TypeIndex::new(index);
        let ty = registry
            .get(index)
            .ok_or("dependency index is absent from reflection")?;
        let inheritance = types::inheritance(&registry, index).map_err(std::io::Error::other)?;
        let fields = types::fields(&registry, index).map_err(std::io::Error::other)?;
        layouts.push(json!({"metadata":ty,"inheritance":inheritance,
            "effective_fields":fields.into_iter().map(|(owner,field)| json!({"declaring_type_index":owner,"field":field})).collect::<Vec<_>>()}));
    }
    let mut limitations = vec![
        "Access roles require exact-image original scheduler evidence. Descriptor optionality, native ABIs, engine execution contexts and gameplay effects remain unproven",
    ];
    if captured_hash.is_none() {
        limitations.push("The legacy function snapshot has no SHA256. Its timestamp and image size match, but this is not exact executable identity proof");
    }
    if registry_format == "provider-verification" {
        limitations.push("The explicit legacy capture adapter preserves captured registration hashes, runtime sizes and runtime/template names. Type metadata comes from the freshly parsed executable. Independent live template hashes and sizes were not recorded by this capture format");
    }
    let result = json!({"schema_version":1,"kind":"runtime-system-dependency-inspection",
        "executable":executable,"image":{"sha256":sha256,"timestamp":timestamp,"size":image_size},
        "sources":{"functions":function_path,"registrations":registration_path,"registry_format":registry_format,
            "function_snapshot_identity":if captured_hash.is_some() {"sha256"} else {"timestamp-and-image-size-only"},
            "types":"fresh executable extraction; no type cache","resolver":"actual execution/reflection/functions.rs"},
        "coverage":coverage,"system_dependency_layout":inventory["system_dependency_layout"],
        "native_inventory_count":entries.len(),"systems":systems,"dependency_type_layouts":layouts,
        "complete_engine_api":false,"limitations":limitations});
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = BufWriter::new(File::create(&output_path)?);
    serde_json::to_writer_pretty(&mut output, &result)?;
    output.flush()?;
    println!("{}", serde_json::to_string(&coverage)?);
    println!(
        "Wrote {}. No game files or memory changed",
        output_path.display()
    );
    Ok(())
}
