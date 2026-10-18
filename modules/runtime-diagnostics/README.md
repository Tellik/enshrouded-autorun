# Runtime diagnostics module

This ShroudForge module records live loader health and Lua callback timings. It reads diagnostic snapshots from the KFC Runtime provider and adds loader-owned measurements such as callback duration, queue state, and mod errors. It does not inspect game memory independently and does not create or edit KFC Runtime profiles.

Runtime ECS discovery and active query-search diagnostics are included in the provider snapshot alongside loader health and bounded callback measurements. This first-party module has its own descriptor, workspace crate, CLI mode, status schema, and Modloader Settings entry.

Entity-manager changes and layout epochs are measured ECS-context transitions. They are not assumed to be world-join events. A missing provider or export is reported as a concrete failure. Measurements do not grant compatibility.

## Configuration and controls

The only configuration is `modules.runtimeDiagnostics` in `shroudforge/config/modloader-config.json`. The `enabled` setting activates the module. The separate `sessionActive` setting represents a running diagnostic session. The selected `jobId` chooses a built-in job or a validated custom job definition. The runtime polls controls every 500 ms and runs an accepted job on its existing game-process update loop. Sessions stop at `maximumDurationSeconds` and persist `sessionActive: false`. Starting a session increments `requestId` so an active session can be restarted intentionally. The loader schema rejects invalid settings.

Built-in jobs and their collectors are grouped under `diagnostics-jobs/`. Custom JSON definitions are loaded from the configured Runtime data folder under `diagnostics/jobs/`, which defaults to `shroudforge/runtime/diagnostics/jobs/`. The Modloader Settings page lists valid definitions and reports invalid files. Definitions select existing capabilities only. See [`diagnostics-jobs/README.md`](diagnostics-jobs/README.md) for the file format.

The job collectors call the KFC Runtime diagnostic export and loader-owned measurement points. The KFC provider keeps its native implementation, and execution modules retain the small measurement calls at the operations they measure. Job selection, sampling, aggregation and reporting stay in this module.

Use Settings → Modules → Runtime Diagnostics, or run:

```powershell
shroudforge\shroudforge.exe --runtime-diagnostics --root "C:\Games\Enshrouded" --start
shroudforge\shroudforge.exe --runtime-diagnostics --root "C:\Games\Enshrouded" --job runtime.queue --start
shroudforge\shroudforge.exe --runtime-diagnostics --root "C:\Games\Enshrouded" --snapshot
shroudforge\shroudforge.exe --runtime-diagnostics --root "C:\Games\Enshrouded" --stop
shroudforge\shroudforge.exe --runtime-diagnostics --root "C:\Games\Enshrouded" --status
```

Commands are saved requests. A running loader consumes them on its next update. If no game runtime is active, the next game-runtime start consumes the request. Status is reported through the generated `shroudforge/state.json` and validated against the module schema. The UI displays the active job, remaining duration and last sample time. It does not reconstruct elapsed work. Parser and API startup checks, build and profile validation, and compatibility decisions belong to other modules.

Diagnostics are written to the existing client or server log under the `diagnostics` source. They appear in the ShroudForge Debug Log tab with other loader messages. Follow the shared [logging rules](../../docs/development/write-and-review-logs.en.md). `logging.minimumLevel` controls which entries are written. The Debug Console's Display selector filters entries already present in the log. INFO summaries may be filtered when the minimum level is WARN or ERROR. Slow callbacks and measurement failures use WARN.

Profile investigation tools, live ECS capture, and component-registry audits are one-shot developer actions under `dev/profiles/`. They inspect a selected executable or process and save evidence for profile review. They are not jobs run by the loader. See that directory's README and the [game build and runtime profile guide](../../docs/development/inspect-game-build-and-create-runtime-profile.de.md). This module owns ShroudForge's diagnostic session, status file, logging, and UI controls.

## Find an action

| Action | File |
| --- | --- |
| Create a diagnostics session | `create_diagnostic_session.rs` |
| Apply saved settings every 500 ms | `refresh_session_settings.rs` |
| Begin a selected job | `start_diagnostic_job.rs` |
| Advance an active session | `tick_diagnostic_session.rs` |
| Collect a selected job | `diagnostics-jobs/capture.rs` |
| Collect native runtime health | `diagnostics-jobs/runtime_health.rs` |
| Collect dispatcher health | `diagnostics-jobs/queue_health.rs` |
| Measure mod callbacks | `diagnostics-jobs/mod_callbacks.rs` |
| Measure native and Lua operations | `diagnostics-jobs/operation_timings.rs` |
| Stop a job and save its result | `stop_diagnostic_job.rs` |
| Publish or read status | `publish_diagnostic_status.rs`, `read_diagnostic_status.rs` |
| Request start, stop, or snapshot | `request_diagnostic_action.rs` |
| Parse existing CLI arguments | `run_diagnostic_command.rs` |
| Display diagnostics controls | `ui/show-diagnostics.tsx` |

The job session preserves the existing duration limit, request IDs and callback measurements. A continuous job runs only while the in-game runtime is active and stops at its configured maximum duration. Older `areas` settings are discarded while loading configuration. The selected job defaults to `runtime.overview`.
