use crate::windows::provider::provider;

pub fn can_write() -> bool {
    let _span = shroudforge_runtime_diagnostics::operation_timings::span("native.ecs.can_write");
    provider().is_some_and(|value| unsafe { (value.can_write)() })
}
