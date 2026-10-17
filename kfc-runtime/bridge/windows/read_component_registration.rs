use super::load_native_provider::provider;
use super::registry;
use std::ffi::CString;

pub fn registration(name: &str) -> Result<serde_json::Value, String> {
    let _span = shroudforge_runtime_diagnostics::measure_runtime_operations::span(
        "native.ecs.registration",
    );
    let Some(fetch) = provider().and_then(|provider| provider.registration) else {
        return registry();
    };
    let name = CString::new(name).map_err(|_| "invalid registration name")?;
    let mut size = unsafe { fetch(name.as_ptr(), std::ptr::null_mut(), 0) };
    for _ in 0..3 {
        if !(2..=1024 * 1024).contains(&size) {
            return Err("invalid native registration response size".into());
        }
        let mut bytes = vec![0u8; size];
        let required = unsafe { fetch(name.as_ptr(), bytes.as_mut_ptr().cast(), bytes.len()) };
        if required > size {
            size = required;
            continue;
        }
        if required < 2 || bytes[required - 1] != 0 {
            return Err("invalid native registration response".into());
        }
        return serde_json::from_slice(&bytes[..required - 1]).map_err(|e| e.to_string());
    }
    Err("native registration changed repeatedly during read".into())
}
