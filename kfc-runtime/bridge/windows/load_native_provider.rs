use super::native_abi::*;
use crate::KFC_RUNTIME_ABI_VERSION;
use std::{
    ffi::{c_char, c_void},
    sync::OnceLock,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *const c_void;
}

static PROVIDER: OnceLock<Provider> = OnceLock::new();
pub(super) static PROVIDER_ERROR: OnceLock<String> = OnceLock::new();

pub(super) fn provider() -> Option<&'static Provider> {
    if let Some(provider) = PROVIDER.get() {
        return Some(provider);
    }
    let loaded = unsafe {
        (|| -> Result<Provider, String> {
            let module_name: Vec<u16> = "kfc-runtime.dll\0".encode_utf16().collect();
            let module = GetModuleHandleW(module_name.as_ptr());
            if module.is_null() {
                Err("provider-module-not-loaded".to_string())
            } else {
                macro_rules! symbol {
                    ($name:literal, $kind:ty) => {{
                        let pointer = GetProcAddress(module, concat!($name, "\0").as_ptr().cast());
                        if pointer.is_null() {
                            return Err(concat!("missing-export:", $name).to_string());
                        }
                        std::mem::transmute::<*const c_void, $kind>(pointer)
                    }};
                }
                let abi = symbol!("KfcRuntimeAbi", unsafe extern "C" fn() -> u32);
                let actual_abi = abi();
                if actual_abi != KFC_RUNTIME_ABI_VERSION {
                    Err(format!(
                        "provider-abi-mismatch:expected={KFC_RUNTIME_ABI_VERSION},actual={actual_abi}"
                    ))
                } else {
                    Ok(Provider {
                        configure: symbol!("KfcRuntimeEcsConfigure", Configure),
                        ready: symbol!("KfcRuntimeEcsReady", Ready),
                        prop_query_ready: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsPropQueryReady".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, Ready>(pointer))
                        },
                        can_write: symbol!("KfcRuntimeEcsCanWrite", Ready),
                        can_write_component: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsCanWriteComponent".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, RuntimePatchAvailable>(pointer)
                            })
                        },
                        describe: symbol!("KfcRuntimeEcsDescribe", Describe),
                        registry: {
                            let pointer = GetProcAddress(module, c"KfcRuntimeEcsRegistry".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, Registry>(pointer))
                        },
                        registration: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsRegistration".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, Registration>(pointer)
                            })
                        },
                        functions: {
                            let pointer = GetProcAddress(module, c"KfcRuntimeFunctions".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, Registry>(pointer))
                        },
                        function_code: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeFunctionCode".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, FunctionCode>(pointer)
                            })
                        },
                        service_read: {
                            let pointer = GetProcAddress(module, c"KfcRuntimeServiceRead".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, ServiceRead>(pointer))
                        },
                        service_compare_exchange: {
                            let pointer = GetProcAddress(
                                module,
                                c"KfcRuntimeServiceCompareExchange".as_ptr(),
                            );
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, ServiceCompareExchange>(
                                    pointer,
                                )
                            })
                        },
                        resources_by_type: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeResourcesByType".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, ResourcesByType>(pointer)
                            })
                        },
                        resource_read: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeResourceRead".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, ResourceRead>(pointer)
                            })
                        },
                        resource_read_scalar_batch: {
                            let pointer = GetProcAddress(
                                module,
                                c"KfcRuntimeResourceReadScalarBatch".as_ptr(),
                            );
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, ResourceReadScalarBatch>(
                                    pointer,
                                )
                            })
                        },
                        resource_resolve_item: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeResourceResolveItem".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, ResourceResolveItem>(pointer)
                            })
                        },
                        resource_status: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeResourceStatus".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, Registry>(pointer))
                        },
                        resource_can_write: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeResourceCanWrite".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, Ready>(pointer))
                        },
                        resource_compare_exchange_scalar_for_session: {
                            let pointer = GetProcAddress(
                                module,
                                c"KfcRuntimeResourceCompareExchangeScalarForSession".as_ptr(),
                            );
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<
                                    *const c_void,
                                    ResourceCompareExchangeScalarForSession,
                                >(pointer)
                            })
                        },
                        resolve_type: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsResolveType".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, ResolveType>(pointer))
                        },
                        entity_template_uuid: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsTemplateUuid".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, EntityTemplateUuid>(pointer)
                            })
                        },
                        query: symbol!("KfcRuntimeEcsQuery", Query),
                        query_bounds: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsQueryBounds".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, QueryBounds>(pointer))
                        },
                        resolve_entity: symbol!("KfcRuntimeEcsResolve", ResolveEntity),
                        read: symbol!("KfcRuntimeEcsRead", Read),
                        read_many: {
                            let pointer = GetProcAddress(module, c"KfcRuntimeEcsReadMany".as_ptr());
                            (!pointer.is_null())
                                .then(|| std::mem::transmute::<*const c_void, ReadMany>(pointer))
                        },
                        compare_exchange: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeEcsCompareExchange".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, CompareExchange>(pointer)
                            })
                        },
                        world_operation_available: symbol!(
                            "KfcRuntimeWorldOperationAvailable",
                            WorldOperationAvailable
                        ),
                        world_session_id: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeWorldSessionId".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, WorldSessionId>(pointer)
                            })
                        },
                        world_context_active: symbol!(
                            "KfcRuntimeWorldContextActive",
                            WorldContextActive
                        ),
                        world_context_kind: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeWorldContextKind".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, WorldContextKind>(pointer)
                            })
                        },
                        world_entity_context_ready: symbol!(
                            "KfcRuntimeWorldEntityContextReady",
                            WorldEntityContextReady
                        ),
                        world_entity_query_props: symbol!(
                            "KfcRuntimeWorldEntityQueryProps",
                            WorldEntityQueryProps
                        ),
                        world_entity_query_props_in_bounds: symbol!(
                            "KfcRuntimeWorldEntityQueryPropsInBounds",
                            WorldEntityQueryPropsInBounds
                        ),
                        world_entity_register_prop_recipes: symbol!(
                            "KfcRuntimeWorldEntityRegisterPropRecipes",
                            WorldEntityRegisterPropRecipes
                        ),
                        world_entity_get_transform: symbol!(
                            "KfcRuntimeWorldEntityGetTransform",
                            WorldEntityGetTransform
                        ),
                        world_entity_identity: symbol!(
                            "KfcRuntimeEcsEntityIdentity",
                            WorldEntityIdentity
                        ),
                        world_entity_set_scale: symbol!(
                            "KfcRuntimeWorldEntitySetScale",
                            WorldEntitySetScale
                        ),
                        world_cursor_read: {
                            let pointer =
                                GetProcAddress(module, c"KfcRuntimeWorldCursorRead".as_ptr());
                            (!pointer.is_null()).then(|| {
                                std::mem::transmute::<*const c_void, WorldCursorRead>(pointer)
                            })
                        },
                        world_voxel_read: symbol!("KfcRuntimeWorldVoxelRead", WorldVoxelRead),
                        world_voxel_write: symbol!("KfcRuntimeWorldVoxelWrite", WorldVoxelWrite),
                        world_grid_get_spec: symbol!(
                            "KfcRuntimeWorldGridGetSpec",
                            WorldGridGetSpec
                        ),
                        world_entity_spawn: symbol!("KfcRuntimeWorldEntitySpawn", WorldEntitySpawn),
                        world_entity_place: symbol!(
                            "KfcRuntimeWorldEntityPlace",
                            WorldEntityPlacement
                        ),
                        world_entity_destroy: symbol!(
                            "KfcRuntimeWorldEntityDestroy",
                            WorldEntityPlacement
                        ),
                        world_entity_destroy_handle: symbol!(
                            "KfcRuntimeWorldEntityDestroyHandle",
                            WorldEntityDestroyHandle
                        ),
                        world_entity_finish: symbol!(
                            "KfcRuntimeWorldEntityFinishBuilding",
                            WorldEntityFinish
                        ),
                        runtime_patch_available: symbol!(
                            "KfcRuntimePatchAvailable",
                            RuntimePatchAvailable
                        ),
                        runtime_patch_set_enabled: symbol!(
                            "KfcRuntimePatchSetEnabled",
                            RuntimePatchSetEnabled
                        ),
                        abi: actual_abi,
                        status: symbol!(
                            "KfcRuntimeStatus",
                            unsafe extern "C" fn(*mut c_char, usize)
                        ),
                    })
                }
            }
        })()
    };
    match loaded {
        Ok(provider) => {
            let _ = PROVIDER.set(provider);
            PROVIDER.get()
        }
        Err(error) => {
            // The runtime DLL may be loaded just after Lua environments are
            // created. Treat that as a transient startup condition and try
            // again on the next API call instead of poisoning this process.
            if error != "provider-module-not-loaded" {
                let _ = PROVIDER_ERROR.set(error);
            }
            None
        }
    }
}

pub(super) fn observed_abi() -> Option<u32> {
    unsafe {
        let module_name: Vec<u16> = "kfc-runtime.dll\0".encode_utf16().collect();
        let module = GetModuleHandleW(module_name.as_ptr());
        if module.is_null() {
            return None;
        }
        let pointer = GetProcAddress(module, c"KfcRuntimeAbi".as_ptr());
        if pointer.is_null() {
            return None;
        }
        let abi: unsafe extern "C" fn() -> u32 = std::mem::transmute(pointer);
        Some(abi())
    }
}
