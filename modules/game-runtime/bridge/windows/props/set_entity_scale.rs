use crate::windows::provider::provider;

pub fn world_entity_set_scale(handle: u32, scale: [f64; 3]) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    if scale.iter().any(|value| !value.is_finite()) {
        return Err("entity scale must contain finite values".into());
    }
    let ok = unsafe { (provider.world_entity_set_scale)(handle, scale.as_ptr()) };
    if ok {
        Ok(())
    } else {
        Err("native entity scale update failed or timed out".into())
    }
}
