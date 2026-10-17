use super::format_operation_error::operation_error;
use super::load_native_provider::provider;

pub fn world_entity_finish_building(complete: bool) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let mut outcome = 1u32;
    let ok = unsafe { (provider.world_entity_finish)(complete, &mut outcome) };
    if ok {
        Ok(())
    } else {
        Err(operation_error(
            "runtime.world.entity.finish_building",
            outcome,
        ))
    }
}
