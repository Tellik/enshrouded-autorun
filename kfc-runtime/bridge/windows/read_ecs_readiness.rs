use super::load_native_provider::provider;

pub fn ready() -> bool {
    provider().is_some_and(|value| unsafe { (value.ready)() })
}
