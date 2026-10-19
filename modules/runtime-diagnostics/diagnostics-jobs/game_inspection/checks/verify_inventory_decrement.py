"""Execute original inventory decrement helpers in Unicorn, never in the game.

Tests the zero quantity passed by zero_resource_argument against normal quantities.
Engine lookups/record allocation are isolated stubs; this is NOT a live factory test.
Legacy exact-image roots remain supported. Other images require a freshly checked
structural contract from inspect_inventory_transactions, without new RVA catalogs.
"""
import argparse
import json
import struct
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from capstone import Cs, CS_ARCH_X86, CS_MODE_64
from capstone.x86 import X86_OP_IMM
from unicorn import Uc, UC_ARCH_X86, UC_MODE_64, UC_HOOK_CODE
from unicorn.x86_const import UC_X86_REG_RSP, UC_X86_REG_RIP, UC_X86_REG_RAX, UC_X86_REG_RCX, UC_X86_REG_RDX, UC_X86_REG_R8, UC_X86_REG_R9
from game_inspection.inspect.read_pe_image import PE


BUILDS = {
    '001c1b40ed091d8c1aee583adde3800d7c858ae2c7f4dff54fca2938b2be1637': (0x164170, 0x1642f0),
    'af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781': (0x3825b0, 0x382730),
}


def run(pe, root, explicit_slot, amount):
    function = next(f for f in pe.functions if f['beginRva'] == root)
    code = pe.read(root, function['endRva'] - root)
    instructions = list(Cs(CS_ARCH_X86, CS_MODE_64).disasm(code, root))
    calls = [i for i in instructions if i.mnemonic == 'call']
    if len(calls) != 4 or any(not i.op_str.startswith('0x') for i in calls):
        raise ValueError('unexpected helper call contract')
    # Both exact images have: slot key, writable-slot check, stack decode, journal allocation.
    call_index = {pe.base + i.address: index for index, i in enumerate(calls)}
    uc = Uc(UC_ARCH_X86, UC_MODE_64)
    page = (pe.base + root) & ~0xfff
    uc.mem_map(page, 0x2000)
    uc.mem_write(pe.base + root, code)
    data, stack, stop = 0x20000000, 0x30000000, 0x40000000
    uc.mem_map(data, 0x10000)
    uc.mem_map(stack, 0x10000)
    uc.mem_map(stop, 0x1000)
    rsp = stack + 0x8008
    result, context, view, slots, journal = (data + x for x in (0, 0x100, 0x1000, 0x2000, 0x3000))
    item, initial = 455684957, 20
    uc.mem_write(view, struct.pack('<QQ', slots, 0))
    uc.mem_write(slots, struct.pack('<III', item, initial, 0) + bytes(84))
    uc.mem_write(rsp, struct.pack('<Q', stop))
    args = (0, item, amount) if explicit_slot else (item, amount)
    for index, value in enumerate(args):
        uc.mem_write(rsp + 40 + index * 8, struct.pack('<Q', value))
    for register, value in [(UC_X86_REG_RSP, rsp), (UC_X86_REG_RCX, result),
                            (UC_X86_REG_RDX, context), (UC_X86_REG_R8, view), (UC_X86_REG_R9, 123)]:
        uc.reg_write(register, value)

    def hook(machine, address, size, unused):
        if address not in call_index:
            return
        kind = call_index[address]
        if kind == 0:
            machine.reg_write(UC_X86_REG_RAX, 123)
        elif kind == 1:
            machine.reg_write(UC_X86_REG_RAX, 1)
        elif kind == 2:
            source, target = machine.reg_read(UC_X86_REG_RDX), machine.reg_read(UC_X86_REG_RCX)
            machine.mem_write(target, bytes(machine.mem_read(source, 12)))
            machine.reg_write(UC_X86_REG_RAX, target)
        else:
            machine.reg_write(UC_X86_REG_RAX, journal)
        machine.reg_write(UC_X86_REG_RIP, address + size)

    uc.hook_add(UC_HOOK_CODE, hook)
    uc.emu_start(pe.base + root, stop, count=5000)
    if uc.reg_read(UC_X86_REG_RIP) != stop:
        raise AssertionError('helper did not return within instruction budget')
    remaining = struct.unpack('<I', uc.mem_read(slots + 4, 4))[0]
    recorded = struct.unpack('<H', uc.mem_read(journal + 0x24, 2))[0]
    error = uc.mem_read(result, 1)[0]
    assert remaining == max(0, initial - amount), (hex(root), amount, remaining)
    assert recorded == min(initial, amount), (hex(root), amount, recorded)
    assert error == 0, (hex(root), amount, error)
    return dict(functionRva=root, explicitSlot=explicit_slot, requestedQuantity=amount,
                before=initial, after=remaining, journalQuantity=recorded)


def structural_contract(pe, report_path=None):
    """Select one structural helper and recheck every stubbed CALL against the image."""
    if report_path:
        report = json.loads(report_path.read_text(encoding='utf-8-sig'))
    else:
        from game_inspection.inspect.extract_executable_reflection import extract
        from game_inspection.inspect.inspect_inventory_transactions import inspect
        reflection = dict(image=pe.identity(), **extract(pe))
        report = inspect(pe, reflection)
    if (report.get('kind') != 'original-native-inventory-transaction-inspection' or
            report.get('image') != pe.identity()):
        raise ValueError('transaction inspection belongs to another image or report kind')
    candidates = report.get('candidates', {}).get('quantityDecrement', [])
    if len(candidates) != 1:
        raise ValueError(f'expected one quantity decrement candidate, found {len(candidates)}')
    candidate = candidates[0]
    contract = candidate.get('emulationContract', {})
    expected = dict(version=1, argumentCount=7, quantityArgument=7, slotArgument=5,
                    itemArgument=6, viewArgument=3, resultArgument=1, transactionArgument=2,
                    sourceCountOffset=4, journalQuantityOffset=0x24, journalKindOffset=0x26,
                    journalKind=1)
    if not contract.get('verifiedContract') or any(contract.get(k) != v for k, v in expected.items()):
        raise ValueError('quantity helper lacks the supported verified structural emulation contract')
    root = candidate['functionRva']
    if contract.get('helperFunctionRva') != root or not candidate.get('cfgConverged'):
        raise ValueError('quantity helper root or argument provenance is unresolved')
    if not any(row.get('edge', {}).get('calleeFunctionRva') == root
               for row in report.get('quantityArgumentForwarders', [])):
        raise ValueError('no original parent forwards its sixth quantity argument to this helper')
    ranges = [f for f in pe.functions if f['primaryBeginRva'] == root]
    if not ranges or not any(f['beginRva'] == root for f in ranges):
        raise ValueError('quantity helper has no original compiler entry range')
    decoder = Cs(CS_ARCH_X86, CS_MODE_64)
    decoder.detail = True
    instructions = {}
    for function in ranges:
        begin, end = function['beginRva'], function['endRva']
        cursor = begin
        for instruction in decoder.disasm(pe.read(begin, end - begin), begin):
            instructions[instruction.address] = instruction
            cursor = instruction.address + instruction.size
        if cursor != end:
            raise ValueError('quantity helper has undecoded original instructions')
    roles = ('slotKey', 'writableSlotCheck', 'decodeAvailableCount', 'journalAppend',
             'trackedWritableAccessor')
    dependencies = contract.get('dependencyCalls', [])
    counts = {role: sum(row.get('role') == role for row in dependencies) for role in roles}
    if (counts != dict(slotKey=1, writableSlotCheck=1, decodeAvailableCount=1,
                       journalAppend=1, trackedWritableAccessor=2) or len(dependencies) != 6):
        raise ValueError('unexpected quantity helper dependency roles')
    calls = {}
    for row in dependencies:
        instruction = instructions.get(row['instructionRva'])
        if (instruction is None or instruction.mnemonic != 'call' or
                len(instruction.operands) != 1 or instruction.operands[0].type != X86_OP_IMM or
                instruction.operands[0].imm != row['targetRva'] or instruction.address in calls):
            raise ValueError('structural dependency does not match a fresh original direct CALL')
        calls[instruction.address] = dict(role=row['role'], targetRva=row['targetRva'],
                                         size=instruction.size)
    if {i.address for i in instructions.values() if i.mnemonic == 'call'} != set(calls):
        raise ValueError('quantity helper has additional or unresolved dependency calls')
    return contract, ranges, calls


def run_structural(pe, contract, ranges, calls, amount):
    """Execute the original quantity helper, with all engine dependencies stubbed."""
    root = contract['helperFunctionRva']
    uc = Uc(UC_ARCH_X86, UC_MODE_64)
    pages = sorted({page for f in ranges
                    for page in range((pe.base + f['beginRva']) & ~0xfff,
                                      (pe.base + f['endRva'] + 0xfff) & ~0xfff, 0x1000)})
    if len(pages) > 64:
        raise ValueError('quantity helper compiler ranges exceed the isolated code budget')
    for page in pages:
        uc.mem_map(page, 0x1000)
    for function in ranges:
        uc.mem_write(pe.base + function['beginRva'],
                     pe.read(function['beginRva'], function['endRva'] - function['beginRva']))
    data, stack, stop = 0x20000000, 0x30000000, 0x40000000
    uc.mem_map(data, 0x10000)
    uc.mem_map(stack, 0x10000)
    uc.mem_map(stop, 0x1000)
    rsp = stack + 0x8008
    result, context, view, slots, journal = (data + x for x in (0, 0x100, 0x1000, 0x2000, 0x3000))
    item, initial, entity, slot = 455684957, 20, 123, 0
    uc.mem_write(view, struct.pack('<Q', slots) + bytes(40))
    uc.mem_write(slots, struct.pack('<III', item, initial, 0) + bytes(84))
    uc.mem_write(rsp, struct.pack('<Q', stop))
    for index, value in enumerate((slot, item, amount)):
        uc.mem_write(rsp + 40 + index * 8, struct.pack('<Q', value))
    for register, value in [(UC_X86_REG_RSP, rsp), (UC_X86_REG_RCX, result),
                            (UC_X86_REG_RDX, context), (UC_X86_REG_R8, view),
                            (UC_X86_REG_R9, entity)]:
        uc.reg_write(register, value)
    invocations = {row['role']: 0 for row in calls.values()}
    original_view = bytes(uc.mem_read(view, 48))
    original_context = bytes(uc.mem_read(context, 0x100))

    def hook(machine, address, size, unused):
        dependency = calls.get(address - pe.base)
        if dependency is None:
            return
        role = dependency['role']
        invocations[role] += 1
        rcx, rdx = machine.reg_read(UC_X86_REG_RCX), machine.reg_read(UC_X86_REG_RDX)
        if role == 'slotKey':
            assert rcx == entity and rdx == slot, (role, rcx, rdx)
            returned = entity | (slot << 32)
        elif role == 'writableSlotCheck':
            assert rcx == context and rdx == (entity | (slot << 32)), (role, rcx, rdx)
            returned = 1
        elif role == 'decodeAvailableCount':
            assert rdx == slots, (role, rdx)
            machine.mem_write(rcx, struct.pack('<III', 0, 0, initial))
            returned = rcx
        elif role == 'journalAppend':
            assert rcx == context + 0xf0, (role, rcx)
            returned = journal
        else:
            assert role == 'trackedWritableAccessor' and rcx == view, (role, rcx)
            returned = slots
        machine.reg_write(UC_X86_REG_RAX, returned)
        machine.reg_write(UC_X86_REG_RIP, address + dependency['size'])

    uc.hook_add(UC_HOOK_CODE, hook)
    uc.emu_start(pe.base + root, stop, count=5000)
    if uc.reg_read(UC_X86_REG_RIP) != stop:
        raise AssertionError('quantity helper did not return within instruction budget')
    if uc.reg_read(UC_X86_REG_RAX) != result:
        raise AssertionError('quantity helper did not return its result storage')
    after_id, after_count, after_pide = struct.unpack('<III', uc.mem_read(slots, 12))
    recorded = struct.unpack('<H', uc.mem_read(journal + contract['journalQuantityOffset'], 2))[0]
    journal_kind = uc.mem_read(journal + contract['journalKindOffset'], 1)[0]
    error, auxiliary, remainder = struct.unpack('<III', uc.mem_read(result, 12))
    expected_removed = min(initial, amount)
    assert after_count == initial - expected_removed, (amount, after_count)
    assert after_id == (0 if amount >= initial else item), (amount, after_id)
    assert after_pide == 0, (amount, after_pide)
    assert bytes(uc.mem_read(slots + 12, 84)) == bytes(84)
    assert recorded == expected_removed, (amount, recorded)
    assert journal_kind == contract['journalKind'], (amount, journal_kind)
    assert struct.unpack('<Q', uc.mem_read(journal, 8))[0] == (entity | (slot << 32))
    assert struct.unpack('<I', uc.mem_read(journal + 8, 4))[0] == item
    assert uc.mem_read(journal + 0x30, 1)[0] == int(amount >= initial)
    assert error == 0 and auxiliary == 0 and remainder == max(0, amount - initial), (amount, error, auxiliary, remainder)
    assert invocations == dict(slotKey=1, writableSlotCheck=1, decodeAvailableCount=1,
                               journalAppend=1, trackedWritableAccessor=int(amount > 0)), invocations
    assert bytes(uc.mem_read(view, 48)) == original_view
    assert bytes(uc.mem_read(context, 0x100)) == original_context
    return dict(functionRva=root, explicitSlot=True, requestedQuantity=amount,
                before=initial, after=after_count, sourceItemIdAfter=after_id,
                journalQuantity=recorded, journalKind=journal_kind, remainingRequestedQuantity=remainder,
                status=error, stubInvocations=invocations, originalInstructionsExecuted=True,
                originalNativeDependenciesExecuted=False, nativeTrackingRecorderExecuted=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('exe', type=Path)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--inspection', type=Path,
                        help='exact-image structural transaction report, otherwise inspect unknown images afresh')
    args = parser.parse_args()
    pe = PE(args.exe)
    if pe.sha256 in BUILDS and not args.inspection:
        roots = BUILDS[pe.sha256]
        cases = [run(pe, root, index == 0, amount)
                 for index, root in enumerate(roots) for amount in (0, 1, 10, 20)]
        resolution = dict(method='legacy-exact-image-catalog')
    else:
        contract, ranges, calls = structural_contract(pe, args.inspection)
        cases = [run_structural(pe, contract, ranges, calls, amount) for amount in (0, 1, 10, 20, 25)]
        resolution = dict(method='original-instruction-structural-contract', contract=contract,
                          inspection=str(args.inspection) if args.inspection else None)
    report = dict(image=pe.identity(), cases=cases, engineDependenciesStubbed=True,
                  liveFactoryConsumptionTested=False, nativeAbiProven=False, callable=False,
                  resolution=resolution)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(f'{len(cases)} original-code cases passed; zero quantity preserves the source stack')


if __name__ == '__main__':
    main()
