use crate::windows::provider::provider;
use std::ffi::CString;

pub fn can_write_component(name: &str) -> bool {
    let _span =
        shroudforge_runtime_diagnostics::operation_timings::span("native.ecs.can_write_component");
    let Some(provider) = provider() else {
        return false;
    };
    let Ok(name) = CString::new(name) else {
        return false;
    };
    unsafe {
        provider
            .can_write_component
            .map_or_else(|| (provider.can_write)(), |check| check(name.as_ptr()))
    }
}
