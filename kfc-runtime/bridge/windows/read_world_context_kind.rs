use super::load_native_provider::provider;

pub fn world_context_kind() -> u32 {
    provider()
        .and_then(|value| value.world_context_kind)
        .map_or(0, |get| unsafe { get() })
}
