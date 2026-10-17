use std::{ffi::OsString, path::PathBuf};

pub(crate) fn run(args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    let game = PathBuf::from(&args[2]);
    let id = args[3].to_str().ok_or("mod id must be valid UTF-8")?;
    let name = args[4].to_str().ok_or("mod name must be valid UTF-8")?;
    let capabilities = args
        .get(5)
        .and_then(|value| value.to_str())
        .unwrap_or("patch")
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    shroudforge_package::create_mod(&game, id, name, &capabilities, |root, target, force| {
        shroudforge_api::export_lua_definitions(root, target, force)
    })?;
    Ok(())
}
