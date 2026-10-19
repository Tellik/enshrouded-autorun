"""Export an unambiguous registry already observed by the production reader.

Captured hashes and sizes are retained. Entity access is never authorized.
"""
import argparse
import json
from pathlib import Path


def export_observed_registry(source):
    capture = json.loads(source.read_text(encoding='utf-8'))
    discovery = capture['discovery']
    candidates = discovery.get('catalogCandidates', [])
    comparison = {
        'available': capture['available'],
        'discovery': {key:value for key,value in discovery.items() if key != 'catalogCandidates'},
        'candidateCounts': [len(candidate['entries']) for candidate in candidates],
        'identicalCompleteEntries': bool(candidates) and all(candidate['entries'] == candidates[0]['entries'] for candidate in candidates),
        'entityAccessAuthorized': False,
    }
    if not capture['available'] or not comparison['identicalCompleteEntries']:
        return None, comparison
    entries = []
    for row in candidates[0]['entries']:
        row = dict(row)
        for key in ('runtime_type', 'template_type'):
            if not row[key]['qualified_name']:
                row[key] = None
        entries.append(row)
    output = {'schema_version':1,'available':True,
              'source':'canonical-export-of-production-resident-discovery-evidence',
              'capture':str(source),'scope':discovery['scope'],'count':len(entries),
              'entries':entries,'entity_access_authorized':False}
    return output, comparison


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('capture', type=Path, help='registry-autodiscovery.json from the production reader')
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--comparison', type=Path, required=True)
    args = parser.parse_args()
    output, comparison = export_observed_registry(args.capture)
    args.comparison.parent.mkdir(parents=True, exist_ok=True)
    args.comparison.write_text(json.dumps(comparison, indent=2), encoding='utf-8')
    if output is None:
        raise SystemExit('No unambiguous observed catalog was published. Comparison evidence saved only.')
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(output, indent=2), encoding='utf-8')
    print(json.dumps({'registrations':len(output['entries']), 'observedRegistries':len(comparison['candidateCounts']),
                      'completeCatalogsIdentical':True, 'worldAuthority':False, 'output':str(args.out)}))


if __name__ == '__main__':
    main()
