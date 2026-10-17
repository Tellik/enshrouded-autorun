use super::load_native_provider::provider;

pub fn world_context_active() -> bool {
    provider().is_some_and(|value| unsafe { (value.world_context_active)() })
}
