use super::*;

impl AppState {
    pub(crate) fn loaded_resource_indexes(
        &self,
    ) -> RefMut<'_, crate::env::game::LoadedResourceIndexes> {
        self.loaded_resource_indexes.borrow_mut()
    }

    pub(crate) fn advance_loaded_resource_indexes(&self) {
        crate::env::game::tick_loaded_resource_indexes(self);
    }

    pub(crate) fn invalidate_loaded_resource_indexes(&self) {
        self.loaded_resource_indexes.borrow_mut().invalidate();
    }

    pub(crate) fn compare_exchange_loaded_resource_scalar(
        &self,
        resource: &ResourceId,
        resource_size: usize,
        offset: usize,
        expected: &[u8],
        desired: &[u8],
        session: u64,
    ) -> Result<(), String> {
        let result =
            crate::env::loader::runtime_provider::resource_compare_exchange_scalar_for_session(
                resource,
                resource_size,
                offset,
                expected,
                desired,
                session,
            );
        if result.is_ok() {
            self.loaded_resource_indexes
                .borrow_mut()
                .invalidate_resource(resource, offset, desired.len());
        } else if result
            .as_ref()
            .is_err_and(|reason| reason.to_lowercase().contains("uncertain"))
        {
            self.invalidate_loaded_resource_indexes();
        }
        result
    }

    pub(crate) fn get_loaded_resource_info_for_session(
        &self,
        resource_id: &ResourceId,
        creation_session: u64,
    ) -> Rc<ResourceInfo> {
        Rc::new(ResourceInfo::loaded(*resource_id, creation_session))
    }

    pub(crate) fn get_loaded_resource_info_from_snapshot(
        &self,
        resource_id: &ResourceId,
        session: u64,
        snapshot: Vec<u8>,
    ) -> Rc<ResourceInfo> {
        Rc::new(ResourceInfo::loaded_snapshot(
            *resource_id,
            session,
            snapshot,
        ))
    }
}

impl ResourceInfo {
    pub(super) fn loaded(resource_id: ResourceId, creation_session: u64) -> Self {
        Self {
            resource_id,
            source: ResourceSource::LoadedRuntime,
            creation_session,
            runtime_snapshot: OnceCell::new(),
            runtime_write_fault: RefCell::new(None),
            original_value: OnceCell::new(),
            value: RefCell::default(),
            is_dirty: Cell::new(false),
        }
    }

    pub(super) fn loaded_snapshot(
        resource_id: ResourceId,
        session: u64,
        snapshot: Vec<u8>,
    ) -> Self {
        let info = Self::loaded(resource_id, session);
        let _ = info.runtime_snapshot.set(Rc::from(snapshot));
        info
    }

    pub(crate) fn creation_session(&self) -> Option<u64> {
        (self.source == ResourceSource::LoadedRuntime).then_some(self.creation_session)
    }

    pub(crate) fn require_runtime_write_session(&self, current: u64) -> Result<u64, String> {
        if self.source != ResourceSource::LoadedRuntime {
            return Err("write_loaded_field requires a ResourceService snapshot".into());
        }
        if let Some(reason) = self.runtime_write_fault.borrow().as_ref() {
            return Err(reason.clone());
        }
        if self.creation_session == 0 || current == 0 || self.creation_session != current {
            return Err("loaded resource belongs to another or an inactive world session. Get a fresh world-bound resource snapshot".into());
        }
        Ok(self.creation_session)
    }

    pub(crate) fn stop_runtime_writes(&self, reason: &str) {
        *self.runtime_write_fault.borrow_mut() = Some(reason.to_owned());
    }

    pub(crate) fn loaded_snapshot_bytes(&self) -> Result<&Rc<[u8]>, String> {
        if self.source != ResourceSource::LoadedRuntime {
            return Err("resource is not a runtime snapshot".into());
        }
        self.runtime_snapshot.get_or_try_init(|| {
            let session = crate::env::loader::runtime_provider::world_session_id();
            if self.creation_session != 0 && self.creation_session != session {
                return Err(
                    "world session changed before this resource snapshot was captured".into(),
                );
            }
            let bytes = crate::env::loader::runtime_provider::resource_read(&self.resource_id)?;
            if self.creation_session != 0
                && crate::env::loader::runtime_provider::world_session_id() != self.creation_session
            {
                return Err("world session changed during resource snapshot capture".into());
            }
            Ok(Rc::from(bytes))
        })
    }
}

#[cfg(test)]
mod resource_snapshot_tests {
    use super::*;

    #[test]
    fn loaded_resource_write_scope_rejects_menu_and_foreign_world_snapshots() {
        let menu = ResourceInfo::loaded(ResourceId::NONE, 0);
        assert_eq!(menu.creation_session(), Some(0));
        assert!(menu.require_runtime_write_session(0).is_err());
        assert!(menu.require_runtime_write_session(42).is_err());
        let world = ResourceInfo::loaded(ResourceId::NONE, 42);
        assert_eq!(world.require_runtime_write_session(42).unwrap(), 42);
        assert!(world.require_runtime_write_session(0).is_err());
        assert!(world.require_runtime_write_session(43).is_err());
        let mut file = ResourceInfo::loaded(ResourceId::NONE, 42);
        file.source = ResourceSource::KfcFile;
        assert_eq!(file.creation_session(), None);
        assert!(file.require_runtime_write_session(42).is_err());
        let lua = mlua::Lua::new();
        lua.globals()
            .set("menu", crate::env::game::Resource::new(Rc::new(menu)))
            .unwrap();
        lua.globals()
            .set("world", crate::env::game::Resource::new(Rc::new(world)))
            .unwrap();
        lua.globals()
            .set("file", crate::env::game::Resource::new(Rc::new(file)))
            .unwrap();
        lua.load("assert(menu.session_id==0 and world.session_id==42 and file.session_id==nil)")
            .exec()
            .unwrap();
        assert!(lua.load("world.session_id=43").exec().is_err());
    }

    #[test]
    fn verified_resource_keeps_its_exact_readback_and_stopped_source_cannot_repeat() {
        let verified = ResourceInfo::loaded_snapshot(ResourceId::NONE, 42, vec![1, 2, 3, 4]);
        // This succeeds on non-Windows as well: a verified result must never
        // issue a later native read to manufacture its purported readback.
        assert_eq!(
            verified.loaded_snapshot_bytes().unwrap().as_ref(),
            &[1, 2, 3, 4]
        );
        assert!(
            verified
                .runtime_snapshot
                .set(Rc::from(vec![9, 9, 9, 9]))
                .is_err()
        );
        assert_eq!(
            verified.loaded_snapshot_bytes().unwrap().as_ref(),
            &[1, 2, 3, 4]
        );
        verified.stop_runtime_writes("native-write-outcome-uncertain");
        assert_eq!(
            verified.require_runtime_write_session(42).unwrap_err(),
            "native-write-outcome-uncertain"
        );
        assert_eq!(
            ResourceInfo::loaded(ResourceId::NONE, 42)
                .require_runtime_write_session(42)
                .unwrap(),
            42
        );
    }
}
