use crate::windows::native_abi::PropRecord;
use crate::windows::provider::provider;

pub fn world_entity_query_props_in_bounds(
    bounds: [f64; 6],
) -> Result<Vec<PropRecord>, &'static str> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable");
    };
    let count = unsafe {
        (provider.world_entity_query_props_in_bounds)(bounds.as_ptr(), std::ptr::null_mut(), 0)
    };
    if count == usize::MAX - 1 {
        return Err("Live prop query is still scanning. Retry on the next update");
    }
    if count == usize::MAX {
        return Err("native recipe-bounds prop query failed");
    }
    if count > 1_000_000 {
        return Err("native prop query exceeded the 1,000,000-prop safety limit");
    }
    let mut props = vec![PropRecord::default(); count];
    if count == 0 {
        return Ok(props);
    }
    let actual = unsafe {
        (provider.world_entity_query_props_in_bounds)(
            bounds.as_ptr(),
            props.as_mut_ptr(),
            props.len(),
        )
    };
    if actual == usize::MAX - 1 {
        return Err("Live prop query is still scanning. Retry on the next update");
    }
    if actual == usize::MAX {
        return Err("native recipe-bounds prop query failed while retrieving results");
    }
    if actual > props.len() {
        return Err("native prop query changed between count and retrieval");
    }
    props.truncate(actual);
    Ok(props)
}
