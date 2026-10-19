# Security Policy

## Supported versions

Security fixes are provided for the latest stable ShroudForge release. Older releases and development builds may not receive security updates; please update to the latest stable release before reporting an issue.

## Reporting a vulnerability

Please report suspected vulnerabilities privately. If GitHub private vulnerability reporting is enabled for this repository, use [Report a vulnerability](https://github.com/bonsaibauer/shroudforge/security/advisories/new). Do not open a public issue or include exploit details in a discussion.

If private reporting is unavailable, contact the repository maintainer through the contact information on their GitHub profile. If you cannot find a private contact route, open an issue asking for one without including sensitive technical details.

Please include the affected ShroudForge version, the relevant environment and steps to reproduce the issue. Give maintainers a reasonable opportunity to investigate and prepare a fix before publishing details.

## Automated scans

GitHub Actions runs Trivy filesystem scans on source code and dependencies. Before publication, the release workflows scan the packaged payload with Microsoft Defender, ClamAV, and Trivy. Defender and ClamAV detections block publication. Trivy findings are reported through SARIF and the release report, but do not block publication under the current workflow policy.

Each release run produces a unified Markdown summary and JSON report containing the source Trivy scan, release Trivy scan, Microsoft Defender scan, and ClamAV scan. The summary is added to the GitHub Actions run summary. The report artifact includes available SARIF files, raw scanner logs, scanner outcomes, and version or signature metadata. Reports and raw scan artifacts are retained for 90 days. Missing or skipped scanner results are identified as errors or skipped scans, not as clean results.

The Defender scan uses a run-specific temporary directory. It checks the release ZIP files and their extracted contents, then matches Defender threat records to the scan directory or original ZIP paths. It does not filter detections by their recorded detection time. This ensures that a real-time detection recorded during extraction is still included in the report.

## Review of issue #15

[Issue #15](https://github.com/bonsaibauer/shroudforge/issues/15) concerns the `dinput8.dll` distributed in ShroudForge release 1.7.0, build `39026384728`. Windows Defender on the affected system identified that DLL as `Trojan:Win32/Bearfoos.A!ml`. The issue was open when reviewed on 2026-10-19. The issue does not include a DLL sample, file hash, local Defender log, or screenshot.

### Comparison with the 1.7.0 workflow

The [1.7.0 Defender job](https://github.com/bonsaibauer/shroudforge/actions/runs/39026384728/job/117378105024) updated Defender signatures, extracted the release ZIP, and logged that it found no malware. That clean CI result conflicts with the detection reported for the released DLL. In the [workflow at that release](https://github.com/bonsaibauer/shroudforge/blob/v1.7.0-build.39026384728/.github/workflows/build.yml), the detection timestamp was set after extraction. The workflow then accepted only detections recorded after that timestamp. A Defender real-time detection raised while extracting the ZIP could therefore be excluded from the workflow result.

The timestamp filtering defect is confirmed in the old workflow. The historical run log does not show whether Defender recorded the 1.7.0 DLL detection and the filter discarded it. The filtering defect is a plausible explanation for the conflicting results, but the available log does not prove that this is what happened.

| Area | 1.7.0 workflow | Updated workflow |
| --- | --- | --- |
| Defender timing | Set the timestamp after extraction and discarded earlier detections. | Uses a run-specific scan directory and considers matching detections regardless of detection time. |
| Defender output | Reported the result in the job log. | Produces a structured JSON report with status, detections, scanned file count, signature update result, engine and signature versions, and protection metadata. |
| Scanner reporting | Displayed Trivy summaries and uploaded individual SARIF and ClamAV artifacts. | Combines source Trivy, release Trivy, Defender, and ClamAV results in a Markdown summary and JSON report. |
| Artifact retention | Trivy SARIF and ClamAV logs were retained for 14 days. | Security reports, SARIF files, and available raw logs are retained for 90 days. |
| Release gate | Defender and ClamAV failures blocked publication. Trivy findings were non-blocking. | Defender and ClamAV findings still block publication. Trivy findings remain non-blocking and are included in the report. |

### Assessment and follow-up

The Defender reporting blind spot is fixed in the current workflow implementation in `.github/workflows/build.yml` and `.github/workflows/dev_build.yml`, with scan logic in `dev/build/scan_release_with_defender.ps1` and report generation in `dev/build/write_release_security_report.py`.

The reported Defender detection against the 1.7.0 `dinput8.dll` is the incident under review. The workflow change fixes the CI reporting gap, but it does not establish whether the DLL is malicious or a false positive. The exact 1.7.0 release artifact still needs to be scanned with the updated workflow, and any detection needs review alongside the DLL's hash and Defender threat details. Keep issue #15 open until that review determines the status of the reported DLL.

The Python and PowerShell scripts passed syntax checks, and `git diff --check` was run.

### Latest GitHub Actions attempt

The latest run failed during submodule checkout, before the build and security scans started. ShroudForge referenced `kfc-parser` commit `3c6565b0ce96a9d6ab8cf141ef3351a917bfa615`, but that commit was not available from the submodule remote. GitHub Actions reported `upload-pack: not our ref`. This run produced no Defender, ClamAV, Trivy, or SARIF result. It is a checkout failure, not a clean scan or a malware finding.

The submodule pin is now set to `742b1cb796238ebdec143842545a5a8f5fc79459`, the reachable ShroudForge-compatible revision on `kfc4_compatibility`. It includes the current upstream `main` commit `618bd0eb56dbd9c6a38842965bcf9665b9a5bab6` as an ancestor while retaining the parser integration APIs used by ShroudForge. A new GitHub Actions run is needed to confirm checkout and produce scan results.
