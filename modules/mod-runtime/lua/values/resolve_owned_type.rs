use super::validate_owned_type;
use crate::{env::registry, lua::LuaValue, type_aliases::TypeHandle};
use kfc::reflection::TypeRegistry;
use std::rc::Rc;

pub(super) fn handle(
    registry: &Rc<TypeRegistry>,
    selector: &LuaValue,
) -> Result<TypeHandle, String> {
    let index = registry::resolve(registry, selector)?;
    validate_owned_type(registry, index)?;
    Ok(TypeHandle::new(registry.clone(), index))
}
