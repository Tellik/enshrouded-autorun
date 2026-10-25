"""Validate the wand allowlist against original ShroudForge resource exports.

Inputs: export_kfc_resources output (ActorSequenceResource and ItemInfo), plus
the exact-image reflection capture. This is offline and never alters game files.
"""
import argparse
import collections
import json
from pathlib import Path
import re


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('resources', type=Path)
    parser.add_argument('reflection', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    capture = json.loads(args.reflection.read_text())
    assert capture['image']['sha256'] == 'd64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882'
    types = capture['types']
    resources = json.loads(args.resources.read_text())['resources']
    by_id = {r['id']: r for r in resources}
    sequences = collections.defaultdict(list)
    ids = collections.defaultdict(set)
    for resource in resources:
        if resource['qualifiedType'] != 'keen::ItemInfo': continue
        item = resource['value']
        if 'Weapon_' not in item['debugName'] or 'Wand' not in item['debugName']: continue
        for sequence in item['sequences']:
            if sequence['triggeredBy'] == 1 and sequence['requiredSkill'] is None:
                sequences[sequence['sequence']].append(item['debugName'])
                ids[sequence['sequence']].add(sequence['sequenceId']['value'])
    assert len(sequences) == 9
    source = (Path(__file__).resolve().parents[1] / 'mods/wand-durability/native/attack_cost.h').read_text()
    allowlist = set(map(int, re.findall(r'\b([0-9]{6,})u\b', source)))
    assert allowlist == {value for values in ids.values() for value in values}
    for resource in resources:
        if resource['qualifiedType'] == 'keen::ItemInfo':
            item = resource['value']
            if any(s['sequence'] in sequences for s in item['sequences']):
                assert 'Weapon_' in item['debugName'] and 'Wand' in item['debugName']
    rows = []
    for guid, names in sequences.items():
        assert len(ids[guid]) == 1
        row = dict(resource=guid, sequenceId=next(iter(ids[guid])), itemCount=len(names), subSequences=[])
        for sub in by_id[guid]['value']['subSequences']:
            events = []
            for event in sub['events']:
                if types[event['$type']]['qualifiedName'] != 'keen::actor::SpawnEntityEvent': continue
                value = event['$value']
                if value['isProjectile']:
                    events.append({k: value[k] for k in ['time', 'chance', 'requiredSkill', 'minSpawnCount', 'maxSpawnCount']})
            assert events and events[0]['chance'] == 1 and events[0]['requiredSkill'] is None
            row['subSequences'].append(dict(name=sub['name'], projectileEvents=events))
        rows.append(row)
    args.out.write_text(json.dumps(dict(image=capture['image'], wandItemCount=sum(map(len,sequences.values())),
        sequences=rows, gameplayValidated=False, dedicatedServerSupported=False), indent=2) + '\n')
    print(f'Confirmed {len(rows)} primary wand sequences for {sum(map(len,sequences.values()))} wand items; no other weapon shares these sequences')


if __name__ == '__main__':
    main()
