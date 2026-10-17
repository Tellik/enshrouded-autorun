use super::format_operation_error::operation_error;
use super::load_native_provider::provider;

pub fn world_entity_spawn(
    uuid: [u64; 2],
    position: [f64; 3],
    rotation: [f64; 4],
    tracking: u32,
    flags: u32,
) -> Result<u32, String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let mut token = 0u32;
    let mut outcome = 1u32;
    let ok = unsafe {
        (provider.world_entity_spawn)(
            uuid.as_ptr(),
            position.as_ptr(),
            rotation.as_ptr(),
            tracking,
            flags,
            &mut token,
            &mut outcome,
        )
    };
    if ok {
        Ok(token)
    } else {
        Err(operation_error("runtime.world.entity.spawn", outcome))
    }
}
