"""Find a captured runtime type by name or hash, retaining every collision."""
import argparse
import json
from pathlib import Path
import sys
from write_inspection_result import write_result
from read_capture_reports import checked_reports

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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('capture', type=Path)
    parser.add_argument('query')
    parser.add_argument('--hash-kind', choices=['qualifiedHash', 'internalHash', 'nameHash', 'impactHash'], default='qualifiedHash')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    write_result(lookup(args.capture, args.query, args.hash_kind), args.out)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
