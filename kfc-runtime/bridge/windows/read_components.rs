use super::load_native_provider::provider;
use std::ffi::CString;

pub fn read_many(
    entity: u32,
    selections: &[(String, u32)],
) -> Result<std::collections::HashMap<String, Vec<u8>>, String> {
    let _span =
        shroudforge_runtime_diagnostics::measure_runtime_operations::span("native.ecs.read_many");
    if selections.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let provider = provider().ok_or("native ECS provider unavailable")?;
    if selections.len() > 64 {
        return Err("component batch expects 1 to 64 selections".into());
    }
    let total = selections
        .iter()
        .try_fold(0usize, |size, (_, width)| {
            if *width == 0 {
                return None;
            }
            size.checked_add(*width as usize)
        })
        .filter(|total| *total <= 1024 * 1024)
        .ok_or("component batch size exceeds 1 MiB")?;
    let mut result = std::collections::HashMap::new();
    if selections
        .iter()
        .any(|(name, _)| selections.iter().filter(|(other, _)| name == other).count() != 1)
    {
        return Err("component batch names must be unique".into());
    }
    if let Some(fetch) = provider.read_many {
        let names = selections
            .iter()
            .map(|(name, _)| CString::new(name.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "invalid component batch name")?;
        let pointers: Vec<_> = names.iter().map(|name| name.as_ptr()).collect();
        let sizes: Vec<_> = selections.iter().map(|(_, size)| *size).collect();
        let mut buffer = vec![0u8; total];
        let mut present = vec![0u8; selections.len()];
        if !unsafe {
            fetch(
                entity,
                pointers.as_ptr(),
                sizes.as_ptr(),
                selections.len(),
                buffer.as_mut_ptr(),
                buffer.len(),
                present.as_mut_ptr(),
            )
        } {
            return Err("component batch snapshot is unavailable or changed".into());
        }
        let mut start = 0;
        for ((name, width), flag) in selections.iter().zip(present) {
            let end = start + *width as usize;
            match flag {
                0 => {}
                1 => {
                    result.insert(name.clone(), buffer[start..end].to_vec());
                }
                _ => return Err("component batch returned an invalid presence marker".into()),
            }
            start = end;
        }
    } else {
        // Attribute root selection needs proven absence for every omitted
        // candidate. Legacy read() cannot distinguish absence from a failed
        // read, so it cannot safely substitute for this optional extension.
        return Err("native provider lacks checked component batch reads. Install the matching runtime provider".into());
    }
    Ok(result)
}
