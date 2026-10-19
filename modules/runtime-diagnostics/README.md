# Runtime diagnostics

This module owns diagnostic job selection, execution and result exports. All jobs use one schema, catalog and session lifecycle. Lua, Python, registered mod actions and compiled Rust diagnostic actions use that lifecycle.

## Source ownership

| Directory | Responsibility |
| --- | --- |
| `catalog/` | The shared job schema, validation and recursive job discovery. |
| `execution/` | The execution contract, Python adapter and verified result exports. `write_job_exports.rs` owns `JobExports`. Mod-runtime hosts Lua, mod-action and compiled Rust actions through its existing job host. |
| `session/` | Settings, requests, start, update, duration limits, cancellation and CLI controls. |
| `collectors/` | Runtime health, queue activity, callback measurements and operation timings. |
| `status/` | Existing diagnostic and shared mod-status documents and their schemas. |
| `diagnostics-jobs/runtime/` | Five collector job definitions. |
| `diagnostics-jobs/world-editor/` | Six World Editor jobs, including system and cursor type inspection through existing Python tools. |
| `diagnostics-jobs/game_inspection/` | Python inspection tools and five selectable capture or analysis jobs grouped by task, including the native Rust ECS dependency report. |
| `tests/` | Job, lifecycle, Lua fixture and Python checks. Tests are not bundled as runnable jobs. |
| `ui/` | The existing job dropdown, controls and report display. |

`build.rs` discovers job definitions and scripts recursively and bundles them automatically. Adding a job does not require another registration list. `lib.rs` preserves the existing public module API. See [AGENTS.md](AGENTS.md) for implementation boundaries and the [job guide](diagnostics-jobs/README.md) for the format.

## Running jobs

Use Settings → Modules → Runtime Diagnostics, select a job and use the existing Start, Stop or Snapshot control. The default remains `runtime.overview`. The module must be enabled. Merely listing a job never runs it.

The only session settings remain `modules.runtimeDiagnostics` in the loader configuration. The runtime consumes requests every 500 ms. Each request has its existing `requestId`, duration limit and status publication. Collector snapshots finish after one sample. Script jobs run to completion or the configured limit, including when requested through Snapshot.

Lua and mod-action jobs use an active mod's existing API permissions and world/thread checks. They do not activate a mod. Python jobs use the configured interpreter, run without a shell and have no Lua mod identity. Compiled Rust jobs are limited to named built-in actions handled by the existing mod-runtime host. They do not load source files or invoke Cargo at runtime. All jobs share the same stop, timeout and shutdown lifecycle.

Python requires an existing interpreter and the packages in `diagnostics-jobs/game_inspection/requirements.txt`. A job can set `execution.interpreter`. Otherwise the adapter uses `SHROUDFORGE_DIAGNOSTICS_PYTHON` or `python`. It does not install Python or packages.

## Output

Result files use the configured exports folder through `paths::export_dir`, under `diagnostics/<job-id>/`. The default is `shroudforge/exports/diagnostics/<job-id>/`. Lua jobs publish `report.json`. The voxel test also keeps `baseline.json` and blocks repetition when previous evidence exists. Python jobs write their original results, `stdout.log`, `stderr.log` and the adapter's `report.json` into the same job export folder. Temporary bundled sources use the configured cache under `diagnostics/`. User job files are not overwritten.

Session messages and collector samples use the shared Debug Log with source `diagnostics`. Regular collector samples are DEBUG entries and require `logging.minimumLevel` set to DEBUG or TRACE. Lua script messages retain the active mod's log identity. The existing status document continues to serve internal state and the UI. Existing installed files, captures, exports and recovery data are not moved or deleted.

The game-profile approval CLI and original-code emulation remain distinct developer evidence. Moving their Python implementation into this module does not grant runtime compatibility or prove native call signatures.
