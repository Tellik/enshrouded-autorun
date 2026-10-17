//! A reflected inline field and its exact read/write codec.
use super::{decode, validate_runtime_scalar};
use crate::{
    env::game::value::{convert_lua_to_value, validate_and_clone_lua_value},
    lua::LuaValue,
    type_aliases::TypeHandle,
};
use kfc::reflection::{PrimitiveType, TypeIndex};

/// One explicitly selected inline field. Other fields in the component may own
/// dynamic storage. Their descriptors are never decoded or included in this mask.
pub(crate) struct InlineField {
    pub(super) ty: TypeHandle,
    pub(super) offset: usize,
    pub(super) leaves: Vec<(PrimitiveType, std::ops::Range<usize>)>,
}

impl InlineField {
    pub(crate) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.leaves
                .capacity()
                .saturating_mul(std::mem::size_of::<(PrimitiveType, std::ops::Range<usize>)>()),
        )
    }
    pub(crate) fn type_index(&self) -> TypeIndex {
        self.ty.index
    }

    pub(crate) fn byte_range(&self) -> std::ops::Range<usize> {
        self.offset..self.offset + self.ty.size as usize
    }

    pub(crate) fn read(&self, lua: &mlua::Lua, component: &[u8]) -> Result<LuaValue, String> {
        let bytes = component
            .get(self.offset..self.offset + self.ty.size as usize)
            .ok_or("field outside component snapshot")?;
        decode(lua, &self.ty, bytes)
    }

    pub(crate) fn write_into(
        &self,
        lua: &mlua::Lua,
        value: &LuaValue,
        component: &mut [u8],
        mask: &mut [u8],
    ) -> Result<(), String> {
        let checked =
            validate_and_clone_lua_value(value, &self.ty, lua).map_err(|e| e.to_string())?;
        let value = convert_lua_to_value(&checked, &self.ty).map_err(|e| e.to_string())?;
        let encoded = value
            .to_bytes(self.ty.type_registry(), &self.ty)
            .map_err(|e| e.to_string())?;
        if encoded.len() != self.ty.size as usize {
            return Err("serialized inline field size changed".into());
        }
        for (kind, range) in &self.leaves {
            let bytes = &encoded[range.clone()];
            validate_runtime_scalar(*kind, bytes)?;
            let start = self.offset + range.start;
            let end = self.offset + range.end;
            let destination = component
                .get_mut(start..end)
                .ok_or("field outside component")?;
            let selected = mask.get_mut(start..end).ok_or("field outside write mask")?;
            if selected.iter().any(|byte| *byte != 0) {
                return Err("selected fields overlap".into());
            }
            // Copy reflected value leaves only. Struct and array padding stays
            // byte-for-byte identical to the live component snapshot.
            destination.copy_from_slice(bytes);
            selected.fill(1);
        }
        Ok(())
    }
}
