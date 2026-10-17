use super::native_abi::Registry;

pub(super) fn fetch_json(fetch: Registry, maximum: usize) -> Result<serde_json::Value, String> {
    let mut size = unsafe { fetch(std::ptr::null_mut(), 0) };
    for _ in 0..3 {
        if !(2..=maximum).contains(&size) {
            return Err("invalid native catalog response size".into());
        }
        let mut buffer = vec![0u8; size];
        let required = unsafe { fetch(buffer.as_mut_ptr().cast(), buffer.len()) };
        if required > size {
            size = required;
            continue;
        }
        if required < 2 || buffer[required - 1] != 0 {
            return Err("invalid component registry response".into());
        }
        return serde_json::from_slice(&buffer[..required - 1]).map_err(|e| e.to_string());
    }
    Err("component registry changed repeatedly during read".into())
}
