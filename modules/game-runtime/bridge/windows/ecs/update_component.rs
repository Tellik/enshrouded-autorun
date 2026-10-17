use super::can_write_component;
use crate::windows::provider::provider;
use std::ffi::CString;

pub fn compare_exchange(
    entity: u32,
    name: &str,
    expected: &[u8],
    mask: &[u8],
    bytes: &[u8],
) -> Result<(), String> {
    if !can_write_component(name) {
        return Err("component write context or original command phase is unavailable".into());
    }
    let function = provider()
        .and_then(|p| p.compare_exchange)
        .ok_or("native runtime lacks guarded attribute updates")?;
    let name = CString::new(name).map_err(|e| e.to_string())?;
    if expected.len() != bytes.len() || mask.len() != bytes.len() {
        return Err("attribute update buffer sizes disagree".into());
    }
    match unsafe {
        function(
            entity,
            name.as_ptr(),
            expected.as_ptr().cast(),
            mask.as_ptr().cast(),
            bytes.as_ptr().cast(),
            bytes.len(),
        )
    } {
        1 => Ok(()),
        2 => Err(
            "Attribute snapshot changed before update. Read current values and retry".into(),
        ),
        3 => Err(
            "ECS write outcome is uncertain because the original bytes could not be verified after rollback".into(),
        ),
        _ => Err("guarded attribute update failed".into()),
    }
}
