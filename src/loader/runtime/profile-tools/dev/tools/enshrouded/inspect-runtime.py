#!/usr/bin/env python3
"""Query captured registries by hash/name and compare executable builds."""
import argparse
import json
from pathlib import Path
import sys

from discovery.matching import compare_functions, trace_anchors


def read(directory, name, optional=False):
    path = directory / name
    if optional and not path.is_file():
        return None
    return json.loads(path.read_text(encoding='utf-8'))


def checked_reports(directory):
    reflection = read(directory, 'reflection.json')
    functions = read(directory, 'functions.json', True)
    live = read(directory, 'live.json', True)
    audit = read(directory, 'profile-audit.json', True)
    for report in (functions, live, audit):
        if report and report['image'] != reflection['image']:
            raise ValueError('capture directory mixes executable identities')
    return reflection, functions, live, audit


def lookup(directory, query, hash_kind):
    reflection, functions, live, audit = checked_reports(directory)
    try:
        value = int(query, 0)
        if not 0 <= value <= 0xffffffff:
            raise ValueError('hash must be an unsigned 32-bit integer')
        matches = [t for t in reflection['types'] if t[hash_kind] == value]
    except ValueError:
        matches = [t for t in reflection['types'] if t['qualifiedName'] == query]
    result = dict(image=reflection['image'], query=query, matchCount=len(matches), matches=[])
    for ty in matches:
        index = ty['index']
        code = []
        for fn in (functions or {}).get('functions', []):
            refs = [r for r in fn['metadataReferences'] if r['typeIndex'] == index]
            hashes = [r for r in fn['hashCandidates'] if any(c['typeIndex'] == index for c in r['candidates'])]
            if refs or hashes:
                code.append(dict(id=fn['id'], beginRva=fn['beginRva'], endRva=fn['endRva'],
                                 metadataReferences=refs, hashCandidates=hashes, callable=False))
        tables = [dict(address=t['address'], status=t['status'], index=row['index'],
                       stableReadback=t['stableReadback'])
                  for t in (live or {}).get('tables', []) for row in t['rows'] if row['typeIndex'] == index]
        result['matches'].append(dict(type=ty, liveTables=tables, codeReferences=code))
    return result


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
        from discovery.pe import PE
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
        from discovery.functions import generate_signature
        signatures = {}
        for operation in result['operationCandidates']:
            for candidate in operation.get('rankedCandidates', [])[:5]:
                rva = candidate['targetRva']
                if rva not in signatures:
                    signatures[rva] = generate_signature(pe, rva)
                candidate['entryPointSignature'] = dict(signatures[rva],
                    purpose='current-procedure-entry-only', hookOrPatchSite=False)
    return result


def trace(scan_path, executable, catalog_path=None, capture_out=None, maximum_depth=6):
    from discovery.pe import PE
    from discovery.reflection import extract
    from discovery.functions import discover
    from discovery.systems import discover as discover_systems

    scan = json.loads(scan_path.read_text(encoding='utf-8-sig'))
    catalog = json.loads(catalog_path.read_text(encoding='utf-8-sig')) if catalog_path else None
    pe = PE(executable)
    identity = pe.identity()
    if scan.get('image') != identity:
        raise ValueError('anchor scan belongs to another executable image')
    if capture_out:
        previous = capture_out / 'reflection.json'
        if previous.is_file() and json.loads(previous.read_text(encoding='utf-8'))['image'] != identity:
            raise ValueError('capture output directory belongs to another executable image')
    reflection = dict(image=identity, **extract(pe))
    functions = dict(image=identity, **discover(pe, reflection, progress=lambda message: print(message, file=sys.stderr, flush=True)))
    groups = {group['beginRva']: group for group in functions['functionGroups']}
    descriptors = discover_systems(pe, functions)
    for descriptor in descriptors:
        groups[descriptor['functionRva']].setdefault('engineDescriptors', []).append(descriptor)
    functions['engineDescriptorCount'] = len(descriptors)
    result = trace_anchors(pe, reflection, functions, scan, catalog, maximum_depth)
    if capture_out:
        capture_out.mkdir(parents=True, exist_ok=True)
        for name, value in (('reflection', reflection), ('functions', functions)):
            (capture_out / (name + '.json')).write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    result['sources'] = dict(scan=str(scan_path), executable=str(pe.path), catalog=str(catalog_path) if catalog_path else None,
                             capture=str(capture_out) if capture_out else None,
                             discovery='fresh reflection and compiler-range disassembly from exact executable')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    query = sub.add_parser('lookup')
    query.add_argument('capture', type=Path)
    query.add_argument('query')
    query.add_argument('--hash-kind', choices=['qualifiedHash', 'internalHash', 'nameHash', 'impactHash'], default='qualifiedHash')
    diff = sub.add_parser('compare')
    diff.add_argument('source', type=Path)
    diff.add_argument('target', type=Path)
    diff.add_argument('--source-profile', type=Path, help='exact source build profile with current operation anchors')
    diff.add_argument('--target-scan', type=Path, help='native function-scan JSON for the exact target image')
    diff.add_argument('--target-executable', type=Path, help='exact target EXE to generate candidate entry-point signatures')
    diff.add_argument('--source-components', type=Path, help='exact source image live component callback capture')
    diff.add_argument('--target-components', type=Path, help='exact target image live component callback capture')
    anchors = sub.add_parser('trace-anchors', help='fresh exact-image trace from known byte signatures to original procedures and callers')
    anchors.add_argument('scan', type=Path, help='exact-image JSON with features name/signature/matches')
    anchors.add_argument('--executable', type=Path, required=True)
    anchors.add_argument('--catalog', type=Path, help='optional registration entries, retaining signatureless data mechanisms')
    anchors.add_argument('--capture-out', type=Path, help='save fresh reflection.json and functions.json for reuse')
    anchors.add_argument('--max-caller-depth', type=int, choices=range(7), default=6)
    for command in (query, diff, anchors):
        command.add_argument('--out', type=Path)
    args = parser.parse_args()
    if args.command == 'lookup':
        result = lookup(args.capture, args.query, args.hash_kind)
    elif args.command == 'trace-anchors':
        result = trace(args.scan, args.executable, args.catalog, args.capture_out, args.max_caller_depth)
    else:
        result = compare(args.source, args.target, args.source_profile, args.target_scan, args.target_executable,
                         args.source_components, args.target_components)
    text = json.dumps(result, indent=2, ensure_ascii=False) + '\n'
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text, encoding='utf-8')
        print(args.out)
    else:
        print(text)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
