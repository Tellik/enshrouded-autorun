"""Verify runtime profile mechanisms against the exact executable and its captures.

This action does not claim coverage of historical, removed mod packages.
"""
import argparse
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
from game_inspection.inspect.read_pe_image import PE
from game_inspection.verify.validate_patch_payload import validate_payload
from game_inspection.verify.verify_runtime_patch_origins import verify_runtime_patch_origins
from game_inspection.verify.verify_world_operation_origins import verify_world_operation_origins
from game_inspection.verify.verify_runtime_hook_origins import verify_runtime_hook_origins


def audit(pe, profile, functions, resources):
    mods, attribute_count, descriptor_count = verify_runtime_patch_origins(pe, profile, functions, resources)
    world = verify_world_operation_origins(pe, profile)
    verify_runtime_hook_origins(pe, profile)
    mods.append(dict(mod='world-editor', manifest_target='client', operations=world,
                     note='Client cursor/UI; direct world calls and optional ordinary ClientPlayerInput queue. Dispatch, replication and persistence are distinct; this static audit does not execute gameplay.'))
    return dict(image=pe.identity(), attribute_ids=attribute_count, engine_descriptors=descriptor_count,
                mods=mods, complete_engine_api=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--profile', type=Path, required=True)
    parser.add_argument('--functions', type=Path, required=True)
    parser.add_argument('--resources', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    read = lambda path: json.loads(path.read_text(encoding='utf-8-sig'))
    result = audit(PE(args.executable), read(args.profile), read(args.functions), read(args.resources))
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(f"Verified {len(result['mods'])} runtime mechanism origins, {result['attribute_ids']} attribute IDs, {result['engine_descriptors']} engine descriptors")


if __name__ == '__main__':
    main()
