//! Read-only inspection through the same KFC parser used by the Lua API.
//! Usage: cargo run -p shroudforge-api --example export_kfc_resources -- EXE OUTPUT.json [--backup] [--type QUALIFIED_NAME ...] [--building-inputs]
use kfc::{
    container::{KFCFile, KFCReader, KFCReaderOptions},
    reflection::{LookupKey, TypeRegistry},
    resource::value::Value,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args_os().skip(1).collect();
    let legacy_building_export = env!("CARGO_CRATE_NAME") == "inspect_building_inputs";
    let requested_building_export = args
        .iter()
        .skip(2)
        .any(|argument| argument == "--building-inputs");
    let building_inputs = legacy_building_export || requested_building_export;
    if requested_building_export && !legacy_building_export {
        args.retain(|argument| argument != "--building-inputs");
    }
    if legacy_building_export && !(args.len() == 2 || (args.len() == 3 && args[2] == "--backup")) {
        return Err("expected EXE OUTPUT.json [--backup]".into());
    }
    if args.len() < 2 {
        return Err("expected EXE OUTPUT.json [--backup] [--type QUALIFIED_NAME ...]".into());
    }
    let mut backup = false;
    let mut selected_types = Vec::new();
    let mut extra = args[2..].iter();
    while let Some(argument) = extra.next() {
        if argument == "--backup" {
            backup = true;
        } else if argument == "--type" {
            selected_types.push(
                extra
                    .next()
                    .ok_or("--type requires a qualified name")?
                    .to_str()
                    .ok_or("invalid type name")?
                    .to_owned(),
            );
        } else {
            return Err(format!("unknown argument {}", argument.to_string_lossy()).into());
        }
    }
    if selected_types.is_empty() {
        selected_types = if building_inputs {
            &[
                "keen::ItemInfo",
                "keen::VoxelBlueprintItemRegistryResource",
                "keen::VoxelMaterialResolvedList",
                "keen::VoxelBlueprintMaterialPoolRegistryResource",
            ][..]
        } else {
            &[
                "keen::AttributeContainerResource",
                "keen::BaseAttributeResource",
                "keen::BalancingTable",
                "keen::GameKnowledgeResource",
                "keen::GameKnowledgeQueryResourceDb",
            ][..]
        }
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    }
    let exe = PathBuf::from(&args[0]);
    let directory = exe.parent().ok_or("missing game directory")?;
    let stem = exe
        .file_stem()
        .and_then(|v| v.to_str())
        .ok_or("invalid executable name")?;
    let registry = TypeRegistry::load_from_executable(&exe)?;
    let options = if backup {
        KFCReaderOptions {
            kfc_extension: "kfc.bak",
            resource_extension: "kfc_resources.bak",
            ..Default::default()
        }
    } else {
        KFCReaderOptions::default()
    };
    let kfc_path = exe.with_extension(options.kfc_extension);
    let file = KFCFile::from_path(&kfc_path, false)?;
    let version = KFCFile::get_version_tag(&kfc_path)?;
    let mut reader = KFCReader::new_with_options(directory, stem, options)?.into_cursor()?;
    let mut resources = Vec::new();
    for name in &selected_types {
        let Some(ty) = registry.get_by_name(LookupKey::Qualified(name)) else {
            continue;
        };
        for id in kfc::kfc_format::resources_by_type(&file, &registry, name) {
            let mut bytes = Vec::new();
            if !reader.read_resource_into(&id, &mut bytes)? {
                return Err(format!("missing resource {id}").into());
            }
            let value = Value::from_bytes(&registry, ty, &bytes)?;
            let mut value = serde_json::to_value(value)?;
            if building_inputs && name == "keen::ItemInfo" {
                value = serde_json::json!({"itemId":value["itemId"],"debugName":value["debugName"],"equipment":value["equipment"]});
            }
            resources.push(serde_json::json!({"id":id.to_string(),"qualifiedType":name,"typeHash":ty.qualified_hash,"value":value}));
        }
    }
    let output = PathBuf::from(&args[1]);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = BufWriter::new(File::create(output)?);
    let mut report = serde_json::json!({"version":version,"executable":exe,"container":kfc_path,
        "type_source":"fresh executable extraction; no type cache", "resources":resources});
    if !building_inputs {
        report["attribute_component_types"] = serde_json::to_value(
            registry
                .iter()
                .filter(|ty| {
                    ty.qualified_name.starts_with("keen::ecs::")
                        && registry
                            .get_inheritance_chain(ty)
                            .iter()
                            .any(|t| t.qualified_name == "keen::ecs::AttributeComponent")
                })
                .collect::<Vec<_>>(),
        )?;
    }
    serde_json::to_writer_pretty(&mut output, &report)?;
    output.flush()?;
    if building_inputs {
        println!(
            "Read {} building input resources; no game files or memory changed",
            resources.len()
        );
    } else {
        println!(
            "Read {} resources; no game files or memory changed",
            resources.len()
        );
    }
    Ok(())
}
