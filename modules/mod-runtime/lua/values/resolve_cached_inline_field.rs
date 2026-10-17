use super::{InlineField, inline_field};
use kfc::reflection::{TypeIndex, TypeRegistry};
use std::rc::Rc;

pub(super) fn cached_inline_field(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    root: TypeIndex,
    path: &str,
) -> Result<Rc<InlineField>, String> {
    if let Some(state) = lua.app_data_ref::<crate::env::AppState>() {
        if Rc::ptr_eq(state.type_registry(), registry) {
            return state.inline_field(root, path);
        }
    }
    Ok(Rc::new(inline_field(registry, root, path)?))
}
