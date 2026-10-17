use super::load_native_provider::provider;
use super::world_session_id;
use std::ffi::CString;

pub fn service_read(key: &str, type_hash: u32, size: u32, session: u64) -> Result<Vec<u8>, String> {
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err("invalid reflected service snapshot size".into());
    }
    if session == 0 || world_session_id() != session {
        return Err("service snapshot belongs to another or an inactive world session".into());
    }
    let key = CString::new(key).map_err(|_| "invalid service key")?;
    let read = provider()
        .and_then(|provider| provider.service_read)
        .ok_or("native provider lacks typed service snapshots")?;
    let mut bytes = vec![0; size as usize];
    if !unsafe {
        read(
            key.as_ptr(),
            type_hash,
            size,
            session,
            bytes.as_mut_ptr(),
            bytes.len(),
        )
    } {
        return Err("guarded service snapshot is unavailable or changed during retrieval".into());
    }
    if world_session_id() != session {
        return Err("world session changed during service snapshot retrieval".into());
    }
    Ok(bytes)
}
