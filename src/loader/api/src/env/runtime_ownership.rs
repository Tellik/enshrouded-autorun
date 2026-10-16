//! Cooperative exclusive claims for the original storage changed by Lua mods.
//! Claims store no baseline or pointers. A leaked active token keeps its claim,
//! because silently releasing it would let another mod capture a changed value.
use super::{AppState, game::Resource, loader, util::add_function_with_mod};
use crate::lua::FunctionArgs;
use kfc::guid::ResourceId;
use mlua::{Lua, Table, UserData};
use mod_loader::Mod;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Ecs {
        session: u64,
        entity: u32,
        qualified_name: String,
    },
    // ResourceService payloads can survive a world change. Session belongs to
    // the token's authority, never to this shared storage identity.
    Resource(ResourceId),
    Service {
        session: u64,
        key: String,
    },
}

impl Target {
    fn belongs_to(&self, current_session: u64) -> bool {
        match self {
            Self::Ecs { session, .. } | Self::Service { session, .. } => {
                *session == current_session
            }
            Self::Resource(_) => true,
        }
    }

    fn label(&self) -> String {
        match self {
            Self::Ecs {
                entity,
                qualified_name,
                ..
            } => format!("component {qualified_name} on entity {entity}"),
            Self::Resource(resource) => format!("loaded resource {resource}"),
            Self::Service { key, .. } => format!("named service {key}"),
        }
    }
}

struct Record {
    owner: String,
    tokens: HashSet<u64>,
}

#[derive(Default)]
pub(crate) struct OwnershipStore {
    next_token: u64,
    records: HashMap<Target, Record>,
}

impl OwnershipStore {
    fn token(&mut self) -> Result<u64, String> {
        self.next_token = self
            .next_token
            .checked_add(1)
            .ok_or("runtime claim token space exhausted")?;
        Ok(self.next_token)
    }

    fn prune(&mut self, session: u64) {
        // An unavailable owner is not evidence that a world ended. Keep its
        // claims until another nonzero session is independently established.
        if session == 0 {
            return;
        }
        self.records.retain(|target, _| target.belongs_to(session));
    }

    fn check(&self, owner: &str, target: &Target) -> Result<(), String> {
        if let Some(record) = self.records.get(target) {
            if record.owner != owner {
                return Err(format!(
                    "{} is owned by runtime mod '{}'. Disable its setting and restore it before another mod writes this storage",
                    target.label(),
                    record.owner
                ));
            }
        }
        Ok(())
    }

    fn acquire(&mut self, owner: &str, target: &Target, token: u64) -> Result<(), String> {
        self.check(owner, target)?;
        self.records
            .entry(target.clone())
            .or_insert_with(|| Record {
                owner: owner.into(),
                tokens: HashSet::new(),
            })
            .tokens
            .insert(token);
        Ok(())
    }

    fn release(&mut self, owner: &str, target: &Target, token: u64) {
        let Some(record) = self.records.get_mut(target) else {
            return;
        };
        if record.owner != owner {
            return;
        }
        record.tokens.remove(&token);
        if record.tokens.is_empty() {
            self.records.remove(target);
        }
    }
}

/// Called by every public typed writer immediately before its native mutation.
/// Existing direct callers may write unclaimed storage. A different mod's
/// active API override always blocks them, including after that mod stops.
pub(crate) fn check_write(state: &AppState, owner: &Mod, target: &Target) -> Result<(), String> {
    let store = state.runtime_ownership();
    let mut store = store.borrow_mut();
    store.prune(loader::runtime_provider::world_session_id());
    store.check(&owner.info().id, target)
}

pub(crate) fn component_target(entity: u32, qualified_name: &str) -> Target {
    Target::Ecs {
        session: loader::runtime_provider::world_session_id(),
        entity,
        qualified_name: qualified_name.into(),
    }
}

pub(crate) struct RuntimeWriteClaim {
    owner: Mod,
    target: Target,
    session: u64,
    token: u64,
    active: Cell<bool>,
    store: Rc<RefCell<OwnershipStore>>,
}

fn require_session(session: u64) -> Result<(), String> {
    if session == 0 || loader::runtime_provider::world_session_id() != session {
        return Err("runtime claim belongs to another or an inactive world session".into());
    }
    Ok(())
}

fn get_claim(
    state: &AppState,
    owner: &Mod,
    target: Target,
    session: u64,
) -> Result<RuntimeWriteClaim, String> {
    require_session(session)?;
    let store = state.runtime_ownership();
    let token = store.borrow_mut().token()?;
    Ok(RuntimeWriteClaim {
        owner: owner.clone(),
        target,
        session,
        token,
        active: Cell::new(false),
        store,
    })
}

impl RuntimeWriteClaim {
    fn acquire(&self, state: &AppState) -> Result<(), String> {
        require_session(self.session)?;
        for operation in if matches!(self.target, Target::Resource(_)) {
            &["runtime.ecs.write", "runtime.assets.write"][..]
        } else {
            &["runtime.ecs.write"][..]
        } {
            if let Some(reason) = loader::runtime_denial_reason(state, &self.owner, operation) {
                return Err(reason);
            }
            if !loader::available(state, &self.owner, operation) {
                return Err(format!("{operation} is currently unavailable"));
            }
        }
        if let Target::Ecs {
            entity,
            qualified_name,
            ..
        } = &self.target
        {
            let component = loader::runtime_provider::resolve(qualified_name)
                .ok_or("claimed component layout is unavailable")?;
            if !loader::runtime_provider::can_write_component(qualified_name)
                || loader::runtime_provider::read(*entity, qualified_name, component.size).is_none()
            {
                return Err(
                    "claimed entity generation or component write authority is unavailable".into(),
                );
            }
        }
        require_session(self.session)?;
        let mut store = self.store.borrow_mut();
        store.prune(self.session);
        store.acquire(&self.owner.info().id, &self.target, self.token)?;
        self.active.set(true);
        Ok(())
    }

    fn release(&self) {
        if self.active.replace(false) {
            self.store
                .borrow_mut()
                .release(&self.owner.info().id, &self.target, self.token);
        }
    }
}

impl UserData for RuntimeWriteClaim {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("acquire", |lua, this, ()| {
            let state = lua.app_data_ref::<AppState>().unwrap();
            Ok(match this.acquire(&state) {
                Ok(()) => (true, None),
                Err(reason) => (false, Some(reason)),
            })
        });
        methods.add_method("release", |_, this, ()| {
            this.release();
            Ok(())
        });
    }
}

pub(crate) fn attach(lua: &Lua, overrides: &Table, owner: &Mod) -> mlua::Result<()> {
    add_function_with_mod(
        lua,
        overrides,
        "get_component_claim",
        owner,
        claim_component,
    )?;
    add_function_with_mod(lua, overrides, "get_resource_claim", owner, claim_resource)?;
    add_function_with_mod(lua, overrides, "get_service_claim", owner, claim_service)?;
    add_function_with_mod(lua, overrides, "get_template_claim", owner, claim_template)?;
    Ok(())
}

fn result(value: Result<RuntimeWriteClaim, String>) -> (Option<RuntimeWriteClaim>, Option<String>) {
    match value {
        Ok(claim) => (Some(claim), None),
        Err(reason) => (None, Some(reason)),
    }
}

fn claim_component(
    lua: &Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<RuntimeWriteClaim>, Option<String>)> {
    Ok(result((|| {
        if args.len() != 2 {
            return Err("get_component_claim expects entity and original component type".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let entity = args.get::<u32>(0).map_err(|e| e.to_string())?;
        let name = loader::runtime_type_name(lua, &args, 1).map_err(|e| e.to_string())?;
        let component = loader::runtime_provider::resolve(&name)
            .ok_or("claimed component layout is unavailable")?;
        let ty = state
            .type_registry()
            .iter()
            .find(|ty| ty.qualified_name == name)
            .ok_or("claimed component is absent from active reflection")?;
        if ty.size != component.size {
            return Err("claimed component reflection and live storage disagree".into());
        }
        let session = loader::runtime_provider::world_session_id();
        get_claim(
            &state,
            owner,
            Target::Ecs {
                session,
                entity,
                qualified_name: name,
            },
            session,
        )
    })()))
}

fn claim_resource(
    lua: &Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<RuntimeWriteClaim>, Option<String>)> {
    Ok(result((|| {
        if args.len() != 1 {
            return Err("get_resource_claim expects one current loaded Resource".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let resource = args.get::<&Resource>(0).map_err(|e| e.to_string())?;
        let session = resource
            .info()
            .require_runtime_write_session(loader::runtime_provider::world_session_id())?;
        get_claim(
            &state,
            owner,
            Target::Resource(resource.info().resource_id),
            session,
        )
    })()))
}

fn claim_service(
    lua: &Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<RuntimeWriteClaim>, Option<String>)> {
    Ok(result((|| {
        if args.len() != 1 {
            return Err("get_service_claim expects one current RuntimeService view".into());
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let view = args
            .get::<&super::runtime_services::RuntimeService>(0)
            .map_err(|e| e.to_string())?;
        let (key, session) = view.ownership_identity(owner)?;
        get_claim(
            &state,
            owner,
            Target::Service {
                session,
                key: key.into(),
            },
            session,
        )
    })()))
}

fn claim_template(
    lua: &Lua,
    args: FunctionArgs,
    owner: &Mod,
) -> mlua::Result<(Option<RuntimeWriteClaim>, Option<String>)> {
    Ok(result((|| {
        if args.len() != 1 {
            return Err(
                "get_template_claim expects one current RuntimeTemplateComponent view".into(),
            );
        }
        let state = lua.app_data_ref::<AppState>().unwrap();
        let view = args
            .get::<&super::runtime_templates::RuntimeTemplateComponent>(0)
            .map_err(|e| e.to_string())?;
        let (resource, session) = view.ownership_identity(owner)?;
        get_claim(&state, owner, Target::Resource(resource), session)
    })()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(session: u64) -> Target {
        Target::Ecs {
            session,
            entity: 7,
            qualified_name: "keen::ecs::Stamina".into(),
        }
    }

    #[test]
    fn independent_mods_cannot_capture_another_mods_baseline() {
        let mut store = OwnershipStore::default();
        let a = store.token().unwrap();
        let b = store.token().unwrap();
        store.acquire("first-mod", &component(1), a).unwrap();
        assert!(
            store
                .acquire("second-mod", &component(1), b)
                .unwrap_err()
                .contains("first-mod")
        );
        assert!(store.check("second-mod", &component(1)).is_err());
        store.release("second-mod", &component(1), a);
        assert!(store.check("second-mod", &component(1)).is_err());
        store.release("first-mod", &component(1), a);
        store.acquire("second-mod", &component(1), b).unwrap();
    }

    #[test]
    fn one_mods_storage_claim_survives_until_every_own_token_is_released() {
        let mut store = OwnershipStore::default();
        let a = store.token().unwrap();
        let b = store.token().unwrap();
        for token in [a, b] {
            store.acquire("first-mod", &component(1), token).unwrap();
        }
        store.release("first-mod", &component(1), a);
        assert!(store.check("second-mod", &component(1)).is_err());
        store.release("first-mod", &component(1), b);
        assert!(store.check("second-mod", &component(1)).is_ok());
    }

    #[test]
    fn world_changes_clear_entity_and_service_claims_but_keep_shared_resources() {
        let mut store = OwnershipStore::default();
        let resource = Target::Resource(ResourceId::default());
        let service = Target::Service {
            session: 1,
            key: "ecs::GameSettings".into(),
        };
        for target in [&component(1), &service, &resource] {
            let token = store.token().unwrap();
            store.acquire("first-mod", target, token).unwrap();
        }
        store.prune(0);
        assert!(store.check("second-mod", &component(1)).is_err());
        assert!(store.check("second-mod", &service).is_err());
        store.prune(2);
        assert!(store.check("second-mod", &component(1)).is_ok());
        assert!(store.check("second-mod", &service).is_ok());
        assert!(store.check("second-mod", &resource).is_err());
    }
}
