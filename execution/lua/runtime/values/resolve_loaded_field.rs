use super::{InlineField, resource_field, validate_runtime_scalar};
use kfc::reflection::TypeIndex;
use std::rc::Rc;

/// Fixed inline plans are independent of a payload. Owned container/variant
/// paths still resolve and validate their descriptors against each snapshot.
pub(crate) fn loaded_field(
    state: &crate::env::AppState,
    root: TypeIndex,
    path: &str,
    bytes: &[u8],
) -> Result<Rc<InlineField>, String> {
    if bytes.len() > u32::MAX as usize {
        return Err("resource snapshot exceeds 32-bit blob offset range".into());
    }
    if let Ok(field) = state.inline_field(root, path) {
        let root_size = state
            .type_registry()
            .get(root)
            .ok_or("unresolved resource root")?
            .size as usize;
        let range = field.byte_range();
        if bytes.len() < root_size
            || range.len() != 4
            || range.start % 4 != 0
            || field.leaves.len() != 1
            || field.leaves[0].1 != (0..4)
        {
            return Err(
                "resource field requires a complete root and one aligned 32-bit leaf".into(),
            );
        }
        validate_runtime_scalar(
            field.leaves[0].0,
            bytes
                .get(range)
                .ok_or("resource field outside current snapshot")?,
        )?;
        return Ok(field);
    }
    Ok(Rc::new(resource_field(
        state.type_registry(),
        root,
        path,
        bytes,
    )?))
}
