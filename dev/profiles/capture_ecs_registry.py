#!/usr/bin/env python3
"""Follow sparse ECS arrays to full engine registrations and entity managers."""
import argparse
import json
from pathlib import Path
from pe_image import PE
from component_registry import discover, export_catalog


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--capture', type=Path, required=True)
    parser.add_argument('--export-only', action='store_true', help='export saved registrations without opening a process')
    parser.add_argument('--catalog-out', type=Path, help='write a profile-generator component catalog')
    parser.add_argument('--id', help='explicit profile id for the component catalog')
    args = parser.parse_args()
    if args.export_only and not args.catalog_out:
        parser.error('--export-only requires --catalog-out')
    if bool(args.catalog_out) != bool(args.id):
        parser.error('--catalog-out and --id must be supplied together')
    pe = PE(args.executable)
    reflection = json.loads((args.capture / 'reflection.json').read_text(encoding='utf-8'))
    if reflection['image'] != pe.identity():
        raise ValueError('capture identity mismatch')
    destination = args.capture / 'components.json'
    if args.export_only:
        report = json.loads(destination.read_text(encoding='utf-8'))
        if report['image'] != pe.identity():
            raise ValueError('component capture identity mismatch')
    else:
        from read_process_memory import Process
        live = json.loads((args.capture / 'live.json').read_text(encoding='utf-8'))
        if live['image'] != pe.identity():
            raise ValueError('live capture identity mismatch')
        process = Process(live['pid'], pe)
        try:
            if int(live['moduleBase'], 16) != process.base:
                raise ValueError('process restarted: capture again')
            report = discover(process, pe, reflection, live)
        finally:
            process.close()
        destination.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    if args.catalog_out:
        catalog = export_catalog(report, reflection, args.id)
        catalog['source'].update(liveCapture=str(destination.resolve()),
                                 reflection=str((args.capture / 'reflection.json').resolve()))
        args.catalog_out.parent.mkdir(parents=True, exist_ok=True)
        args.catalog_out.write_text(json.dumps(catalog, indent=2) + '\n', encoding='utf-8')
        print(f'component catalog: {len(catalog["components"])} runtime layouts, '
              f'{len(catalog["unresolved"])} unresolved -> {args.catalog_out}')
    print(json.dumps([dict(count=r['count'], runtime=r['runtimeTypes'],
                          template=r['templateTypes'], templateOnly=r['templateOnly'],
                          managers=r['managers']) for r in report['registries']], indent=2))


if __name__ == '__main__':
    main()
