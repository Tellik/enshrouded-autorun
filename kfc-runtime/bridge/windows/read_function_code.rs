use super::load_native_provider::provider;

pub fn function_code(rva: u32) -> Option<Vec<u8>> {
    let fetch = provider()?.function_code?;
    let mut code = vec![0; 512];
    let count = unsafe { fetch(rva, code.as_mut_ptr(), code.len()) };
    if count == 0 || count > code.len() {
        return None;
    }
    code.truncate(count);
    Some(code)
}
