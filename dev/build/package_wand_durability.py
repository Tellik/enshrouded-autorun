"""Build or verify the standalone Wand Durability ZIP; exclude game data."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import zipfile

ROOT = Path(__file__).resolve().parents[2]
MOD = ROOT / 'mods' / 'wand-durability'
FILES = ('mod.json', 'extended.mod.json', 'native-plugin.ini', 'README.md',
         'LICENSE', 'LICENSE.nlohmann')


def verify(path):
    expected = {'wand-durability/' + p for p in (*FILES, 'bin/wand-durability.dll')}
    with zipfile.ZipFile(path) as archive:
        assert len(archive.namelist()) == len(expected) and set(archive.namelist()) == expected
        assert archive.testzip() is None
        manifest = json.loads(archive.read('wand-durability/mod.json'))
        assert manifest['id'] == 'wand-durability' and manifest['version'] == '0.1.0'
        settings = json.loads(archive.read('wand-durability/extended.mod.json'))
        assert settings['enabled'] is False and settings['targets'] == ['client']
        assert archive.read('wand-durability/native-plugin.ini').decode().splitlines() == [
            '[Plugin]', 'Enabled=1', 'Dll=bin/wand-durability.dll']
        dll = archive.read('wand-durability/bin/wand-durability.dll')
        assert dll[:2] == b'MZ'
        nt = struct.unpack_from('<I', dll, 0x3c)[0]
        assert dll[nt:nt+4] == b'PE\0\0'
        assert struct.unpack_from('<H', dll, nt+4)[0] == 0x8664
        assert struct.unpack_from('<H', dll, nt+22)[0] & 0x2000
    print(f'Verified {path.name}: standalone Windows x64 plugin, disabled by default')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dll', type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--verify', type=Path)
    args = parser.parse_args()
    if args.verify:
        verify(args.verify)
        return
    if not args.dll or not args.out:
        parser.error('--dll and --out are required when building')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(args.out, 'w', zipfile.ZIP_DEFLATED) as archive:
        for name in FILES:
            archive.write(MOD / name, 'wand-durability/' + name)
        archive.write(args.dll, 'wand-durability/bin/wand-durability.dll')
    verify(args.out)
    digest = hashlib.sha256(args.out.read_bytes()).hexdigest()
    args.out.with_suffix(args.out.suffix + '.sha256').write_text(f'{digest}  {args.out.name}\n')


if __name__ == '__main__':
    main()
