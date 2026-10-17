use super::load_native_provider::provider;

pub fn world_session_id() -> u64 {
    provider()
        .and_then(|value| value.world_session_id)
        .map_or(0, |get| unsafe { get() })
}
