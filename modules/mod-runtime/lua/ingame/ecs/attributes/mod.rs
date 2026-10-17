//! Lua attribute bindings. Resolution lives in runtime_resolution.
//! Lua attribute actions. Resolution lives with the reflection owner.
pub(crate) use crate::runtime_resolution::attributes::Catalog;
use mlua::Value as LuaValue;
use serde_json::Value;

impl Catalog {
    pub(crate) fn get(&self, selector: &LuaValue) -> Result<&Value, String> {
        match selector {
            LuaValue::Integer(id) => self
                .entries
                .get(&u32::try_from(*id).map_err(|_| "attribute ID outside u32")?)
                .ok_or_else(|| "attribute ID not found".into()),
            LuaValue::String(name) => {
                let name = name.to_str().map_err(|e| e.to_string())?;
                self.by_name(name.as_ref())
            }
            _ => Err("expected original attribute name or unsigned attribute ID".into()),
        }
    }
}

mod read_attribute_storage_range;
use read_attribute_storage_range::storage_range;
mod resolve_attribute_storage;
use resolve_attribute_storage::resolve_storage;
mod read_entity_attribute_storage;
use read_entity_attribute_storage::entity_storage;
mod check_attribute_component;
use check_attribute_component::is_attribute_component;
mod validate_attribute_bits;
use validate_attribute_bits::validate_attribute_bits;
mod recalculate_bound_component;
use recalculate_bound_component::recalculate_bound_component_update;
mod recalculate_component_update;
pub(crate) use recalculate_component_update::recalculate_component_update;
mod expect_root_values;
use expect_root_values::expect_root_values;
mod format_attribute_root;
use format_attribute_root::root_result;
mod encode_attribute_scalar;
use encode_attribute_scalar::encode_attribute_scalar;
#[cfg(test)]
use encode_attribute_scalar::encode_scalar;
mod evaluate_attributes;
mod read_attribute;
mod read_attribute_catalog;
mod read_attribute_descriptor;
mod read_attributes;
mod register_attribute_api;
mod update_attribute;
mod write_attribute_storage;
pub(crate) use register_attribute_api::attach;

#[cfg(test)]
use crate::runtime_resolution::attributes::uint;
#[cfg(test)]
use kfc::guid::Guid;
#[cfg(test)]
use mlua::{Lua, Table};
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
mod tests;
