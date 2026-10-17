# How system update checks and installation work

The updater has several entry points, but they use one per-installation check lock, one cooldown record, one release lookup and one saved result. The Modloader button, automatic check and startup check do not each maintain their own GitHub checksum request loop.

## One release check

The UI check enters through `modules/updater/check_system_update.rs`. Automatic and startup checks enter through `modules/updater/system-updates/headless_check.rs`. Both use `modules/updater/system-updates/check_lock.rs` before contacting GitHub.

The lock is shared by ShroudForge processes for the same installation. Automatic checks respect `checkMinutes` in the loader configuration. The default is 60 minutes and accepted values are limited to 5 through 1,440 minutes. A manual check can bypass that interval, but cannot run alongside another check. The last attempt and last completed result are saved in the same loader configuration.

`modules/updater/system-updates/fetch_system_release.rs` makes one request for the GitHub releases list. It selects the newest eligible release that contains a ShroudForge ZIP and its matching `.sha256` asset. It then downloads that selected checksum once and saves it with the release result. A check with no eligible release makes no checksum request. Otherwise, the checksum request is made once for the selected release, not once per mod or UI row. A manual check requests it again because manual checks bypass the automatic interval.

The saved `latestRelease`, `lastCheckAt`, and `lastCheckAttemptAt` let the UI and startup flow reuse the same result. Automatic checks also reuse the recent check result until their configured interval allows another attempt.

## Download and install

Finding a release does not download its ZIP. The player explicitly queues the system update. The queued action downloads the selected archive, verifies it against the saved SHA-256 checksum, and stages the files. Installation waits until Enshrouded has exited so files currently used by the game can be replaced safely.

The startup check also verifies files managed by the installed release. Missing or invalid managed files open the updater's repair flow. A normal optional update can be shown once for its version. A required update blocks startup until the repair or installation step is accepted. If GitHub cannot be reached during an ordinary startup check, the game continues and ShroudForge records the failure.

## Where to follow the code

| Step | File |
| --- | --- |
| UI check and result state | `modules/updater/check_system_update.rs` |
| Automatic and startup check coordination | `modules/updater/system-updates/headless_check.rs` |
| Per-installation lock and interval | `modules/updater/system-updates/check_lock.rs` |
| GitHub release and checksum requests | `modules/updater/system-updates/fetch_system_release.rs` |
| Persisted result restored for the UI | `modules/updater/restore_checked_release.rs` |
| Check interval and enable switch | `modules/loader-config/loader.schema.json` and `modules/loader-config/loader.default.json` |

Mod catalog downloads and installation are separate queued operations. Their files and UI live under `modules/updater/mod-installation/` and `modules/updater/ui/`.
