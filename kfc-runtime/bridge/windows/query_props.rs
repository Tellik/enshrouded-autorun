use super::load_native_provider::provider;
use super::native_abi::PropRecord;

pub fn world_entity_query_props(
    bounds: [f64; 6],
    padding: f64,
) -> Result<Vec<PropRecord>, &'static str> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable");
    };
    let count = unsafe {
        (provider.world_entity_query_props)(bounds.as_ptr(), padding, std::ptr::null_mut(), 0)
    };
    if count == usize::MAX {
        return Err("native prop query failed");
    }
    if count > 1_000_000 {
        return Err("native prop query exceeded the 1,000,000-prop safety limit");
    }
    let mut props = vec![PropRecord::default(); count];
    if count == 0 {
        return Ok(props);
    }
    let actual = unsafe {
        (provider.world_entity_query_props)(
            bounds.as_ptr(),
            padding,
            props.as_mut_ptr(),
            props.len(),
        )
    };
    if actual == usize::MAX {
        return Err("native prop query failed while retrieving results");
    }
    if actual > 1_000_000 {
        return Err("native prop query exceeded the 1,000,000-prop safety limit");
    }
    if actual > props.len() {
        return Err("native prop query changed between count and retrieval");
    }
    props.truncate(actual);
    Ok(props)
}
