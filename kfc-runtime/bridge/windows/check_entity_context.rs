use super::load_native_provider::provider;

pub fn world_entity_context_ready() -> bool {
    provider().is_some_and(|value| unsafe { (value.world_entity_context_ready)() })
}
