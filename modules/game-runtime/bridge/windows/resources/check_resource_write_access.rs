use crate::windows::provider::provider;

pub fn resource_can_write() -> bool {
    provider()
        .and_then(|value| value.resource_can_write)
        .is_some_and(|ready| unsafe { ready() })
}
