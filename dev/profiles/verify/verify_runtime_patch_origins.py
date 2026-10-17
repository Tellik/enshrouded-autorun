"""Verify exact-image runtime patch bytes, attribute requirements and original callers."""
from collections import defaultdict
from pathlib import Path
import struct
import sys
from capstone import Cs, CS_ARCH_X86, CS_MODE_64
from capstone.x86 import X86_REG_RBP, X86_REG_RSP, X86_OP_MEM, X86_OP_REG
sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from dev.profiles.inspect.read_system_descriptors import discover as discover_systems
from dev.profiles.verify.validate_patch_payload import validate_payload

def verify_runtime_patch_origins(pe, profile, functions, resources):
    assert profile['image']['sha256'] == pe.sha256, 'profile belongs to another image'
    assert functions['image']['sha256'] == pe.sha256, 'call inventory belongs to another image'
    assert Path(resources['executable']).resolve().samefile(pe.path), 'KFC export belongs to another executable'
    descriptors = discover_systems(pe, functions)
    names = defaultdict(set)
    for d in descriptors:
        names[d['functionRva']].add(d['name'])
    edges = defaultdict(set)
    for function in functions['functions']:
        for branch in function['directBranches']:
            if branch['kind'] == 'call': edges[function['primaryBeginRva']].add(branch['targetRva'])
    attributes = {}
    for resource in resources['resources']:
        if resource['qualifiedType'] == 'keen::AttributeContainerResource':
            for root in resource['value']['attributes']:
                assert len(root['ids']) == len(root['debugNames']) == len(root['structure'])
                for index, (id_, name) in enumerate(zip(root['ids'], root['debugNames'])):
                    assert id_['value'] not in attributes
                    attributes[id_['value']] = dict(name=name, index=index, root_hash=root['ids'][0]['value'])
    engine = Cs(CS_ARCH_X86, CS_MODE_64)
    engine.detail = True
    mods = []
    for alias, spec in profile['runtimePatches'].items():
        validate_payload(spec)
        modifier = spec['modifier']
        site = spec['function']['beginRva'] + spec['function']['targetOffset']
        owner = next(f for f in pe.functions if f['beginRva'] <= site < f['endRva'])
        assert owner['primaryBeginRva'] == modifier['function_rva']
        signature = bytes(0 if x in ('?', '??') else int(x, 16) for x in spec['signature'].split())
        original = pe.read(site, len(signature))
        assert all(x in ('?', '??') or original[i] == int(x, 16) for i, x in enumerate(spec['signature'].split()))
        if 'argument_index' in modifier:
            rsp_delta, rbp_delta = 0, None
            for instruction in engine.disasm(pe.read(owner['primaryBeginRva'], 96), owner['primaryBeginRva']):
                if instruction.mnemonic == 'push': rsp_delta -= 8
                elif instruction.mnemonic == 'sub' and instruction.operands[0].reg == X86_REG_RSP:
                    rsp_delta -= instruction.operands[1].imm
                elif instruction.mnemonic == 'lea' and instruction.operands[0].reg == X86_REG_RBP and instruction.operands[1].mem.base == X86_REG_RSP:
                    rbp_delta = rsp_delta + instruction.operands[1].mem.disp
                    break
            load = next(engine.disasm(original, site))
            assert rbp_delta is not None and load.operands[1].type == X86_OP_MEM and load.operands[1].mem.base == X86_REG_RBP
            offset = rbp_delta + load.operands[1].mem.disp
            # Return address + 32-byte home space => fifth argument at +0x28.
            assert offset >= 0x28 and (offset - 0x28) % 8 == 0
            assert 5 + (offset - 0x28) // 8 == modifier['argument_index']
        if 'engine_name' in modifier:
            assert modifier['engine_name'] in names[owner['primaryBeginRva']]
        for requirement in modifier.get('attribute_requirements', []):
            assert attributes[requirement['hash']] == {k: requirement[k] for k in ['name', 'index', 'root_hash']}
            # The selected function must actually reference the stated ID. It is
            # insufficient for the ID merely to exist in a separate resource.
            assert any(struct.pack('<I', requirement['hash']) in pe.read(f['beginRva'], f['endRva'] - f['beginRva'])
                       for f in pe.functions if f['primaryBeginRva'] == owner['primaryBeginRva'])
        for caller in modifier['callers']:
            path = caller['path_rvas']
            assert caller['name'] in names[path[0]] and path[-1] == owner['primaryBeginRva']
            assert all(b in edges[a] for a, b in zip(path, path[1:]))
        mods.append(dict(mod='sf-' + alias.rsplit('.', 1)[1].replace('_', '-'),
                         modifier=modifier, original_instructions=[f'{i.mnemonic} {i.op_str}' for i in engine.disasm(original[:spec['overwriteBytes']], site)],
                         code_and_origin_verified=True, gameplay_executed_by_audit=False))
    return mods, len(attributes), len(descriptors)
