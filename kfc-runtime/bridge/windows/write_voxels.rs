use super::load_native_provider::provider;

pub fn world_voxel_write(
    origin: [i32; 3],
    dimensions: [u32; 3],
    values: &[u16],
) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let mut outcome = 1u32;
    let ok = unsafe {
        (provider.world_voxel_write)(
            origin.as_ptr(),
            dimensions.as_ptr(),
            values.as_ptr(),
            values.len(),
            &mut outcome,
        )
    };
    if ok {
        return Ok(());
    }
    Err(match outcome {
        1 => "voxel write was rejected before changing the world".into(),
        2 => "Voxel write failed. The previous voxel region was restored and read back".into(),
        _ => "Voxel write outcome is uncertain. Rollback could not be verified".into(),
    })
}
