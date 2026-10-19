"""Trace exact-image byte signatures to original procedures and their callers."""
import argparse
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from game_inspection.analyze.write_inspection_result import write_result
from collections import defaultdict
from game_inspection.analyze.index_captured_functions import FunctionIndex
from game_inspection.inspect.read_pe_image import PE
from game_inspection.inspect.extract_executable_reflection import extract
from game_inspection.inspect.read_system_descriptors import discover as discover_systems

def trace_anchors(pe, reflection, functions, scan, catalog=None, maximum_depth=6):
    """Trace every fresh byte match to original procedures and named callers.

    The caller graph retains instruction RVAs and all equally short incoming
    edges. It establishes static code relationships, never an ABI or effect.
    """
    import re
    import capstone as cs
    from collections import deque
    from game_inspection.inspect.discover_executable_functions import profile_audit

    identity = pe.identity()
    if not 0 <= maximum_depth <= 6:
        raise ValueError('caller depth must be between zero and six')
    if scan.get('image') != identity or reflection.get('image') != identity or functions.get('image') != identity:
        raise ValueError('anchor scan and captures must belong to the exact executable image')
    index = FunctionIndex(reflection, functions)
    features = scan.get('features')
    if not isinstance(features, list):
        raise ValueError('anchor scan requires a features array')
    if any(not isinstance(row, dict) for row in features):
        raise ValueError('anchor features must be objects')
    rows = [dict(row) for row in features]
    catalog_entries = {row.get('name'): row for row in (catalog or {}).get('entries', [])}
    present = {row.get('name') for row in rows}
    for row in (catalog or {}).get('entries', []):
        if row.get('name') not in present:
            rows.append(dict(row))
            present.add(row.get('name'))
    valid, invalid = {}, {}
    for ordinal, row in enumerate(rows):
        tokens = str(row.get('signature') or '').split()
        if not tokens:
            continue
        if (len(tokens) > 4096 or any(not re.fullmatch(r'[0-9a-fA-F]{2}|\?\??', t) for t in tokens)
                or sum(t not in ('?', '??') for t in tokens) < 4):
            invalid[ordinal] = 'invalid-or-insufficiently-specific-byte-signature'
        else:
            valid[str(ordinal)] = dict(signature=' '.join(tokens))
    audited = profile_audit(pe, dict(id='trace-anchors', target=identity['target'], image=identity, hooks=valid))
    matches = {int(row['id'][5:]): row['matches'] for row in audited['operations']}
    edges = defaultdict(list)
    for caller, group in index.groups.items():
        for fragment in group['ranges']:
            for branch in index.ranges[fragment].get('directBranches', []):
                callee = index.containing(branch['targetRva'])
                if branch['kind'] == 'call' and callee is not None and caller != callee:
                    edges[callee].append(dict(callerFunctionRva=caller, calleeFunctionRva=callee,
                                             instructionRva=branch['instructionRva'], targetRva=branch['targetRva']))
    decoder = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
    procedures, caller_functions, referenced_types, anchors = {}, {}, set(), []
    type_names = defaultdict(list)
    for ty in reflection['types']:
        for name in {ty.get('name'), ty['qualifiedName']} - {None}:
            type_names[name].append(ty['index'])

    def descriptor_types(group):
        dependencies = [dict(descriptor=d['name'], slotOffset=s['offset'], name=name,
                             reflectionCandidates=type_names[name], associationUnique=len(type_names[name]) == 1)
                        for d in group.get('engineDescriptors', [])
                        for s in d.get('dependencySlots', []) for name in s.get('entries', [])]
        referenced_types.update(t for row in dependencies for t in row['reflectionCandidates'])
        return dependencies

    def procedure(root):
        key = str(root)
        if key not in procedures:
            group = index.groups[root]
            ranges = []
            for rva in group['ranges']:
                native = index.ranges[rva]
                code = pe.read(rva, native['endRva'] - rva)
                instructions = [dict(rva=i.address, bytes=i.bytes.hex(), asm=i.mnemonic + ' ' + i.op_str)
                                for i in decoder.disasm(code, rva)]
                ranges.append(dict(native, originalBytes=code.hex(), instructions=instructions))
                referenced_types.update(r['typeIndex'] for r in native.get('metadataReferences', []))
                referenced_types.update(c['typeIndex'] for h in native.get('hashCandidates', []) for c in h['candidates'])
            procedures[key] = dict(group, ranges=ranges, callable=False, nativeAbiProven=False,
                                   gameplayEffectsProven=False, descriptorTypeReferences=descriptor_types(group))
        return procedures[key]

    def callers(root):
        queue, distance, paths, found, graph = deque([root]), {root: 0}, {root: []}, [], {}
        while queue:
            current = queue.popleft()
            names = sorted(index.features[current]['descriptor'])
            if names:
                found.append(dict(functionRva=current, names=names, depth=distance[current],
                                  oneShortestCallPath=paths[current]))
                continue
            if distance[current] == maximum_depth:
                continue
            for edge in sorted(edges[current], key=lambda e: (e['callerFunctionRva'], e['instructionRva'])):
                caller, depth = edge['callerFunctionRva'], distance[current] + 1
                if caller == root or distance.get(caller, depth) < depth:
                    continue
                graph[edge['instructionRva']] = edge
                caller_functions[str(caller)] = dict(index.groups[caller], callable=False, nativeAbiProven=False,
                    descriptorTypeReferences=descriptor_types(index.groups[caller]))
                if caller not in distance:
                    distance[caller] = depth
                    paths[caller] = [edge, *paths[current]]
                    queue.append(caller)
        return dict(nearestNamedCallers=found, directCallEdges=list(graph.values()), maximumDepth=maximum_depth,
                    pathPolicy='one shortest path per named procedure; graph retains alternative shortest edges',
                    includesIndirectCalls=False, nativeArgumentsProven=False)

    for ordinal, row in enumerate(rows):
        sites = matches.get(ordinal, [])
        result = dict(name=row.get('name'), signature=row.get('signature'), reportedMatches=row.get('matches'),
                      matches=sites, matchCount=len(sites), sites=[], callable=False, nativeAbiProven=False)
        metadata = catalog_entries.get(row.get('name'), {})
        result['catalogMetadata'] = metadata or None
        if ordinal in invalid:
            result.update(status='unresolved-invalid-signature', reason=invalid[ordinal])
        elif not row.get('signature'):
            result.update(status='unresolved-data-mechanism', reason='no-byte-signature-supplied')
        else:
            result['status'] = 'unresolved-no-match' if not sites else 'ambiguous-byte-matches' if len(sites) > 1 else 'unique-byte-match'
        for site in sites:
            root = index.containing(site)
            detail = dict(siteRva=site, primaryFunctionRva=root, callable=False)
            if root is None:
                detail.update(instructionBoundaryVerified=False, reason='no-compiler-described-procedure')
            else:
                group = procedure(root)
                instructions = [i for part in group['ranges'] for i in part['instructions']]
                position = next((n for n, i in enumerate(instructions) if i['rva'] == site), None)
                detail.update(instructionBoundaryVerified=position is not None,
                              instructionContext=instructions[max(0, position - 6):position + 10] if position is not None else [],
                              callerTrace=callers(root))
            offset = metadata.get('siteOffset', metadata.get('siteOffset130') if metadata.get('kind20') == 0
                                  else metadata.get('siteOffset178'))
            if isinstance(offset, int) and not isinstance(offset, bool):
                target, target_root = site + offset, index.containing(site + offset)
                instructions = [i for part in procedure(target_root)['ranges'] for i in part['instructions']] if target_root is not None else []
                position = next((n for n, i in enumerate(instructions) if i['rva'] == target), None)
                detail['catalogSite'] = dict(offset=offset, rva=target, primaryFunctionRva=target_root,
                    instructionBoundaryVerified=position is not None,
                    instructionContext=instructions[max(0, position - 6):position + 10] if position is not None else [],
                    evidence='catalog-declared offset, independently checked instruction boundary', callable=False)
            result['sites'].append(detail)
        result['reportedMatchesAgree'] = sorted(row['matches']) == sites if 'matches' in row else None
        anchors.append(result)
    return dict(schemaVersion=1, kind='exact-image-original-anchor-trace', image=identity, anchors=anchors,
                coverage=dict(anchors=len(anchors), signatureAnchors=len(valid), matchedAnchors=sum(bool(a['matches']) for a in anchors),
                              ambiguousAnchors=sum(len(a['matches']) > 1 for a in anchors), procedureGroups=len(procedures)),
                procedures=procedures, callerFunctions=caller_functions,
                reflectedTypes=[t for t in reflection['types'] if t['index'] in referenced_types],
                callable=False, nativeAbiProven=False, gameplayEffectsProven=False,
                limitations=['Byte matches and type-hash immediates do not establish function arguments or effects',
                             'All byte matches are retained, including non-instruction-boundary matches',
                             'PE unwind groups omit some leaf functions, inlining and generated code',
                             'Caller traces use captured direct CALL instructions only'])

def trace(scan_path, executable, catalog_path=None, capture_out=None, maximum_depth=6):
    from game_inspection.inspect.discover_executable_functions import discover

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
    parser.add_argument('scan', type=Path, help='exact-image JSON with features name/signature/matches')
    parser.add_argument('--executable', type=Path, required=True)
    parser.add_argument('--catalog', type=Path, help='optional registration entries, retaining signatureless data mechanisms')
    parser.add_argument('--capture-out', type=Path, help='save fresh reflection.json and functions.json for reuse')
    parser.add_argument('--max-caller-depth', type=int, choices=range(7), default=6)
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    write_result(trace(args.scan, args.executable, args.catalog, args.capture_out, args.max_caller_depth), args.out)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
