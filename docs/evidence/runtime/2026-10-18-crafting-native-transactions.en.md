# Crafting and inventory native transaction investigation

Date: 2026-10-18. This is a dated investigation owned by `docs/evidence/runtime/`, not an approved runtime contract or an implementation claim. It records the evidence needed for a future typed runtime operation used by Crafting Essentials. The older modifier names describe ShroudForge interventions and must not be treated as original engine method names.

## Scope and executable identity

The executable was read from `C:/Program Files (x86)/Steam/steamapps/common/Enshrouded/enshrouded.exe`. Its identity was recomputed with the existing PE reader:

| Property | Value |
| --- | --- |
| Steam build associated with the capture | `25988036` |
| SHA-256 | `82e5020a8736348cd0374f004b2f5e5ef456b5bd27aa5420df34c7dd241cfeeb` |
| PE timestamp | `1792148057` |
| SizeOfImage | `44146688` |
| Version recorded in the existing capture | `1269579\|^/game38/branches/release_1.0\|2026-10-16T12:58:37.402390Z` |

All addresses below are RVAs in that image unless explicitly labelled historical. They are investigation evidence, not mod settings or stable Lua identifiers. The running game remained open as PID `10068`. This investigation did not invoke a game native function or modify the process.

Inputs were the original current executable, the existing function/reflection capture under `dev/profiles/captures/enshrouded-client-25988036/native-function-discovery/`, and the historical [1076226 profile](../../../modules/game-runtime/profiles/enshrouded/client/1076226.json). The local capture reports are ignored developer data. The function catalog contains 722 named engine execution descriptors. Its functions remain marked `semanticStatus: unverified` and `callable: false` unless a separate verified adapter supplies a contract.

## Historical intervention and present quantity-removal path

The historical `zero_resource_argument` modifier belongs to function `0x3822B0`, with patch site `0x3822D1`, in the exact old image selected by profile `1076226`. It replaced the load of the sixth Windows x64 integer argument with zero. The profile explicitly says `complete_native_signature: false`. Its image hash, byte guard and RVAs do not authorize any current call.

The cross-build report ranks current function `0x40A080` as the corresponding candidate. Inspection of its complete current instructions establishes a quantity-removal path with the following argument roles. These C++ names are explanatory aliases reconstructed from operands. They are not recovered source symbols or a currently published API:

```cpp
struct QuantityResult {
    std::uint8_t status;             // +0
    std::uint8_t padding[3];
    std::uint32_t auxiliary;         // +4, zeroed in the examined paths
    std::uint32_t remaining;         // +8
};

QuantityResult* remove_quantity(
    QuantityResult* result,          // RCX, also returned in RAX
    InventoryTransaction* tx,        // RDX, current stack-owned transaction
    std::uint32_t owner_entity,      // R8D
    std::uint32_t item_id,           // R9D
    std::uint64_t preferred_slot,    // argument 5, entry RSP + 0x28
    std::uint32_t quantity);         // argument 6, entry RSP + 0x30
```

At `0x40A0AC` the function loads the sixth argument into `r15d`. It resolves an inventory setup for the entity, tries a supplied slot, then traverses linked inventories and their eight slots until the requested quantity is exhausted. `0xE83DF0` decodes the fifth argument into an entity ID in its low 32 bits and a slot byte from the high 32 bits. This is consistent with the reflected `keen::InventorySlotId` fields `entityId` and `slotIndex`.

The small wrapper at `0x40A050` forwards the requested quantity and supplies the default preferred-slot value from the engine image. It is an ordinary direct native call, not a registered service callback.

Both removal branches of `0x40A080` call `0x40A360`. The latter receives seven argument roles: result, transaction, writable inventory view, inventory entity, slot byte, item ID and quantity. It checks the item ID and slot write eligibility, decodes the available stack count, clamps the removal to the available amount, records a journal entry and changes the original stack through the original tracked accessor.

| Current instruction | Observed effect |
| --- | --- |
| `0x40A3EF` through `0x40A403` | Compute `min(requested, available)` |
| `0x40A40E` | Append a transaction journal record through `0x412390` |
| `0x40A42D` | Store the removed quantity as UInt16 at journal record `+0x24` |
| `0x40A432` | Set journal kind byte `+0x26` to `1` |
| `0x40A46D` | Obtain the original tracked writable view for full removal |
| `0x40A478` and `0x40A47E` | Clear the stack payload in the full-removal branch |
| `0x40A49B` through `0x40A49E` | Read source count at slot `+4`, then subtract the removed amount |
| `0x40A4AB` | Obtain the original tracked writable view for partial removal |
| `0x40A4B0` | Write the resulting source count |
| `0x40A483` or `0x40A4B5` | Compute remaining requested quantity |

For a valid stack with a positive available count, quantity zero produces removed amount zero and remaining amount zero. The partial-removal branch compares the unchanged count and skips the writable accessor. This establishes why changing that argument can preserve the stack while reporting that the quantity request is satisfied. It does not establish that every earlier validation, transaction consumer or gameplay outcome is equivalent to the old build. In particular, the function still validates its inventory setup and supplied slot. A zero-count or malformed stack is not covered by the positive-stack statement.

## Source removal and the historical split intervention

The old `preserve_split_source` modifier suppressed `sub dword ptr [rsi+4], ebp` at historical `0x38A574` in owner `0x38A3D0`. It only suppressed that partial-removal instruction. A full-removal branch that clears the stack was a separate original path. Consequently, the old effect is not accurately described as preserving every source stack during every transfer.

The current source-removal function at `0x411F50` has these reconstructed argument roles:

```cpp
SourceRemovalResult* remove_from_source(
    SourceRemovalResult* result,     // RCX, 24-byte result storage
    InventoryTransaction* tx,        // RDX, opaque borrowed transaction
    std::uint64_t source_slot,       // R8, InventorySlotId by value
    std::uint32_t requested_count,   // R9D
    bool clear_item_id_on_full);     // argument 5
```

It resolves the source slot through `0x40F340`, whose slot decoding reaches `0xE83DF0`. The reflected storage is `keen::ecs::Inventory.slots`: eight `keen::ItemStack` values of size 12. Each stack has item ID at `+0`, count at `+4`, and persistent-item entity ID (`pide`) at `+8`. The available-count helper `0xE79830` reports count for an ordinary stack and one for a persistent item.

At `0x412013` source removal appends a record through `0x412390`, writes quantity at record `+0x24` and journal kind `4` at `+0x26`. The exact current partial-removal instruction is `44 29 66 04` at `0x4120F4`, or `sub dword ptr [rsi+4], r12d`, followed by `EB C4` at `0x4120F8`. The full-removal path instead clears the stack at `0x4120AD`/`0x4120B0`, or clears count and persistent-item ID at `0x4120B5` depending on the fifth argument. Setting that boolean false therefore does not preserve a fully removed source count.

Two observed direct paths are:

```text
inventory_actions  0x3FD610 → 0x3F3290 → 0x412110 → 0x411F50
equipment_actions 0x381E70 → 0x37FA40 → 0x40DAD0 → 0x411F50
```

Passing zero as the removal count would also produce zero output quantity for subsequent destination operations. It would not reproduce the old intervention, which suppressed only the source subtraction while leaving the calculated transfer result and journal quantity intact.

The reflected `keen::ecs::InventoryTransferAction` supplies `type`, `flags` and a UInt16 `amount`. For ordinary compatible stacks, with source count `S` and available destination capacity `C`, the traced direct-slot paths compute:

| Original action | Actual transferred amount `M` |
| --- | --- |
| `All` | `min(S, C)` |
| `Half` | `min(floor(S / 2), C)` |
| `One` | `min(S, C, 1)` |
| `CustomAmount` | `min(S, C, uint16(amount))` |

The public `One`, `Half` and `All` enum values are 0, 1 and 2, while wrapper `0x412110` uses internal modes 2, 1 and 0 respectively. The compatible direct-slot path at `0x40D6D0` bounds the request by destination space. The custom-amount path at `0x40DAD0` additionally bounds it by the supplied amount. Positive successful paths remove from the source through `0x411F50` before crediting the destination through `0x40FE80`. The separate source and destination journal kinds are 4 and 5. The source-to-entity path can return unplaced quantities to the source through the original insertion helper.

`TransferItemToInventoryFlag` contains `Gifting`, `RefillStacks`, `MoveAllStacks`, `ToActionbar` and `ToBackpack`. The examined handlers use them for validation, refilling, inventory traversal and destination selection. None supplies independent source debit zero and destination credit `M`. There is no flag test between the source journal and partial subtraction. A zero amount or identical source/destination slot produces a no-op. The instruction/reflection record is `split-action-dataflow-20261018.json` in the current client capture directory, containing 13 original procedures and 2,897 decoded instructions. This is static dataflow evidence, not a live transfer test.

## The forwarded boolean is not an established item-consumption switch

The old `clear_item_use_argument` intervention changed a forwarded boolean at historical `0x385C2A` in owner `0x3857D0`. Its name does not establish the original boolean's meaning. The old profile only documents that a sixth boolean is forwarded as false, and retains `complete_native_signature: false`.

The current callgraph/operand counterpart is `0x40D130`. Its source-removal call at `0x40D564` reaches `0x411F50` before the boolean is loaded. At `0x40D584` it loads its sixth argument, passes that value as the seventh argument of `0x40DE90`, then reaches `0x410060`. In that downstream function the boolean selects inventory filtering and free-slot search. The false branch at `0x41018D` reaches reads at offsets `+0xA0` and `+0xA4`, consistent with the reflected `SlotSelection.actionbarEntityIds` layout. The opaque native context's complete type has not been verified live. This is not a demonstrated global "consume item" gate. Applying false here cannot be advertised as disabling all item consumption.

There is a separate real reflected `keen::actor::PayUsageCost` configuration. It has size 120, `ignoreCostMask` at `+112` and `consume` at `+113`. The original `actor_pay_usage_cost` system checks event type `0x4E5C0937` at `0x224107` and calls `0x21FC30` at `0x224149`. That helper receives an event payload in RCX, configuration in RDX and the current system frame in R8. The configuration `consume` byte is tested at `0x220068`. When true, the later inventory path requests quantity one through `0x40A080` at `0x220180` and commits through `0x408B50` at `0x2201A2`.

Mana, stamina and other usage-cost handling precede that `consume` test and remain separate behavior. Setting the original configuration's reflected `consume` field false is therefore a concrete data-driven path for this event's item consumption. It does not require changing the helper's instructions or invoking it separately. This event path does not establish that crafting, equipment transfers and all other item consumers use the same gate.

The later read-only `actor-sequence-consumer-resource-link-live.json` report closes the ownership relationship. All 2,157 live `ActionSequenceRegistry` pointers match their canonical `ResourceService` resource spans, with no mismatches. The original `actor_pay_usage_cost` system record identifies `ActionSequenceRegistry` as its first read service, and both normal and alternate bindings identify the same live registry. The report records 232 `PayUsageCost` event payloads, including 75 with `consume=true`, and revalidates the final owners and every observed event field. Their inline variant payload addresses are inside the same resources exposed by reflected `ActorSequenceResource.subSequences[].events[]` paths. These are original consumer data, rather than detached copies. This was a memory readback, not a configuration write or item-use gameplay test.

`keen::actor::ApplyBuffEvent.consume` is a second established event gate. In `actor_apply_buff`, instructions `0x25E5F8`/`0x25E603` form the pointer to the original variant payload's reflected byte at `+0x70`. The test at `0x25E819` jumps directly to original buff application at `0x25EBBE` when false. Its true branch creates the inventory transaction and reaches quantity removal at `0x25EA64`. The mod therefore selects these two original event types and overrides only their `consume` leaves. The separate `ignoreCostMask` is not modified.

The historical executable was not available in this investigation. The old/current item-use correspondence is based on historical profile/capture evidence and current operands and call relationships. It must not be reported as an independently executed comparison of both original binaries.

## Actual named callers of quantity removal

The following current paths were obtained by resolving original execution descriptors and following direct call edges in the current image. The outer names are original engine execution names. The inner helper names have not been recovered as source symbols.

| Engine execution name | Current direct call path |
| --- | --- |
| `actor_apply_buff` | `0x25E4A0 → 0x40A080` |
| `actor_pay_usage_cost` | `0x224040 → 0x21FC30 → 0x40A080` |
| `actor_spawn_entity` | `0x2241D0 → 0x221230 → 0x40A080` |
| `inventory_actions` | `0x3FD610 → 0x3F8C00 → 0x40A080` |
| `player_crafting` | `0x2E0E50 → 0x2C18E0 → 0x3F0830 → 0x40A050 → 0x40A080` |
| `factory_station` | `0x330660 → 0x31FF90 → 0x3236B0 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `equipment_actions` | `0x381E70 → 0x37FA40 → 0x40A050 → 0x40A080` |
| `animal_food_consumption` | `0x22B880 → 0x225850 → 0x40A050 → 0x40A080` |
| `server_player_fishing` | `0x398B60 → 0x39A4A0 → 0x3891F0 → 0x40A080` |
| `player_cooking` | `0x2DF030 → 0x2C2DC0 → 0x40A050 → 0x40A080` |
| `player_ui` | `0x2B34F0 → 0x3F0830 → 0x40A050 → 0x40A080` |
| `player_skilltree` | `0x2E3380 → 0x2C90C0 → 0x2C8A30 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `handle_crafting_events` | `0x274A40 → 0x470F10 → 0x3F1CB0 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `player_building_blueprints` | `0x2D30D0 → 0x48EB30 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `player_building_dismantle` | `0x2D3AE0 → 0x48EB30 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `player_building_place_prop` | `0x2D4E30 → 0x48EB30 → 0x3F1760 → 0x40A050 → 0x40A080` |
| `player_building_undo` | `0x2D5980 → 0x48EB30 → 0x3F1760 → 0x40A050 → 0x40A080` |

The inventory helper therefore has broader reach than player crafting alone. This table records found paths, not an exhaustive proof of every possible caller or indirect call.

## Original context, mutation tracking and lifetime

The original `actor_pay_usage_cost` system at `0x224040` receives a query iterator in RCX. Its first pointer identifies the current execution owner. `0x9BAB60` initializes its per-iteration output frame, whose size for this system is `0x140`. `0x9B5A50` advances the iterator, including its cursor at `iterator+8`, and binds current component/service views into that frame.

A wrapper that enumerates this iterator before calling the original would consume original iteration state. Reusing an advanced iterator is not a valid callback adapter. The iterator's live owner, access phase, mutable accessor data and restoration requirements must be established separately.

For usage-cost processing the next helper, `0x21FC30`, constructs an inventory transaction on its own stack. Its observed sequence is:

```text
0x21DD00   initialize transaction and inline scratch buffers
0x411DF0   bind current native contexts and accessors
0x40A080   remove requested quantity
0x408B50   process successful transaction effects
0x21DE50   destroy transaction and release grown buffers
```

The transaction includes these observed regions:

| Offset | Role |
| --- | --- |
| `+0xF0` | Journal data pointer |
| `+0xF8` | Journal record count |
| `+0x100` | Journal capacity, initialized to `0x200` |
| `+0x108` | Optional journal growth callback |
| `+0x118` | Inline journal storage, `0x34` bytes per record |
| `+0x7140` | Affected-inventory collection |
| `+0x71A8` | Internal flag, reset by initialization and binding |

These are private transaction internals. They are not reflected stable types and cannot be exposed as durable Lua-owned memory. The context and accessors remain borrowed from the engine while the original call is active. A callback entry or remembered thread ID alone does not supply them.

The writable-view helper at `0x408630` receives the original view structure:

| View offset | Observed role |
| --- | --- |
| `+0x00` | Component/slots pointer |
| `+0x08`, `+0x10`, `+0x18` | Arguments forwarded to the original mutation recorder |
| `+0x20` | Component registration index, UInt16 |
| `+0x22` | Execution-phase byte |
| `+0x28` | Original recorder context |

When the recorder context exists, the helper calls the direct original recorder at `0x9C5DF0`, then returns the component pointer. This is a direct call, not a function pointer stored in the view.

The production [ECS write implementation](../../../modules/game-runtime/native/ecs/write_entity_components.cpp) and [scene write guard](../../../modules/game-runtime/native/engine-context/lock_scene_access.cpp) deliberately restrict the current direct writer to components with neither original tracking-category bit set. The earlier `inventory-stack-dataflow-20261016.json` capture for build `25966634` records both categories for `Inventory`/`InventorySetup`. That historical classification is not an independently sampled classification for the current image. Removing the production restriction would not establish the missing native transaction phase, queue capacity, stamps or alias tracking.

## Structural resolution and callback replacement are different operations

The existing [named-service resolver](../../../modules/game-runtime/native/named-services/resolve_named_service.cpp) demonstrates a production path from the held registry owner to original system records. A system record contains the original descriptor at `+0` and callback at `+0x28`. The resolver validates descriptor identity, callback identity, full service name, mutable dependency ordinal and both normal/alternate bindings.

That path is useful evidence for resolving and validating a named system. It does not currently implement callback replacement or transfer the engine's synchronization and ownership rights to Lua.

Even a correctly synchronized wrapper around such a registered callback only intercepts the outer system boundary. The quantity-removal and source-stack operations above are internal direct calls. Replacing an outer callback does not change their arguments. The journal growth callback is also not a per-transaction interception point because the original append helper uses it only when its capacity is insufficient.

A new typed native transaction policy needs a proven intervention point for the actual argument or mutation, a validated context lifetime and completion semantics, and restoration on mod/session shutdown. A profile can record and validate those contracts, but a name, cross-build score or matching call topology alone does not prove them. The existing profile schema is the canonical location for approved build contracts. This report does not add an alternate profile format or authorize copying old patch bytes into the current profile.

Calling the quantity-removal helper separately from Lua would perform another inventory operation. It would not suppress the engine's existing request. Restoring removed stacks after a system callback would also occur after some original transaction effects and cannot be presented as the old zero-argument behavior.

## Verification and remaining work

The executable identity, current native instruction streams and listed direct call paths were checked with the existing PE reader, descriptor reader, captured function graph and `objdump -d -Mintel`. The [structural inventory inspector](../../../dev/profiles/inspect/inspect_inventory_transactions.py) now derives its candidates from original named system descriptors, reflected fields, compiler ranges and decoded argument provenance. Its quantity candidate includes an isolated-emulation contract that identifies six direct call sites by their five dependency roles. It contains no build hash or RVA catalog entries for the current helper.

The final inspector run used the current executable directly, without `--capture`, `--reflection` or `--functions`, and read PID `10068` through the optional `--pid` path. Quantity decrement, usage-cost consumption and the legacy forwarded transfer boolean each resolve to one candidate with complete required instruction/origin evidence. Partial split has one candidate but remains `unique: false` with status `incomplete-instruction-or-origin-evidence`: its slot resolver's pointer output is not modelled. Matching the subtraction instructions alone does not prove that missing origin. The analysis preserves unknown origins at CFG merges and derives the relevant count getter's return values from its original CFG and stores. The traversal reached 265 procedure groups, decoded 271 groups including additional leaf dependencies, and traced origins in 28 groups. Ten reached groups contain undecoded ranges and are listed separately in the report. This is not a complete callgraph audit or proof that every indirect/inlined path has been discovered.

The final inspector compared all compiler ranges of the three fully resolved owner groups against the running process. All 3,502 bytes matched the original executable, with no changed bytes. An earlier run read 3,940 bytes across four candidates, but its earlier origin conclusions were superseded by the stricter analysis above. The report explicitly retains `liveVerified: false`, `callable: false`, `processMemoryWritten: false` and `consistentSnapshot: false`. The readback verifies selected code identity, not arguments, a coherent transaction snapshot, invocation safety or gameplay effects. Supplying an older image's reflection capture was also checked and correctly rejected with exit status 1.

The final local reports are `dev/profiles/captures/enshrouded-client-25988036/native-function-discovery/inventory-transactions-20261018.json` and `inventory-decrement-20261018.json` in the same directory. They are ignored capture artifacts, not distributed approval data. With the pinned discovery dependencies installed, the equivalent commands from the repository root are:

```powershell
python dev/profiles/inspect/inspect_inventory_transactions.py `
  'C:/Program Files (x86)/Steam/steamapps/common/Enshrouded/enshrouded.exe' `
  --pid 10068 `
  --out dev/profiles/captures/enshrouded-client-25988036/native-function-discovery/inventory-transactions-20261018.json

python dev/profiles/checks/verify_inventory_decrement.py `
  'C:/Program Files (x86)/Steam/steamapps/common/Enshrouded/enshrouded.exe' `
  --inspection dev/profiles/captures/enshrouded-client-25988036/native-function-discovery/inventory-transactions-20261018.json `
  --out dev/profiles/captures/enshrouded-client-25988036/native-function-discovery/inventory-decrement-20261018.json
```

The existing [inventory decrement check](../../../dev/profiles/checks/verify_inventory_decrement.py) consumes that contract, checks its exact-image identity and redecodes every dependency call against the original image. It rejects additional calls, unresolved argument provenance and ambiguous helper candidates. It maps the original helper instructions into a private Unicorn machine. Slot-key generation, eligibility, count decoding, journal allocation and the tracked writable accessor are explicit stubs. No original engine dependency or native tracking recorder is executed.

Both supported current-image paths were run: automatic fresh structural inspection and `--inspection` with the inspector's exact-image JSON report. The existing command shape remains supported, and the older exact-image `BUILDS` entries were preserved. The original legacy server image with SHA-256 `001c1b40ed091d8c1aee583adde3800d7c858ae2c7f4dff54fca2938b2be1637` passed its original eight cases through the unchanged command shape.

Current original-code results, with an initial ordinary stack of 20 and no persistent-item entity:

| Requested quantity | Source count after | Journal quantity | Remaining request | Tracked-accessor stub calls |
| --- | --- | --- | --- | --- |
| 0 | 20 | 0 | 0 | 0 |
| 1 | 19 | 1 | 0 | 1 |
| 10 | 10 | 10 | 0 | 1 |
| 20 | 0 | 20 | 0 | 1 |
| 25 | 0 | 20 | 5 | 1 |

The check also asserts journal kind, slot key, item ID, full-removal flag, returned result pointer, unchanged remaining inventory slots, and unchanged private context/view buffers. These are executions of original current instructions with controlled engine dependencies. They do not validate a live engine ABI, real journal allocation, tracking queues, commit effects, or gameplay equivalence across all callers.

An earlier read-only spot check in the same session compared live code against disk at quantity entry `0x40A080` (64 bytes), slot quantity helper `0x40A360` (64 bytes), consumption gate `0x220068` (16 bytes), and partial split `0x4120F4` (six bytes). All matched the original executable. The final inspector run additionally checked every compiler range of its three fully resolved owner groups. No new native call or gameplay action was triggered by either check.

No live native arguments were sampled and no new runtime hook was executed by this investigation. The unresolved requirement for reproducing the broad historical quantity/split interventions is an approved transaction contract at the inner original operation. Original reflected event configuration and production resource/service fields provide narrower existing data paths, documented below, that do not need such an interception. Claims of their actual gameplay effects still require the respective crafting, item-use and split actions in game.

## Native production time and the reflected Lua fields

The original `factory_station` execution is at `0x330660`. Its reflected mutable `keen::ecs::FactoryStation` has `runningRecipe` at `+4`, `nextRecipe` at `+40`, `recipeStart` at `+80`, and `recipePauseDuration` at `+88`. Its original read services include `BalancingRegistry` and `g38_items::RecipeRegistry`. `keen::RecipeInfo` has size 168, `recipeId` at `+16`, and `craftingDuration` at `+56`. The latter is `keen::Time.value`, an inline signed 64-bit nanosecond value. `keen::RecipeRegistryResource.recipes` is the relative array at root `+8`.

In the Running state, `0x330824` loads the original `RecipeRegistry`, `0x330828` loads `runningRecipe`, and `0x33082B` calls the original map lookup `0xE7FBD0`. It returns the actual `RecipeInfo` pointer. At `0x33083C`, the original `BalancingRegistry` getter `0xE767D0` reads the live GameSettings pointer at registry `+0x90`, then `factoryProductionSpeedFactor` at settings `+0x3C`. For ordinary finite positive inputs its returned float is `1.0f / factor`. There is no 0.25–2.0 clamp in this consumer. The getter explicitly returns zero if the absolute factor is at most approximately `1e-20`, rather than performing that reciprocal. Negative, non-finite, and sentinel time behavior are not proposed as useful settings.

The instructions at `0x330848` through `0x330869` compute the effective duration. `f32` below means IEEE binary32 rounding, and `trunc64` means conversion to a signed integer by truncation toward zero:

```text
baseNs      = RecipeInfo.craftingDuration.value
inverse     = f32(1.0 / GameSettings.factoryProductionSpeedFactor)
baseSeconds = f32(double(baseNs) * 1e-9)
durationNs  = trunc64(double(f32(baseSeconds * inverse)) * 1e9)
deadline    = TimeAdd(FactoryStation.recipeStart, durationNs)
complete    = nativeSimulationTime >= deadline
```

The constants are original double `1e-9` at `0x13B3670` and double `1e9` at `0x13B7B90`. `0x33086E` calls the original Time addition at `0x1926F0`, which additionally handles the engine's `INT64_MIN`/`INT64_MAX` sentinels. The normal finite branch simply adds the two nanosecond values. The completion comparison is at `0x330884`/`0x330887`. The subsequent output helper `0x3282A0` can still enter `MissingOutputSpace`; elapsed time does not bypass material, power, environment or output-space requirements.

The factor is reread on each Running invocation, while `recipeStart` remains the existing start. A changed factor therefore changes the deadline of an already running recipe from that start. It is not a rule that only affects later starts. It also follows that a one-second minimum for the recipe's base duration does not impose a one-second minimum on the effective duration after speed scaling.

### Pause, resume and progress

The state enum is `NoRecipe=0`, `MissingMaterials=1`, `Paused=2`, `Stopped=3`, `Running=4`, `MissingOutputSpace=5`. The start path `0x3286E0` checks/consumes input through `0x31FF90`, enters Running, copies `nextRecipe` to `runningRecipe`, and stores the original simulation time as `recipeStart` at `0x3288CC`.

The pause helper `0x323BE0` enters Paused, computes `now - recipeStart`, and stores that elapsed active time in `recipePauseDuration` at `0x323D16`. The resume helper `0x34BDB0` restores Running when the pause conditions clear and stores `now - recipePauseDuration` in `recipeStart` at `0x34BF68`. Despite its name, that pause field is the preserved elapsed production time, not the remaining duration. Ordinary active progress can consequently be derived from `(now - recipeStart) / durationNs`; this report does not claim that a specific UI widget uses that exact unclamped ratio.

The display helper `0xB1C0A0`, reached through `0xD1EB50`, performs the same float duration conversion, then uses its separate snapshot's count at `+0xA0` and start at `+0x28` to compute `(count + 1) * duration - (now - start)` on ordinary finite values. That input is not the `FactoryStation` component layout. Its complete reflected snapshot identity is unresolved, so those offsets must not be exposed as invented FactoryStation fields.

### Private execution of the original arithmetic

The existing PE reader and Unicorn executed the original `0xE767D0` getter, the original `0x330841` arithmetic block and original `0x1926F0` Time addition with private recipe/settings/station buffers. Original image constants were mapped unchanged. There were no stubbed dependencies, game process writes or native calls in the running game. Twenty cases covered base durations `1,000,000,000`, `3,000,000,000`, `100,000,000,001`, and `900,000,000,000` nanoseconds and factors `0.25`, `1`, `1.3`, `2`, and `4`.

| Three-second base, stored factor | Original-code effective duration |
| --- | --- |
| 0.25 | 12 seconds |
| 1 | 3 seconds |
| 1.3 | 2.307692289 seconds |
| 2 | 1.5 seconds |
| 4 | 0.75 seconds |

`100,000,000,001` nanoseconds at factor 1 becomes `100,000,000,000` after the float conversion, which demonstrates the actual precision loss. The factor-4 case also establishes that the previously reported 25–200 percent "game limit" was not a limit in this original consumer. Any such restriction in Crafting Essentials is mod validation. The record is `production-time-formula-20261018.json` in the current client capture directory.

### Live ownership joins and actual control paths

A read-only session resolved the current owner chain and original named services. `BalancingRegistry+0x90` was exactly the address resolved independently as named `ecs::GameSettings`, with a stored factor of 1.0. Two observed FactoryStations, entities 1030 and 1032, were both `MissingMaterials`, with no `runningRecipe` and `nextRecipe=2859124257`. Both original map lookups resolve to `RecipeInfo` address `0x2001CB58270`, whose duration at `0x2001CB582A8` was 1,000,000,000 nanoseconds. No production cycle was started by the investigation. Four production-related code snippets, 501 bytes in total, matched the original executable. The record is `production-live-read-20261018.json` in the current client capture directory.

The full `recipe-registry-consumer-resource-link-live.json` closes the resource relationship, rather than assuming that the map stores a copy. The original current ResourceService resolves GUID `5e57de0f-280c-4559-8191-f02c1ba656e2`, part 0, handle 48593, to span `0x2001CB394F8`, size 927,539. Its reflected `recipes` relative-array header is at `0x2001CB39500`, with first element at `0x2001CB398E0`. All 2,536 occupied `g38_items::RecipeRegistry` entries point exactly at their corresponding inline `RecipeInfo` elements in that loaded resource. There are no mismatches or runtime-only IDs. Recipe 2859124257 is Lua ordinal 747, and its `craftingDuration.value` address is the same `0x2001CB582A8` consumed above. Original resource callbacks, owner vtables/backpointer, ItemRegistry's bound resource, map bounds, and unchanged map keys/values/occupancy were checked. The process was not suspended, so this is still not an atomic world snapshot.

This establishes two concrete type-based Lua control paths:

```lua
local service, reason = runtime.services.get("ecs::GameSettings")
local observed = assert(service:read_fields({ "factoryProductionSpeedFactor" }))
local ok, error = service:write_field(
    "factoryProductionSpeedFactor", desiredFactor,
    observed.factoryProductionSpeedFactor)

-- With a freshly resolved loaded RecipeRegistryResource and its original ordinal:
local updated, error = game.assets.write_loaded_field(
    resource, "recipes[747].craftingDuration.value", desiredNs, observedNs)
```

These paths use reflected names and the original data consumed by the engine. The mod must discover the current resource and ordinals from their reflected values, not embed the example GUID, addresses or ordinal. Named service resolution and scalar writing remain guarded by the active world session, original owner/gate validation and expected-value checks. The new resource scalar support for 1/2/4/8-byte leaves admits the signed 64-bit `Time.value` and byte event fields, rather than replacing them with unrelated 32-bit fields. Successful compilation/private fixtures do not mean that the new DLL has been loaded into the existing game session or that a workshop/item action has been tested.

## Narrow additional cost fields found in original game code

The reflected `keen::ecs::ShroudForge` is an original game template configuration, not loader-owned configuration merely because its name resembles this project. It has `maxGemLevel` at `+0x24`, `craftingCostModifier` at `+0x28` and `allowAllMaterialTiersForUpgraderInsert` at `+0x20`. Its current registration is template-only. The reflected modifier has default 1 and UI metadata range 0–2.

The actual `equipment_actions → 0x37FA40` gem-action branches load that modifier for Slot and Upgrade. Slot calls `0xE78260` at `0x3802D2`, then skips its `0x40A050` cost request when the result is zero. For ordinary nonnegative inputs, the helper computes `ceilf(roundf((insertBaseCost - maxGemLevel) * modifier) * materialScale)` if `insertBaseCost > maxGemLevel`, otherwise zero. Upgrade calls `0xE782E0` at `0x380040`, likewise skips cost on zero, and computes `ceilf(roundf(((maxUpgradeCost / maxGemLevel) * currentLevel + (maxUpgradeCost * increasePerLevelPercentage) * currentLevel) * modifier) * materialScale)` with the original float operations. Here `roundf` is the observed half-away-from-zero conversion using an added signed 0.5 before truncation, and the helper's subsequent casts are UInt32-based on normal nonnegative costs. Missing balancing data or zero maximum level gives zero.

The balancing inputs are the reflected `BalancingTable.gemCrafting` fields: `maxUpgradeCost`, `insertBaseCost`, and `increasePerLevelPercentage`. Optional material scaling comes from `RecipeRegistryResource.inputCategories[].items[].scaling` via `0xE7FD30`; without the template's allow-all-material flag it is 1. Unslot calls `0xE781F0`, whose current original code is simply `xor eax,eax; ret`, so the existence of an `extractBaseCost` reflected field does not establish that this build consumes it here. Recycle calls the separate reward helper `0xE78320`, including `salvageGainPercentage`, without passing this cost modifier.

Thus zero is a genuine native cost-free case for those Slot/Upgrade branches. This field is not an established global recipe/building/resource-cost switch. Its entire template-to-loaded-resource write relationship and gameplay effect were not verified in this investigation, and it is not automatically added as a general crafting option. Similarly, setting `RecipeInputResource.itemStack.count` to zero is not a substitute for the historical quantity argument intervention. The `0x3F0830` handling specifically belongs to **AutomaticTransfer**, where a zero resolved count produces a zero craftable limit. It must not be generalized to the actual CraftRecipe consumer, which follows a different path. The historical helper intervention retained the positive original requirement before replacing the removal argument. The follow-up below records the corrected consumer relationship.

## Runtime scalar implementation checks reported by the owning agent

The owning implementation agent completed `build_kfc_runtime.ps1` after fixing the MSVC byte compare/exchange intrinsic. The existing native `kfc-runtime-test-resource` fixture was rebuilt and passed 795 checks, including 1/2/8-byte scalar cases, CAS conflicts, preserved sibling bytes, restoration, rejected misalignment and the advertised supported-width list. The existing Rust command `cargo test -p shroudforge-api --lib lua::values::tests::` passed six tests, with two pre-existing installed-capture tests ignored. These are source/private-fixture checks of the scalar data path. The new native API was not yet loaded into PID 10068 when these results were reported. They do not establish an actual in-game resource write, completed workshop recipe or non-consuming item action.

The Release Modloader build also completed through `build_modloader.ps1 -SkipUiBuild`, preserving the existing built UI. The existing `bundled_lua` and `runtime_overrides` targets each passed one check against the current sources. Crafting Essentials now contains the reversible typed data bindings for live recipe duration and the two item-consumption events. The global historical quantity intervention and partial-source preservation remain unimplemented. No new code hook was installed, and the running game still needs the rebuilt DLL pair before the new field widths can be exercised through Lua.

The process later restarted as PID 18064 while the investigation was in progress. Its loaded module paths and installed file hashes still identify the previous DLL pair. Its runtime status advertises only the older four-byte scalar writer. No attempt was made to replace DLLs in that running process. Installation and gameplay verification therefore remain pending a closed game, rather than being reported as completed by the restart alone.

## Live installation and hybrid settings follow-up

After the user closed the game, the rebuilt native and Lua runtime DLLs and current Crafting Essentials package were installed with backups and matching source/destination SHA-256 checks. Existing setting values were preserved. The next process, PID 8944, advertises `resourceWriteWidths=[1,2,4,8]`, guarded local-authority writes, and no resource write fault. Its startup asset pass changed 368 timed recipes to three seconds.

The installation initially retained the older release `shroudforge.exe`. Its three separate UI processes rejected newer module configuration fields and repeatedly logged `Additional properties are not allowed ('enabled' was unexpected)`. The DLL and new CLI embed the current schema. The old UI hosts were stopped through their existing manual-reset stop events, the events were reset, the matching new CLI was installed, and the same three role command lines were restarted. The game remained PID 8944 throughout. The repeated schema errors stopped. This was an incomplete local deployment, not a Crafting Lua exception or evidence of a broken release updater.

Independent `ReadProcessMemory` observations then resolved the current original owner chain and named registries again. All 2,536 RecipeRegistry pointers and 2,157 ActionSequenceRegistry pointers joined exactly to canonical loaded resource payloads. Baseline observations were:

| Original consumer field | Independently observed live value |
| --- | --- |
| 368 positive recipe durations | 3,000,000,000 nanoseconds each |
| 232 PayUsageCost consume fields | false |
| 120 ApplyBuffEvent consume fields | false |
| Normal and alternate GameSettings production factor | same pointer, 1.0 |

The observer rechecked 2,890 scalar values, world owners, registry maps and resource spans. It did not acquire the engine gates or suspend the process, so this remains a sequentially revalidated observation, not an atomic world snapshot. The user reported that production worked after supplying ingredients. No stopwatch duration or item-use stack comparison was supplied.

A temporary Lua action probe was refused by the existing startup asset fingerprint after its source changed. It never activated or performed a probe write. The clean installed source was restored byte-for-byte and the ordinary Runtime reload succeeded. No startup record was forged or bypassed.

Changing the ordinary Keep Used Items setting to false exposed another concrete issue: `refresh_runtime_configuration.rs` gated both activation and setting changes on `runtime && !patch`. The false setting was therefore not delivered to the hybrid mod. The independent capture still observed all consume fields false. This is **not a successful restoration test**. The saved setting was returned to true.

The settings refresh now admits hybrid value changes only when every changed setting is marked `applyAt=live` and `restartRequired=false`. Runtime-only behavior and hybrid activation boundaries are preserved. Existing lifecycle tests were extended for live hybrid values, deferred restart values, runtime-only behavior and capability checks. All five lifecycle tests passed and the Release workspace build completed. The updated DLL requires another ordinary game restart before the setting transitions and restoration can be verified live.

After a second confirmed ordinary game shutdown, the matching native DLL, Lua runtime DLL, CLI and updater outputs were installed together with another backup and hash record. PID 15296 started with this build. While still outside a ready world, changing Keep Used Items to false produced `Runtime settings applied live` and the mod's `Keep Used Items is off` status. Returning it to true was also delivered live. This confirms the repaired hybrid delivery condition in the loaded DLL. Actual resource restoration, the five-second 64-bit write and duration scaling still require a ready world.

An additional independent original-file comparison matched all 352 live consume fields. Exactly 75 PayUsageCost and 11 ApplyBuffEvent fields were originally true and were observed false in PID 8944. The other 266 fields were already false. All observed PayUsageCost ignoreCostMask values remained equal to the original data. This establishes 86 actual changed byte flags, rather than assuming that every false field represents a write. The existing bundled Lua compilation and override recovery targets were rerun after the hybrid fix and each passed one test.

Installation hashes, backup files, read-only observer, baseline and disabled-setting captures are under the ignored `dev/profiles/captures/enshrouded-client-25988036/live-crafting-20261018/` directory.

## Corrected actual recipe-cost consumer

The current `CraftRecipe=0` dispatch produces CraftRecipeEvent and reaches `handle_crafting_events → 0x470F10 → 0x3F1CB0`. Its UI simulation calls the same `0x3F1CB0` operation and then discards the transaction. `AutomaticTransfer=1` instead reaches `0x3F0830`.

Direct ingredient amounts originate from `RecipeRegistryResource.recipes[i].input[j].itemStack.count`. Category amounts originate from `inputItemCategory.count`, with native category material scaling and `ceilf`. Separate `RecipeInfo.waterCharges` controls water requests. Zero input counts really can permit crafting with no ingredient inventory in the actual CraftRecipe consumer. Thirty isolated original-code cases covered crafting flags, missing ingredients, category rounding and water/batch amounts. Engine lookups and output recording were explicitly stubbed, so these are not gameplay tests.

Factory start uses the input-only transaction flag `0x08`, environmental validation additionally uses `0x40`, and completion uses `0x10` to generate output without charging inputs twice. Refund reconstructs **current** recipe inputs and water charges. Editing recipe counts consequently changes requirements, automatic filling and cancellation refunds, including existing orders. These data controls are not a global 1:1 replacement for the historical quantity-argument intervention. No recipe-cost setting was silently substituted for that original feature.

The detailed read-only report, original instruction ranges and private execution results are in the ignored `recipe-material-cost-native-dataflow-20261018.json` capture.

## Source-free native insertion and remaining split coupling

The original admin creation path is `inventory_actions → 0x3F5940 → 0x40B170 → 0x40B760 → 0x408B50`. `0x40FE80` provides a concrete ordinary-item insertion operation with destination slot and quantity, journal kind 5, and no source-removal call. The existing AdminInventoryCreateAction consumer nevertheless ignores its count field and chooses one item or a full stack with automatic target selection.

The original split UI publishes TransferItemToInventoryEvent through `0xB04A80`, type `0x2C`, with target/source SlotIds, CustomAmount and a UInt16 amount. This request still enters the original source-removal transaction and has no established skip-debit flag.

The internal insertion helpers require the original running ECS program dependency views, mutable tracking context and correctly initialized transaction buffers. `0x411DF0` copies context and flags but does not initialize the full transaction. Original callers additionally initialize three owned native lists, commit or discard the journal, and free their buffers. A generic game-thread callback alone does not establish this inventory-specific phase or context. No direct adapter or automatic 1:1 split replacement is installed by these findings. The old partial SUB suppression also retained source journal kind 4.

The ignored `admin-item-create-dataflow-20261018.json` capture contains 23 original procedures or leaves and 5,651 decoded instructions. The commit's embedded jump-table data is explicitly recorded as an incomplete linear decode.

The further `free-crafting-request-producers-20261018.json` capture follows the nine captured direct call edges of the relevant crafting targets. Among these original callers, only Factory completion sets the free-input flag. Normal CraftRecipeEvent has no corresponding reflected selector. The existing admin create request is item granting, not recipe production.

The further `native-function-discovery/admin-item-create-producer.json` capture follows the original input publisher: `0x3A2470` copies ClientPlayerInput admin commands into PlayerInput, and `0x954E40` publishes a deep-copied input history entry keyed by entity and tick. The history is bounded to 64 entries and an already-present tick is not replaced. These original routines do not establish a thread-safe external producer. The specific AdminInventoryCreateAction producer and safe queue/phase remain unresolved, so `nativeAbiProven` and `callable` remain false. Neither direct PlayerInput writes nor an invented transaction context were admitted from these pointer relationships.

## Live data checks and recipe-cost implementation follow-up

PID 15296 was started with the matching 1/2/4/8-byte scalar runtime and hybrid-setting delivery fix. During a ready world, the read-only observer rejoined all 2,536 recipe registry entries and 2,157 action-sequence entries to their canonical resource payloads and revalidated 2,890 scalar values per capture.

The user changed Production Time from 3 to 5 seconds while the process remained open. The observer recorded intermediate batches, then all 368 positive recipe durations at exactly 5,000,000,000 ns. Setting it back to 3 seconds returned all 368 to exactly 3,000,000,000 ns. At 200 percent, the named GameSettings service's `factoryProductionSpeedFactor` read back as 0.5 with the world baseline at 1.0. The setting file was returned to 100 percent. The world owner chain became unavailable before a final 100-percent readback, so that last value is configuration restoration, not a confirmed live service readback. The user had previously reported that production worked after supplying ingredients. This pass did not time an actual production cycle.

Turning Keep Used Items off restored exactly the 75 original true `PayUsageCost.consume` flags and 11 original true `ApplyBuffEvent.consume` flags. Turning it back on returned the observed applicable flags to false. The final all-false capture reported 232 `PayUsageCost` and 120 `ApplyBuffEvent` values, with owner and registry revalidation passing. This proves live field changes and restoration against the original data. It does not prove a practical item-use stack comparison.

The mod now implements **No Recipe Resource Costs** through the original typed `keen::RecipeRegistryResource` data before startup. It zeros direct `input[].itemStack.count`, category `input[].inputItemCategory.count`, and `waterCharges`, retaining IDs, categories and outputs. The setting explicitly requires restart, while item-use and both production settings remain live. This gives the original CraftRecipe and factory consumers genuine zero-count inputs, but it changes displayed requirements, automatic filling and cancellation refunds. It remains a recipe-scoped behavior and is not equivalent to the historical shared quantity-removal modifier for building, upgrades and other inventory callers.

Per-setting `applyAt` and `restartRequired` metadata is accepted by the extended-mod schema and parser. The package test verifies explicit live and restart schedules survive source inference on hybrid mods. All 12 `shroudforge-package` unit tests passed, and the Release Modloader build completed with `build_modloader.ps1 -SkipUiBuild`.

After a normal game restart, PID 2164 logged the startup pass changing 368 timed recipes to 3 seconds and removing cost values from 2,536 recipes. The read-only live capture `crafting-values-recipe-costs-after-startup-retry-20261018T181150565167Z.json` rejoined and revalidated 2,536 canonical recipe resources. It observed zero positive direct ingredient rows, category ingredient rows and water charges across 7,173 input rows. All 16,882 recipe cost fields revalidated. It also read all 368 positive durations as exactly 3,000,000,000 ns and the named `ecs::GameSettings.factoryProductionSpeedFactor` as 1.0.

The first live capture raced a changing scalar during the mod's incremental runtime writes and was discarded. The retry revalidated owners, registry maps, 2,890 scalar values and all recipe cost fields. A later settled capture, `crafting-values-recipe-costs-after-startup-settled-20261018T181320638899Z.json`, again found all recipe costs zero and confirmed that all 232 `PayUsageCost.consume` and 120 `ApplyBuffEvent.consume` fields were false. The user then confirmed that a recipe worked in the loaded world without its ingredients. This is a user-reported gameplay result, while the read-only process capture independently proves the underlying recipe values. Building, upgrade and other shared inventory costs are outside the recipe-data change. Keep Source Stack When Splitting remains unavailable because guarded live Inventory writes are not exposed by this runtime build.
