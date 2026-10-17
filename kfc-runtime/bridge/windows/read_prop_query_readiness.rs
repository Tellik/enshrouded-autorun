use super::load_native_provider::provider;

pub fn world_entity_query_props_ready() -> bool {
    provider().is_some_and(|value| unsafe {
        value
            .prop_query_ready
            .map_or_else(|| (value.ready)(), |ready| ready())
    })
}
