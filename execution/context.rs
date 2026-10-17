use std::{
    cell::{Cell, RefCell, RefMut},
    collections::{
        HashMap, HashSet,
        hash_map::{self, Entry},
    },
    io::Read,
    path::PathBuf as NativePathBuf,
    rc::Rc,
    sync::mpsc::{self, Receiver, TryRecvError},
};

use bitflags::bitflags;
use kfc::kfc_format;
use kfc::{
    container::{KFCCursor, KFCFile, KFCReader, KFCWriter},
    guid::{ContentHash, ResourceId},
    reflection::{TypeHandle, TypeIndex, TypeRegistry},
    resource::value::Value,
};
use mod_loader::{Capability, Mod, ModEnvironment};
use once_cell::unsync::OnceCell;

use crate::{
    RunArgs, RuntimePhase, ShroudForgeApi,
    type_aliases::{MappedValue, PathBuf},
    cache::CacheDiff,
    env::{
        Type,
        game::value::is_dirty_lua_value,
        value::{convert_lua_to_value, convert_value_to_lua, validate_and_clone_lua_value},
    },
    logging::{error, warn},
    lua::{LuaError, LuaValue},
};

pub struct AppState {
    pub(crate) runtime_configured: Cell<bool>,
    runtime_active_mods: RefCell<HashSet<String>>,
    runtime_effects: RefCell<HashMap<String, serde_json::Value>>,
    runtime_write_fault: RefCell<Option<(u64, String)>>,
    runtime_ownership: Rc<RefCell<super::runtime_ownership::OwnershipStore>>,
    native_dlls: RefCell<Vec<NativeDll>>,
    pending_native_dlls: RefCell<Vec<PendingNativeDll>>,
    env: ModEnvironment,
    api: ShroudForgeApi,
    config: AppConfig,

    type_registry: Rc<TypeRegistry>,
    attribute_catalog: OnceCell<Result<super::runtime_attributes::Catalog, String>>,
    inline_fields: RefCell<HashMap<(TypeIndex, String), Rc<super::runtime_values::InlineField>>>,
    inline_field_bytes: Cell<usize>,

    ref_file: Rc<KFCFile>,
    reader: RefCell<KFCCursor<KFCReader>>,
    writer: RefCell<Option<KFCWriter<Rc<KFCFile>, Rc<TypeRegistry>>>>,
    asset_transaction: Cell<bool>,
    asset_stem: String,

    // NOTE: the `Vec<u8>` must always be treated as immutable
    new_contents: RefCell<HashMap<ContentHash, Vec<u8>>>,
    resources: RefCell<HashMap<ResourceId, Rc<ResourceInfo>>>,
    loaded_resource_indexes: RefCell<super::game::LoadedResourceIndexes>,
    types: RefCell<HashMap<TypeIndex, LuaValue>>,
}

pub struct AppConfig {
    is_server: bool,
    phase: RuntimePhase,

    feature_flags: AppFeatures,
    export_dir: PathBuf,
}

bitflags! {
    pub struct AppFeatures: u32 {
        const PATCH = 1 << 0;
        const EXPORT = 1 << 1;
        const RUNTIME_DLL = 1 << 2;

        const ALL = !0;
    }
}

impl AppState {
    pub fn new(
        env: ModEnvironment,
        api: Option<ShroudForgeApi>,
        args: RunArgs,
        cache_diff: &CacheDiff,
    ) -> Result<Self, ()> {
        let game_dir = env.game_dir();
        let cache_dir = env.cache_dir();
        let file_name = &args.file_name;

        // prepare configuration

        let options = args.options;
        let game_root = env.game_dir().as_std_path();
        let default_export_dir = PathBuf::from_path_buf(shroudforge_loader_config::paths::export_dir(game_root))
            .expect("a UTF-8 game path joined with fixed loader paths remains UTF-8");
        let configured_export_dir = options.export_dir.and_then(|directory| {
            use std::path::Component;
            if directory.is_absolute()
                || directory.components().any(|component| {
                    !matches!(component, Component::Normal(_) | Component::CurDir)
                })
            {
                warn!(
                    path = %directory.display(),
                    "Export directory must be a relative path inside the game directory"
                );
                return None;
            }
            let directory = game_root.join(directory);
            if let Err(error) = std::fs::create_dir_all(&directory) {
                warn!(error = %error, path = %directory.display(), "Failed to create export directory");
                return None;
            }
            let canonical_root = match std::fs::canonicalize(game_root) {
                Ok(path) => path,
                Err(error) => {
                    warn!(error = %error, "Failed to resolve game directory for export path validation");
                    return None;
                }
            };
            let canonical_directory = match std::fs::canonicalize(&directory) {
                Ok(path) => path,
                Err(error) => {
                    warn!(error = %error, "Failed to resolve export directory");
                    return None;
                }
            };
            if !canonical_directory.starts_with(&canonical_root) {
                warn!(path = %directory.display(), "Export directory resolves outside the game directory");
                return None;
            }
            PathBuf::from_path_buf(canonical_directory).ok()
        });
        let export_dir = configured_export_dir.unwrap_or(default_export_dir);

        let phase = options.phase;
        let is_server = options
            .is_server
            .unwrap_or_else(|| file_name.to_lowercase().contains("server"));

        let mut feature_flags = AppFeatures::empty();

        if options.patch {
            feature_flags |= AppFeatures::PATCH;
        }

        if options.export {
            feature_flags |= AppFeatures::EXPORT;
        }

        #[cfg(windows)]
        if phase == RuntimePhase::Ingame {
            feature_flags |= AppFeatures::RUNTIME_DLL;
        }

        let config = AppConfig {
            export_dir,

            is_server,
            phase,

            feature_flags,
        };

        // load and attach to resources

        let (type_registry, is_dirty) =
            crate::game_files::load_type_registry(game_dir, cache_dir, file_name)?;
        if phase == RuntimePhase::Pregame {
            crate::game_files::export_lua_definitions(
                cache_dir,
                &type_registry,
                options.skip_cache || is_dirty || cache_diff.build_id_changed(),
            );
        }
        let type_registry = Rc::new(type_registry);

        let kfc_path = game_dir.join(file_name).with_extension("kfc");
        let (ref_file, reader) = if phase == RuntimePhase::Ingame {
            (
                crate::game_files::load_kfc_file(&kfc_path)?,
                crate::game_files::create_reader_with_extension(game_dir, file_name, "kfc")?,
            )
        } else {
            let bak_path = crate::game_files::create_backup(&kfc_path)?;
            (
                crate::game_files::load_kfc_file(&bak_path)?,
                crate::game_files::create_reader(game_dir, file_name)?,
            )
        };
        let api = match api {
            Some(api) => api,
            None => {
                let operations =
                    crate::runtime_operations(game_dir.as_std_path()).map_err(|error| {
                        error!(%error, "Invalid installed API contract");
                    })?;
                let schema = kfc::schema_from_registry(&type_registry, &ref_file);
                ShroudForgeApi::new(
                    shroudforge_compatibility::Compatibility::new(operations).resolve(schema),
                )
            }
        };
        let writer = if options.patch {
            let stage =
                shroudforge_game_assets::transaction::begin(game_dir.as_std_path(), file_name)
                    .map_err(|error| {
                        error!(error = %error, "Failed to begin typed asset transaction");
                    })?;
            let staged_writer = (|| {
                for suffix in ["kfc", "kfc_resources"] {
                    let name = format!("{file_name}.{suffix}");
                    std::fs::copy(game_dir.join(&name), stage.join(&name)).map_err(|error| {
                        error!(error = %error, file = %name, "Failed to stage game asset container");
                    })?;
                }
                let stage = PathBuf::from_path_buf(stage.clone()).map_err(|_| {
                    error!("Asset staging path is not valid UTF-8");
                })?;
                crate::game_files::create_writer(&stage, file_name, &type_registry, &ref_file)
            })();
            match staged_writer {
                Ok(writer) => Some(writer),
                Err(()) => {
                    if let Err(error) = shroudforge_game_assets::transaction::abort(
                        game_dir.as_std_path(),
                        file_name,
                    ) {
                        warn!(error = %error, "Failed to roll back asset initialization");
                    }
                    return Err(());
                }
            }
        } else {
            None
        };

        Ok(Self {
            runtime_configured: Cell::new(false),
            runtime_active_mods: RefCell::new(HashSet::new()),
            runtime_effects: RefCell::new(HashMap::new()),
            runtime_write_fault: RefCell::new(None),
            runtime_ownership: Rc::new(RefCell::new(
                super::runtime_ownership::OwnershipStore::default(),
            )),
            native_dlls: RefCell::new(Vec::new()),
            pending_native_dlls: RefCell::new(Vec::new()),
            env,
            api,
            config,

            type_registry,
            attribute_catalog: OnceCell::new(),
            inline_fields: RefCell::new(HashMap::new()),
            inline_field_bytes: Cell::new(0),

            ref_file,
            reader: RefCell::new(reader),
            writer: RefCell::new(writer),
            asset_transaction: Cell::new(options.patch),
            asset_stem: file_name.clone(),

            new_contents: RefCell::new(HashMap::new()),
            resources: RefCell::new(HashMap::new()),
            loaded_resource_indexes: RefCell::new(super::game::LoadedResourceIndexes::default()),
            types: RefCell::new(HashMap::new()),
        })
    }

    #[inline]
    pub fn env(&self) -> &ModEnvironment {
        &self.env
    }

    #[inline]
    pub fn api(&self) -> &ShroudForgeApi {
        &self.api
    }

    #[inline]
    pub fn is_server(&self) -> bool {
        self.config.is_server
    }

    #[inline]
    pub fn is_client(&self) -> bool {
        !self.config.is_server
    }

    #[inline]
    pub fn phase(&self) -> RuntimePhase {
        self.config.phase
    }

    pub(crate) fn set_runtime_mod_active(&self, id: &str, active: bool) {
        let native_dll_state = self.native_dll_state(id);
        let mut active_mods = self.runtime_active_mods.borrow_mut();
        if active {
            active_mods.insert(id.to_owned());
        } else {
            active_mods.remove(id);
        }
        drop(active_mods);
        if !active {
            self.loaded_resource_indexes.borrow_mut().remove_owner(id);
        }
        if let Some(state) = native_dll_state {
            let detail = match state {
                "loading" => "native DLL is still loading",
                _ => "native DLL remains loaded for this game session",
            };
            self.report_runtime_effect(id, state, detail);
        } else {
            let mut effects = self.runtime_effects.borrow_mut();
            if effects
                .get(id)
                .is_some_and(|effect| effect["state"] != "restart-required")
            {
                effects.remove(id);
            }
        }
    }

    pub(crate) fn runtime_mod_is_active(&self, id: &str) -> bool {
        self.runtime_active_mods.borrow().contains(id)
    }

    pub(crate) fn runtime_ownership(
        &self,
    ) -> Rc<RefCell<super::runtime_ownership::OwnershipStore>> {
        self.runtime_ownership.clone()
    }

    pub(crate) fn runtime_write_fault(&self) -> Option<String> {
        let session = super::loader::runtime_provider::world_session_id();
        self.runtime_write_fault
            .borrow()
            .as_ref()
            .filter(|(original, _)| session != 0 && *original == session)
            .map(|(_, reason)| reason.clone())
    }

    pub(crate) fn record_uncertain_runtime_write(&self, session: u64, reason: &str) {
        if session == 0 {
            return;
        }
        let mut fault = self.runtime_write_fault.borrow_mut();
        if fault
            .as_ref()
            .is_some_and(|(original, _)| *original == session)
        {
            return;
        }
        *fault = Some((session, reason.to_owned()));
    }

    pub(crate) fn report_runtime_effect(&self, id: &str, state: &str, detail: &str) {
        let value = serde_json::json!({
            "state": state,
            "processTarget": if self.is_server() { "server" } else { "client" },
            "scope": "this-process",
            "detail": detail.chars().take(240).collect::<String>(),
            "updatedAt": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
        let mut effects = self.runtime_effects.borrow_mut();
        if effects.get(id).is_some_and(|previous| {
            previous["state"] == value["state"] && previous["detail"] == value["detail"]
        }) {
            return;
        }
        effects.insert(id.to_owned(), value);
    }

    pub(crate) fn runtime_effects(&self) -> serde_json::Value {
        serde_json::Value::Object(
            self.runtime_effects
                .borrow()
                .iter()
                .map(|(id, value)| (id.clone(), value.clone()))
                .collect(),
        )
    }

    #[inline]
    pub fn export_dir(&self) -> &PathBuf {
        &self.config.export_dir
    }

    #[inline]
    pub fn has_feature(&self, feature: AppFeatures) -> bool {
        self.config.feature_flags.contains(feature)
    }

    #[inline]
    pub fn type_registry(&self) -> &Rc<TypeRegistry> {
        &self.type_registry
    }

    pub(crate) fn attribute_catalog(&self) -> Result<&super::runtime_attributes::Catalog, String> {
        self.attribute_catalog
            .get_or_init(|| {
                super::runtime_attributes::Catalog::load(
                    &self.type_registry,
                    &self.ref_file,
                    &mut self.reader.borrow_mut(),
                )
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    pub(crate) fn inline_field(
        &self,
        root: TypeIndex,
        path: &str,
    ) -> Result<Rc<super::runtime_values::InlineField>, String> {
        let key = (root, path.to_owned());
        if let Some(plan) = self.inline_fields.borrow().get(&key) {
            return Ok(plan.clone());
        }
        let plan = Rc::new(super::runtime_values::inline_field(
            &self.type_registry,
            root,
            path,
        )?);
        let mut fields = self.inline_fields.borrow_mut();
        // Bound metadata retention. A full cache never discards a live value,
        // baseline or claim and does not turn a valid field into an error.
        let retained = plan
            .retained_bytes()
            .saturating_add(path.len())
            .saturating_add(128);
        if fields.len() < 4096
            && self.inline_field_bytes.get().saturating_add(retained) <= 16 * 1024 * 1024
        {
            self.inline_field_bytes
                .set(self.inline_field_bytes.get() + retained);
            fields.insert(key, plan.clone());
        }
        Ok(plan)
    }

    pub fn get_type(
        &self,
        lua: &mlua::Lua,
        type_index: TypeIndex,
    ) -> mlua::Result<Option<LuaValue>> {
        match self.types.borrow_mut().entry(type_index) {
            Entry::Occupied(entry) => Ok(Some(entry.get().clone())),
            Entry::Vacant(entry) => {
                if self.type_registry.get(type_index).is_none() {
                    return Ok(None);
                }

                let value = Type::new(TypeHandle::new(self.type_registry.clone(), type_index));
                let value = LuaValue::UserData(lua.create_userdata(value)?);

                entry.insert(value.clone());
                Ok(Some(value))
            }
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        if self.asset_transaction.get() {
            if let Err(error) = shroudforge_game_assets::transaction::abort(
                self.env.game_dir().as_std_path(),
                &self.asset_stem,
            ) {
                warn!(error = %error, "Failed to abort typed asset transaction");
            }
        }
        self.shutdown_native_dlls();
    }
}

#[path = "assets/session.rs"]
mod asset_session;
#[path = "loaded-resources/session.rs"]
mod loaded_session;
#[path = "native_mods.rs"]
mod native_mods;

pub use asset_session::ResourceInfo;
use asset_session::ResourceSource;
use native_mods::{NativeDll, PendingNativeDll};
