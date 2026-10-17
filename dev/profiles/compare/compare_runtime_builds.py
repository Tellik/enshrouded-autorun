"""Compare captured reflected layouts and rank procedure evidence across builds."""
import argparse
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from dev.profiles.analyze.write_inspection_result import write_result
from dev.profiles.analyze.read_capture_reports import checked_reports
from dev.profiles.compare.rank_cross_build_functions import compare_functions
from dev.profiles.inspect.read_pe_image import PE

def layout(ty, types):
    def name(index):
        return types[index]['qualifiedName'] if index is not None else None
    return dict(size=ty['size'], alignment=ty['alignment'], primitive=ty['primitiveType'],
                flags=ty['flagsBits'], innerType=name(ty.get('innerType')),
                fields={key: dict(type=name(f['type']), offset=f['dataOffset']) for key, f in ty['structFields'].items()},
                enums=ty['enumFields'])

def compare(source, target, source_profile=None, target_scan=None, target_executable=None,
            source_components=None, target_components=None):
    left, lf, _, audit = checked_reports(source)
    right, rf, _, right_audit = checked_reports(target)
    profile = None
    if source_profile:
        profile = json.loads(source_profile.read_text(encoding='utf-8-sig'))
        identity = left['image']
        image = profile['image']
        if (profile['target'].lower() != identity['target'].lower() or
                image['timestamp'] != identity['timestamp'] or image['size'] != identity['size'] or
                (image.get('sha256') and image['sha256'] != identity['sha256'])):
            raise ValueError('source profile belongs to another captured executable')
    scan = None
    if target_scan:
        scan = json.loads(target_scan.read_text(encoding='utf-8-sig'))
        identity = right['image']
        if (scan.get('timestamp') != identity['timestamp'] or scan.get('imageSize') != identity['size'] or
                scan.get('sha256') != identity['sha256']):
            raise ValueError('target function scan belongs to another captured executable')
    pe = None
    if target_executable:
        pe = PE(target_executable)
        if pe.identity() != right['image']:
            raise ValueError('target executable belongs to another captured image')

    def components(directory, explicit, identity):
        path = explicit if explicit is not None else directory / 'components.json'
        if explicit is None and not path.is_file():
            return None
        report = json.loads(path.read_text(encoding='utf-8-sig'))
        if report['image'] != identity:
            raise ValueError('component callback capture belongs to another executable image')
        return report

    left_components = components(source, source_components, left['image'])
    right_components = components(target, target_components, right['image'])
    a = {t['qualifiedName']: t for t in left['types']}
    b = {t['qualifiedName']: t for t in right['types']}
    common = sorted(a.keys() & b.keys())
    changed = [dict(name=name, source=layout(a[name], left['types']), target=layout(b[name], right['types']))
               for name in common if layout(a[name], left['types']) != layout(b[name], right['types'])]
    result = dict(source=left['image'], target=right['image'],
                  addedTypes=sorted(b.keys() - a.keys()), removedTypes=sorted(a.keys() - b.keys()),
                  changedLayouts=changed, unchangedLayouts=len(common) - len(changed), operationCandidates=[])
    result.update(compare_functions(left, lf, audit, right, rf, right_audit, profile, scan,
                                    left_components, right_components))
    if pe:
        from dev.profiles.inspect.discover_executable_functions import generate_signature
        signatures = {}
        for operation in result['operationCandidates']:
            for candidate in operation.get('rankedCandidates', [])[:5]:
                rva = candidate['targetRva']
                if rva not in signatures:
                    signatures[rva] = generate_signature(pe, rva)
                candidate['entryPointSignature'] = dict(signatures[rva],
                    purpose='current-procedure-entry-only', hookOrPatchSite=False)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('target', type=Path)
    parser.add_argument('--source-profile', type=Path, help='exact source build profile with current operation anchors')
    parser.add_argument('--target-scan', type=Path, help='native function-scan JSON for the exact target image')
    parser.add_argument('--target-executable', type=Path, help='exact target EXE to generate candidate entry-point signatures')
    parser.add_argument('--source-components', type=Path, help='exact source image live component callback capture')
    parser.add_argument('--target-components', type=Path, help='exact target image live component callback capture')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    write_result(compare(args.source, args.target, args.source_profile, args.target_scan,
                         args.target_executable, args.source_components, args.target_components), args.out)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
