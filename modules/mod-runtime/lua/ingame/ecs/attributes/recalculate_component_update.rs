use super::is_attribute_component;
use super::recalculate_bound_component_update;
use crate::env::AppState;
use crate::env::loader;

/// Generic `ecs.write` and `ecs.write_fields` preserve AttributeComponent
/// headers and execute the original initialized root's full calculation before
/// the native expected-snapshot CAS. Other component types keep their codec.
pub(crate) fn recalculate_component_update(
    state: &AppState,
    name: &str,
    before: &[u8],
    after: &mut [u8],
    mask: &mut [u8],
) -> Result<(), String> {
    if !is_attribute_component(state.type_registry(), name)? {
        return Ok(());
    }
    let catalog = state.attribute_catalog()?;
    let functions = loader::runtime_provider::functions()?;
    recalculate_bound_component_update(
        catalog,
        name,
        before,
        after,
        mask,
        functions["attribute_calculation_model"]
            .as_str()
            .unwrap_or(""),
    )
    .map(|_| ())
}
