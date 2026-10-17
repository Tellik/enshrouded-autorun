use crate::windows::provider::provider;
use std::ffi::CString;

pub fn resolve_type(name: &str) -> Option<String> {
    let resolve = provider()?.resolve_type?;
    let name = CString::new(name).ok()?;
    let mut buffer = [0u8; 512];
    if !unsafe { resolve(name.as_ptr(), buffer.as_mut_ptr().cast(), buffer.len()) } {
        return None;
    }
    let length = buffer.iter().position(|byte| *byte == 0)?;
    String::from_utf8(buffer[..length].to_vec()).ok()
}
