#!/usr/bin/env python3
"""Create and verify the independent, directory-installed Autorun mod ZIP."""

import argparse
import hashlib
import json
from pathlib import Path
import struct
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = ROOT / "standalone" / "autorun"
FILES = (
    "mod.json",
    "extended.mod.json",
    "native-plugin.ini",
    "README.md",
)


def check_dll(data: bytes) -> None:
    if data[:2] != b"MZ" or len(data) < 0x100:
        raise ValueError("the native plugin is not a PE DLL")
    offset = struct.unpack_from("<I", data, 0x3C)[0]
    if data[offset : offset + 4] != b"PE\0\0":
        raise ValueError("the native plugin has no PE header")
    machine = struct.unpack_from("<H", data, offset + 4)[0]
    if machine != 0x8664:
        raise ValueError("the native plugin is not Windows x64")


def expected_members(dll: bytes) -> dict[str, bytes]:
    manifest = json.loads((PACKAGE / "mod.json").read_text())
    extension = json.loads((PACKAGE / "extended.mod.json").read_text())
    if manifest["id"] != "autorun" or extension["targets"] != ["client"]:
        raise ValueError("unexpected mod identity or target")
    if extension["enabled"]:
        raise ValueError("the standalone test package must default to disabled")
    if b"Dll=bin/autorun.dll" not in (PACKAGE / "native-plugin.ini").read_bytes():
        raise ValueError("native-plugin.ini does not reference the packaged DLL")
    check_dll(dll)
    members = {f"autorun/{name}": (PACKAGE / name).read_bytes() for name in FILES}
    members["autorun/bin/autorun.dll"] = dll
    members["autorun/LICENSE"] = (ROOT / "LICENSE").read_bytes()
    return members


def package(dll_path: Path, output: Path) -> None:
    members = expected_members(dll_path.read_bytes())
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, content in sorted(members.items()):
            entry = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o644 << 16
            archive.writestr(entry, content, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    verify(output, members)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    output.with_name(output.name + ".sha256").write_text(f"{digest}  {output.name}\n")
    print(f"{output} sha256={digest}")


def verify(path: Path, members: dict[str, bytes] | None = None) -> None:
    with zipfile.ZipFile(path) as archive:
        if archive.testzip() is not None:
            raise ValueError("ZIP contains a corrupt member")
        names = set(archive.namelist())
        expected = {f"autorun/{name}" for name in (*FILES, "bin/autorun.dll", "LICENSE")}
        if names != expected:
            raise ValueError(f"unexpected ZIP layout: {sorted(names ^ expected)}")
        check_dll(archive.read("autorun/bin/autorun.dll"))
        if members:
            for name, content in members.items():
                if archive.read(name) != content:
                    raise ValueError(f"packaged file differs from source: {name}")
        json.loads(archive.read("autorun/mod.json"))
        json.loads(archive.read("autorun/extended.mod.json"))
    print(f"Verified standalone mod ZIP: {path}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--dll", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--verify", type=Path)
    args = parser.parse_args()
    try:
        if args.verify:
            verify(args.verify)
        elif args.dll and args.out:
            package(args.dll, args.out)
        else:
            parser.error("use --verify ZIP or --dll DLL --out ZIP")
    except (OSError, ValueError, KeyError, zipfile.BadZipFile) as error:
        sys.exit(str(error))
