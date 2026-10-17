use crate::windows::provider::provider;
use std::ffi::CString;

pub fn query_bounds(
    components: &[String],
    bounds: [f64; 6],
    padding: f64,
) -> Result<Vec<u32>, &'static str> {
    let Some(provider) = provider() else {
        return Err("native ECS provider unavailable");
    };
    let Some(query) = provider.query_bounds else {
        return Err("Native spatial ECS query is unavailable. Update KFC Runtime");
    };
    let Ok(names) = components
        .iter()
        .map(|name| CString::new(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
    else {
        return Err("invalid ECS component name");
    };
    let pointers: Vec<_> = names.iter().map(|name| name.as_ptr()).collect();
    let mut entities = vec![0; 1 << 20];
    let actual = unsafe {
        query(
            pointers.as_ptr(),
            pointers.len(),
            bounds.as_ptr(),
            padding,
            entities.as_mut_ptr(),
            entities.len(),
        )
    };
    if actual == usize::MAX {
        return Err("native spatial ECS query failed or timed out");
    }
    if actual > entities.len() {
        return Err("native spatial ECS query exceeded the entity safety limit");
    }
    entities.truncate(actual);
    Ok(entities)
}
