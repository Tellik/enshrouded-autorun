use super::load_native_provider::provider;
use std::ffi::CString;

pub fn world_operation_available(name: &str) -> bool {
    let Some(provider) = provider() else {
        return false;
    };
    let Ok(name) = CString::new(name) else {
        return false;
    };
    unsafe { (provider.world_operation_available)(name.as_ptr()) }
}
