use super::*;
use crate::env::loader::runtime_provider;
use crate::lua::values as runtime_values;
use crate::session::loaded_resource_indexes::{Lookup, Progress, Projection, Selector};
use crate::type_aliases::TypeHandle;
use kfc::reflection::{PrimitiveType, TypeIndex, TypeRegistry};
use mod_loader::Mod;
use std::{collections::HashSet, rc::Rc, time::Instant};
struct ScalarField {
    field: Rc<runtime_values::InlineField>,
    kind: PrimitiveType,
    root_size: usize,
}

impl ScalarField {
    #[cfg(test)]
    fn resolve(registry: &Rc<TypeRegistry>, root: TypeIndex, path: &str) -> Result<Self, String> {
        if path.is_empty() || path.len() > 1024 {
            return Err("indexed field path must contain 1 to 1024 bytes".into());
        }
        // inline_field rejects DS pointers, blob traversal and overlapping
        // descriptors. The source offset is fixed for every resource of this type.
        let field = Rc::new(runtime_values::inline_field(registry, root, path)?);
        Self::from_field(registry, root, field)
    }

    fn from_field(
        registry: &Rc<TypeRegistry>,
        root: TypeIndex,
        field: Rc<runtime_values::InlineField>,
    ) -> Result<Self, String> {
        let range = field.byte_range();
        if range.len() != 4 || range.start % 4 != 0 {
            return Err("loaded resource indexes require an aligned 32-bit scalar field".into());
        }
        let mut index = field.type_index();
        let mut seen = HashSet::new();
        let kind = loop {
            if !seen.insert(index) || seen.len() > 64 {
                return Err("cyclic indexed scalar typedef".into());
            }
            let ty = registry
                .get(index)
                .ok_or("unresolved indexed scalar type")?;
            if ty.size != 4 {
                return Err("indexed scalar typedef has an inconsistent storage size".into());
            }
            if ty.primitive_type != PrimitiveType::Typedef {
                break ty.primitive_type;
            }
            index = ty.inner_type.ok_or("indexed typedef has no value type")?;
        };
        if !matches!(
            kind,
            PrimitiveType::UInt32
                | PrimitiveType::SInt32
                | PrimitiveType::Float32
                | PrimitiveType::Enum
        ) {
            return Err(
                "loaded resource indexes support 32-bit integers, finite floats and enums".into(),
            );
        }
        Ok(Self {
            field,
            kind,
            root_size: registry.get(root).ok_or("unresolved resource type")?.size as usize,
        })
    }

    fn wanted(
        &self,
        lua: &mlua::Lua,
        registry: &Rc<TypeRegistry>,
        value: &LuaValue,
    ) -> Result<[u8; 4], String> {
        let ty = TypeHandle::new(registry.clone(), self.field.type_index());
        let checked = super::value::validate_and_clone_lua_value(value, &ty, lua)
            .map_err(|e| e.to_string())?;
        let encoded = super::value::convert_lua_to_value(&checked, &ty)
            .map_err(|e| e.to_string())?
            .to_bytes(registry, &ty)
            .map_err(|e| e.to_string())?;
        runtime_values::validate_runtime_scalar(self.kind, &encoded)?;
        encoded
            .try_into()
            .map_err(|_| "indexed scalar codec did not produce four bytes".into())
    }

    fn key(&self, bytes: &[u8]) -> Result<[u8; 4], String> {
        if bytes.len() < self.root_size {
            return Err("indexed resource snapshot is smaller than its reflected root".into());
        }
        let scalar = bytes
            .get(self.field.byte_range())
            .ok_or("indexed scalar is outside resource snapshot")?;
        runtime_values::validate_runtime_scalar(self.kind, scalar)?;
        scalar
            .try_into()
            .map_err(|_| "indexed scalar has an invalid width".into())
    }
}

fn require_session(expected: u64) -> Result<(), String> {
    // Session zero remains a supported read-only menu snapshot.
    if runtime_provider::world_session_id() != expected {
        return Err("world session changed during loaded resource indexing".into());
    }
    Ok(())
}

fn request(
    state: &AppState,
    args: &FunctionArgs,
) -> Result<(Selector, ScalarField, Projection), String> {
    if state.phase() != crate::RuntimePhase::Ingame {
        return Err("loaded ResourceService access is available only in the ingame phase".into());
    }
    let registry = state.type_registry();
    let metadata = get_type(args, 0, registry.as_ref()).map_err(|error| error.to_string())?;
    let path = args.get::<String>(1).map_err(|error| error.to_string())?;
    if path.is_empty() || path.len() > 1024 {
        return Err("indexed field path must contain 1 to 1024 bytes".into());
    }
    let scalar = ScalarField::from_field(
        registry,
        metadata.index,
        state.inline_field(metadata.index, &path)?,
    )?;
    let type_id = kfc_format::resource_id(registry, &metadata.qualified_name, Guid::NONE, 0)
        .ok_or("indexed resource type is absent from the active reflection")?;
    let projection = Projection {
        type_hash: type_id.type_hash(),
        root_size: scalar.root_size,
        offset: scalar.field.byte_range().start,
        kind: scalar.kind,
    };
    Ok((
        Selector {
            root: metadata.index,
            path,
        },
        scalar,
        projection,
    ))
}

fn progress_table(lua: &mlua::Lua, progress: Progress) -> mlua::Result<Table> {
    let result = lua.create_table()?;
    result.raw_set("session_id", progress.session_id)?;
    result.raw_set("scanned", progress.scanned)?;
    result.raw_set("total", progress.total)?;
    result.raw_set("payload_reads", progress.payload_reads)?;
    result.raw_set("complete", progress.complete)?;
    result.raw_set("phase", progress.phase)?;
    Ok(result)
}

/// Idempotent preparation declaration. It survives a later nonzero world
/// session and is removed with its owner, not after a particular gameplay action.
pub(crate) fn prepare(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(bool, Option<String>, Option<Table>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let prepare = || -> Result<(Progress, Option<String>), String> {
        if let Some(reason) =
            crate::env::loader::runtime_denial_reason(&state, owner, "runtime.ecs.read")
        {
            return Err(reason);
        }
        if args.len() != 2 {
            return Err(
                "prepare_loaded_index expects a type and fixed inline scalar field path".into(),
            );
        }
        let (selector, _, projection) = request(&state, &args)?;
        let mut indexes = state.loaded_resource_indexes();
        let position = indexes.register(
            runtime_provider::world_session_id(),
            &owner.info().id,
            selector,
            projection,
            Instant::now(),
        )?;
        let progress = indexes.progress(position);
        let reason = if progress.complete {
            None
        } else {
            Some(
                indexes
                    .pending_reason(position)
                    .unwrap_or_else(|| "loaded resource index preparation is queued".into()),
            )
        };
        Ok((progress, reason))
    };
    Ok(match prepare() {
        Ok((progress, reason)) => (
            progress.complete,
            reason,
            Some(progress_table(lua, progress)?),
        ),
        Err(reason) => (false, Some(reason), None),
    })
}

pub(super) fn get(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<Resource>, Option<String>, Option<Table>)> {
    let state = lua.app_data_ref::<AppState>().unwrap();
    let lookup = || -> Result<Lookup, String> {
        if let Some(reason) =
            crate::env::loader::runtime_denial_reason(&state, owner, "runtime.ecs.read")
        {
            return Err(reason);
        }
        if args.len() != 3 {
            return Err("get_loaded_resource_by_field expects a type, fixed inline scalar field path and value".into());
        }
        let (selector, scalar, projection) = request(&state, &args)?;
        let wanted = scalar.wanted(
            lua,
            state.type_registry(),
            args.get::<LuaValue>(2).map_err(|error| error.to_string())?,
        )?;
        let session = runtime_provider::world_session_id();
        let mut indexes = state.loaded_resource_indexes();
        let position = indexes.register(
            session,
            &owner.info().id,
            selector,
            projection,
            Instant::now(),
        )?;
        let result = indexes.lookup(
            position,
            wanted,
            Instant::now(),
            |id| {
                require_session(session)?;
                let bytes = runtime_provider::resource_read(id)?;
                require_session(session)?;
                Ok(bytes)
            },
            |bytes| scalar.key(bytes),
        );
        if let Err(reason) = require_session(session) {
            indexes.observe_session(runtime_provider::world_session_id(), Instant::now());
            return Err(reason);
        }
        result
    };
    Ok(match lookup() {
        Ok(result) => {
            let resource = result.resource.map(|(id, bytes)| {
                Resource::new(state.get_loaded_resource_info_from_snapshot(
                    &id,
                    result.progress.session_id,
                    bytes,
                ))
            });
            (
                resource,
                result.reason,
                Some(progress_table(lua, result.progress)?),
            )
        }
        Err(reason) => (None, Some(reason), None),
    })
}

/// The single runtime owner advances every registered job. No Lua callback or
/// repeated mod lookup is needed to finish its initial index.
pub(crate) fn tick(state: &AppState) {
    if state.phase() != crate::RuntimePhase::Ingame {
        return;
    }
    let mut indexes = state.loaded_resource_indexes();
    if indexes.is_empty() {
        return;
    }
    let session = runtime_provider::world_session_id();
    indexes.advance(
        session,
        Instant::now(),
        |type_hash| {
            require_session(session)?;
            let resources = runtime_provider::resources_by_type(type_hash)?;
            require_session(session)?;
            Ok(resources)
        },
        |ids, projection| {
            require_session(session)?;
            let keys = runtime_provider::resource_scalar_batch(
                ids,
                projection.type_hash,
                projection.root_size,
                projection.offset,
                session,
            )?;
            require_session(session)?;
            Ok(keys)
        },
    );
    if runtime_provider::world_session_id() != session {
        indexes.observe_session(runtime_provider::world_session_id(), Instant::now());
    }
}

#[cfg(test)]
#[path = "index_scalar_tests.rs"]
mod tests;
