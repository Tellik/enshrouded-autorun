use super::load_native_provider::provider;
use std::ffi::CString;

pub fn runtime_patch_set_enabled(name: &str, enabled: bool) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let name = CString::new(name).map_err(|_| "invalid patch name")?;
    let mut outcome = 1u32;
    let ok = unsafe { (provider.runtime_patch_set_enabled)(name.as_ptr(), enabled, &mut outcome) };
    if ok {
        return Ok(());
    }
    Err(format!(
        "runtime patch '{}' rejected the operation or failed its live-byte safety check (outcome={outcome})",
        name.to_string_lossy()
    ))
}
