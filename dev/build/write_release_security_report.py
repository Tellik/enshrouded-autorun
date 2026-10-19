#!/usr/bin/env python3
"""Write per-scanner and unified security reports for ShroudForge releases."""

from __future__ import annotations

import argparse
import json
import os
import re
from pathlib import Path
from typing import Any


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def sarif_findings(path: Path) -> tuple[list[dict[str, str]], dict[str, int], str | None]:
    if not path.is_file():
        return [], {"error": 0, "warning": 0, "note": 0, "other": 0}, "Die SARIF-Datei wurde nicht erzeugt."
    try:
        sarif = json.loads(read_text(path))
    except json.JSONDecodeError as error:
        return [], {"error": 0, "warning": 0, "note": 0, "other": 0}, f"Die SARIF-Datei enthält ungültiges JSON: {error}"
    if not isinstance(sarif, dict):
        return [], {"error": 0, "warning": 0, "note": 0, "other": 0}, "Die SARIF-Datei muss ein JSON-Objekt enthalten."

    findings: list[dict[str, str]] = []
    counts = {"error": 0, "warning": 0, "note": 0, "other": 0}
    for run in sarif.get("runs", []) or []:
        if not isinstance(run, dict):
            continue
        rules: dict[str, str] = {}
        tool = run.get("tool") or {}
        driver = tool.get("driver") or {}
        for rule in driver.get("rules", []) or []:
            if not isinstance(rule, dict):
                continue
            description = rule.get("shortDescription")
            rules[rule.get("id", "")] = description.get("text", "") if isinstance(description, dict) else ""
        for result in run.get("results", []) or []:
            if not isinstance(result, dict):
                continue
            level = str(result.get("level", "other")).lower()
            if level not in counts:
                level = "other"
            counts[level] += 1
            location = ""
            locations = result.get("locations", []) or []
            if locations:
                physical = locations[0].get("physicalLocation") or {}
                artifact_location = physical.get("artifactLocation") or {}
                artifact = artifact_location.get("uri", "")
                region = physical.get("region") or {}
                line = region.get("startLine")
                location = f"{artifact}:{line}" if artifact and line else str(artifact)
            rule_id = str(result.get("ruleId", ""))
            message_entry = result.get("message") or {}
            message = str(message_entry.get("text", "")) if isinstance(message_entry, dict) else str(message_entry)
            findings.append(
                {
                    "level": level,
                    "rule_id": rule_id,
                    "title": rules.get(rule_id, "") or rule_id or "Finding",
                    "message": message,
                    "location": location,
                }
            )
    return findings, counts, None


def write_trivy_report(args: argparse.Namespace) -> None:
    sarif_path = Path(args.sarif)
    findings, counts, parse_error = sarif_findings(sarif_path)
    outcome = args.scan_outcome.lower()
    if outcome == "success" and parse_error is None:
        status = "findings" if findings else "clean"
    elif outcome in {"skipped", "cancelled"}:
        status = "skipped"
    else:
        status = "error"
    error = parse_error
    if outcome == "success" and parse_error:
        status = "error"

    report = {
        "scanner": "Trivy",
        "target": args.target,
        "version": args.version,
        "status": status,
        "scan_step_outcome": args.scan_outcome,
        "sarif_upload_status": args.sarif_upload_outcome,
        "sarif_file": sarif_path.name if sarif_path.is_file() else None,
        "finding_count": len(findings),
        "counts": counts,
        "findings": findings[:100],
        "error": error,
    }
    write_json(Path(args.output), report)


def write_clamav_report(args: argparse.Namespace) -> None:
    log_path = Path(args.log)
    log = read_text(log_path)
    infected_match = re.findall(r"Infected files:\s*(\d+)", log, re.IGNORECASE)
    scanned_match = re.findall(r"Scanned files:\s*(\d+)", log, re.IGNORECASE)
    detected = [line.strip() for line in log.splitlines() if re.search(r"\sFOUND\s*$", line)]
    infected_count = int(infected_match[-1]) if infected_match else len(detected)
    scan_outcome = args.scan_outcome.lower()
    if scan_outcome == "success" and (infected_match or infected_count == 0):
        status = "findings" if infected_count else "clean"
    elif scan_outcome in {"skipped", "cancelled"}:
        status = "skipped"
    elif infected_count:
        status = "findings"
    else:
        status = "error"

    error = None
    if scan_outcome == "success" and not infected_match:
        error = "ClamAV hat den Scan ohne Abschlussübersicht im erfassten Protokoll beendet."
        status = "error"
    elif scan_outcome == "failure" and not infected_count:
        error = "Der ClamAV-Scan ist fehlgeschlagen. Siehe das angehängte Rohprotokoll."
    if args.signature_update_outcome.lower() not in {"success", "skipped"}:
        error = error or "Das ClamAV-Signaturupdate wurde nicht erfolgreich abgeschlossen."
        if status == "skipped":
            status = "error"

    report = {
        "scanner": "ClamAV",
        "target": "Release ZIP files and extracted contents",
        "version": args.version or None,
        "status": status,
        "scan_step_outcome": args.scan_outcome,
        "signature_update_status": args.signature_update_outcome,
        "scanned_file_count": int(scanned_match[-1]) if scanned_match else None,
        "finding_count": infected_count,
        "findings": [{"level": "malware", "message": item} for item in detected[:100]],
        "error": error,
    }
    write_json(Path(args.output), report)


def load_report(path: Path, scanner: str, target: str, job_result: str) -> dict[str, Any]:
    if path.is_file():
        try:
            value = json.loads(read_text(path))
            if isinstance(value, dict):
                return value
        except json.JSONDecodeError as error:
            return {
                "scanner": scanner,
                "target": target,
                "status": "error",
                "finding_count": None,
                "error": f"Report JSON is invalid: {error}",
            }
    missing_status = "skipped" if job_result.lower() == "skipped" else "error"
    reason = "Der Scanner-Job wurde vor der Berichtserstellung übersprungen." if missing_status == "skipped" else "Der Scanner-Bericht fehlt."
    return {
        "scanner": scanner,
        "target": target,
        "status": missing_status,
        "finding_count": None,
        "error": reason,
    }


def markdown_cell(value: Any) -> str:
    text = str(value if value is not None else "—")
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("|", "\\|")
        .replace("\r", " ")
        .replace("\n", " ")
        .replace("`", "\\`")
    )


def status_label(report: dict[str, Any], is_trivy: bool = False) -> str:
    status = report.get("status", "error")
    if status == "clean":
        return "Keine Funde"
    if status == "findings":
        return "Funde, Veröffentlichung nicht blockiert" if is_trivy else "Funde, Veröffentlichung blockiert"
    if status == "skipped":
        return "Übersprungen"
    if status == "error":
        return "Fehler"
    return str(status)


def job_result_label(value: Any) -> str:
    return {
        "success": "Erfolgreich",
        "failure": "Fehlgeschlagen",
        "cancelled": "Abgebrochen",
        "skipped": "Übersprungen",
    }.get(str(value).lower(), "Unbekannt")


def sarif_status_label(value: Any) -> str:
    return {
        "success": "Hochgeladen",
        "failure": "Fehlgeschlagen",
        "skipped": "Übersprungen",
        "cancelled": "Abgebrochen",
    }.get(str(value).lower(), "Unbekannt")


def signature_status_label(value: Any) -> str:
    return {
        "success": "aktualisiert",
        "warning": "Warnung, vorhandene Signaturen verwendet",
        "failure": "fehlgeschlagen",
        "skipped": "übersprungen",
        "not_attempted": "nicht gestartet",
    }.get(str(value).lower(), str(value))


def finding_count(report: dict[str, Any]) -> Any:
    value = report.get("finding_count")
    if value is not None:
        return value
    return len(report.get("detections", []))


def report_notes(report: dict[str, Any]) -> str:
    notes: list[str] = []
    if report.get("version"):
        notes.append(f"Version: {report['version']}")
    if report.get("signature_update_status"):
        notes.append(f"Signaturupdate: {signature_status_label(report['signature_update_status'])}")
    if report.get("signature_update_message"):
        notes.append(str(report["signature_update_message"]))
    metadata = report.get("metadata", {})
    if isinstance(metadata, dict):
        if metadata.get("signature_version"):
            notes.append(f"Defender-Signatur: {metadata['signature_version']}")
        if metadata.get("signature_last_updated"):
            notes.append(f"aktualisiert: {metadata['signature_last_updated']}")
        if metadata.get("engine_version"):
            notes.append(f"Engine: {metadata['engine_version']}")
        if metadata.get("cloud_protection_level") is not None:
            notes.append(f"Cloud-Schutzwert: {metadata['cloud_protection_level']}")
        if metadata.get("real_time_protection_enabled") is not None:
            notes.append(f"Echtzeitschutz: {metadata['real_time_protection_enabled']}")
    for warning in report.get("metadata_warnings", []) or []:
        notes.append(str(warning))
    if report.get("scanned_file_count") is not None:
        notes.append(f"Dateien: {report['scanned_file_count']}")
    if report.get("sarif_upload_status"):
        notes.append(f"SARIF: {sarif_status_label(report['sarif_upload_status'])}")
    if report.get("error"):
        notes.append(str(report["error"]))
    return ", ".join(notes) or "—"


def count_cell(report: dict[str, Any], key: str) -> Any:
    counts = report.get("counts", {})
    if isinstance(counts, dict) and key in counts:
        return counts[key]
    return "—"


def collect_details(reports: list[dict[str, Any]]) -> list[dict[str, str]]:
    details: list[dict[str, str]] = []
    for report in reports:
        scanner = str(report.get("scanner", "Scanner"))
        if scanner.lower() == "trivy":
            scanner = f"Trivy ({report.get('target', 'Scan')})"
        entries = report.get("findings", [])
        if scanner.lower().startswith("microsoft defender"):
            entries = report.get("detections", [])
        for entry in entries or []:
            if scanner.lower().startswith("microsoft defender"):
                title = str(entry.get("threat_name") or "Defender detection")
                location = ", ".join(str(item) for item in entry.get("resources", []))
                level = "Malware"
                message = title
            else:
                title = str(entry.get("title") or entry.get("rule_id") or entry.get("message") or "Finding")
                location = str(entry.get("location", ""))
                level = str(entry.get("level", "Finding"))
                message = str(entry.get("message", ""))
            details.append({"scanner": scanner, "level": level, "finding": title, "message": message, "location": location})
    return details[:50]


def write_aggregate(args: argparse.Namespace) -> None:
    report_root = Path(args.reports_dir)
    build_result = os.environ.get("BUILD_JOB_RESULT", "unknown")
    source_job = os.environ.get("TRIVY_SOURCE_JOB_RESULT", "unknown")
    defender_job = os.environ.get("DEFENDER_JOB_RESULT", "unknown")
    release_job = os.environ.get("RELEASE_SCAN_JOB_RESULT", "unknown")
    run_id = os.environ.get("GITHUB_RUN_ID", "")
    repository = os.environ.get("GITHUB_REPOSITORY", "")

    source = load_report(report_root / "trivy-filesystem" / "trivy-filesystem.json", "Trivy", "Quellcode und Abhängigkeiten", source_job)
    defender = load_report(report_root / "defender" / "defender-release-report.json", "Microsoft Defender Antivirus", "Release-Paket", defender_job)
    release_trivy = load_report(report_root / "release" / "trivy-release.json", "Trivy", "Release-Paket", release_job)
    clamav = load_report(report_root / "release" / "clamav-release.json", "ClamAV", "ZIP und entpackte Release-Dateien", release_job)

    packages_built = build_result.lower() == "success"
    if not packages_built:
        overall = "not_applicable" if build_result.lower() == "skipped" else "blocked"
    else:
        required_success = (
            build_result.lower() == "success"
            and source_job.lower() == "success"
            and defender_job.lower() == "success"
            and release_job.lower() == "success"
            and defender.get("status") == "clean"
            and clamav.get("status") == "clean"
            and source.get("status") in {"clean", "findings"}
            and release_trivy.get("status") in {"clean", "findings"}
        )
        overall = "ready" if required_success else "blocked"

    reports = [source, defender, release_trivy, clamav]
    job_results = {
        "build_and_package": build_result,
        "trivy_filesystem": source_job,
        "defender": defender_job,
        "release_scanners": release_job,
    }
    artifact_url = f"https://github.com/{repository}/actions/runs/{run_id}/artifacts" if repository and run_id else None
    aggregate = {
        "run_id": run_id or None,
        "repository": repository or None,
        "overall_status": overall,
        "job_results": job_results,
        "scanners": reports,
        "report_artifact_url": artifact_url,
    }

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    write_json(output_dir / "release-security-report.json", aggregate)

    overall_label = {
        "ready": "Prüfungen abgeschlossen. Veröffentlichung ist freigegeben.",
        "blocked": "Prüfung fehlgeschlagen oder blockierende Funde vorhanden. Veröffentlichung ist blockiert.",
        "not_applicable": "Für dieses Ereignis wurde kein Release-Paket gebaut.",
    }[overall]
    lines = [
        "# Einheitlicher Release-Sicherheitsbericht",
        "",
        f"**Run:** {run_id or 'unbekannt'}",
        f"**Gesamtergebnis:** {overall_label}",
        "",
        "## Prüfungen",
        "",
        "| Dienst | Umfang | Ergebnis | Total | Errors | Warnings | Notes | SARIF | Version und Signaturen |",
        "|---|---|---|---:|---:|---:|---:|---|---|",
        f"| Build und Paketierung | Release-ZIP | {markdown_cell(job_result_label(build_result))} | — | — | — | — | — | — |",
    ]
    for report, is_trivy in ((source, True), (defender, False), (release_trivy, True), (clamav, False)):
        notes = report_notes(report)
        lines.append(
            f"| {markdown_cell(report.get('scanner'))} | {markdown_cell(report.get('target'))} | "
            f"{markdown_cell(status_label(report, is_trivy))} | {markdown_cell(finding_count(report))} | "
            f"{markdown_cell(count_cell(report, 'error'))} | {markdown_cell(count_cell(report, 'warning'))} | "
            f"{markdown_cell(count_cell(report, 'note'))} | {markdown_cell(report.get('sarif_upload_status', '—'))} | "
            f"{markdown_cell(notes)} |"
        )
    lines.extend(
        [
            "",
            "Trivy-Funde werden gezählt. Die SARIF-Datei bleibt als Workflow-Artefakt erhalten und wird bei erfolgreichem Upload in GitHub Code Scanning veröffentlicht. Trivy-Funde blockieren gemäß Workflow-Richtlinie nicht. Defender- und ClamAV-Funde blockieren die Veröffentlichung.",
            "",
            "## Job-Ergebnisse",
            "",
            "| Job | Ergebnis |",
            "|---|---|",
            f"| Build und Paketierung | {markdown_cell(job_result_label(build_result))} |",
            f"| Trivy-Dateisystemscan | {markdown_cell(job_result_label(source_job))} |",
            f"| Microsoft Defender | {markdown_cell(job_result_label(defender_job))} |",
            f"| Trivy-Paketscan und ClamAV | {markdown_cell(job_result_label(release_job))} |",
        ]
    )

    details = collect_details(reports)
    lines.extend(["", "## Gemeldete Funde", ""])
    if details:
        lines.extend(
            [
                "| Dienst | Einstufung | Fund | Datei oder Fundstelle |",
                "|---|---|---|---|",
            ]
        )
        for item in details:
            location = item["location"] or item["message"]
            lines.append(
                f"| {markdown_cell(item['scanner'])} | {markdown_cell(item['level'])} | "
                f"{markdown_cell(item['finding'])} | {markdown_cell(location)} |"
            )
    else:
        lines.append("Die verfügbaren Berichte führen keine einzelnen Funde auf. Übersprungene und fehlgeschlagene Prüfungen sind oben markiert.")

    lines.extend(["", "## Dateien", "", "Der zusammengefasste Bericht sowie SARIF- und Rohberichte liegen im Workflow-Artefakt."])
    if artifact_url:
        lines.append(f"[Berichte und SARIF-Artefakte öffnen]({artifact_url})")
    markdown = "\n".join(lines) + "\n"
    (output_dir / "release-security-report.md").write_text(markdown, encoding="utf-8")
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as summary:
            summary.write(markdown)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    trivy = subparsers.add_parser("trivy")
    trivy.add_argument("--target", required=True)
    trivy.add_argument("--version", default="aquasecurity/trivy-action@v0.36.0")
    trivy.add_argument("--sarif", required=True)
    trivy.add_argument("--scan-outcome", required=True)
    trivy.add_argument("--sarif-upload-outcome", required=True)
    trivy.add_argument("--output", required=True)
    trivy.set_defaults(function=write_trivy_report)

    clamav = subparsers.add_parser("clamav")
    clamav.add_argument("--scan-outcome", required=True)
    clamav.add_argument("--signature-update-outcome", required=True)
    clamav.add_argument("--version", default="")
    clamav.add_argument("--log", required=True)
    clamav.add_argument("--output", required=True)
    clamav.set_defaults(function=write_clamav_report)

    aggregate = subparsers.add_parser("aggregate")
    aggregate.add_argument("--reports-dir", required=True)
    aggregate.add_argument("--output-dir", required=True)
    aggregate.set_defaults(function=write_aggregate)

    args = parser.parse_args()
    args.function(args)


if __name__ == "__main__":
    main()
