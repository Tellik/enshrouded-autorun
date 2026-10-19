# Diagnostic jobs

All jobs use [`../catalog/diagnostic-job.schema.json`](../catalog/diagnostic-job.schema.json). Definitions and scripts are grouped by task. The catalog and build discover files recursively. No second registration list is required.

| Group | Contents |
| --- | --- |
| `runtime/` | Runtime overview, provider health, queue activity, callbacks and operation timings. |
| `world-editor/` | API read comparison, typed inputs, registered Impact calls, building-system and cursor type inspection, and a voxel write test with restoration. |
| `game_inspection/capture/` | Game-build and factory captures with their existing Python components. |
| `game_inspection/inspect/` | Inventory transactions, captured ECS system dependencies and other inspection components. |
| `game_inspection/checks/` | Inventory differential checks and other original-code evidence. |
| `game_inspection/analyze/`, `compare/`, `verify/` | Existing shared analysis and checks. Helper files are not independently selectable jobs. |

## A job definition

A job describes its ID, name, purpose and collector capabilities. Without `execution`, it collects the selected measurements as before. `execution.kind` selects the adapter: `lua`, `python`, `modAction` or a named built-in `rust` action. All use the same requests, duration limits, export paths and status messages.

```json
{
  "schemaVersion": 1,
  "id": "user.input-observation",
  "name": "Observe inputs",
  "description": "Reads typed inputs through an active mod.",
  "capabilities": ["runtime"],
  "execution": {
    "kind": "lua",
    "modId": "world-editor",
    "script": "world-editor/observe_typed_inputs.lua"
  }
}
```

Custom definitions and scripts live under `diagnostics/jobs/` in the configured Runtime data folder. Their IDs start with `user.`. Script paths are relative to this shared job folder. The referenced file must exist there. Subfolders are searched too. Unknown fields, duplicate IDs, paths outside the job folder and oversized files are rejected. JSON files are limited to 64 KiB and each script to 1 MiB.

## Adapters

- `lua` requires `modId` and `script`. The script provides `on_load`, `on_update` and `on_unload` as needed, plus an optional `update_interval_ms`. The host passes the actual elapsed monotonic time between callbacks. Normal mod permissions and native API boundaries remain in force. `diagnostic.call_mod(modId, action, value?)` invokes the same registered action as the UI.
- `modAction` requires `modId` and `action`, with an optional string `value`. The target mod must be active and the action registered.
- `python` requires `script`, with optional `args` and `interpreter`. The adapter passes arguments directly without a shell, provides the shared Python package and captures the exit code and console output. Cancellation and duration limits terminate the started process. The adapter does not install dependencies.
- `rust` requires a built-in `action` selected from the job schema. Implementations run in the existing mod-runtime host and use the same job lifecycle. Rust source paths and `cargo run` are not accepted. `inspect.system-dependencies` reads the exact-matching executable, function and live component registration reports from `capture.runtime`, then writes `system-dependencies.json` in its export folder.

Python arguments can use `{pid}`, `{gameExe}`, `{gameRoot}`, `{jobs}`, `{exports}` and `{diagnosticExports}`. `{exports}` is the current job's export folder, and `{diagnosticExports}` is the shared `diagnostics/` export root. For `capture.factory-state`, `capture.runtime` must first produce matching `reflection.json` and `components.json` files. The World Editor inspection jobs read matching `reflection.json` and `functions.json`. `inspect.system-dependencies` additionally validates the matching live registration capture. No manager addresses are guessed.

## Results and restarts

Result files live under `diagnostics/<job-id>/` in the shared configured exports folder. The default path is `shroudforge/exports/diagnostics/<job-id>/`. `JobExports` in `execution/write_job_exports.rs` uses the existing `paths::export_dir` resolver. Verified Lua exports use individual filenames of at most 80 ASCII characters and are limited to 4 MiB. Python programs retain their own result sizes and also save `stdout.log`, `stderr.log` and the adapter report in the same job export folder.

Session messages and collector samples use the shared Debug Log with source `diagnostics`. Regular collector samples are DEBUG entries and require `logging.minimumLevel` set to DEBUG or TRACE. Lua script messages retain the active mod's log identity. The existing status document continues to serve internal state and the UI. Temporary bundled scripts live under `diagnostics/` in the existing configured cache. Existing installed data, exports and recovery files are not moved or deleted.

| Lua call | Effect |
| --- | --- |
| `diagnostic.publish(report)` | Saves a JSON object as `report.json`, verifies the file by reading it back and updates the status. |
| `diagnostic.encode(value)` | Encodes supported Lua values as JSON. |
| `diagnostic.write_file(name, text)` | Saves and verifies a result file in the job export folder. |
| `diagnostic.read_file(name)` / `exists(name)` | Reads or checks a result file in the job export folder. |
| `diagnostic.finish(report?)` | Ends the job and optionally updates the status without overwriting a file. |

The World Editor voxel job is intended for a disposable test world. It saves an 8³ block, checks a no-op, changes one occupied cell and verifies restoration. An existing `report.json` or `baseline.json` blocks a restart and is preserved. Uncertain write results and cancellation do not trigger another world mutation.
