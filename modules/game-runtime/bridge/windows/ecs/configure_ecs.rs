use crate::windows::provider::provider;
use std::ffi::CString;

pub fn configure(types: &[(String, u32)]) -> bool {
    let Some(provider) = provider() else {
        return false;
    };
    let Ok(names) = types
        .iter()
        .map(|(name, _)| CString::new(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
    else {
        return false;
    };
    let pointers: Vec<_> = names.iter().map(|name| name.as_ptr()).collect();
    let sizes: Vec<_> = types.iter().map(|(_, size)| *size).collect();
    unsafe { (provider.configure)(pointers.as_ptr(), sizes.as_ptr(), pointers.len()) }
}
