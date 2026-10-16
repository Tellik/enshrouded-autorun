# Public Lua runtime API audit, 2026-10-16

This audit follows the public Lua entry points into their shared reflection,
calculation, authority and storage writers. Its purpose is to establish whether
independent third-party mods can use the same paths as SF Survival and SF
Unlimited Flight. It does not establish the gameplay effect of every reflected
field or an ABI for calling arbitrary original engine functions.

## Declared API inventory at the start of the audit

`src/loader/api/src/shroudforge/v1/definitions/runtime.lua` contained 86 explicit
`function` declarations. This includes five log methods and both documented
overloads of `runtime.world.entity.destroy`. `assets/manager.lua` contained 14
declarations, including both overloads of `create_resource`.

Functions documented as callable fields, such as `runtime.report_effect`,
`RuntimeNativeBinding.call` and the two `RuntimeFunctionModifier` methods, are
listed separately below and are not included in those declaration counts.

| Namespace | Explicit signatures | Commands | Implementation |
| --- | ---: | --- | --- |
| `runtime` | 4 | `has`, `require`, `status`, `get_operations` | `env/loader.rs` |
| `runtime.overrides` and returned tracker | 10 | `new`, `begin_update`, `waiting`, `apply`, `restore_disabled`, `restore_all`, `has_ownership`, `has_shared_ownership`, `reset`, `summary` | `shroudforge/v1/lua/overrides.lua`, loaded per mod through `env/loader.rs` at audit start |
| `runtime.services` and service view | 3 | `get`, `read_fields`, `write_field` | `env/runtime_services.rs`, `native/service/service_runtime.cpp` |
| `runtime.network` | 7 | `status`, `send`, `send_mod`, `accept`, `connected_peers`, `receive`, `receive_mod` | `env/runtime_networking.rs`, Steam networking transport |
| `runtime.values` | 3 | `decode`, `encode`, `new` | `env/runtime_values.rs`, shared KFC value codec |
| `runtime.functions` | 8 | `list_native`, `list`, `find`, `get`, `get_native`, `bind`, `get_operations`, `bind_modifier` | `env/runtime_functions.rs`, `runtime_resolution/functions.rs`, checked wrappers in `env/loader.rs` |
| `runtime.ecs` | 21 | `get_registry`, `get_component`, `get_attributes`, `get_attribute`, `read_attribute`, `write_attribute_storage`, `read_attributes`, `evaluate_attributes`, `update_attribute`, `get_components`, `get_catalog`, `query`, `query_bounds`, `resolve`, `read`, `read_bytes`, `read_fields`, `get_template`, `get_template_component`, `write_fields`, `write` | `env/loader.rs`, `env/runtime_attributes.rs`, `env/runtime_templates.rs`, `native/ecs/ecs_runtime.cpp` |
| Template component view | 2 | `read_fields`, `write_field` | `env/runtime_templates.rs`, shared native ResourceService scalar CAS |
| `runtime.world` including nested APIs | 21 | `cursor.get`, `building.submit`, `building.status`, `building.cancel`, `voxel.get_grid_spec`, `voxel.read`, `voxel.write`, `operation_available`, `context_active`, `context_kind`, `session_id`, `entity.query_props`, `entity.register_prop_recipes`, `entity.query_props_in_bounds`, `entity.get_transform`, `entity.set_scale`, `entity.spawn`, `entity.place`, both `entity.destroy` overloads, `entity.finish_building` | `env/loader.rs`, `env/runtime_building.rs`, checked native world adapters |
| `runtime.patch` | 2 | `available`, `set_enabled` | `env/loader.rs`, exact-image native patch provider |
| Mod log | 5 | `trace`, `debug`, `info`, `warn`, `error` | `env/log.rs` |
| `game.assets` | 14 | `get_resource`, `get_resource_parts`, `get_resources_by_type`, `get_loaded_resource`, `get_loaded_resources_by_type`, `get_loaded_resource_by_field`, `write_loaded_field`, `get_all_resources`, `get_resource_types`, both `create_resource` overloads, `get_content`, `get_all_contents`, `create_content` | `env/game/assets.rs`, `env/game/assets/loaded_index.rs`, resource/content wrappers |

Callable fields outside that count are `runtime.report_effect`,
`RuntimeNativeBinding.call`, `RuntimeFunctionModifier.available` and
`RuntimeFunctionModifier.set_enabled`. Lifecycle callbacks are `on_load`,
`on_update` and `on_unload`, supplied by each mod and called by the common runner.

The typed override targets, claim factories and the live resource field reader
introduced while closing the findings below extend this initial inventory.
Their current signatures are documented alongside the implementation in
`runtime.lua` and `assets/manager.lua`.

The final declared surface contains 99 runtime declarations and 15 assets
declarations. Added runtime entries are the five typed target factories, four
advanced claim factories, two claim methods, `restore_unused` and
`values.normalize_field`. Assets adds `read_loaded_field`. `ecs.write_fields`
keeps its existing first two results and additionally returns confirmed fields.

## Shared behavior established by the source trace

The production data API contains no branch granting SF Survival or SF Unlimited
Flight special access based on either mod ID. Mod IDs identify the caller for
capabilities, lifecycle, effects and claims. References to particular survival
types in the reviewed API tests are fixtures, not a runtime allowlist for those
mods.

| Data path | Type and storage identity | Calculation and writes | Authority and result |
| --- | --- | --- | --- |
| ECS component | Canonical qualified reflection type, original registration/runtime role, matching live size, opaque entity handle with generation | Shared inline codec, selected leaf masks, complete-snapshot native CAS. AttributeComponent subtypes use the complete original root calculation | Active ingame mod with Runtime capability, current original world/Scene and component tracker permission, independent native readback |
| Named attribute | Original KFC attribute name or stored ID, initialized root ID and definition identity on the specific entity, verified inline storage | Complete original root program. Named and generic component mutation paths now validate every root slot before CAS | Same ECS authority and generation guards as component writers |
| Detached values and calculations | Active parser registry and owned bytes or caller-provided root values | Owned serialization/evaluation, no engine pointer traversal or live mutation | No native engine call is implied |
| Template configuration | Current entity-to-template UUID, concrete original template registration, unique owned BlobVariant selection | Same resource field codec and single aligned 32-bit native CAS | Original nonzero session, both runtime asset and ECS write gates, shared resource scope |
| Loaded ResourceService data | ResourceId constructed by the active KFC format and active reflection, immutable owned snapshots | Single aligned 32-bit reflected scalar, complete fresh payload CAS. Prior immutable snapshot or explicit expected scalar guards caller intent | Active Runtime mod, current nonzero creation session and original authoritative gates, complete native readback |
| Original named service | Exact original DI name, matching reflection, individually admitted plain-data contract and original mutable system binding | Shared inline codec, mandatory expected field, optional related input expectations, complete-snapshot native CAS | Current original authoritative session and Scene, same Runtime capability, native binding revalidation and readback |

`runtime.functions.bind` returns already checked operation wrappers or an
independently proven owned-buffer transformation. Finding a system descriptor,
its dependencies or an RVA does not establish its native arguments, safe
execution phase, side effects or caller ABI. Most original code candidates are
therefore intentionally inspectable and uncallable.

`runtime.functions.bind_modifier` and `runtime.patch` are explicit exact-image
code patch interfaces. They do not become reflection writes and do not acquire
storage claims. The existing runtime patch capability is the Runtime capability
plus the distinct `runtime.gameplay.patch` feature. The separate Patch capability
is currently used for the pregame asset transaction.

## Findings and changes

### Fixed: named updates used a weaker root validation path

Before this audit, `runtime.ecs.update_attribute` checked the selected attribute
descriptor and called `Catalog::recalculate` directly. `write_attribute_storage`,
`ecs.write` and `ecs.write_fields` instead used
`recalculate_bound_component_update`, which checks every root descriptor and
each input/result scalar against its bound storage interpretation.

The direct arithmetic evaluator can accept a root whose untouched sibling has
unverified decoding evidence. It can also produce an integer result outside the
common nonnegative domain allowed when declared and storage signedness differ.
The previous named update did not perform those complete checks before CAS.

`update_attribute` now routes through the same complete validator. The validator
returns its existing calculation trace so the public result remains compatible.
The regression
`named_attribute_updates_validate_unselected_programs_and_derived_result_domains`
demonstrates that arithmetic evaluation accepts both constructed inputs while
the live mutation validator rejects them without changing the owned candidate
or mask.

Evidence: `env/runtime_attributes.rs`,
`runtime_resolution/attributes.rs::Catalog::recalculate` and
`runtime_resolution/calculation.rs`.

### Closed in the shared API: independent overrides could capture changed baselines

The original tracker kept its ownership map inside one mod instance. Expected
values and native CAS protected writes from stale bytes, but did not coordinate
baselines across mods. For example, mod A could change 100 to 200, then mod B
could capture 200 and change it to 300. If A observed B's value it relinquished
ownership. Restoring B could leave 200 with no owner of the original 100.

`env/runtime_ownership.rs` adds conservative exclusive claims on the actual
storage object. An ECS claim uses session, opaque entity and canonical runtime
component name. A service claim uses session and original DI key. A resource
claim uses ResourceId without a session because a loaded resource can survive
world changes. A template view and a loaded resource view for the same payload
share that resource claim.

Claims belong to the calling mod and use independent tokens. Several tokens
from one mod may share an object. A different mod is blocked until all tokens
have been explicitly released after restoration. Dropping an active token does
not silently release the record. An unavailable session zero does not erase
claims. A confirmed different nonzero session can remove old ECS/service claims.
Resource claims remain until explicit release. Typed adapters also retain their
creation session and validate it for every read/write, rather than rebinding an
old baseline to the next world's data.

The guards cover component `write` and `write_fields`, named attribute
`update_attribute` and `write_attribute_storage`, template `write_field`, service
`write_field`, `game.assets.write_loaded_field` and the separate original
`world.entity.set_scale` adapter, which changes CurrentTransform storage.
Unclaimed raw writes remain available. Claims cannot identify writes made by the
game itself, an unrelated native library or another process. Expected-value and
readback checks continue to handle external edits.

### Closed: restoration depended on each mod implementing cleanup correctly

The loader now registers every `runtime.overrides.new()` tracker privately by
mod ID. It restores registered scopes even without a mod `on_unload` callback,
and also when a callback fails. Busy cleanup keeps the writer scope available,
pauses regular callbacks and retries. Rebuilding the same mod factory preserves
its existing tracker registry. An uncertain helper result blocks subsequent
apply and restore writes. Errors in target calculations release freshly acquired
unmodified claims instead of leaking them.

The five typed target factories own the repeated readers, root calculations,
expected-value maps, field codecs and confirmed result handling. Survival and
Flight now use them directly. Their Lua packages contain feature policy and
original field names. Neither package imports or depends on the other.

### Closed: a successful resource write could be mistaken for a failed write

The previous Flight adapter reread live storage after confirmed native success.
A busy or changed subsequent observation could drop the first baseline and let
the next update multiply an already modified value. Typed resource targets now
decode the exact immutable snapshot returned by the successful writer, using
the same production field codec as the write. Integration coverage includes a
busy read immediately after success, a new multiplier and a deferred restore.

### Newly traced: upward pitch can use a post-clamp native input

The fixed intermediate target clamp is still a code literal. Complete private
execution of the original glider branch proves that a negative finite
`ItemInfo.equipment.gliderConfig.pitchAngleSpeed` can nevertheless produce an
upward final quaternion after that clamp. A signed-rate controller supports the
original `YawSpeedPitchSpeed` input mode. The mod restores the rate after landing,
unsupported input mode, multiple players, disable or unload.

108 parameter cases, 120 original rotation steps, 12 interpolation cases and a
177-frame sampled trajectory pass. The last trajectory uses the live captured
player template's rotation speed 200 and ExponentialLerp mode, with mixed frame
times. It tilts up to approximately -84.709 degrees, holds and tilts down.
This is an actively sampled controller. A negative stored rate has no native
timeout. If Lua stalls, the last rate can outlive the assumed sampling horizon.
Target-angle input mode is unsupported. Actual flight physics and live input
response remain unverified.

Evidence: `devdata/embervale-toolkit-3.1.0/glider-post-clamp-original-20261016.json`
and `player-glider-rotation-template-live-20261016.json`.

### Live R7 follow-up: gaps between query snapshots

R7 was installed and both DLL hashes were confirmed in process 18332. The user
reported normal flight with no observable improvement. Captured runtime status
repeatedly returned an incomplete query between successful player snapshots.
The native query cache expired after 50 ms and discarded its result while a new
incremental full-world scan was running. Flight restored its pitch override on
those pending responses. This made that query contract unsuitable for continuous
control even after its initial discovery completed.

The common provider now advances the refresh while returning the last complete
snapshot's members. Every returned handle is rechecked against its original ID,
generation, current entity-table membership and requested components. New matches
appear after the refresh completes. The first scan still returns pending. A
different binding epoch never reuses the previous snapshot. Idle query state
expires after two seconds. Reads and writes retain their independent validation.

The private original-owner fixture passes 126 checks including refresh continuity,
removal, generation changes, component removal, first-scan pending, epoch isolation
and discovery of a newly matching entity. R7 live samples are recorded in
`build/flight-survival-r7-live-samples.jsonl`. The corrected provider requires a
new DLL and a subsequent live test.

The same follow-up found that Flight tested only stored `Actor.currentState`.
The already traced original state getter applies the active action's
`setStateMask`, then removes `unsetStateMask`. Flight now reads all those reflected
fields and follows that original predicate. A regression covers HangGliding
supplied only by the action and removal taking precedence. This is mod feature
selection using public typed reads. It adds no private writer or mod-ID exception.

### Remaining API boundaries

1. Reflected storage does not reveal every native calculation constant. The
   independently traced glider pitch clamp is a code literal. The newly traced
   signed-rate input is an actively controlled later stage. Reflection still
   cannot manufacture a missing mutable field or original system entry point.
2. A component field write proves a guarded memory result. It does not prove
   that every original consumer reads the field again, publishes an event,
   refreshes UI, replicates to another process or saves it. Services and shared
   configuration expose that distinction explicitly.
3. ECS and service writes currently require the validated authoritative local
   world path. Calls are not automatically forwarded from a joined client to a
   remote server. Networking is an explicit separate protocol surface.
4. Complete ECS values containing unsupported native DS ownership cannot be
   constructed or written. Supported inline siblings remain available through
   `read_fields` and `write_fields`. Live resource/template mutation is limited
   to aligned 32-bit leaves. No generic allocation or container resize is
   exposed.
5. Named services require admitted exact-image plain-data contracts. A type's
   mere presence in reflection does not authorize every object with that name.
6. Loaded-resource indexes establish uniqueness in their captured index
   snapshot. They reread the selected candidate and invalidate after public
   resource writes. Concurrent additions or changes elsewhere in the
   ResourceService do not form a transaction with that lookup. This is stated
   in the public contract.
7. The claim is exclusive per complete storage object. Independent mods that
   target different fields of the same component, resource or service can
   therefore conflict conservatively. No implicit cross-mod dependency or
   ordered stack of calculated effects is invented.

## Review coverage

The initial primary data API review scope contained 18 source files. Eleven
were read completely, a whole-file coverage of 11 / 18 = 61.1%. Seven large
implementation files were traced at the entry points and relevant helpers.
This is a focused source/control-flow audit, not a claim of a repository-wide
line-by-line review.

Completely read in that initial scope:

- `shroudforge/v1/definitions/runtime.lua`
- `eml/v1/definitions/assets/manager.lua`
- `eml/v1/definitions/assets/resource.lua`
- `shroudforge/v1/lua/overrides.lua`
- `env/game/assets.rs`
- `env/game/assets/resource.rs`
- `env/game/assets/loaded_index.rs`
- `env/runtime_attributes.rs`
- `env/runtime_templates.rs`
- `env/runtime_services.rs`
- `runtime_resolution/calculation.rs`

Targeted reads in that initial scope:

- `env/runtime_values.rs`, production inline/resource codecs and policy, selected existing regressions
- `env/runtime_functions.rs`, production inventory, resolver binding and checked adapters
- `env/loader.rs`, public registration, operation availability, lifecycle gates, type mapping and all typed mutation paths
- `env/app_state.rs`, shared state, write faults and runtime ResourceInfo lifetime/snapshots
- `lib.rs`, lifecycle activation, disabling, refresh and unload behavior
- `runtime_resolution/attributes.rs`, storage binding and root arithmetic evaluation
- `runtime_resolution/functions.rs`, dependency and live registration evidence

Supporting provider evidence was inspected selectively in
`native/ecs/ecs_runtime.cpp`, `native/resource/resource_runtime.cpp`,
`native/service/service_runtime.cpp`, `native/provider/engine_context.cpp` and
`native/ecs/component_registry.h`. The small `env/game.rs`, `env/util.rs` and
`env/type_lookup.rs` were additionally read completely. The newly added
`env/runtime_ownership.rs` was read completely and checked separately.
The new `env/runtime_overrides.rs` and `shroudforge/v1/lua/override_targets.lua`
were also read completely during integration, together with both current mod
entry points and their integration fixtures.

Network packet handling, the full world editing subsystem, the native code
candidate inventory, vendored parser internals, generated Lua definitions and
all gameplay consumers were not subjected to a complete source review in this
scope. Public network/world commands were inventoried and their registration
and checked dispatch boundary inspected.

## Verification

The focused attribute suite passed after the canonical-validator change:
10 passed, 0 failed, two existing installed-executable tests ignored because
their environment inputs were not supplied. The new claim tests exercise
independent mod exclusion, multiple same-mod tokens and session/resource
lifetime rules. The combined API/library and Survival/Flight/restore integrations
passed after adapter integration, with 63 unit tests and three integration suites
passing and six environment-dependent unit tests ignored. Subsequent build
results are recorded by the release run.

No live game writes were made by this audit.
