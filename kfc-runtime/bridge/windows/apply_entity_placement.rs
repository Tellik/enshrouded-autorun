use super::format_operation_error::operation_error;
use super::load_native_provider::provider;

pub fn world_entity_placement(
    destroy: bool,
    position: [f64; 3],
    rotation: [f64; 4],
    bounds: [f32; 6],
    tracking: u32,
    feedback: u32,
) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let mut outcome = 1u32;
    let function = if destroy {
        provider.world_entity_destroy
    } else {
        provider.world_entity_place
    };
    let ok = unsafe {
        function(
            position.as_ptr(),
            rotation.as_ptr(),
            bounds.as_ptr(),
            tracking,
            feedback,
            &mut outcome,
        )
    };
    if ok {
        Ok(())
    } else {
        Err(operation_error(
            if destroy {
                "runtime.world.entity.destroy"
            } else {
                "runtime.world.entity.place"
            },
            outcome,
        ))
    }
}
