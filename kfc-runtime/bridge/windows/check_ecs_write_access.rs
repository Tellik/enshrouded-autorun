use super::load_native_provider::provider;

pub fn can_write() -> bool {
    let _span =
        shroudforge_runtime_diagnostics::measure_runtime_operations::span("native.ecs.can_write");
    provider().is_some_and(|value| unsafe { (value.can_write)() })
}
