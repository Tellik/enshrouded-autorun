//! Owned KFC values. No process pointers or engine allocation are exposed.
use super::{
    game::value::{convert_lua_to_value, convert_value_to_lua, validate_and_clone_lua_value},
    registry,
};
use crate::{
    alias::{MappedValue, TypeHandle},
    lua::LuaValue,
};
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use std::{collections::HashSet, rc::Rc};

/// One explicitly selected inline field. Other fields in the component may own
/// dynamic storage. Their descriptors are never decoded or included in this mask.
pub(crate) struct InlineField {
    ty: TypeHandle,
    offset: usize,
    leaves: Vec<(PrimitiveType, std::ops::Range<usize>)>,
}

fn cached_inline_field(
    lua: &mlua::Lua, registry: &Rc<TypeRegistry>, root: TypeIndex, path: &str,
) -> Result<Rc<InlineField>, String> {
    if let Some(state) = lua.app_data_ref::<super::AppState>() {
        if Rc::ptr_eq(state.type_registry(), registry) {
            return state.inline_field(root, path);
        }
    }
    Ok(Rc::new(inline_field(registry, root, path)?))
}

/// Fixed inline plans are independent of a payload. Owned container/variant
/// paths still resolve and validate their descriptors against each snapshot.
pub(crate) fn loaded_field(
    state: &super::AppState, root: TypeIndex, path: &str, bytes: &[u8],
) -> Result<Rc<InlineField>, String> {
    if bytes.len() > u32::MAX as usize {
        return Err("resource snapshot exceeds 32-bit blob offset range".into());
    }
    if let Ok(field) = state.inline_field(root, path) {
        let root_size = state.type_registry().get(root).ok_or("unresolved resource root")?.size as usize;
        let range = field.byte_range();
        if bytes.len() < root_size || range.len() != 4 || range.start % 4 != 0
            || field.leaves.len() != 1 || field.leaves[0].1 != (0..4)
        { return Err("resource field requires a complete root and one aligned 32-bit leaf".into()); }
        validate_runtime_scalar(field.leaves[0].0,
            bytes.get(range).ok_or("resource field outside current snapshot")?)?;
        return Ok(field);
    }
    Ok(Rc::new(resource_field(state.type_registry(), root, path, bytes)?))
}

/// Shared scalar policy for every runtime writer, including whole components.
pub(crate) fn validate_runtime_scalar(kind: PrimitiveType, bytes: &[u8]) -> Result<(), String> {
    let finite = match kind {
        PrimitiveType::Float32 => f32::from_le_bytes(bytes.try_into().map_err(|_| "invalid float32 width")?).is_finite(),
        PrimitiveType::Float64 => f64::from_le_bytes(bytes.try_into().map_err(|_| "invalid float64 width")?).is_finite(),
        _ => true,
    };
    if !finite {
        return Err("runtime field writes require finite floating-point values".into());
    }
    Ok(())
}

impl InlineField {
    pub(crate) fn retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.leaves.capacity().saturating_mul(std::mem::size_of::<(PrimitiveType, std::ops::Range<usize>)>()))
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

#[derive(Clone, Copy)]
enum FieldStep<'a> {
    Name(&'a str),
    Index(usize),
}

/// Check caller expectations through the exact inline field codec used by writes.
/// The subsequent native CAS still compares the complete fresh component snapshot.
pub(crate) fn expect_inline_fields(
    lua: &mlua::Lua,
    registry: &Rc<TypeRegistry>,
    component: TypeIndex,
    before: &[u8],
    expected: &mlua::Table,
) -> Result<(), String> {
    let mut candidate = before.to_vec();
    let mut mask = vec![0; before.len()];
    let mut count = 0;
    for pair in expected.clone().pairs::<String, LuaValue>() {
        let (path, value) = pair.map_err(|error| error.to_string())?;
        count += 1;
        if count > 1024 {
            return Err("at most 1024 expected field paths may be selected".into());
        }
        cached_inline_field(lua, registry, component, &path)?
            .write_into(lua, &value, &mut candidate, &mut mask)
            .map_err(|error| format!("expected {path}: {error}"))?;
    }
    if count == 0 {
        return Err("expected fields must be nonempty when provided".into());
    }
    if mask
        .iter()
        .enumerate()
        .any(|(index, selected)| *selected != 0 && candidate[index] != before[index])
    {
        return Err(
            "Selected field snapshot changed before update. Read current fields and retry".into(),
        );
    }
    Ok(())
}

fn field_path(mut path: &str) -> Result<Vec<FieldStep<'_>>, String> {
    if path.is_empty() || path.len() > 1024 {
        return Err("expected a nonempty field path of at most 1024 bytes".into());
    }
    let mut steps = Vec::new();
    while !path.is_empty() {
        if steps.len() >= 64 {
            return Err("field path exceeds 64 steps".into());
        }
        if let Some(tail) = path.strip_prefix('[') {
            let close = tail.find(']').ok_or("unterminated array index")?;
            let digits = &tail[..close];
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err("invalid array index".into());
            }
            let position = digits
                .parse::<usize>()
                .map_err(|_| "array index overflow")?;
            if position == 0 {
                return Err("array indices are one-based".into());
            }
            steps.push(FieldStep::Index(position - 1));
            path = &tail[close + 1..];
        } else {
            let end = path.find(['.', '[', ']']).unwrap_or(path.len());
            if end == 0 {
                return Err("expected a reflected field name".into());
            }
            steps.push(FieldStep::Name(&path[..end]));
            path = &path[end..];
        }
        if let Some(tail) = path.strip_prefix('.') {
            if tail.is_empty() || tail.starts_with(['.', '[', ']']) {
                return Err("expected a field name after '.'".into());
            }
            path = tail;
        } else if !path.is_empty() && !path.starts_with('[') {
            return Err("expected '.' or an array index in field path".into());
        }
    }
    Ok(steps)
}

fn inline_leaves(
    registry: &TypeRegistry,
    index: TypeIndex,
    base: usize,
    limit: usize,
    stack: &mut HashSet<TypeIndex>,
    leaves: &mut Vec<(PrimitiveType, std::ops::Range<usize>)>,
) -> Result<(), String> {
    if stack.len() >= 64 || !stack.insert(index) {
        return Err("cyclic or excessively nested inline field layout".into());
    }
    let ty = registry.get(index).ok_or("unresolved inline field type")?;
    let end = base
        .checked_add(ty.size as usize)
        .filter(|end| *end <= limit)
        .ok_or_else(|| {
            format!(
                "inline field outside reflected storage: {}",
                ty.qualified_name
            )
        })?;
    match ty.primitive_type {
        PrimitiveType::Typedef => {
            let inner = ty.inner_type.ok_or("missing typedef value type")?;
            inline_leaves(registry, inner, base, end, stack, leaves)?;
        }
        PrimitiveType::Struct => {
            for (_, field) in registry::fields(registry, index)? {
                let offset =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                let start = base.checked_add(offset).ok_or("field offset overflow")?;
                inline_leaves(registry, field.r#type, start, end, stack, leaves)?;
            }
        }
        PrimitiveType::StaticArray => {
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if (inner.size as usize).checked_mul(ty.field_count as usize) != Some(ty.size as usize)
            {
                return Err("static array size differs from reflected element count".into());
            }
            for position in 0..ty.field_count as usize {
                inline_leaves(
                    registry,
                    inner.index,
                    base + position * inner.size as usize,
                    end,
                    stack,
                    leaves,
                )?;
            }
        }
        PrimitiveType::DsArray
        | PrimitiveType::DsString
        | PrimitiveType::DsOptional
        | PrimitiveType::DsVariant
        | PrimitiveType::BlobArray
        | PrimitiveType::BlobString
        | PrimitiveType::BlobOptional
        | PrimitiveType::BlobVariant
        | PrimitiveType::ObjectReference => {
            return Err(format!(
                "{} has external storage or reference ownership. Select an inline sibling field",
                ty.qualified_name
            ));
        }
        kind => {
            let width = match kind {
                PrimitiveType::Bool
                | PrimitiveType::UInt8
                | PrimitiveType::SInt8
                | PrimitiveType::Bitmask8 => 1,
                PrimitiveType::UInt16 | PrimitiveType::SInt16 | PrimitiveType::Bitmask16 => 2,
                PrimitiveType::UInt32
                | PrimitiveType::SInt32
                | PrimitiveType::Float32
                | PrimitiveType::Bitmask32 => 4,
                PrimitiveType::UInt64
                | PrimitiveType::SInt64
                | PrimitiveType::Float64
                | PrimitiveType::Bitmask64 => 8,
                PrimitiveType::Guid => 16,
                PrimitiveType::Enum => {
                    let inner = ty
                        .inner_type
                        .and_then(|i| registry.get(i))
                        .ok_or("missing enum storage type")?;
                    if !matches!(
                        inner.primitive_type,
                        PrimitiveType::UInt8
                            | PrimitiveType::SInt8
                            | PrimitiveType::UInt16
                            | PrimitiveType::SInt16
                            | PrimitiveType::UInt32
                            | PrimitiveType::SInt32
                            | PrimitiveType::UInt64
                            | PrimitiveType::SInt64
                    ) {
                        return Err("enum has no integer storage type".into());
                    }
                    inner.size as usize
                }
                _ => {
                    return Err(format!(
                        "{} is not an inline value field",
                        ty.qualified_name
                    ));
                }
            };
            if ty.size as usize != width || leaves.len() >= 65_536 {
                return Err("invalid or excessively large inline field layout".into());
            }
            leaves.push((kind, base..end));
        }
    }
    stack.remove(&index);
    Ok(())
}

fn reject_external_overlap(
    registry: &TypeRegistry,
    index: TypeIndex,
    base: usize,
    selected: &std::ops::Range<usize>,
    stack: &mut HashSet<TypeIndex>,
) -> Result<(), String> {
    let ty = registry
        .get(index)
        .ok_or("unresolved enclosing field type")?;
    let end = base
        .checked_add(ty.size as usize)
        .ok_or("enclosing field offset overflow")?;
    if end <= selected.start || base >= selected.end {
        return Ok(());
    }
    if stack.len() >= 64 || !stack.insert(index) {
        return Err("cyclic or excessively nested enclosing field layout".into());
    }
    match ty.primitive_type {
        PrimitiveType::Typedef => {
            reject_external_overlap(
                registry,
                ty.inner_type.ok_or("missing typedef value type")?,
                base,
                selected,
                stack,
            )?;
        }
        PrimitiveType::Struct => {
            for (_, field) in registry::fields(registry, index)? {
                let relative =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                let offset = base.checked_add(relative).ok_or("field offset overflow")?;
                reject_external_overlap(registry, field.r#type, offset, selected, stack)?;
            }
        }
        PrimitiveType::StaticArray => {
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if inner.size == 0
                || (inner.size as usize).checked_mul(ty.field_count as usize)
                    != Some(ty.size as usize)
            {
                return Err("invalid enclosing static array layout".into());
            }
            let width = inner.size as usize;
            let first = selected.start.saturating_sub(base) / width;
            let last = (selected.end.min(end) - base - 1) / width;
            for position in first..=last {
                reject_external_overlap(
                    registry,
                    inner.index,
                    base + position * width,
                    selected,
                    stack,
                )?;
            }
        }
        PrimitiveType::DsArray
        | PrimitiveType::DsString
        | PrimitiveType::DsOptional
        | PrimitiveType::DsVariant
        | PrimitiveType::BlobArray
        | PrimitiveType::BlobString
        | PrimitiveType::BlobOptional
        | PrimitiveType::BlobVariant
        | PrimitiveType::ObjectReference => {
            return Err(format!(
                "selected field overlaps external storage or reference descriptor {}",
                ty.qualified_name
            ));
        }
        _ => {}
    }
    stack.remove(&index);
    Ok(())
}

/// Resolve names and one-based static indices, such as `transform.position[1]`.
/// Resolution follows reflected inline containment only, never live pointers.
pub(crate) fn inline_field(
    registry: &Rc<TypeRegistry>,
    component: TypeIndex,
    path: &str,
) -> Result<InlineField, String> {
    let root = registry.get(component).ok_or("unresolved component type")?;
    let mut index = component;
    let mut offset = 0usize;
    for step in field_path(path)? {
        let chain = registry::inheritance(registry, index)?;
        for current in chain {
            let ty = registry.get(current).ok_or("unresolved field type")?;
            if ty.primitive_type != PrimitiveType::Typedef {
                index = current;
                break;
            }
        }
        let ty = registry.get(index).ok_or("unresolved field type")?;
        let (child, relative) = if let FieldStep::Index(position) = step {
            if ty.primitive_type != PrimitiveType::StaticArray {
                return Err(format!("{} is not a static array", ty.qualified_name));
            }
            let inner = ty
                .inner_type
                .and_then(|i| registry.get(i))
                .ok_or("missing static array element type")?;
            if position >= ty.field_count as usize
                || (inner.size as usize).checked_mul(ty.field_count as usize)
                    != Some(ty.size as usize)
            {
                return Err("static array index or reflected layout is invalid".into());
            }
            (inner.index, position * inner.size as usize)
        } else {
            if ty.primitive_type != PrimitiveType::Struct {
                return Err(format!(
                    "{} has no reflected inline fields",
                    ty.qualified_name
                ));
            }
            let FieldStep::Name(name) = step else {
                unreachable!()
            };
            let fields = registry::fields(registry, index)?;
            let (_, field) = fields
                .iter()
                .find(|(_, field)| field.name == name)
                .ok_or_else(|| {
                    format!("reflected field not found: {}.{name}", ty.qualified_name)
                })?;
            (
                field.r#type,
                usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?,
            )
        };
        let child_type = registry.get(child).ok_or("unresolved field type")?;
        if relative
            .checked_add(child_type.size as usize)
            .is_none_or(|end| end > ty.size as usize)
        {
            return Err("field exceeds its reflected parent storage".into());
        }
        offset = offset
            .checked_add(relative)
            .ok_or("field offset overflow")?;
        index = child;
    }
    let ty = TypeHandle::new(registry.clone(), index);
    if offset
        .checked_add(ty.size as usize)
        .is_none_or(|end| end > root.size as usize)
    {
        return Err("field exceeds component storage".into());
    }
    let mut leaves = Vec::new();
    inline_leaves(
        registry,
        index,
        0,
        ty.size as usize,
        &mut HashSet::new(),
        &mut leaves,
    )?;
    leaves.sort_by_key(|(_, range)| range.start);
    if leaves.is_empty()
        || leaves
            .windows(2)
            .any(|pair| pair[0].1.end > pair[1].1.start)
    {
        return Err("inline field has empty or overlapping reflected storage".into());
    }
    reject_external_overlap(
        registry,
        component,
        0,
        &(offset..offset + ty.size as usize),
        &mut HashSet::new(),
    )?;
    Ok(InlineField { ty, offset, leaves })
}

/// Resolve one aligned 32-bit leaf inside an owned resource snapshot. The KFC
/// mapper selects fields, indices and concrete variants. Source offsets follow
/// its exact BlobArray/BlobVariant format, because that mapper does not expose
/// source positions. See kfc-resource/src/mapped/mod.rs:498-514,563-603 and
/// src/value/write.rs:817-821 in the pinned, unchanged parser dependency.
/// This scope never follows Ds pointers or changes container descriptors.
pub(crate) fn resource_field(
    registry: &Rc<TypeRegistry>,
    root_type: TypeIndex,
    path: &str,
    bytes: &[u8],
) -> Result<InlineField, String> {
    fn span(bytes: &[u8], offset: usize, size: usize) -> Result<(), String> {
        offset
            .checked_add(size)
            .filter(|end| *end <= bytes.len())
            .map(|_| ())
            .ok_or_else(|| "field outside owned resource snapshot".into())
    }
    fn u32_at(bytes: &[u8], offset: usize) -> Result<usize, String> {
        span(bytes, offset, 4)?;
        Ok(u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize)
    }
    let steps = field_path(path)?;
    let root = registry.get(root_type).ok_or("unresolved resource type")?;
    if bytes.len() > u32::MAX as usize {
        return Err("resource snapshot exceeds 32-bit blob offset range".into());
    }
    span(bytes, 0, root.size as usize)?;
    let owned: Rc<[u8]> = bytes.into();
    let mut mapped = MappedValue::from_bytes(registry, root, &owned).map_err(|e| e.to_string())?;
    let mut index = root_type;
    let mut offset = 0usize;
    let mut containers = Vec::new();
    let mut descriptors = Vec::new();
    for step in steps {
        for current in registry::inheritance(registry, index)? {
            if registry
                .get(current)
                .ok_or("unresolved resource field type")?
                .primitive_type
                != PrimitiveType::Typedef
            {
                index = current;
                break;
            }
        }
        let ty = registry
            .get(index)
            .ok_or("unresolved resource field type")?;
        span(bytes, offset, ty.size as usize)?;
        match (step, ty.primitive_type, &mapped) {
            (FieldStep::Name(name), PrimitiveType::Struct, MappedValue::Struct(value)) => {
                if !Rc::ptr_eq(value.data(), &owned) || value.r#type().index != index {
                    return Err("mapped struct does not belong to this resource scope".into());
                }
                let fields = registry::fields(registry, index)?;
                let (_, field) = fields
                    .iter()
                    .find(|(_, field)| field.name == name)
                    .ok_or_else(|| {
                        format!("reflected field not found: {}.{name}", ty.qualified_name)
                    })?;
                let child = registry
                    .get(field.r#type)
                    .ok_or("unresolved resource child type")?;
                let relative =
                    usize::try_from(field.data_offset).map_err(|_| "field offset overflow")?;
                if relative
                    .checked_add(child.size as usize)
                    .is_none_or(|end| end > ty.size as usize)
                {
                    return Err("field exceeds its reflected parent storage".into());
                }
                containers.push((index, offset));
                offset = offset
                    .checked_add(relative)
                    .ok_or("field offset overflow")?;
                index = child.index;
                span(bytes, offset, child.size as usize)?;
                mapped = value
                    .get(name)
                    .map_err(|e| e.to_string())?
                    .ok_or("mapped field missing")?;
            }
            (
                FieldStep::Index(position),
                PrimitiveType::StaticArray | PrimitiveType::BlobArray,
                MappedValue::Array(value),
            ) => {
                if !Rc::ptr_eq(value.data(), &owned) || value.r#type().index != index {
                    return Err("mapped array does not belong to this resource scope".into());
                }
                let inner = value.element_type();
                if inner.size == 0 || position >= value.len() {
                    return Err("resource array index or element size is invalid".into());
                }
                let length = value
                    .len()
                    .checked_mul(inner.size as usize)
                    .ok_or("array size overflow")?;
                let start = if ty.primitive_type == PrimitiveType::BlobArray {
                    if ty.size != 8 || value.len() != u32_at(bytes, offset + 4)? {
                        return Err("invalid reflected blob array descriptor".into());
                    }
                    descriptors.push(offset..offset + 8);
                    let start = offset
                        .checked_add(u32_at(bytes, offset)?)
                        .ok_or("blob offset overflow")?;
                    if start < offset + 8 || start < root.size as usize {
                        return Err("blob array payload overlaps its resource descriptors".into());
                    }
                    start
                } else {
                    if value.len() != ty.field_count as usize || length != ty.size as usize {
                        return Err("invalid reflected static array layout".into());
                    }
                    offset
                };
                span(bytes, start, length)?;
                offset = start
                    .checked_add(
                        position
                            .checked_mul(inner.size as usize)
                            .ok_or("array element offset overflow")?,
                    )
                    .ok_or("array element offset overflow")?;
                index = inner.index;
                mapped = value
                    .get(position)
                    .map_err(|e| e.to_string())?
                    .ok_or("mapped array element missing")?;
            }
            (FieldStep::Name("value"), PrimitiveType::BlobVariant, MappedValue::Variant(value)) => {
                if ty.size != 12
                    || value.base_type().index != index
                    || !Rc::ptr_eq(value.value().data(), &owned)
                {
                    return Err("mapped variant does not belong to this resource scope".into());
                }
                let concrete = value.variant_type();
                let relative = u32_at(bytes, offset + 4)?;
                if relative == 0
                    || u32_at(bytes, offset)? != concrete.qualified_hash as usize
                    || u32_at(bytes, offset + 8)? != concrete.size as usize
                {
                    return Err("invalid, absent or mismatched resource variant descriptor".into());
                }
                let base = registry
                    .get(ty.inner_type.ok_or("missing variant base type")?)
                    .ok_or("unresolved variant base type")?;
                if concrete.primitive_type != PrimitiveType::Struct
                    || !registry.is_sub_type(base, concrete)
                {
                    return Err("resource variant is not a reflected subtype".into());
                }
                descriptors.push(offset..offset + 12);
                let start = offset
                    .checked_add(4)
                    .and_then(|n| n.checked_add(relative))
                    .ok_or("variant payload offset overflow")?;
                if start < offset + 12 || start < root.size as usize {
                    return Err("variant payload overlaps its resource descriptors".into());
                }
                span(bytes, start, concrete.size as usize)?;
                offset = start;
                index = concrete.index;
                mapped = MappedValue::Struct(value.value().clone());
            }
            _ => {
                return Err(format!(
                    "{} does not support this owned resource field step",
                    ty.qualified_name
                ));
            }
        }
    }
    let ty = TypeHandle::new(registry.clone(), index);
    let mut leaves = Vec::new();
    inline_leaves(
        registry,
        index,
        0,
        ty.size as usize,
        &mut HashSet::new(),
        &mut leaves,
    )?;
    if ty.size != 4 || offset % 4 != 0 || leaves.len() != 1 || leaves[0].1 != (0..4) {
        return Err("resource writes require one aligned, contiguous 32-bit value leaf".into());
    }
    span(bytes, offset, 4)?;
    let selected = offset..offset + 4;
    if descriptors
        .iter()
        .any(|range| range.start < selected.end && selected.start < range.end)
    {
        return Err("resource field overlaps a traversed container descriptor".into());
    }
    for (container, base) in containers {
        reject_external_overlap(registry, container, base, &selected, &mut HashSet::new())?;
    }
    if leaves[0].0 == PrimitiveType::Float32
        && !f32::from_le_bytes(bytes[selected].try_into().unwrap()).is_finite()
    {
        return Err("resource field contains a nonfinite floating-point value".into());
    }
    Ok(InlineField { ty, offset, leaves })
}

fn supported(
    registry: &TypeRegistry,
    index: TypeIndex,
    seen: &mut HashSet<TypeIndex>,
) -> Result<(), String> {
    if !seen.insert(index) {
        return Ok(());
    }
    let ty = registry.get(index).ok_or("unresolved value type")?;
    if matches!(
        ty.primitive_type,
        PrimitiveType::DsArray
            | PrimitiveType::DsString
            | PrimitiveType::DsOptional
            | PrimitiveType::DsVariant
    ) {
        return Err(format!(
            "{} requires engine allocation and ownership. The KFC value codec cannot construct it",
            ty.qualified_name
        ));
    }
    registry::inheritance(registry, index)?;
    if let Some(inner) = ty.inner_type {
        supported(registry, inner, seen)?;
    }
    for field in ty.struct_fields.values() {
        supported(registry, field.r#type, seen)?;
    }
    Ok(())
}

fn handle(registry: &Rc<TypeRegistry>, selector: &LuaValue) -> Result<TypeHandle, String> {
    let index = registry::resolve(registry, selector)?;
    validate_owned_type(registry, index)?;
    Ok(TypeHandle::new(registry.clone(), index))
}

pub(crate) fn validate_owned_type(registry: &TypeRegistry, index: TypeIndex) -> Result<(), String> {
    supported(registry, index, &mut HashSet::new())
}

fn decode(lua: &mlua::Lua, ty: &TypeHandle, bytes: &[u8]) -> Result<LuaValue, String> {
    if bytes.len() < ty.size as usize {
        return Err(format!(
            "truncated {}: expected at least {} bytes",
            ty.qualified_name, ty.size
        ));
    }
    let owned: Rc<[u8]> = bytes.into();
    let value =
        MappedValue::from_bytes(ty.type_registry(), ty, &owned).map_err(|e| e.to_string())?;
    convert_value_to_lua(&value, lua).map_err(|e| e.to_string())
}

fn outcome(result: Result<LuaValue, String>) -> (Option<LuaValue>, Option<String>) {
    match result {
        Ok(value) => (Some(value), None),
        Err(reason) => (None, Some(reason)),
    }
}

pub(crate) fn create(lua: &mlua::Lua, registry: Rc<TypeRegistry>) -> mlua::Result<mlua::Table> {
    let table = lua.create_table()?;
    let registry_field = registry.clone();
    table.set("normalize_field", lua.create_function(move |lua, (selector, path, value): (LuaValue, String, LuaValue)| {
        let normalize = || -> Result<LuaValue, String> {
            let root = handle(&registry_field, &selector)?;
            let field = cached_inline_field(lua, &registry_field, root.index, &path)?;
            let mut bytes = vec![0; root.size as usize];
            let mut mask = vec![0; bytes.len()];
            field.write_into(lua, &value, &mut bytes, &mut mask)?;
            field.read(lua, &bytes)
        };
        Ok(outcome(normalize()))
    })?)?;
    let registry_decode = registry.clone();
    table.set(
        "decode",
        lua.create_function(move |lua, (selector, bytes): (LuaValue, mlua::String)| {
            Ok(outcome(
                handle(&registry_decode, &selector)
                    .and_then(|ty| decode(lua, &ty, &bytes.as_bytes())),
            ))
        })?,
    )?;
    let registry_new = registry.clone();
    table.set(
        "new",
        lua.create_function(move |lua, selector: LuaValue| {
            Ok(outcome(handle(&registry_new, &selector).and_then(|ty| {
                let bytes = ty
                    .default_value
                    .as_ref()
                    .ok_or_else(|| format!("no reflected default for {}", ty.qualified_name))?;
                decode(lua, &ty, bytes)
            })))
        })?,
    )?;
    table.set(
        "encode",
        lua.create_function(move |lua, (selector, value): (LuaValue, LuaValue)| {
            let result = handle(&registry, &selector).and_then(|ty| {
                let checked =
                    validate_and_clone_lua_value(&value, &ty, lua).map_err(|e| e.to_string())?;
                let value = convert_lua_to_value(&checked, &ty).map_err(|e| e.to_string())?;
                let bytes = value.to_bytes(&registry, &ty).map_err(|e| e.to_string())?;
                lua.create_string(bytes)
                    .map(LuaValue::String)
                    .map_err(|e| e.to_string())
            });
            Ok(outcome(result))
        })?,
    )?;
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resource_fixture() -> Rc<TypeRegistry> {
        let mut types = Vec::new();
        for (i, (name, kind, size, inner, flags, count)) in [
            ("F32", "FLOAT32", 4, None, "", 0),
            ("Component", "STRUCT", 0, None, "", 0),
            ("FallDamage", "STRUCT", 8, Some(1), "", 2),
            (
                "Variant",
                "BLOB_VARIANT",
                12,
                Some(1),
                "HAS_BLOB_VARIANT",
                1,
            ),
            ("Components", "BLOB_ARRAY", 8, Some(3), "HAS_BLOB_ARRAY", 0),
            (
                "Template",
                "STRUCT",
                8,
                None,
                "HAS_BLOB_ARRAY | HAS_BLOB_VARIANT",
                1,
            ),
            ("DsArray", "DS_ARRAY", 24, Some(0), "HAS_DS", 0),
            ("DsRoot", "STRUCT", 24, None, "HAS_DS", 1),
            ("Values", "BLOB_ARRAY", 8, Some(0), "HAS_BLOB_ARRAY", 0),
            ("ValueRoot", "STRUCT", 8, None, "HAS_BLOB_ARRAY", 1),
            ("Bool", "BOOL", 1, None, "", 0),
            ("BoolValues", "BLOB_ARRAY", 8, Some(10), "HAS_BLOB_ARRAY", 0),
            ("BoolRoot", "STRUCT", 8, None, "HAS_BLOB_ARRAY", 1),
            ("String", "BLOB_STRING", 8, None, "HAS_BLOB_STRING", 0),
            ("StringRoot", "STRUCT", 8, None, "HAS_BLOB_STRING", 1),
        ]
        .into_iter()
        .enumerate()
        {
            let qualified = format!("resource_test::{name}");
            types.push(
                serde_json::json!({"index":i,"name":name,"qualifiedName":qualified,
                "impactName":name,"size":size,"alignment":4,"elementAlignment":4,
                "fieldCount":count,"primitiveType":kind,"innerType":inner,"flags":flags,
                "nameHash":kfc::hash::fnv(name),"qualifiedHash":kfc::hash::fnv(&qualified),
                "impactHash":kfc::hash::fnv(name),"internalHash":i}),
            );
        }
        types[2]["structFields"] = serde_json::json!({
            "start":{"name":"start","type":0,"dataOffset":0},
            "lethal":{"name":"lethal","type":0,"dataOffset":4}});
        for (index, field, child) in [
            (5, "components", 4),
            (7, "items", 6),
            (9, "items", 8),
            (12, "items", 11),
            (14, "text", 13),
        ] {
            types[index]["structFields"] =
                serde_json::json!({field:{"name":field,"type":child,"dataOffset":0}});
        }
        Rc::new(
            serde_json::from_value(serde_json::json!({"version":"resource-fixture","types":types}))
                .unwrap(),
        )
    }

    fn resource_bytes(payload: usize) -> Vec<u8> {
        let mut bytes = vec![0; payload + 16];
        bytes[0..4].copy_from_slice(&8u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
        for (descriptor, start) in [(8, payload), (20, payload + 8)] {
            bytes[descriptor..descriptor + 4]
                .copy_from_slice(&kfc::hash::fnv("resource_test::FallDamage").to_le_bytes());
            bytes[descriptor + 4..descriptor + 8]
                .copy_from_slice(&((start - descriptor - 4) as u32).to_le_bytes());
            bytes[descriptor + 8..descriptor + 12].copy_from_slice(&8u32.to_le_bytes());
            bytes[start..start + 4].copy_from_slice(&8f32.to_le_bytes());
            bytes[start + 4..start + 8].copy_from_slice(&16f32.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn field_normalization_uses_the_live_writer_codec_and_float_domain() {
        let lua = mlua::Lua::new();
        lua.globals().set("values", create(&lua, resource_fixture()).unwrap()).unwrap();
        lua.load(r#"
            local canonical, reason = values.normalize_field('resource_test::FallDamage', 'start', 0.1)
            assert(reason == nil and canonical == 0.10000000149011612)
            local invalid, why = values.normalize_field('resource_test::FallDamage', 'start', math.huge)
            assert(invalid == nil and why)
            local dynamic, failure = values.normalize_field('resource_test::DsRoot', 'items[1]', 1)
            assert(dynamic == nil and failure)
        "#).exec().unwrap();
    }

    #[test]
    fn resource_blob_paths_match_the_pinned_mapper_and_change_only_the_selected_word() {
        let registry = resource_fixture();
        let lua = mlua::Lua::new();
        for payload in (32..=256).step_by(4) {
            let mut bytes = resource_bytes(payload);
            let owned: Rc<[u8]> = bytes.clone().into();
            let mapped = MappedValue::from_bytes(
                &registry,
                registry.get(TypeIndex::new(5)).unwrap(),
                &owned,
            )
            .unwrap();
            let components = mapped
                .as_struct()
                .unwrap()
                .get("components")
                .unwrap()
                .unwrap();
            for (position, name, expected) in [(1, "start", 8.0), (2, "lethal", 16.0)] {
                let path = format!("components[{position}].value.{name}");
                let field = resource_field(&registry, TypeIndex::new(5), &path, &bytes).unwrap();
                let variant = components
                    .as_array()
                    .unwrap()
                    .get(position - 1)
                    .unwrap()
                    .unwrap();
                let actual = variant
                    .as_variant()
                    .unwrap()
                    .value()
                    .get(name)
                    .unwrap()
                    .unwrap()
                    .as_f32()
                    .unwrap();
                let decoded = field.read(&lua, &bytes).unwrap();
                assert_eq!(actual, expected);
                assert!(matches!(decoded, LuaValue::Number(value) if value == actual as f64));
            }
            let field = resource_field(
                &registry,
                TypeIndex::new(5),
                "components[1].value.start",
                &bytes,
            )
            .unwrap();
            assert_eq!(field.byte_range(), payload..payload + 4);
            let before = bytes.clone();
            let mut mask = vec![0; bytes.len()];
            field
                .write_into(&lua, &LuaValue::Number(16.0), &mut bytes, &mut mask)
                .unwrap();
            assert_eq!(&bytes[payload..payload + 4], &16f32.to_le_bytes());
            for i in 0..bytes.len() {
                assert_eq!(mask[i], u8::from((payload..payload + 4).contains(&i)));
                if mask[i] == 0 {
                    assert_eq!(bytes[i], before[i]);
                }
            }
            assert!(
                field
                    .write_into(
                        &lua,
                        &LuaValue::Number(f64::INFINITY),
                        &mut bytes,
                        &mut vec![0; before.len()]
                    )
                    .is_err()
            );
        }
        assert!(inline_field(&registry, TypeIndex::new(5), "components[1].value.start").is_err());
    }

    #[test]
    fn resource_paths_reject_bad_indices_descriptors_truncation_and_nonfinite_values() {
        let registry = resource_fixture();
        let bytes = resource_bytes(32);
        let resolve =
            |path: &str, bytes: &[u8]| resource_field(&registry, TypeIndex::new(5), path, bytes);
        for path in [
            "",
            "components[0].value.start",
            "components[3].value.start",
            "components[-1].value.start",
            "components[+1].value.start",
            "components[1]value.start",
            "components[1].value.",
            "components[1].start",
            "components[1].type",
            "components[1].value",
            "components[1].value.missing",
        ] {
            assert!(resolve(path, &bytes).is_err(), "accepted {path}");
        }
        for length in 0..40 {
            assert!(resolve("components[1].value.start", &bytes[..length]).is_err());
        }
        for (offset, value) in [
            (0, u32::MAX),
            (0, 4),
            (4, u32::MAX),
            (8, 0),
            (12, u32::MAX),
            (12, 0),
            (16, 4),
        ] {
            let mut corrupt = bytes.clone();
            corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(
                resolve("components[1].value.start", &corrupt).is_err(),
                "accepted corruption {offset}={value}"
            );
        }
        let mut corrupt = bytes.clone();
        corrupt[32..36].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(resolve("components[1].value.start", &corrupt).is_err());
    }

    #[test]
    fn resource_scalar_arrays_are_bounded_and_ds_strings_and_bool_words_are_rejected() {
        let registry = resource_fixture();
        let mut bytes = vec![0; 16];
        bytes[0..4].copy_from_slice(&8u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&3f32.to_le_bytes());
        bytes[12..16].copy_from_slice(&7f32.to_le_bytes());
        assert_eq!(
            resource_field(&registry, TypeIndex::new(9), "items[2]", &bytes)
                .unwrap()
                .byte_range(),
            12..16
        );
        for (index, path) in [
            (7, "items[1]"),
            (12, "items[1]"),
            (14, "text"),
            (14, "text[1]"),
        ] {
            assert!(resource_field(&registry, TypeIndex::new(index), path, &[0; 24]).is_err());
        }
        let mut unaligned = bytes.clone();
        unaligned[0..4].copy_from_slice(&9u32.to_le_bytes());
        unaligned.push(0);
        assert!(resource_field(&registry, TypeIndex::new(9), "items[1]", &unaligned).is_err());
    }

    #[test]
    #[ignore = "set SHROUDFORGE_TEST_REGISTRY and SHROUDFORGE_TEST_RESOURCE_CAPTURE to owned exports"]
    fn installed_template_snapshot_resolves_resource_fields_when_explicitly_requested() {
        let registry: Rc<TypeRegistry> = Rc::new(
            serde_json::from_slice(
                &std::fs::read(std::env::var("SHROUDFORGE_TEST_REGISTRY").unwrap()).unwrap(),
            )
            .unwrap(),
        );
        let capture: serde_json::Value = serde_json::from_slice(
            &std::fs::read(std::env::var("SHROUDFORGE_TEST_RESOURCE_CAPTURE").unwrap()).unwrap(),
        )
        .unwrap();
        let lua = mlua::Lua::new();
        let root = registry
            .iter()
            .find(|ty| ty.qualified_name == "keen::ecs::TemplateResource")
            .unwrap();
        let config = registry
            .iter()
            .find(|ty| ty.qualified_name == "keen::ecs::FallDamage")
            .unwrap();
        let mut checked = 0;
        for service in capture["player"]["services"].as_array().unwrap() {
            for resource in service["templates"].as_array().unwrap() {
                let bytes: Vec<u8> = resource["resourceBytes"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                    .collect();
                assert_eq!(bytes.len(), resource["size"].as_u64().unwrap() as usize);
                for variant in resource["fallDamage"].as_array().unwrap() {
                    for (name, value_position) in [
                        ("fallDamageStartDistance", 0),
                        ("fallDamageLethalDistance", 1),
                    ] {
                        let path = format!(
                            "components[{}].value.{name}",
                            variant["ordinal"].as_u64().unwrap()
                        );
                        let field = resource_field(&registry, root.index, &path, &bytes).unwrap();
                        let expected_offset = variant["payloadOffset"].as_u64().unwrap() as usize
                            + config.struct_fields[name].data_offset as usize;
                        assert_eq!(field.byte_range(), expected_offset..expected_offset + 4);
                        let expected = variant["values"][value_position].as_f64().unwrap();
                        assert!(
                            matches!(field.read(&lua, &bytes).unwrap(), LuaValue::Number(value) if value == expected)
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 0);
        eprintln!(
            "Production resource walker resolved {checked} original fall-damage fields through the pinned KFC mapper"
        );
    }

    #[test]
    fn lua_values_round_trip_inherited_fields_defaults_and_validate_errors() {
        let lua = mlua::Lua::new();
        let registry = Rc::new(super::super::registry::tests::fixture());
        lua.globals()
            .set("values", create(&lua, registry).unwrap())
            .unwrap();
        lua.load(
            r#"
            local a, reason = values.new('test::Child')
            assert(a and not reason and a.x == 17 and a.y == 23)
            local b = assert(values.new('test::Child'))
            a.x = 1234
            assert(b.x == 17)
            local bytes = assert(values.encode('test::Child', a))
            assert(#bytes == 8)
            local decoded = assert(values.decode('test::Child', bytes))
            assert(decoded.x == 1234 and decoded.y == 23)
            local missing, why = values.decode('test::Child', 'short')
            assert(missing == nil and why:find('truncated'))
            local invalid, error = values.encode('test::U32', -1)
            assert(invalid == nil and error)
            local ds, ds_error = values.new('test::HasDs')
            assert(ds == nil and ds_error:find('engine allocation'))
        "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn named_byte_enum_expected_fields_use_the_production_codec_and_preserve_siblings() {
        let lua = mlua::Lua::new();
        let mut fixture = serde_json::to_value(super::super::registry::tests::fixture()).unwrap();
        let types = fixture["types"].as_array_mut().unwrap();
        types.push(serde_json::json!({"index":6,"name":"State","qualifiedName":"test::State","impactName":"State",
            "size":1,"alignment":1,"elementAlignment":1,"fieldCount":2,"primitiveType":"ENUM","flags":"","innerType":8,
            "nameHash":1,"impactHash":2,"qualifiedHash":3,"internalHash":4,
            "enumFields":{"Walking":{"name":"Walking","value":2},"BuildingFlying":{"name":"BuildingFlying","value":11}}}));
        types.push(serde_json::json!({"index":7,"name":"Locomotion","qualifiedName":"test::Locomotion","impactName":"Locomotion",
            "size":8,"alignment":4,"elementAlignment":4,"fieldCount":2,"primitiveType":"STRUCT","flags":"",
            "nameHash":5,"impactHash":6,"qualifiedHash":7,"internalHash":8,
            "structFields":{"counter":{"name":"counter","type":0,"dataOffset":0},"state":{"name":"state","type":6,"dataOffset":5}}}));
        types.push(serde_json::json!({"index":8,"name":"U8","qualifiedName":"test::U8","impactName":"U8",
            "size":1,"alignment":1,"elementAlignment":1,"fieldCount":0,"primitiveType":"UINT8","flags":"",
            "nameHash":9,"impactHash":10,"qualifiedHash":11,"internalHash":12}));
        let registry = Rc::new(serde_json::from_value::<TypeRegistry>(fixture).unwrap());
        let field = inline_field(&registry, TypeIndex::new(7), "state").unwrap();
        let before = [99, 0, 0, 0, 0xa5, 2, 0x5a, 0xff];
        assert_eq!(
            field
                .read(&lua, &before)
                .unwrap()
                .as_string()
                .unwrap()
                .to_str()
                .unwrap(),
            "Walking"
        );
        let expected = lua.create_table().unwrap();
        expected.set("state", "Walking").unwrap();
        expect_inline_fields(&lua, &registry, TypeIndex::new(7), &before, &expected).unwrap();
        let mut after = before;
        let mut mask = [0; 8];
        field
            .write_into(
                &lua,
                &LuaValue::String(lua.create_string("BuildingFlying").unwrap()),
                &mut after,
                &mut mask,
            )
            .unwrap();
        assert_eq!(after, [99, 0, 0, 0, 0xa5, 11, 0x5a, 0xff]);
        assert_eq!(mask, [0, 0, 0, 0, 0, 1, 0, 0]);
        assert!(
            expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected)
                .unwrap_err()
                .contains("snapshot changed")
        );
        expected.set("state", "BuildingFlying").unwrap();
        expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected).unwrap();
        expected.set("state", "InventedState").unwrap();
        assert!(
            expect_inline_fields(&lua, &registry, TypeIndex::new(7), &after, &expected).is_err()
        );
        assert!(
            expect_inline_fields(
                &lua,
                &registry,
                TypeIndex::new(7),
                &after,
                &lua.create_table().unwrap()
            )
            .is_err()
        );
    }

    #[test]
    #[ignore = "set SHROUDFORGE_TEST_REGISTRY to an installed parser cache"]
    fn installed_registries_resolve_all_indices_when_explicitly_requested() {
        use mlua::ObjectLike;
        let path =
            std::env::var("SHROUDFORGE_TEST_REGISTRY").expect("registry cache path required");
        let registry: Rc<TypeRegistry> =
            Rc::new(serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap());
        let lua = mlua::Lua::new();
        let definitions = crate::definition::generator::generate(&registry);
        for ty in registry.iter() {
            assert_eq!(
                registry::resolve(&registry, &LuaValue::Integer(ty.qualified_hash as i64)).unwrap(),
                ty.index
            );
            registry::inheritance(&registry, ty.index).unwrap();
            registry::fields(&registry, ty.index).unwrap();
            let value = lua
                .create_userdata(super::super::Type::new(TypeHandle::new(
                    registry.clone(),
                    ty.index,
                )))
                .unwrap();
            assert_eq!(value.get::<u8>("flags_bits").unwrap(), ty.flags.bits());
            assert_eq!(value.get::<usize>("index").unwrap(), ty.index.as_usize());
            assert_eq!(
                value.get::<u32>("qualified_hash").unwrap(),
                ty.qualified_hash
            );
            if ty.primitive_type == PrimitiveType::Struct {
                assert!(
                    definitions.contains(&format!(
                        "---@class {}",
                        ty.qualified_name.replace("::", ".")
                    )),
                    "missing generated type: {}",
                    ty.qualified_name
                );
            }
        }
        eprintln!(
            "Lua metadata verified for all {} types ({})",
            registry.len(),
            registry.version
        );
        if let Ok(path) = std::env::var("SHROUDFORGE_TEST_ECS_SNAPSHOTS") {
            let capture: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let snapshots = capture["snapshots"]
                .as_array()
                .expect("capture snapshots required");
            let values = create(&lua, registry.clone()).unwrap();
            let owned_decode = values.get::<mlua::Function>("decode").unwrap();
            let mut results = Vec::new();
            let mut whole_count = 0;
            let mut field_count = 0;
            let mut scalar_count = 0;
            let mut partially_decoded = 0;
            for snapshot in snapshots {
                let name = snapshot["component"].as_str().unwrap();
                let index = registry::resolve(
                    &registry,
                    &LuaValue::String(lua.create_string(name).unwrap()),
                )
                .unwrap();
                let ty = registry.get(index).unwrap();
                assert_eq!(
                    snapshot["qualifiedHash"].as_u64(),
                    Some(u64::from(ty.qualified_hash))
                );
                let hex = snapshot["bytes"].as_str().unwrap();
                assert_eq!(hex.len(), ty.size as usize * 2);
                let bytes: Vec<_> = hex
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| {
                        ((pair[0] as char).to_digit(16).unwrap() * 16
                            + (pair[1] as char).to_digit(16).unwrap()) as u8
                    })
                    .collect();
                assert_eq!(bytes.len(), ty.size as usize);
                let (whole, whole_reason): (LuaValue, Option<String>) = owned_decode
                    .call((name, lua.create_string(&bytes).unwrap()))
                    .unwrap();
                let whole_ok = whole_reason.is_none() && !whole.is_nil();
                whole_count += usize::from(whole_ok);
                let decoded = lua.create_table().unwrap();
                let mut fields_ok = 0;
                let mut field_errors = serde_json::Map::new();
                for (_, field) in registry::fields(&registry, index).unwrap() {
                    match inline_field(&registry, index, &field.name)
                        .and_then(|field| field.read(&lua, &bytes))
                    {
                        Ok(value) => {
                            decoded.raw_set(field.name.as_str(), value).unwrap();
                            fields_ok += 1;
                        }
                        Err(reason) => {
                            field_errors.insert(field.name.clone(), serde_json::json!(reason));
                        }
                    }
                }
                for (field, expected) in snapshot["inlineScalarFields"]
                    .as_object()
                    .into_iter()
                    .flatten()
                {
                    let value = decoded.raw_get::<LuaValue>(field.as_str()).unwrap();
                    match value {
                        LuaValue::Boolean(value) => {
                            assert_eq!(expected.as_bool(), Some(value), "{name}.{field}")
                        }
                        LuaValue::Integer(value) => {
                            assert_eq!(expected.as_f64(), Some(value as f64), "{name}.{field}")
                        }
                        LuaValue::Number(value) => {
                            let field_type = registry::fields(&registry, index)
                                .unwrap()
                                .iter()
                                .find(|(_, metadata)| metadata.name == *field)
                                .and_then(|(_, metadata)| registry.get(metadata.r#type))
                                .unwrap();
                            if field_type.primitive_type == PrimitiveType::Float32 {
                                // JSON parsing can shift the f64 representation of
                                // f32::MAX by one f64 ULP. Compare the actual declared
                                // f32 representation, without an epsilon tolerance.
                                assert_eq!(
                                    (expected.as_f64().unwrap() as f32).to_bits(),
                                    (value as f32).to_bits(),
                                    "{name}.{field}"
                                );
                            } else {
                                assert_eq!(expected.as_f64(), Some(value), "{name}.{field}");
                            }
                        }
                        _ => panic!(
                            "capture scalar did not decode through production field codec: {name}.{field}"
                        ),
                    }
                    scalar_count += 1;
                }
                if name == "keen::ecs::NetworkPlayerAttributes" {
                    lua.globals()
                        .set("captured_player_fields", decoded.clone())
                        .unwrap();
                    lua.load("assert(type(captured_player_fields.health) == 'number'); assert(type(captured_player_fields.stamina) == 'number'); assert(type(captured_player_fields.movementSprint) == 'number')").exec().unwrap();
                }
                field_count += fields_ok;
                partially_decoded += usize::from(!whole_ok && fields_ok > 0);
                results.push(serde_json::json!({"component":name,"whole_value_decoded":whole_ok,"whole_value_reason":whole_reason,
                    "inline_fields_decoded":fields_ok,"inline_field_reasons":field_errors,
                    "identity_only":!whole_ok && fields_ok == 0}));
            }
            let report = serde_json::json!({"source_capture":path,"capture_execution":"external-read-only-process-snapshot",
                "new_dll_or_live_lua_verified":false,"type_count":registry.len(),"snapshot_count":snapshots.len(),
                "whole_values_decoded":whole_count,"partial_values_decoded":partially_decoded,"inline_fields_decoded":field_count,
                "capture_scalars_compared":scalar_count,"entries":results});
            std::fs::write(
                format!("{path}.lua-codec.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            eprintln!(
                "Production Lua codecs checked {} external snapshots: {whole_count} whole values, {partially_decoded} partial values, {field_count} inline fields, {scalar_count} captured scalars compared",
                snapshots.len()
            );
        }
    }
}
