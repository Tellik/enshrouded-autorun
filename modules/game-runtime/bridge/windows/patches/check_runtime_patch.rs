use crate::windows::provider::provider;
use std::ffi::CString;

pub fn runtime_patch_available(name: &str) -> bool {
    let Some(provider) = provider() else {
        return false;
    };
    let Ok(name) = CString::new(name) else {
        return false;
    };
    unsafe { (provider.runtime_patch_available)(name.as_ptr()) }
}
