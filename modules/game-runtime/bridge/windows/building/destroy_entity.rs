use super::format_operation_error::operation_error;
use crate::windows::provider::provider;

pub fn world_entity_destroy_handle(handle: u32) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let mut outcome = 1u32;
    let ok = unsafe { (provider.world_entity_destroy_handle)(handle, &mut outcome) };
    if ok {
        Ok(())
    } else {
        Err(operation_error("runtime.world.entity.destroy", outcome))
    }
}
