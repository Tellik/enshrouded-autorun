# Game build and runtime profile actions

This directory contains the focused C++, Python, and PowerShell actions used to inspect game builds, capture runtime evidence, validate candidate profiles, and approve supported data. The actions and their inputs are listed in the current [game build and runtime profile guide](../../docs/development/inspect-game-build-and-create-runtime-profile.de.md).

The folders separate the native developer CLI, capture tools, inspection, comparison, analysis, verification, explicit checks, and their tests. Catalogs, schemas, and local captures remain at this directory's root because they are data used across those actions.

## Native developer CLI

| File | Action |
| --- | --- |
| `cli/kfc_runtime_dev.cpp` | Starts the `kfc-runtime-dev` command line and dispatches its subcommands. |
| `cli/profile_cli.h` | Declares the shared command context, option types, and command handlers. |
| `cli/read_cli_option.cpp` | Reads named command-line options and their values. |
| `cli/inspect_executable.cpp` | Writes a report about an executable's PE identity and sections. |
| `cli/select_game_build.cpp` | Selects client or server targets and locates exact build catalogs. |
| `cli/discover_function_candidates.cpp` | Finds heuristic function ranges and candidate addresses in a PE image. |
| `cli/scan_function_signatures.cpp` | Scans an executable for cataloged byte signatures. |
| `cli/export_profile_function_catalog.cpp` | Exports a profile's hooks and operations as a function catalog. |
| `cli/capture_process_modules.cpp` | Records a process image path and its loaded modules. |
| `cli/import_legacy_ecs_capture.cpp` | Converts an older tabular ECS capture to the current catalog format. |
| `cli/inspect_provider_abi.cpp` | Checks a provider DLL's ABI version and exports. |
| `cli/create_runtime_profile_draft.cpp` | Creates an unapproved profile draft for review. |
| `cli/validate_runtime_profile.cpp` | Validates profile structure, offsets, operations, and component data. |
| `cli/approve_runtime_profile.cpp` | Checks recorded evidence and requests confirmation before marking a profile approved. |
| `cli/read_pe_image.cpp` | Reads PE file identity, sections, and executable ranges for the native tools. |
| `cli/read_pe_image.h` | Declares the native PE image reader and its data types. |
| `cli/match_function_signatures.cpp` | Parses byte patterns and searches executable ranges for matches. |
| `cli/match_function_signatures.h` | Declares the native byte-pattern matcher. |
| `cli/write_json_report.cpp` | Writes formatted JSON reports for the native commands. |

## Capture tools

| File | Action |
| --- | --- |
| `capture/capture_runtime.ps1` | Orchestrates the selected client or server capture and optional verification steps. |
| `capture/discover_runtime_build.py` | Extracts executable reflection and optional live process, function, component, and system data. |
| `capture/capture_ecs_registry.py` | Discovers ECS registrations and can export a component catalog. |
| `capture/capture_factory_state.py` | Reads factory and linked inventory observations from a running process. |
| `capture/read_process_memory.py` | Provides read-only process memory access and live memory capture. |
| `capture/discover_component_registry.py` | Discovers and validates component registrations, managers, and storage layouts. |
| `capture/find_entity_managers.cpp` | Searches process memory for entity manager candidates and checks their structure. |
| `capture/capture_legacy_component_registry.cpp` | Finds and prints the older ECS component table. |
| `capture/find_live_type_references.cpp` | Counts live process references to selected reflected types. |
| `capture/export_observed_registry.py` | Exports a unique registry already observed by the production reader. |
| `capture/export_runtime_bindings.py` | Exports a runtime discovery catalog and a compact build index. |

## Executable and resource inspection

| File | Action |
| --- | --- |
| `inspect/read_pe_image.py` | Reads PE sections, RVAs, and executable identity for Python tools. |
| `inspect/extract_executable_reflection.py` | Extracts the reflected type catalog directly from an executable. |
| `inspect/discover_executable_functions.py` | Disassembles PE function ranges and records static code references. |
| `inspect/read_system_descriptors.py` | Reads named ECS system execution descriptors. |
| `inspect/inspect_inventory_transactions.py` | Resolves inventory quantity removal, partial source removal, and usage-cost relationships from original named descriptors, reflected layouts, and decoded instructions. It reports evidence without calling or changing the game. |
| `inspect/inspect_component_metadata.cpp` | Examines component metadata fields against component indices. |
| `inspect/export_kfc_resources.rs` | Exports selected KFC resources through the parser. |
| `inspect/inspect_system_dependencies.rs` | Resolves system dependencies using the runtime resolver. |

## Build comparison

| File | Action |
| --- | --- |
| `compare/compare_runtime_builds.py` | Compares layouts and function evidence from two game builds. |
| `compare/rank_cross_build_functions.py` | Ranks static function candidates across builds without authorizing calls. |
| `compare/compare_recipe_resources.py` | Compares recipe resources from exported captures. |

## Capture analysis

| File | Action |
| --- | --- |
| `analyze/index_captured_functions.py` | Indexes captured function ranges, calls, descriptors, and type references. |
| `analyze/find_runtime_type.py` | Finds captured types by name or hash and reports all matching locations. |
| `analyze/trace_function_anchors.py` | Traces signature matches to functions and their static callers. |
| `analyze/read_capture_reports.py` | Reads related reports and rejects captures from different executable identities. |
| `analyze/write_inspection_result.py` | Writes analysis results as JSON or to the console. |

## Verification

| File | Action |
| --- | --- |
| `verify/validate_patch_payload.py` | Checks patch payload lengths and embedded references. |
| `verify/verify_runtime_origins.py` | Coordinates profile patch, world operation, and hook origin checks. |
| `verify/verify_runtime_patch_origins.py` | Checks patch origins, argument evidence, and caller paths. |
| `verify/verify_runtime_hook_origins.py` | Checks hook signatures and complete instruction boundaries. |
| `verify/verify_world_operation_origins.py` | Checks world operation addresses and guards. |
| `verify/audit_live_component_registry.ps1` | Runs the native live registry audit and saves its log and candidate mappings. |
| `verify/audit_live_component_registry.cpp` | Audits live registry tables, guards, and candidate mappings. |

## Explicit checks

| File | Action |
| --- | --- |
| `checks/generate_attribute_reference_vectors.py` | Generates isolated emulator vectors for original game attribute interpreters. |
| `checks/run_callback_corpus_tests.ps1` | Runs the existing callback corpus test against a selected registry capture. |
| `checks/sample_entity_manager_registers.cpp` | Samples entity manager registrations from a live process. |
| `checks/verify_building_event_arguments.py` | Verifies original building event argument mappings in an isolated emulator. |
| `checks/verify_building_input_layout.py` | Verifies the client building input layout and hook contract. |
| `checks/verify_component_copy_adapters.py` | Differentially checks component copy adapters on private buffers. |
| `checks/verify_component_registry.cpp` | Verifies a live component registry against production registry rules. |
| `checks/verify_cursor_voxel_context.py` | Checks the cursor-to-voxel context data flow in the original executable. |
| `checks/verify_inventory_decrement.py` | Emulates the original inventory decrement helpers on isolated data, using established image catalogs or the structural inventory inspection report. Engine dependencies remain stubs. |

## Tool tests

| File | Action |
| --- | --- |
| `tests/test_components.py` | Tests component registry parsing and validation with independent fixtures. |
| `tests/test_discovery.py` | Tests PE, reflection, function, system, and anchor discovery with synthetic files. |
| `tests/test_mod_origins.py` | Tests that known invalid runtime patch payloads fail validation. |

## Shared data and boundaries

| Location | Contents |
| --- | --- |
| `catalogs/` | Maintained function signatures and captured build indexes. |
| `schemas/` | Contracts for profiles, function catalogs, and component catalogs. |
| `captures/` | Ignored local input files, original executable copies, capture results, and drafts. |
| `requirements-discovery.txt` | Python dependencies for executable discovery. |
| `../../modules/game-runtime/profiles/` | Approved runtime profiles shipped with the native runtime. |

The native CLI is compiled directly from `modules/game-runtime/native/CMakeLists.txt`. Its build targets and profile-directory definition point to these source folders and the shared catalog root. Python tools use package-qualified imports so scripts can run directly and their modules can be shared across the folders.

Profile approval remains tied to the exact executable identity and its recorded evidence. A function match is not approval to call an engine function.
