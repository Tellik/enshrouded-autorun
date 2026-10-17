# Runtime discovery, client and dedicated server

The discovery tools extract the complete **reflected type registry** of an EXE,
including fields, enum values, attributes, defaults, four hash domains and
metadata RVAs. They also enumerate every PE exception range, follow chained
unwind information to group procedure fragments, disassemble those ranges,
record direct calls/jumps and hash references, and inspect live metadata tables.
They do **not** claim that every engine function has a known ABI or gameplay
meaning. There is no observed universal `hash -> callable engine function` table.

The source for the reflection layout is
`kfc-parser/crates/kfc-base/src/reflection/extract/parser.rs`.
The extractor preserves addresses which the parser normally discards, without
modifying the parser submodule. Chained unwind parsing follows Microsoft's
[x64 exception-handling format](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64).

## Run a complete capture

From the repository root, use your existing Python installation and install the pinned analysis dependencies when needed:

```powershell
$python = 'python'
& $python -m pip install -r dev/profiles/requirements-discovery.txt
```

Build the shared production verifier, start client and server, then run:

```powershell
cmake --build build/native-runtime --config Release --target kfc-runtime-verify-component-registry
& dev/profiles/capture_runtime.ps1 `
  -Python $python -VerifyBufferAdapters
```

The capture action uses the Steam directories by default. It accepts `-ClientDirectory`,
`-ServerDirectory`, `-Target client|server|both`, `-OutputDirectory`, `-Offline`,
`-NativeVerifier`, `-ResourceExporter`, `-VerifyBufferAdapters`, and `-RunCallbackCorpusTests`. A live capture automatically
verifies a freshly discovered manager with the production reader and writes
`runtime-bindings.json` plus `runtime-index.json`. `-VerifyBufferAdapters` runs the isolated owned-buffer differential check.
The existing Lua corpus is a separate action, selected only with
`-RunCallbackCorpusTests` or `checks/run_callback_corpus_tests.ps1`.
No build or corpus test runs implicitly during a normal capture.
A live pass reads committed readable memory and can take minutes for a large
client. It does not inject code, suspend the process, write memory, or call game
functions. Pages that cannot be read are listed; a running process is not an
atomic snapshot. Every reflected descriptor header and registry slot is checked
against the selected executable; candidate table entries are read back again.

For explicit input and a single process:

```powershell
& $python dev/profiles/capture_runtime.py `
  'C:/Program Files (x86)/Steam/steamapps/common/Enshrouded/enshrouded.exe' `
  --profile kfc-runtime/profiles/enshrouded/client/1076226.json `
  --pid (Get-Process enshrouded).Id --functions `
  --out dev/profiles/captures/my-client-build
```

`--profile`, `--pid`, and `--functions` are independent optional arguments. Static
reflection extraction needs only Python's standard library; live scanning uses
NumPy and function analysis uses NumPy and Capstone. No existing component map
is needed to find the reflection registry or allocation-shaped ECS table
candidates. A supplied profile supplies additional index/size checks, not names
for unknown functions. A changed executable requires a separate output directory.

## Artifacts and evidence levels

| File | Meaning |
| --- | --- |
| `reflection.json` | All entries in the discovered registry, original indices, field types/offsets, metadata addresses and hash collisions. Registry index is **not** an ECS component index. |
| `components.json` | Complete live engine registrations, separate runtime/configuration types, native callback references and independently observed entity-manager links. No component profile is used. |
| `functions.json` | Every exception range and chained procedure group; byte fingerprints, direct branches, immediate hash candidates, metadata references and data pointers into code. |
| `live.json` | Exact EXE/PID identity, regions and unreadable pages, validated metadata, contiguous pointer runs and sparse allocated table candidates. |
| `profile-audit.json` | Executable identity, hook/patch signature hits and world-operation guard checks for the supplied profile. Byte matches do not test gameplay semantics. |
| `runtime-bindings.json` | All reflected types, registrations and provisional native bindings, with separately validated adapters when requested. |
| `runtime-index.json` | Small executable/build index pointing at the full generated catalog. |
| `summary.json` | Counts and measured coverage. `fullRuntimeResolved` remains false. |
| `client-to-server.json` | Name-based layout differences and candidate counterparts of known operations across builds. |

All reports identify the executable by SHA-256, PE timestamp, image size and
target filename. `version` comes separately from the adjacent KFC data header;
it must not substitute for executable identity. Reusing a directory containing
reports from a different EXE is rejected.

Qualified hashes, internal hashes, impact hashes and short-name hashes are
different namespaces. All collisions remain explicit. Duplicate attribute
names follow the parser's last-value behavior, with displaced values retained
in `attributeDuplicates`.

`allocated-ecs-table-candidate` means a sparse array of unique reflected ECS
descriptors with an observed preceding byte-size field. Its engine role remains
a hypothesis. `profile-anchored-component-table` additionally agrees with at
least four existing component mappings and contains no observed profile
conflicts. Allocation shape does not prove every application-level bound.
Empty slots, absent types, and conflicting observations must not be invented.

Matching relocated procedure fingerprints masks only instruction address
operands, retaining other code bytes and immediate constants. All chained
fragments must match. Callee-graph candidates are weaker evidence and are listed
separately. The comparison also ranks caller links, named engine descriptors,
reflected metadata references, and noncolliding type-hash immediates. It includes
hooks and patches as well as world operations. Individual matching unwind
fragments and relative direct CALL/JMP structures provide additional partial
evidence. Multiple code matches retain their individual evidence and unresolved
conflicts. These methods do not prove
the target's context lifetime, thread,
argument types, function effects, or callees' semantics. Candidates cannot be
called through Lua and are never promoted to production profiles automatically.

Exception metadata can omit leaf functions, inlined code and generated code.
Code pointer targets without exception coverage and undecodable tails are
reported. Thus a list of all exception ranges is not a list of all engine APIs.

## Look up types and compare builds

```powershell
& $python dev/profiles/lookup_runtime_type.py `
  dev/profiles/captures/my-client-build `
  'keen::ecs::CurrentTransform' --out target/current-transform-evidence.json

# Decimal or 0x-prefixed hash; qualifiedHash is the default domain.
& $python dev/profiles/lookup_runtime_type.py `
  dev/profiles/captures/my-client-build 0x1399e0ba `
  --hash-kind qualifiedHash --out target/type-evidence.json

& $python dev/profiles/compare_runtime_builds.py `
  <old-capture-directory> <new-capture-directory> `
  --source-profile kfc-runtime/profiles/enshrouded/client/1076226.json `
  --target-executable $executable --out target/build-diff.json
```

The lookup retains **all** matching types, their live table slots, and relevant
code/hash evidence. The comparison rejects reports with mixed EXE identities.
Field comparisons resolve referenced types to names instead of comparing
build-dependent reflection indices.

`--source-profile` adds operation anchors introduced after the old capture and
checks that the profile belongs to the old executable identity. Optional
`--target-scan <native-function-scan.json>` adds current byte-pattern matches as
weak evidence after checking the new executable SHA-256. `functionMatching`
reports reciprocal unique anchors. Each operation retains ranked candidates,
evidence, ambiguity, and `callable: false`. Unsupported data-only operations are
reported explicitly without inventing a function counterpart.

Optional `--source-components` and `--target-components` accept separate live
registration captures for the exact old/new images. Otherwise `components.json`
is read from each capture directory when present. Stable callback ownership
uses registration names, callback origin/slot, layout version and stride. A
callback pointing inside a procedure is not treated as its entry point.
A leaf function without unwind coverage can retain an explicit byte-site
candidate only when the exact source profile guard agrees with the target scan
signature and original bytes. Its native ABI and procedure extent remain unproven.

With `--target-executable`, up to five candidates per operation receive an
`entryPointSignature`. The tool disassembles at instruction boundaries, masks
address operands, and scans all executable sections for uniqueness. It checks
the current EXE identity and bounds each pattern to its compiler-described
range. These signatures identify current procedure entries. They do not relocate
an old patch instruction or prove an operation's arguments and effects.

Export a profile-generator component catalog directly from a full live capture:

```powershell
& $python dev/profiles/capture_ecs_registry.py `
  $executable --capture <capture-directory> --export-only `
  --id enshrouded-client-new-build --catalog-out target/components-live.json
```

The exporter follows the captured runtime/template descriptor indices and
requires exact EXE/reflection/capture identity. Dynamic runtime types can have
different names and sizes from their configuration types. `components` uses
runtime descriptor names, while `registrationMappings` preserves both layouts
and template-only registrations. Real conflicts remain under `unresolved`.
`consistentSnapshot: false` on manager evidence means the running process was
not captured atomically. It does not establish whether a world was loaded.

## Trace known working byte anchors into original code

`trace-anchors` starts with an exact-image scan JSON containing `image` and a
`features` array of `name`, `signature` and previously observed `matches`.
It verifies the EXE SHA-256, extracts fresh reflection and disassembles all
compiler ranges. It then rescans every supplied signature in executable sections.

```powershell
& $python dev/profiles/trace_function_anchors.py `
  <exact-image-signature-scan.json> --executable $executable `
  --catalog <optional-registration-catalog.json> `
  --capture-out target/anchor-trace-capture --out target/anchor-trace.json
```

Every match is retained, including ambiguous and non-instruction-boundary hits.
`procedures` contains full primary unwind groups, original bytes, instructions,
engine descriptors, metadata references and all candidate hash interpretations.
Caller traces follow original direct CALL instructions for at most six edges.
They stop at the nearest named engine procedure on each searched branch and
retain alternative shortest edges in the caller graph. The saved capture can
be reused by `lookup` and `compare`.

Optional catalog `entries` add missing registrations. Entries without a byte
signature are explicitly `unresolved-data-mechanism`. Their metadata is retained.
When a catalog supplies `siteOffset`, or its captured Toolkit field `siteOffset130`
for a simple patch or `siteOffset178` for a cave,
`catalogSite` independently checks the resulting instruction boundary. Signature
start and declared intervention site remain separate evidence. A named caller or matching
type hash does not prove native arguments, entity scope, a writable field or a
gameplay effect. The output never authorizes engine calls or installs patches.

## Lua interface

After building/installing the updated loader and restarting the game:

```lua
local transform = game.types.get_by_qualified_name("keen::ecs::CurrentTransform")
local same_type, reason = game.types.get_by_qualified_hash(transform.qualified_hash)
assert(same_type, reason)

-- query/read/write/query_bounds accept qualified hash, name or Type.
local entities, query_reason = runtime.ecs.query(transform.qualified_hash)
if entities and entities[1] then
    local value, read_reason = runtime.ecs.read(entities[1], transform.qualified_hash)
end

-- Includes unresolved candidates, unlike get_components().
local catalog = runtime.ecs.get_catalog()
for _, entry in ipairs(catalog.entries) do
    print(entry.qualified_name, entry.qualified_hash, entry.resolved,
          entry.read_available, entry.write_available, entry.reason)
end

for operation, status in pairs(runtime.get_operations()) do
    print(operation, status.available, status.reason)
end
```

`Type` exposes `qualified_hash`, `internal_hash`, `name_hash`, and `impact_hash`.
`game.types.get_by_internal_hash(hash)` returns `nil, reason` on ambiguity.
Numeric ECS arguments always mean a **qualified type hash**, never a function
address or component index. Existing phase, capability, live layout, size,
entity generation and write checks still apply. A mapped type need not occur on
an entity; a write-capable component can still contain unsupported dynamic
fields. No API for arbitrary native calls is introduced.

## Validation

The full API contract and examples are in
[runtime-registry.md](../../runtime/runtime-registry-api.en.md). The live runner
now follows sparse storage arrays to full engine registration records. The
[follow-up investigation](2026-10-08-component-registry.md)
explains why 81 registered components have configuration but no entity storage.

`capture_ecs_registry.py EXE --capture DIRECTORY` augments a capture from a still
running process. The native `kfc-runtime-verify-component-registry` target uses
the production C++ reader in a separate read-only process to cross-check it.

```powershell
& $python -m unittest discover -s dev/profiles/tests -v
cargo test -p shroudforge-api --lib --offline
cargo build -p shroudforge-modloader --lib --release --offline
```

See [the 2026-10-08 investigation](2026-10-08-runtime-discovery.md)
for measured client/server results and the remaining gaps.

## Full runtime binding export and callback verification

The native production reader now inventories unwind groups plus code pointers in
loaded data sections. Lua merges in all live component callbacks and exposes every
candidate through `runtime.functions.list/get` with original descriptor names where proven, otherwise `unclear_<RVA>`, and
separate signature/context/effect validation. Code pointers can be internal labels.
The bounded proof checker can provide owned-buffer adapters without making an
unresolved native ABI callable. See [the public schema](../../runtime/runtime-registry-api.en.md).

For each current capture, run the native verifier using its current PID and
manager address, then run the opt-in Rust corpus test and differential verifier:

```powershell
$env:SHROUDFORGE_TEST_FUNCTIONS = "$capture/provider-verification.json"
cargo test -p shroudforge-api --lib --offline production_callback_binding_corpus -- --ignored --nocapture
& $python dev/profiles/checks/verify_component_copy_adapters.py $executable `
  --registry "$capture/provider-verification.json" `
  --adapters "$capture/provider-verification.json.adapters.json" `
  --out "$capture/adapter-verification.json"
& $python dev/profiles/export_runtime_bindings.py $executable `
  --capture $capture --out "$capture/runtime-bindings.json" --index $indexPath
```

The differential verifier executes only the accepted straight-line copy bodies in
its own process with owned buffers. It never writes to a game process. Generated
full exports include all reflected types and all registrations; small indexes
point at them without checking large generated data into source control. Current
server index: `dev/profiles/catalogs/enshrouded/server/1024233.json`. The server executable
profile has also been corrected to `profiles/enshrouded/server/1024233.json`.

## Verify original runtime mechanisms

Build the resource exporter explicitly, then request original-mechanism evidence:

```powershell
cargo build -p shroudforge-api --example export_kfc_resources --offline
./dev/profiles/capture_runtime.ps1 -Python $python -VerifyRuntimeOrigins
```

`-Offline -VerifyRuntimeOrigins` performs EXE/KFC analysis without opening a process.
The action checks the exact profile hash before validating patch bytes, modifier
requirements, callers, world-operation guards, and hook instruction boundaries.
It does not claim coverage of deleted historical mod packages. That old package
assertion and its original source are closed evidence under
[historical source evidence](source-evidence/README.md).
`-InspectModOrigins` remains a parameter alias for existing callers.

Results retain `resources.json`, `mod-origins.json`, and original execution
descriptors in `functions.json`. To verify an existing capture directly:

```powershell
cargo run -p shroudforge-api --example export_kfc_resources --offline -- $executable "$capture/resources.json"
& $python dev/profiles/verify_runtime_origins.py $executable `
  --profile $profile --functions "$capture/functions.json" --resources "$capture/resources.json" `
  --out "$capture/mod-origins.json"
```

The default exporter selects the original attribute/knowledge resource types.
`--type QUALIFIED_NAME` selects explicit types. `--building-inputs` uses the
original building resource selection and retains the ItemInfo projection.
The earlier Cargo example names compile this same implementation.

### Fresh resources and calculation semantics (2026-10-09)

`export_kfc_resources` always extracts types freshly from the EXE; it
never loads the installed JSON cache. Its optional `--backup` switch reads the
matched `.kfc.bak` / `.kfc_resources.bak` pair. Run both variants per target to
compare the 59 relevant resources, not just container timestamps.

```powershell
cargo run -p shroudforge-api --example export_kfc_resources --offline -- $exe target/fresh-resources.json
cargo run -p shroudforge-api --example export_kfc_resources --offline -- $exe target/backup-resources.json --backup
& $python dev/profiles/checks/generate_attribute_reference_vectors.py --exe $exe --resources target/fresh-resources.json --out target/vm-vectors.json
$env:SHROUDFORGE_TEST_VM = "$PWD/target/vm-vectors.json"
cargo test -p shroudforge-api --lib original_engine_differential --offline -- --ignored --nocapture
```

Run the concrete check from the repository root, as shown above.
The Python environment requires the pinned Unicorn package in
`requirements-discovery.txt`. It emulates the original signed, unsigned and
float interpreter code with read-only image memory, bounded steps and private
scratch pages; it never opens a game process. Current output is 65 nonempty KFC
programs Ã— 32 inputs = 2,080 vectors per executable, all matching the Rust model.

Shipped profiles now select `componentResolution: live-registration`; component
captures stay in diagnostics rather than being copied into the runtime profile.
`attributeCalculationModel` is an exact-build semantic authorization. The
profile generator deliberately removes it; relocating a code signature does
not establish VM equivalence in a new build. The public API and its remaining
limits are maintained in [runtime-registry.md](../../runtime/runtime-registry-api.en.md).

### Resolve every system dependency through the production resolver

The read-only inspector below uses the same `execution/reflection/functions.rs`
as the Lua function catalog. It joins every captured engine system descriptor
to freshly extracted parser types and the captured runtime/template roles.
It does not restrict discovery to the operations listed in an execution profile.

```powershell
cargo run -p shroudforge-api --release --example inspect_system_dependencies --offline -- `
  $exe "$capture/provider-verification.json.functions.json" `
  "$capture/provider-verification.json" "$capture/resolved-system-dependencies.json" `
  --registry-format provider-verification
```

The explicit `provider-verification` adapter supports the older external capture
format. Its template identity comes from the freshly parsed executable because
that capture did not record independent live template hashes and sizes. Use the
default `native` format for a production `registry()` capture. The inspector
reports which image identity checks were possible. Empty SHA256 fields are not
treated as exact identity proof.

The corrected KFC4 client inventory contains 722 system descriptors, 2,904
dependency references and 586 distinct dependency types. All names join to fresh
parser types and the captured registration roles. Both native and Python readers
preserve all six spans through descriptor offset `0x68`. The previous five-span
reader omitted 100 references, including the whole `copy_parent_level` descriptor.
Exact-image `systemDependencyLayout` evidence exposes template configuration reads
and runtime read/write roles in the same function catalog. Optionality, native
calling signatures and gameplay effects remain independently unresolved.

The standalone native verifier contains an experimental `--discover` mode for
observing resident private pages. It skips nonresident pages and reports that
limited scope. It is not automatic production discovery and does not establish
an active world, a loaded resource owner or an engine execution phase. Catalog
agreement only describes the allocations observed by that diagnostic run.
Production entity reads require a manager-bound registry and a freshly validated
original owner chain. This read-only path does not require a dispatcher hook.
Writes still require a verified execution phase. Scheduler access metadata does
not grant permission to submit arbitrary mutations into the original frame.

### Recover calculations and the actual engine data path

For a native modification, trace the original instructions after locating its
site. A UI label or a unique AOB does not establish the value's meaning.

1. Record the exact image hash and containing unwind procedure, including its
   chained fragments and direct helper calls.
2. Decode the original memory reads and arithmetic. Trace base-register writes,
   lookup return values, time conversion, limits and vector operations.
3. Compare the observed accesses with complete reflected containment paths.
   Keep Blob and DS layouts separate even when their internal hashes agree.
4. Prove relative descriptor bases against the existing parser codec. A component
   stored in `TemplateResource.components` is serialized template data. It is not
   evidence for an entity's live component column.
5. When a pure helper can run without engine services, bounded private emulation
   can check a reconstructed equation without loading a foreign DLL or changing
   the game. Keep special floating-point cases and gameplay effects explicit.
6. Trace resource identity to its loaded owner and the exact object consumed by
   the engine. Establish lifetime, update phase and dependent copies before
   implementing a live binding. `game.assets.Resource.data` currently operates
   on KFC file copies, not on loaded engine resources. A matching field layout
   does not close that gap.

For example, the KFC4 glider layout distinguishes `accelerationForward`, the
three directional air-resistance coefficients, and yaw/pitch/roll angle speeds.
A speed-labelled slider can scale an air-resistance input. A fixed angle limit
in executable code is a different contract from an angle-speed resource field.
Never substitute one merely because both affect flight.

The requested runtime must resolve gameplay access dynamically without modifying
executable game code. Neither the current hook-driven dispatcher nor the
toolkit's caves meet that requirement. The toolkit proves that particular
patched paths work for its implementation. It does not prove that a patch-free
engine path is impossible or that ShroudForge has already found one.
