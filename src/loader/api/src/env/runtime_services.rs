//! Session-bound reflection views into original named DI services.
use super::{AppState, loader, registry, runtime_values, util::add_function_with_mod};
use crate::lua::{FunctionArgs, LuaValue};
use kfc::reflection::{PrimitiveType, TypeIndex};
use mlua::{MultiValue, Table, UserData};
use mod_loader::Mod;
use std::{cell::RefCell, collections::BTreeSet};

/// Owns identity and session only. Every operation resolves the actual original
/// named binding again. A cached process pointer never escapes the provider.
pub(crate) struct RuntimeService {
    owner_mod: Mod,
    key: String,
    service_type: TypeIndex,
    type_hash: u32,
    size: u32,
    session: u64,
    write_fault: RefCell<Option<String>>,
}

pub(crate) fn create(lua: &mlua::Lua, owner_mod: &Mod) -> mlua::Result<Table> {
    let services = lua.create_table()?;
    add_function_with_mod(lua, &services, "get", owner_mod, get)?;
    Ok(services)
}

fn require_access(state: &AppState, owner_mod: &Mod, operation: &str) -> Result<(), String> {
    if let Some(reason) = loader::runtime_denial_reason(state, owner_mod, operation) {
        return Err(reason);
    }
    if !loader::available(state, owner_mod, operation) {
        return Err(format!("{operation} is currently unavailable"));
    }
    Ok(())
}

fn require_session(session: u64) -> Result<(), String> {
    if session == 0 || loader::runtime_provider::world_session_id() != session {
        return Err("service view belongs to another or an inactive world session".into());
    }
    Ok(())
}

fn get(
    lua: &mlua::Lua,
    args: FunctionArgs,
    owner_mod: &Mod,
) -> mlua::Result<(Option<RuntimeService>, Option<String>)> {
    let resolve = || -> Result<RuntimeService, String> {
        let state = lua.app_data_ref::<AppState>().unwrap();
        require_access(&state, owner_mod, "runtime.ecs.read")?;
        if args.len() != 1 {
            return Err("runtime.services.get expects one original service name".into());
        }
        let key = args.get::<String>(0).map_err(|e| e.to_string())?;
        if key.is_empty() || key.len() > 127 || key.contains('\0') || key.starts_with("keen::") {
            return Err("use the exact original DI name, for example ecs::GameSettings".into());
        }
        let qualified_name = format!("keen::{key}");
        let types = state.type_registry();
        let metadata = types
            .iter()
            .find(|ty| ty.qualified_name == qualified_name)
            .ok_or_else(|| {
                format!("original service has no active reflected type {qualified_name}")
            })?;
        if metadata.primitive_type != PrimitiveType::Struct
            || metadata.size == 0
            || metadata.size > 65536
        {
            return Err("named service requires a bounded reflected plain-data struct".into());
        }
        // A service contract covers its whole object. Reject any pointer,
        // dynamic storage, reference or overlapping descriptor at creation.
        let fields = registry::fields(types, metadata.index)?;
        if fields.is_empty() {
            return Err("named service has no reflected inline fields".into());
        }
        for (_, field) in fields {
            runtime_values::inline_field(types, metadata.index, &field.name)?;
        }
        let session = loader::runtime_provider::world_session_id();
        require_session(session)?;
        loader::runtime_provider::service_read(&key, metadata.qualified_hash, metadata.size, session)
            .map_err(|reason| format!("{key} needs an admitted original plain-data contract and an available authoritative Scene: {reason}"))?;
        require_session(session)?;
        Ok(RuntimeService {
            owner_mod: owner_mod.clone(),
            key,
            service_type: metadata.index,
            type_hash: metadata.qualified_hash,
            size: metadata.size,
            session,
            write_fault: RefCell::new(None),
        })
    };
    Ok(match resolve() {
        Ok(service) => (Some(service), None),
        Err(reason) => (None, Some(reason)),
    })
}

impl RuntimeService {
    pub(crate) fn ownership_identity(&self, owner: &Mod) -> Result<(&str, u64), String> {
        if self.owner_mod.info().id != owner.info().id {
            return Err("service view belongs to a different runtime mod".into());
        }
        Ok((&self.key, self.session))
    }

    fn snapshot(&self, state: &AppState) -> Result<Vec<u8>, String> {
        require_access(state, &self.owner_mod, "runtime.ecs.read")?;
        require_session(self.session)?;
        let ty = state
            .type_registry()
            .get(self.service_type)
            .ok_or("service reflection is unavailable")?;
        if ty.qualified_name != format!("keen::{}", self.key)
            || ty.qualified_hash != self.type_hash
            || ty.size != self.size
        {
            return Err("service reflection changed since view creation".into());
        }
        let bytes = loader::runtime_provider::service_read(
            &self.key,
            self.type_hash,
            self.size,
            self.session,
        )?;
        require_session(self.session)?;
        Ok(bytes)
    }

    fn read_fields(&self, lua: &mlua::Lua, args: MultiValue) -> Result<Table, String> {
        if args.len() != 1 {
            return Err("read_fields expects one dense array of reflected field paths".into());
        }
        let LuaValue::Table(paths) = &args[0] else {
            return Err("read_fields expects a dense array of field paths".into());
        };
        let mut selected = BTreeSet::new();
        for pair in paths.clone().pairs::<LuaValue, LuaValue>() {
            let (key, path) = pair.map_err(|e| e.to_string())?;
            let LuaValue::Integer(index) = key else {
                return Err("field paths must be a dense array".into());
            };
            if index <= 0 || index as usize > paths.raw_len() || selected.len() >= 1024 {
                return Err("select a dense array of at most 1024 field paths".into());
            }
            let LuaValue::String(path) = path else {
                return Err("field paths must be strings".into());
            };
            if !selected.insert(path.to_str().map_err(|e| e.to_string())?.to_owned()) {
                return Err("duplicate field path".into());
            }
        }
        if selected.is_empty() || selected.len() != paths.raw_len() {
            return Err("select a nonempty dense array of field paths".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let bytes = self.snapshot(&state)?;
        let result = lua
            .create_table_with_capacity(0, selected.len())
            .map_err(|e| e.to_string())?;
        for path in selected {
            let field =
                runtime_values::inline_field(state.type_registry(), self.service_type, &path)?;
            result
                .raw_set(path.as_str(), field.read(lua, &bytes)?)
                .map_err(|e| e.to_string())?;
        }
        require_session(self.session)?;
        Ok(result)
    }

    fn write_field(&self, lua: &mlua::Lua, args: MultiValue) -> Result<(), String> {
        if let Some(reason) = self.write_fault.borrow().as_ref() {
            return Err(reason.clone());
        }
        if args.len() != 3 && args.len() != 4 {
            return Err("write_field expects path, value, expected current value and optional expected_fields".into());
        }
        let LuaValue::String(path) = &args[0] else {
            return Err("write_field expects a reflected field path".into());
        };
        let path = path.to_str().map_err(|e| e.to_string())?;
        let state = lua.app_data_ref::<AppState>().unwrap();
        require_access(&state, &self.owner_mod, "runtime.ecs.write")?;
        let before = self.snapshot(&state)?;
        let field = runtime_values::inline_field(state.type_registry(), self.service_type, &path)?;
        let mut expected = before.clone();
        let mut expected_mask = vec![0; before.len()];
        field.write_into(lua, &args[2], &mut expected, &mut expected_mask)?;
        if expected != before {
            return Err(
                "service field changed before update. Read current fields and retry".into(),
            );
        }
        if args.len() == 4 {
            let LuaValue::Table(expected_fields) = &args[3] else {
                return Err("expected_fields must be a nonempty field-path/value map".into());
            };
            runtime_values::expect_inline_fields(
                lua,
                state.type_registry(),
                self.service_type,
                &before,
                expected_fields,
            )?;
        }
        let mut after = before.clone();
        let mut mask = vec![0; before.len()];
        field.write_into(lua, &args[1], &mut after, &mut mask)?;
        require_session(self.session)?;
        super::runtime_ownership::check_write(&state, &self.owner_mod,
            &super::runtime_ownership::Target::Service { session: self.session, key: self.key.clone() })?;
        if let Err(reason) = loader::runtime_provider::service_compare_exchange(
            &self.key,
            self.type_hash,
            self.size,
            self.session,
            &before,
            &mask,
            &after,
        ) {
            if reason.to_lowercase().contains("uncertain") {
                state.record_uncertain_runtime_write(self.session, &reason);
                *self.write_fault.borrow_mut() = Some(reason.clone());
            }
            return Err(reason);
        }
        // The native CAS freshly reads back the entire resulting service under
        // the original Scene gate and revalidates its binding before returning.
        // No second unlocked observation is mistaken for the CAS outcome.
        Ok(())
    }
}

impl UserData for RuntimeService {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("key", |_, this| Ok(this.key.clone()));
        fields.add_field_method_get("scope", |_, _| Ok("host-world-shared"));
        fields.add_field_method_get("session_id", |_, this| Ok(this.session));
        fields.add_field_method_get("write_supported", |_, _| Ok(true));
        fields.add_field_method_get("type", |lua, this| {
            lua.app_data_ref::<AppState>()
                .unwrap()
                .get_type(lua, this.service_type)
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("read_fields", |lua, this, args: MultiValue| {
            Ok(match this.read_fields(lua, args) {
                Ok(values) => (Some(values), None),
                Err(reason) => (None, Some(reason)),
            })
        });
        methods.add_method("write_field", |lua, this, args: MultiValue| {
            Ok(match this.write_field(lua, args) {
                Ok(()) => (true, None),
                Err(reason) => (false, Some(reason)),
            })
        });
    }
}
