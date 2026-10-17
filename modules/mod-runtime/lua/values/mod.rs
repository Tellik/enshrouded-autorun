//! Owned KFC values. No process pointers or engine allocation are exposed.
//! Owned KFC values and explicit reflected field actions.

mod inline_field;
pub(crate) use inline_field::InlineField;
mod resolve_cached_inline_field;
use resolve_cached_inline_field::cached_inline_field;
mod resolve_loaded_field;
pub(crate) use resolve_loaded_field::loaded_field;
mod validate_runtime_scalar;
pub(crate) use validate_runtime_scalar::validate_runtime_scalar;
mod expect_inline_fields;
pub(crate) use expect_inline_fields::expect_inline_fields;
mod parse_field_path;
use parse_field_path::{FieldStep, field_path};
mod collect_inline_leaves;
use collect_inline_leaves::inline_leaves;
mod reject_external_overlap;
use reject_external_overlap::reject_external_overlap;
mod resolve_inline_field;
pub(crate) use resolve_inline_field::inline_field;
mod resolve_resource_field;
pub(crate) use resolve_resource_field::resource_field;
mod validate_owned_type;
pub(crate) use validate_owned_type::validate_owned_type;
mod resolve_owned_type;
use resolve_owned_type::handle;
mod decode_owned_value;
use decode_owned_value::decode;
mod lua_value_result;
use lua_value_result::outcome;
mod register_value_api;
pub(crate) use register_value_api::create;
mod create_default_value;
mod encode_owned_value;
mod normalize_field;

#[cfg(test)]
use crate::{
    env::registry,
    lua::LuaValue,
    type_aliases::{MappedValue, TypeHandle},
};
#[cfg(test)]
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
#[cfg(test)]
use std::rc::Rc;
#[cfg(test)]
mod tests;
