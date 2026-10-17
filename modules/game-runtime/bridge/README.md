# Rust bridge to the native runtime

This library connects the mod runtime to the canonical C ABI in `../native/abi/runtime_api.h`. `modules/mod-runtime` uses it as a regular Cargo dependency.

Native operations live with their actual owners under `windows/`:

| Action | File |
| --- | --- |
| Load the provider and verify its ABI | `windows/provider/load_native_provider.rs` |
| Define C data types and function signatures | `windows/native_abi.rs` |
| Read ECS component registrations | `windows/ecs/read_component_registry.rs` |
| Find entities | `windows/ecs/query_entities.rs` |
| Read a component | `windows/ecs/read_component.rs` |
| Read components in batches | `windows/ecs/read_components.rs` |
| Update a component with validation | `windows/ecs/update_component.rs` |
| Read a resource | `windows/resources/read_resource.rs` |
| Update a resource scalar with validation | `windows/resources/update_resource_scalar.rs` |
| Read or update a service | `windows/named_services/read_service.rs`, `windows/named_services/update_service.rs` |
| Read world state, voxels, props, functions, or patches | `windows/world_context/`, `windows/voxels/`, `windows/props/`, `windows/functions/`, `windows/patches/` |
| Submit the original building-input operation | `building_input.rs`, exported as `player_input` |

Each area declares and exports its own actions through `mod.rs`. `windows/mod.rs` preserves the existing public Rust function names. `runtime_records.rs` owns the shared snapshot records. `unsupported_platform.rs` contains the existing responses for other operating systems. Building input keeps its established symbol-loading path.

Shared measurement of runtime operations belongs to the diagnostics module. The bridge and Lua execution call the same implementation through `shroudforge_runtime_diagnostics::diagnostics_jobs::operation_timings`.
