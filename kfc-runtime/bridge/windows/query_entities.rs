use super::load_native_provider::provider;
use std::ffi::CString;

pub fn query(components: &[String]) -> Result<Vec<u32>, &'static str> {
    let _span =
        shroudforge_runtime_diagnostics::measure_runtime_operations::span("native.ecs.query");
    let Some(provider) = provider() else {
        return Err("native ECS provider unavailable");
    };
    let Ok(names) = components
        .iter()
        .map(|name| CString::new(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
    else {
        return Err("invalid ECS component name");
    };
    let pointers: Vec<_> = names.iter().map(|name| name.as_ptr()).collect();
    // Most player queries fit in one call. Do not count then rescan the
    // whole world on another engine tick for every query.
    let mut entities = vec![0; 256];
    for _ in 0..3 {
        let actual = unsafe {
            (provider.query)(
                pointers.as_ptr(),
                pointers.len(),
                entities.as_mut_ptr(),
                entities.len(),
            )
        };
        if actual == usize::MAX {
            return Err("native ECS query failed or timed out");
        }
        if actual == usize::MAX - 1 {
            return Err("Live ECS query is still scanning. Retry on the next update");
        }
        if actual <= entities.len() {
            entities.truncate(actual);
            return Ok(entities);
        }
        if actual > 1 << 20 {
            return Err("live ECS query returned an invalid entity count");
        }
        entities.resize(actual, 0);
    }
    Err("live ECS query buffer did not stabilize")
}
