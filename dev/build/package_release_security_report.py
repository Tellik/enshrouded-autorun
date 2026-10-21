#!/usr/bin/env python3
"""Bundle release security reports and write a SHA-256 sidecar."""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def package_report(reports_directory: Path, archive_path: Path, required_release: bool) -> None:
    reports_directory = reports_directory.resolve()
    archive_path = archive_path.resolve()
    markdown_report = reports_directory / "unified" / "release-security-report.md"
    if not reports_directory.is_dir():
        raise FileNotFoundError(f"Security reports directory does not exist: {reports_directory}")
    if not markdown_report.is_file():
        raise FileNotFoundError(f"Unified Markdown report does not exist: {markdown_report}")
    if archive_path == reports_directory or reports_directory in archive_path.parents:
        raise ValueError("The security report ZIP must be written outside the reports directory.")

    report_files = sorted(
        path
        for path in reports_directory.rglob("*")
        if path.is_file() and not path.is_symlink()
    )
    if not report_files:
        raise FileNotFoundError(f"No security report files were found in: {reports_directory}")
    if any(path.relative_to(reports_directory).as_posix() == "release-body.md" for path in report_files):
        raise ValueError("The reports directory already contains a release-body.md file.")

    report_body = markdown_report.read_text(encoding="utf-8")
    if required_release:
        report_body = f"ShroudForge-Update: required\n\n{report_body}"

    archive_path.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(archive_path, "w", compression=ZIP_DEFLATED, compresslevel=9) as archive:
        for path in report_files:
            archive.write(path, path.relative_to(reports_directory).as_posix())
        archive.writestr("release-body.md", report_body)

    checksum_path = Path(f"{archive_path}.sha256")
    checksum_path.write_text(f"{sha256_file(archive_path)}  {archive_path.name}\n", encoding="ascii")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reports-directory", type=Path, required=True)
    parser.add_argument("--archive-path", type=Path, required=True)
    parser.add_argument("--required-release", action="store_true")
    args = parser.parse_args()
    package_report(args.reports_directory, args.archive_path, args.required_release)


if __name__ == "__main__":
    main()
