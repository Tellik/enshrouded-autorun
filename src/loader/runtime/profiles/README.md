# Runtime profile format

Each JSON file describes one game executable build and process target. Keep the
sections in this order so the file reads from identity, through runtime layout
and ECS metadata, to executable patch definitions.

| Section | Meaning |
| --- | --- |
| `schemaVersion`, `id`, `target` | Profile format version, stable launcher selection ID, and executable name. |
| `kfcFormat` | KFC directory format expected for this build, currently `3` or `4`. The parser confirms the file's own header marker. |
| `image` | Executable identity: PE timestamp, image size in bytes, and optional SHA-256. |
| `allowStructuralRevalidation` | Whether automatic selection may try this execution profile when executable identity differs within the same KFC format. Manual selection also requires the same KFC format. |
| `ownerLayout` | Exact-image original singleton, client and controller ownership shared by ResourceService and ECS readers. |
| `ecsReadLayout` | Validated original execution-owner, registry-owner and entity-manager relation plus entity storage offsets. Enables stable reads independently of hooks, with `provenance.ecsReadValidated=true`. |
| `resourceLayout` | Original ItemRegistry pointer, ResourceService callbacks used only for validation, resource index and chunk layouts. |
| `resourceWriteLayout` | Exact-image, separately validated content-worker lifetime and private bulk scalar CAS. Requires resourceAtomicScalarWriteValidated and its evidence. Uses the shared thread registry fields in ecsWriteLayout. |
| `ecsWriteLayout` | Original registered local-authority thread lifetime and Scene execution gates. The current untracked-components policy admits only registrations with both original tracking bits clear. |
| `ecsLayout` | Byte offsets used to read the entity manager, entities, and component storage. These are offsets, not absolute addresses. All field names use `camelCase`. `entityDefinitionPointer` is the entity record's pointer to its definition. |
| `entityDefinitionLayout` | Offsets of the entity template UUID and name fields. |
| `hooks` | Signature and original instruction bytes for runtime entry hooks. `captureOffset` is only used by the cursor hook. |
| `worldContexts` | Offsets between the live game services used by world operations. |
| `worldGrids` | Per-build grid IDs, world origins, cell sizes, and maximum dimensions exposed by the native world API. |
| `worldOperations` | Guarded game functions or globals used by `runtime.world.*`. `*Rva` values are relative virtual addresses. |
| `componentResolution` | Shipped profiles use `live-registration`. the provider derives component identities/indices/sizes from the engine. Do not duplicate them in `components`. |
| `components` | Optional legacy fallback only, mutually exclusive with `componentResolution`. |
| `attributeCalculationModel` | Exact-build authorization for the independently tested owned attribute interpreter. Removed when generating a new-build draft. |
| `runtimePatches` | Guarded executable interventions, now attached to `runtime.functions` through `modifier`. legacy `runtime.patch.*` keys remain backend aliases. `function` identifies the unwind fragment. `modifier.function_rva` identifies the chained primary root. |

Numeric byte arrays use decimal values from 0 through 255. Signatures are
space-separated hexadecimal bytes. `??` marks a wildcard. RVA fields are
addresses relative to the executable image, while fields ending in `Offset`
are byte offsets within their documented structure or context.

Read profiles use `provenance.status="read-validated"`, an exact SHA-256 image
identity and `ownerLayout` with `resourceLayout`. They do not contain execution
hooks or native action definitions. Optional `ecsReadLayout` has its own live
validation and owner-chain evidence. Original component registration records
remain the only component name/index/size source for this path.

The optional write sections carry their own exact-image evidence. Resource
writes are limited to one aligned 32-bit scalar in the original private bulk,
with restored data-page protection. They do not mutate executable pages or
publish engine notifications. Component writes preserve complete snapshot CAS
and the original component tracking policy. The profile generator removes both
write sections when producing a draft for another image. Original consumer
dataflow and live effect checks remain separate from structural authorization.

The optional active-scene fields in `ecsReadLayout` additionally require
`provenance.ecsAuthorityReadValidated=true`. They select exactly one ready local
authority scene before falling back to the client replica when no local scene
exists. A transitioning or ambiguous scene is unavailable. Authority selection
does not authorize writes or task submission.

`modifier.engine_name`, when present, must match an original loaded execution
descriptor. `modifier.id` is explicitly a ShroudForge intervention name.
`attribute_requirements` checks stored KFC IDs, names, root IDs and element
indices before Lua can bind an attribute intervention. Static caller paths are
evidence of shared-helper reachability, not complete native signatures.

`inlineReferences` describes displacement/next-instruction/data offsets inside
a trampoline. The provider derives these relative references after copying the
payload. it never reuses an original EXE-relative displacement for inline data.

World operations with `validated: false` are always disabled, including when a
candidate address happens to match an instruction guard. `unresolvedReason`
states the missing evidence. Such entries may omit addresses entirely. a
candidate `functionRva` only supports an inspectable partial function binding.

Do not add fields to only one build profile without updating
`profile.schema.json`, the native profile reader, and the profile generator.

Server `runtime.world.context.active` uses ABI `actor-world-context` with an
instruction guard and store-validation offset. It intentionally has no client
global RVA. The cursor hook is optional (the headless server has none).
World operation guards establish native code availability. live context checks
and the game-thread dispatcher still gate actual calls. Zero in the original
finish event slot is a valid argument.
