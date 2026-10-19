"""Inspect native inventory transaction relationships without calling or changing the game.

Resolve named ECS descriptors, compiler-described procedures and original CALL
instructions afresh. Report instruction and reflection evidence, never approval
to execute a native function or to install a hook.
"""
import argparse
from collections import defaultdict, deque
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from game_inspection.analyze.read_capture_reports import checked_reports
from game_inspection.analyze.write_inspection_result import write_result
from game_inspection.inspect.extract_executable_reflection import extract
from game_inspection.inspect.read_pe_image import PE
from game_inspection.inspect.read_system_descriptors import discover as discover_systems


ROOT_NAMES = ('actor_pay_usage_cost', 'inventory_actions', 'equipment_actions', 'player_crafting')
UNKNOWN = frozenset(('unknown:source',))


def origins_known(values):
    return bool(values) and not any('unknown:' in value for value in values)


def known_origin(values, required):
    return required in values and origins_known(values)


def register_name(name):
    if not name:
        return ''
    for full, aliases in {
        'rax': ('eax', 'ax', 'al', 'ah'), 'rbx': ('ebx', 'bx', 'bl', 'bh'),
        'rcx': ('ecx', 'cx', 'cl', 'ch'), 'rdx': ('edx', 'dx', 'dl', 'dh'),
        'rsi': ('esi', 'si', 'sil'), 'rdi': ('edi', 'di', 'dil'),
        'rbp': ('ebp', 'bp', 'bpl'), 'rsp': ('esp', 'sp', 'spl'),
    }.items():
        if name == full or name in aliases:
            return full
    return re.sub(r'^(r\d+)[dwb]$', r'\1', name)


def instruction_row(instruction):
    return dict(rva=instruction.address, bytes=instruction.bytes.hex(),
                asm=instruction.mnemonic + ' ' + instruction.op_str,
                size=instruction.size, instructionBoundaryVerified=True)


class Inspection:
    def __init__(self, pe, reflection, functions=None):
        import capstone as cs
        from capstone import x86_const as x86

        self.pe, self.reflection, self.cs, self.x86 = pe, reflection, cs, x86
        self.decoder = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
        self.decoder.detail = True
        self.groups = defaultdict(list)
        for native in pe.functions:
            if native['primaryBeginRva'] is not None:
                self.groups[native['primaryBeginRva']].append(native)
        self.decoded, self.traces, self.paths = {}, {}, {}
        self.leaf_preservation = {}
        self.count_summaries = {}
        if functions:
            # Captured addresses must still describe the original compiler ranges.
            expected = {f['beginRva']: (f['endRva'], f['primaryBeginRva']) for f in pe.functions}
            for row in functions.get('functions', []):
                if expected.get(row['beginRva']) != (row['endRva'], row.get('primaryBeginRva')):
                    raise ValueError('function capture disagrees with executable unwind ranges')
        self.descriptors = self.read_descriptors(functions)

    def read_descriptors(self, functions):
        if functions and functions.get('codePointers') is not None:
            pointers = functions['codePointers']
            for row in pointers:
                if self.pe.u('<Q', row['slotRva'])[0] != self.pe.base + row['targetRva']:
                    raise ValueError('captured code pointer disagrees with original executable')
        else:
            pointers = []
            roots = set(self.groups)
            for section in self.pe.sections:
                if section['flags'] & 0x20000000:
                    continue
                data = self.pe.read(section['rva'], section['rawSize'])
                start = (-section['rva']) % 8
                for offset in range(start, len(data) - 7, 8):
                    target = struct.unpack_from('<Q', data, offset)[0] - self.pe.base
                    if target in roots:
                        pointers.append(dict(slotRva=section['rva'] + offset, targetRva=target))
        skeleton = dict(functionGroups=[dict(beginRva=r) for r in self.groups], codePointers=pointers)
        return discover_systems(self.pe, skeleton)

    def root(self, rva):
        index = self.pe.containing_function(rva)
        return self.pe.functions[index]['primaryBeginRva'] if index is not None else None

    def instructions(self, root):
        if root not in self.decoded:
            instructions = {}
            for native in self.groups.get(root, []):
                begin, end = native['beginRva'], native['endRva']
                cursor = begin
                for instruction in self.decoder.disasm(self.pe.read(begin, end - begin), begin):
                    instructions[instruction.address] = instruction
                    cursor = instruction.address + instruction.size
                if cursor != end:
                    # Keep undecoded ranges explicit. No edge is invented across them.
                    instructions[cursor] = None
            self.decoded[root] = instructions
        return self.decoded[root]

    def calls(self, root):
        result = []
        for instruction in self.instructions(root).values():
            if (instruction and instruction.mnemonic == 'call' and len(instruction.operands) == 1
                    and instruction.operands[0].type == self.x86.X86_OP_IMM):
                target = instruction.operands[0].imm
                result.append(dict(callerFunctionRva=root, calleeFunctionRva=self.root(target),
                                   instructionRva=instruction.address, targetRva=target,
                                   instruction=instruction_row(instruction)))
        return result

    def preserved_leaf_registers(self, target):
        """Prove untouched registers from a complete, call-free leaf procedure.

        Compiler stack probes are calls before argument registers are saved.
        Treating them as ordinary ABI calls would lose all incoming arguments.
        Their actual decoded writes, including implicit writes, establish which
        registers survive. An unknown or incomplete procedure proves nothing.
        """
        root = self.root(target)
        if root is None or root != target:
            return set()
        if root not in self.leaf_preservation:
            code = self.instructions(root)
            values = list(code.values())
            if not values or any(i is None or i.mnemonic == 'call' for i in values):
                self.leaf_preservation[root] = set()
            else:
                writes = set()
                complete = True
                for instruction in values:
                    _, changed = instruction.regs_access()
                    writes.update(register_name(self.decoder.reg_name(r)) for r in changed)
                    if (instruction.group(self.cs.CS_GRP_JUMP) and
                            (not instruction.operands or instruction.operands[0].type != self.x86.X86_OP_IMM
                             or instruction.operands[0].imm not in code)):
                        complete = False
                self.leaf_preservation[root] = ({'rax', 'rcx', 'rdx', 'r8', 'r9', 'r10', 'r11'} - writes
                                               if complete else set())
        return self.leaf_preservation[root]

    def leaf_instructions(self, target, maximum_bytes=256):
        """Decode a small direct-call leaf CFG absent from the unwind table."""
        pending, found = deque([target]), {}
        while pending:
            address = pending.popleft()
            if address in found:
                continue
            if not target <= address < target + maximum_bytes or len(found) >= 96:
                return []
            try:
                instruction = next(self.decoder.disasm(self.pe.read(address, 15), address, count=1))
            except (ValueError, StopIteration):
                return []
            if instruction.mnemonic in ('call', 'int3'):
                return []
            found[address] = instruction
            if instruction.group(self.cs.CS_GRP_RET):
                continue
            if instruction.mnemonic != 'jmp':
                pending.append(address + instruction.size)
            if instruction.group(self.cs.CS_GRP_JUMP):
                if len(instruction.operands) != 1 or instruction.operands[0].type != self.x86.X86_OP_IMM:
                    return []
                pending.append(instruction.operands[0].imm)
        return [found[address] for address in sorted(found)]

    def count_output_summary(self, target):
        """Derive the count output of a complete, call-free ItemStack getter.

        The accepted reads, output stores and every return are checked against
        reflection. No dependency address or unmodelled engine call is trusted.
        """
        if target in self.count_summaries:
            return self.count_summaries[target]
        self.count_summaries[target] = None
        code = self.leaf_instructions(target)
        if not code or any(i.mnemonic not in ('mov', 'cmp', 'test', 'xor', 'je', 'jne', 'jz', 'jnz', 'jmp', 'ret') for i in code):
            return None
        stack_type = self.typed('keen::ItemStack')
        data_type = self.typed('keen::ItemData')
        count_offset = stack_type['structFields']['data']['dataOffset'] + data_type['structFields']['count']['dataOffset']
        pide_offset = stack_type['structFields']['data']['dataOffset'] + data_type['structFields']['pide']['dataOffset']
        self.decoded[target] = {i.address: i for i in code}
        trace = self.trace(target)
        if not trace['cfgConverged'] or not trace['instructionDecodingComplete']:
            return None
        observed = trace['observations']
        output_offsets, read_offsets, returns = defaultdict(set), set(), []
        for instruction in code:
            row = observed.get(instruction.address)
            if row is None:
                return None
            if instruction.mnemonic == 'ret':
                if row.get('returnOrigins') != ['arg:1']:
                    return None
                returns.append(row)
            for index, operand in enumerate(instruction.operands):
                if operand.type != self.x86.X86_OP_MEM:
                    continue
                if operand.mem.index:
                    return None
                base = row['memoryBaseOrigins'][index]
                if operand.access & self.cs.CS_AC_WRITE:
                    if (instruction.mnemonic != 'mov' or index != 0 or base != ['arg:1']
                            or not 0 <= operand.mem.disp < 64 or operand.size not in (1, 4, 8)):
                        return None
                    if operand.size == 4:
                        output_offsets[operand.mem.disp].update(row['operandOrigins'][1])
                elif operand.access & self.cs.CS_AC_READ:
                    if base != ['arg:2'] or operand.mem.disp not in (0, count_offset, pide_offset) or operand.size != 4:
                        return None
                    read_offsets.add(operand.mem.disp)
        possible = [offset for offset, origins in output_offsets.items()
                    if f'load(arg:2+{count_offset})' in origins and 'imm:1' in origins]
        if len(possible) != 1 or read_offsets != {0, count_offset, pide_offset} or not returns:
            return None
        output_offset, origins = possible[0], set()
        for row in returns:
            fields = row['outputFieldOrigins']
            values = fields.get(f'heap|arg:1|{output_offset}|4')
            if values is None:
                # The invalid-item return zeroes a wider result member which
                # covers the count. Only a proven zero can be split this way.
                zero_cover = [value for key, value in fields.items()
                              if (lambda parts: int(parts[2]) <= output_offset and
                                  output_offset + 4 <= int(parts[2]) + int(parts[3]))(key.split('|'))
                              and value == ['imm:0']]
                values = ['imm:0'] if zero_cover else None
            if values is None or not origins_known(values):
                return None
            origins.update(values)
        if origins != {f'load(arg:2+{count_offset})', 'imm:0', 'imm:1'}:
            return None
        summary = dict(fieldOffset=output_offset, fieldSize=4, sourceArgument=2,
                       sourceCountOffset=count_offset, sourcePideOffset=pide_offset,
                       possibleOrigins=sorted(origins), returnCount=len(returns),
                       derivation='all original leaf CFG returns write count, unit count or zero',
                       instructions=[instruction_row(i) for i in code])
        self.count_summaries[target] = summary
        return summary

    def reachable(self, maximum_depth):
        for descriptor in self.descriptors:
            if descriptor['name'] not in ROOT_NAMES:
                continue
            root = descriptor['functionRva']
            queue, seen = deque([(root, [])]), {root}
            while queue:
                current, path = queue.popleft()
                self.paths.setdefault(current, {})[descriptor['name']] = path
                if len(path) >= maximum_depth:
                    continue
                for edge in self.calls(current):
                    callee = edge['calleeFunctionRva']
                    if callee is not None and callee not in seen:
                        seen.add(callee)
                        queue.append((callee, path + [edge]))
        return sorted(self.paths)

    def trace(self, root):
        """Conservative intra-procedure register/stack origins along original CFG edges.

        Values are origin sets, not an emulator. Arithmetic/aliasing is deliberately
        limited and unknown stores/calls do not prove a native C++ signature.
        """
        if root in self.traces:
            return self.traces[root]
        code = self.instructions(root)
        initial = {r: frozenset((f'arg:{i}',)) for i, r in enumerate(('rcx', 'rdx', 'r8', 'r9'), 1)}
        initial['rsp'] = frozenset(('stack:0',))
        states, pending, observations = {root: initial}, deque([root]), {}

        def default_origin(key):
            # An untouched incoming stack argument is known even before its
            # first explicit read. Other missing state is unknown, not bottom.
            if key.startswith('mem:'):
                offset = int(key[4:])
                if offset >= 40 and (offset - 40) % 8 == 0:
                    return frozenset((f'arg:{5 + (offset - 40) // 8}',))
            return UNKNOWN

        def stack_offset(values):
            if len(values) == 1:
                token = next(iter(values))
                if token.startswith('stack:'):
                    return int(token[6:])
            return None

        def value(operand, state, address=False):
            if operand.type == self.x86.X86_OP_REG:
                return state.get(register_name(self.decoder.reg_name(operand.reg)), UNKNOWN)
            if operand.type == self.x86.X86_OP_IMM:
                return frozenset((f'imm:{operand.imm}',))
            if operand.type != self.x86.X86_OP_MEM:
                return UNKNOWN
            memory = operand.mem
            if memory.index:
                if not address:
                    return UNKNOWN
                # LEA performs address arithmetic without reading memory.
                # Retain the proven base/index dependencies, including UNKNOWN.
                # Coefficients and displacement remain in the original operands.
                base = (state.get(register_name(self.decoder.reg_name(memory.base)), UNKNOWN)
                        if memory.base else frozenset())
                index = state.get(register_name(self.decoder.reg_name(memory.index)), UNKNOWN)
                return base | index
            base = state.get(register_name(self.decoder.reg_name(memory.base)), UNKNOWN)
            offset = stack_offset(base)
            if offset is not None:
                offset += memory.disp
                if address:
                    return frozenset((f'stack:{offset}',))
                key = f'mem:{offset}'
                if key in state:
                    return state[key]
                return default_origin(key)
            prefix = 'ptr' if address else 'load'
            origins = set(UNKNOWN) if not origins_known(base) else set()
            known = sorted(origin for origin in base if 'unknown:' not in origin)
            if known:
                expression = known[0] if len(known) == 1 else 'address-dependencies[' + '|'.join(known) + ']'
                origins.add(f'{prefix}({expression}+{memory.disp})')
            return frozenset(origins) or UNKNOWN

        def write(operand, values, state):
            if operand.type == self.x86.X86_OP_REG:
                state[register_name(self.decoder.reg_name(operand.reg))] = values
            elif operand.type == self.x86.X86_OP_MEM and not operand.mem.index:
                base = state.get(register_name(self.decoder.reg_name(operand.mem.base)), UNKNOWN)
                offset = stack_offset(base)
                if offset is not None:
                    state[f'mem:{offset + operand.mem.disp}'] = values
                elif base == frozenset(('arg:1',)):
                    state[f'heap|arg:1|{operand.mem.disp}|{operand.size}'] = values

        iterations = 0
        while pending and iterations < max(1000, len(code) * 40):
            iterations += 1
            address = pending.popleft()
            instruction = code.get(address)
            if instruction is None:
                continue
            state, operands = dict(states[address]), instruction.operands
            observation = dict(instruction=instruction_row(instruction),
                               operandOrigins=[sorted(value(o, state)) for o in operands],
                               memoryBaseOrigins=[sorted(state.get(register_name(self.decoder.reg_name(o.mem.base)), UNKNOWN))
                                                  if o.type == self.x86.X86_OP_MEM else [] for o in operands])
            if instruction.mnemonic == 'ret':
                observation['returnOrigins'] = sorted(state.get('rax', UNKNOWN))
                observation['outputFieldOrigins'] = {key: sorted(origins) for key, origins in state.items()
                                                     if key.startswith('heap|arg:1|')}
            if instruction.mnemonic == 'call':
                offset = stack_offset(state.get('rsp', UNKNOWN))
                observation['argumentOrigins'] = [sorted(state.get(r, UNKNOWN)) for r in ('rcx', 'rdx', 'r8', 'r9')]
                observation['argumentOrigins'] += [sorted(state.get(f'mem:{offset + n}', default_origin(f'mem:{offset + n}')))
                                                    if offset is not None else sorted(UNKNOWN) for n in (32, 40, 48, 56)]
                if operands and operands[0].type == self.x86.X86_OP_IMM:
                    summary = self.count_output_summary(operands[0].imm)
                    arguments = observation['argumentOrigins']
                    output = stack_offset(frozenset(arguments[0]))
                    if summary and output is not None:
                        source_values = set()
                        for origin in summary['possibleOrigins']:
                            if origin.startswith('load(arg:2+'):
                                parents = arguments[1]
                                if not origins_known(parents):
                                    source_values.update(UNKNOWN)
                                known = sorted(parent for parent in parents if 'unknown:' not in parent)
                                if known:
                                    # LEA/ADD sources jointly describe one address.
                                    # A scalar index is not a second pointer to dereference.
                                    address_expression = known[0] if len(known) == 1 else 'address-dependencies[' + '|'.join(known) + ']'
                                    source_values.add(f"load({address_expression}+{summary['sourceCountOffset']})")
                            else:
                                source_values.add(origin)
                        state[f"mem:{output + summary['fieldOffset']}"] = frozenset(source_values) or UNKNOWN
                        observation['derivedCountOutput'] = dict(summary, targetRva=operands[0].imm,
                                                                  callerOutputOffset=output + summary['fieldOffset'])
                preserved = (self.preserved_leaf_registers(operands[0].imm)
                             if operands and operands[0].type == self.x86.X86_OP_IMM else set())
                observation['leafRegistersUnmodified'] = sorted(preserved)
                for register in ('rax', 'rcx', 'rdx', 'r8', 'r9', 'r10', 'r11'):
                    if register not in preserved:
                        state[register] = UNKNOWN
                if operands and operands[0].type == self.x86.X86_OP_IMM:
                    if 'rax' not in preserved:
                        state['rax'] = frozenset((f'return:{operands[0].imm}',))
            elif instruction.mnemonic == 'push':
                offset = stack_offset(state.get('rsp', UNKNOWN))
                if offset is not None:
                    state['rsp'] = frozenset((f'stack:{offset - 8}',))
            elif instruction.mnemonic == 'pop':
                offset = stack_offset(state.get('rsp', UNKNOWN))
                if operands:
                    write(operands[0], UNKNOWN, state)
                state['rsp'] = frozenset((f'stack:{offset + 8}',)) if offset is not None else UNKNOWN
            elif instruction.mnemonic in ('mov', 'movzx', 'movsx', 'movsxd') and len(operands) == 2:
                write(operands[0], value(operands[1], state), state)
            elif instruction.mnemonic == 'lea' and len(operands) == 2:
                write(operands[0], value(operands[1], state, address=True), state)
            elif instruction.mnemonic.startswith('cmov') and len(operands) == 2:
                write(operands[0], value(operands[0], state) | value(operands[1], state), state)
            elif instruction.mnemonic in ('sub', 'add') and len(operands) == 2:
                before = value(operands[0], state)
                offset = stack_offset(before)
                operand_value = value(operands[1], state)
                immediate = (int(next(iter(operand_value))[4:]) if len(operand_value) == 1
                             and next(iter(operand_value)).startswith('imm:') else None)
                if offset is not None and immediate is not None:
                    delta = immediate * (-1 if instruction.mnemonic == 'sub' else 1)
                    write(operands[0], frozenset((f'stack:{offset + delta}',)), state)
                else:
                    write(operands[0], before | value(operands[1], state), state)
            elif (instruction.mnemonic in ('xor', 'pxor') and len(operands) == 2
                  and operands[0].type == self.x86.X86_OP_REG and operands[1].type == self.x86.X86_OP_REG
                  and operands[0].reg == operands[1].reg):
                write(operands[0], frozenset(('imm:0',)), state)
            elif instruction.mnemonic not in ('cmp', 'test', 'nop', 'ret', 'int3') and not instruction.group(self.cs.CS_GRP_JUMP):
                # Unknown writers must not retain stale argument provenance.
                for operand in operands:
                    if operand.access & self.cs.CS_AC_WRITE:
                        write(operand, UNKNOWN, state)
            observations[address] = observation
            successors = []
            if not instruction.group(self.cs.CS_GRP_RET) and instruction.mnemonic != 'int3':
                if instruction.mnemonic != 'jmp':
                    successors.append(address + instruction.size)
                if instruction.group(self.cs.CS_GRP_JUMP) and operands and operands[0].type == self.x86.X86_OP_IMM:
                    successors.append(operands[0].imm)
            for successor in successors:
                if successor not in code or code[successor] is None:
                    continue
                previous = states.get(successor)
                if previous is None:
                    states[successor] = state
                    pending.append(successor)
                else:
                    merged = {key: previous.get(key, default_origin(key)) | state.get(key, default_origin(key))
                              for key in previous.keys() | state.keys()}
                    merged = {key: values if len(values) <= 12 else frozenset(('unknown:merged',))
                              for key, values in merged.items()}
                    if merged != previous:
                        states[successor] = merged
                        pending.append(successor)
        result = dict(observations=observations, cfgConverged=not pending,
                      instructionDecodingComplete=all(i is not None for i in code.values()),
                      method='conservative-register-stack-origin-sets-on-original-cfg')
        self.traces[root] = result
        return result

    def paths_for(self, root):
        return [dict(name=name, edges=path) for name, path in sorted(self.paths.get(root, {}).items())]

    def typed(self, name):
        candidates = [t for t in self.reflection['types'] if t['qualifiedName'] == name]
        if len(candidates) != 1:
            raise ValueError(f'expected one reflected {name}, found {len(candidates)}')
        return candidates[0]


def quantity_contract(inspection, root, journals, count_offset, pide_offset):
    """Describe only a fully matched six-call original-code emulator boundary.

    Dependency names describe independently checked operand roles. They do not
    name engine exports or grant permission to call a dependency in the game.
    """
    x86 = inspection.x86
    calls = sorted(inspection.calls(root), key=lambda e: e['instructionRva'])
    trace = inspection.trace(root)
    observed = trace['observations']
    reasons = []
    if len(calls) != 6 or any(i and i.mnemonic == 'call' and
                             (not i.operands or i.operands[0].type != x86.X86_OP_IMM)
                             for i in inspection.instructions(root).values()):
        return dict(version=1, verifiedContract=False, rejectionReasons=['expected exactly six original direct calls'])

    def argument(edge, index):
        rows = observed.get(edge['instructionRva'], {}).get('argumentOrigins', [])
        return rows[index - 1] if len(rows) >= index else []

    key, eligibility, decode, append, writable_a, writable_b = calls
    if not known_origin(argument(key, 1), 'arg:4') or not known_origin(argument(key, 2), 'arg:5'):
        reasons.append('slot-key call does not receive entity argument four and slot argument five')
    if (not known_origin(argument(eligibility, 1), 'arg:2')
            or not known_origin(argument(eligibility, 2), f"return:{key['targetRva']}")):
        reasons.append('write eligibility call does not receive transaction and generated slot key')
    key_code = inspection.leaf_instructions(key['targetRva'])
    key_shape = False
    if len(key_code) == 4 and [i.mnemonic for i in key_code] == ['mov', 'mov', 'mov', 'ret']:
        entity, slot, packed = key_code[:3]
        key_shape = (entity.operands[0].type == x86.X86_OP_MEM and entity.operands[0].size == 4
                     and entity.operands[0].mem.base == x86.X86_REG_RSP
                     and entity.operands[1].type == x86.X86_OP_REG and entity.operands[1].reg == x86.X86_REG_ECX
                     and slot.operands[0].type == x86.X86_OP_MEM and slot.operands[0].size == 1
                     and slot.operands[0].mem.base == x86.X86_REG_RSP
                     and slot.operands[0].mem.disp == entity.operands[0].mem.disp + 4
                     and slot.operands[1].type == x86.X86_OP_REG and slot.operands[1].reg == x86.X86_REG_DL
                     and packed.operands[0].type == x86.X86_OP_REG and packed.operands[0].reg == x86.X86_REG_RAX
                     and packed.operands[1].type == x86.X86_OP_MEM and packed.operands[1].size == 8
                     and packed.operands[1].mem.base == x86.X86_REG_RSP
                     and packed.operands[1].mem.disp == entity.operands[0].mem.disp)
    if not key_shape:
        reasons.append('slot-key dependency does not pack reflected entity and slot fields into eight bytes')
    decode_code = inspection.leaf_instructions(decode['targetRva'])
    reads = {o.mem.disp for i in decode_code for o in i.operands
             if o.type == x86.X86_OP_MEM and o.size == 4 and o.access & inspection.cs.CS_AC_READ}
    unit_count = [i for i in decode_code if i.mnemonic == 'mov' and len(i.operands) == 2
                  and i.operands[0].type == x86.X86_OP_MEM and i.operands[0].size == 4
                  and i.operands[1].type == x86.X86_OP_IMM and i.operands[1].imm == 1]
    if not ({0, count_offset, pide_offset} <= reads and unit_count
            and origins_known(argument(decode, 1)) and any(v.startswith('stack:') for v in argument(decode, 1))
            and known_origin(argument(decode, 2), 'load(arg:3+0)')):
        reasons.append('available-count dependency does not match item/count/pide decoding on original view data')
    matched_journals = [j for j in journals if j['kind'] == 1
                        and known_origin(j['quantityStore'].get('operandOrigins', [[], []])[1], 'arg:7')
                        and known_origin(j['quantityStore'].get('memoryBaseOrigins', [[], []])[0], f"return:{append['targetRva']}")]
    if len(matched_journals) != 1 or not known_origin(argument(append, 1), 'arg:2'):
        reasons.append('journal allocator result and quantity argument seven do not reach one kind-one journal record')
    if (writable_a['targetRva'] != writable_b['targetRva'] or
            any(not known_origin(argument(edge, 1), 'arg:3') for edge in (writable_a, writable_b))):
        reasons.append('both tracked writable accessors must receive the original inventory view')
    count_writes = []
    for instruction in inspection.instructions(root).values():
        if (instruction and instruction.mnemonic == 'mov' and len(instruction.operands) == 2
                and instruction.operands[0].type == x86.X86_OP_MEM and instruction.operands[0].size == 4
                and instruction.operands[0].mem.disp == count_offset):
            row = observed.get(instruction.address, {})
            if (known_origin(row.get('memoryBaseOrigins', [[], []])[0], f"return:{writable_a['targetRva']}")
                    and known_origin(row.get('operandOrigins', [[], []])[1], 'arg:7')):
                count_writes.append(row)
    if not count_writes:
        reasons.append('decremented quantity does not reach original tracked count storage')
    dependencies = [dict(role=role, instructionRva=edge['instructionRva'], targetRva=edge['targetRva'],
                         instruction=edge['instruction']) for role, edge in zip(
                             ('slotKey', 'writableSlotCheck', 'decodeAvailableCount', 'journalAppend',
                              'trackedWritableAccessor', 'trackedWritableAccessor'), calls)]
    contract = dict(version=1, verifiedContract=not reasons and trace['cfgConverged'] and trace['instructionDecodingComplete'],
                    rejectionReasons=reasons, helperFunctionRva=root, argumentCount=7,
                    resultArgument=1, transactionArgument=2, viewArgument=3, entityArgument=4,
                    slotArgument=5, itemArgument=6, quantityArgument=7,
                    sourceCountOffset=count_offset, dependencyCalls=dependencies,
                    slotKeyInstructions=[instruction_row(i) for i in key_code],
                    availableCountInstructions=[instruction_row(i) for i in decode_code],
                    trackedCountWrites=count_writes,
                    verificationScope='static isolated-emulation dependency boundary only',
                    nativeAbiProven=False, liveVerified=False)
    if len(matched_journals) == 1:
        journal = matched_journals[0]
        quantity_ins = inspection.instructions(root)[journal['quantityStore']['instruction']['rva']]
        kind_ins = inspection.instructions(root)[journal['kindStore']['rva']]
        contract.update(journalQuantityOffset=quantity_ins.operands[0].mem.disp,
                        journalKindOffset=kind_ins.operands[0].mem.disp, journalKind=journal['kind'])
    return contract


def inspect(pe, reflection, functions=None, maximum_depth=5):
    inspection = Inspection(pe, reflection, functions)
    roots = inspection.reachable(maximum_depth)
    types = {name: inspection.typed(name) for name in
             ('keen::ItemStack', 'keen::ItemData', 'keen::InventorySlotId',
              'keen::actor::PayUsageCost', 'keen::ecs::SlotSelection')}
    count_offset = (types['keen::ItemStack']['structFields']['data']['dataOffset'] +
                    types['keen::ItemData']['structFields']['count']['dataOffset'])
    pide_offset = (types['keen::ItemStack']['structFields']['data']['dataOffset'] +
                   types['keen::ItemData']['structFields']['pide']['dataOffset'])
    consume_offset = types['keen::actor::PayUsageCost']['structFields']['consume']['dataOffset']
    actionbar_offset = types['keen::ecs::SlotSelection']['structFields']['actionbarEntityIds']['dataOffset']
    x86 = inspection.x86
    candidates = dict(partialSplit=[], quantityDecrement=[], usageConsume=[], legacyTransferBoolean=[])
    journal_owners = {}
    for root in roots:
        code = [i for i in inspection.instructions(root).values() if i]
        journal_types = [i for i in code if i.mnemonic == 'mov' and len(i.operands) == 2
                         and i.operands[0].type == x86.X86_OP_MEM and i.operands[0].size == 1
                         and i.operands[1].type == x86.X86_OP_IMM and i.operands[1].imm in (1, 4)]
        count_subs = [i for i in code if i.mnemonic == 'sub' and len(i.operands) == 2
                      and i.operands[0].type == x86.X86_OP_MEM and i.operands[0].size == 4
                      and i.operands[0].mem.disp == count_offset and i.operands[1].type == x86.X86_OP_REG]
        register_subs = [i for i in code if i.mnemonic == 'sub' and len(i.operands) == 2
                         and all(o.type == x86.X86_OP_REG and o.size == 4 for o in i.operands)]
        consume_tests = [i for i in code if i.mnemonic in ('cmp', 'test')
                         and any(o.type == x86.X86_OP_MEM and o.size == 1 and o.mem.disp == consume_offset
                                 for o in i.operands)]
        if not ((journal_types and (count_subs or register_subs)) or consume_tests):
            continue
        trace = inspection.trace(root)
        observed = trace['observations']
        journal_rows = []
        for kind in journal_types:
            destination = kind.operands[0]
            # Journal kind is immediately after its uint16 quantity field.
            stores = [i for i in code if i.mnemonic == 'mov' and len(i.operands) == 2
                      and i.operands[0].type == x86.X86_OP_MEM and i.operands[0].size == 2
                      and i.operands[0].mem.base == destination.mem.base
                      and i.operands[0].mem.disp + 2 == destination.mem.disp]
            for store in stores:
                journal_rows.append(dict(kind=kind.operands[1].imm, kindStore=instruction_row(kind),
                                         quantityStore=observed.get(store.address, dict(instruction=instruction_row(store)))))
        if journal_rows:
            journal_owners[root] = journal_rows
        for instruction in count_subs:
            row = observed.get(instruction.address, {})
            origins = row.get('operandOrigins', [[], []])[1]
            matching = [j for j in journal_rows if j['kind'] == 4
                        and set(j['quantityStore'].get('operandOrigins', [[], []])[1]) & set(origins)]
            following = inspection.instructions(root).get(instruction.address + instruction.size)
            if 'arg:4' in origins and matching and following and following.mnemonic == 'jmp':
                candidates['partialSplit'].append(dict(functionRva=root, sourceCountFieldOffset=count_offset,
                    subtraction=row, followingBranch=instruction_row(following), journal=matching,
                    namedCallers=inspection.paths_for(root), cfgConverged=trace['cfgConverged'],
                    originEvidenceComplete=origins_known(origins) and all(
                        origins_known(j['quantityStore'].get('operandOrigins', [[], []])[1]) for j in matching),
                    abiArgumentEvidence=dict(quantity=4, slot=3, transaction=2, result=1),
                    scope='partial removal only; full-stack clearing is a separate path',
                    nativeAbiProven=False, liveVerified=False))
        if any(j['kind'] == 1 for j in journal_rows):
            decrement_evidence = []
            for instruction in register_subs:
                row = observed.get(instruction.address, {})
                origins = row.get('operandOrigins', [[], []])
                if any('arg:7' in values for values in origins):
                    decrement_evidence.append(row)
            if decrement_evidence:
                candidates['quantityDecrement'].append(dict(functionRva=root, journal=journal_rows,
                    registerSubtractions=decrement_evidence, namedCallers=inspection.paths_for(root),
                    cfgConverged=trace['cfgConverged'], quantityArgument=7,
                    originEvidenceComplete=all(origins_known(values) for row in decrement_evidence
                                               for values in row.get('operandOrigins', []) if 'arg:7' in values),
                    emulationContract=quantity_contract(inspection, root, journal_rows, count_offset, pide_offset),
                    nativeAbiProven=False, liveVerified=False))
        for instruction in consume_tests:
            row = observed.get(instruction.address, {})
            if not any(known_origin(origins, f'load(arg:2+{consume_offset})') for origins in row.get('operandOrigins', [])):
                continue
            following = inspection.instructions(root).get(instruction.address + instruction.size)
            if not following or following.mnemonic not in ('je', 'jz', 'jne', 'jnz'):
                continue
            target = following.operands[0].imm
            gated_calls = [edge for edge in inspection.calls(root)
                           if following.address < edge['instructionRva'] < target]
            candidates['usageConsume'].append(dict(functionRva=root,
                reflectedType=types['keen::actor::PayUsageCost']['qualifiedName'],
                field='consume', fieldOffset=consume_offset, test=row,
                followingBranch=instruction_row(following), gatedDirectCalls=gated_calls,
                namedCallers=inspection.paths_for(root), cfgConverged=trace['cfgConverged'],
                originEvidenceComplete=True,
                nativeAbiProven=False, liveVerified=False))
    # Follow an original sixth stack argument into the seventh argument of a
    # callee which actually reads the reflected Actionbar inventory IDs.
    for root in roots:
        code = list(inspection.instructions(root).values())
        loads = [i for i in code if i and i.mnemonic == 'movzx' and len(i.operands) == 2
                 and i.operands[1].type == x86.X86_OP_MEM and i.operands[1].size == 1
                 and i.operands[1].mem.base in (x86.X86_REG_RBP, x86.X86_REG_RSP)]
        if not loads or not any(e['calleeFunctionRva'] in journal_owners for e in inspection.calls(root)):
            continue
        trace = inspection.trace(root)
        load_rows = [trace['observations'].get(i.address, {}) for i in loads]
        forwarded = []
        for edge in inspection.calls(root):
            row = trace['observations'].get(edge['instructionRva'], {})
            arguments = row.get('argumentOrigins', [])
            if len(arguments) < 7 or not known_origin(arguments[6], 'arg:6'):
                continue
            byte_stores = [i for i in code if i and i.mnemonic == 'mov' and len(i.operands) == 2
                           and edge['instructionRva'] - 96 <= i.address < edge['instructionRva']
                           and i.operands[0].type == x86.X86_OP_MEM and i.operands[0].size == 1
                           and i.operands[0].mem.base == x86.X86_REG_RSP and i.operands[0].mem.disp == 48
                           and known_origin(trace['observations'].get(i.address, {}).get('operandOrigins', [[], []])[1], 'arg:6')]
            if not byte_stores:
                continue
            callee = edge['calleeFunctionRva']
            if callee is None:
                continue
            downstream = inspection.calls(callee)
            intermediary_trace = inspection.trace(callee)
            roles = []
            for next_edge in downstream:
                final = next_edge['calleeFunctionRva']
                if final is None:
                    continue
                next_row = intermediary_trace['observations'].get(next_edge['instructionRva'], {})
                next_arguments = next_row.get('argumentOrigins', [])
                if len(next_arguments) < 7 or not known_origin(next_arguments[6], 'arg:7'):
                    continue
                field_reads = [instruction_row(i) for i in inspection.instructions(final).values()
                               if i and any(o.type == x86.X86_OP_MEM and o.mem.disp in
                                            (actionbar_offset, actionbar_offset + 4) for o in i.operands)]
                if len(field_reads) >= 2:
                    final_trace = inspection.trace(final)
                    boolean_branches = []
                    for instruction in inspection.instructions(final).values():
                        if not instruction or instruction.mnemonic not in ('cmp', 'test'):
                            continue
                        origin = final_trace['observations'].get(instruction.address, {})
                        following = inspection.instructions(final).get(instruction.address + instruction.size)
                        if (not any(known_origin(values, 'arg:7') for values in origin.get('operandOrigins', []))
                                or not following or following.mnemonic not in ('je', 'jz', 'jne', 'jnz')):
                            continue
                        target = following.operands[0].imm
                        gated = [row for row in field_reads if following.address < row['rva'] < target]
                        if gated:
                            boolean_branches.append(dict(test=origin, branch=instruction_row(following),
                                                         gatedActionbarReads=gated))
                    if not boolean_branches:
                        continue
                    roles.append(dict(edge=next_edge, booleanForwarding=next_row,
                                      actionbarFieldReads=field_reads, booleanBranches=boolean_branches,
                                      cfgConverged=final_trace['cfgConverged'],
                                      instructionDecodingComplete=final_trace['instructionDecodingComplete'],
                                      association='offset-compatible SlotSelection.actionbarEntityIds; context type unresolved'))
            if roles:
                forwarded.append(dict(edge=edge, arguments=row, downstreamRoles=roles,
                                      booleanArgumentStores=[trace['observations'][i.address] for i in byte_stores]))
        if forwarded:
            candidates['legacyTransferBoolean'].append(dict(functionRva=root, booleanLoads=load_rows,
                forwardedCalls=forwarded, namedCallers=inspection.paths_for(root),
                role='inventory insertion/actionbar selection, separate from PayUsageCost.consume',
                originEvidenceComplete=True,
                cfgConverged=trace['cfgConverged'], nativeAbiProven=False, liveVerified=False))
    # Report actual parent forwarding of quantity argument six into child seven.
    decrement_roots = {r['functionRva'] for r in candidates['quantityDecrement']}
    quantity_forwarders = []
    for root in roots:
        relevant = [e for e in inspection.calls(root) if e['calleeFunctionRva'] in decrement_roots]
        if not relevant:
            continue
        trace = inspection.trace(root)
        for edge in relevant:
            row = trace['observations'].get(edge['instructionRva'], {})
            arguments = row.get('argumentOrigins', [])
            if len(arguments) >= 7 and known_origin(arguments[6], 'arg:6'):
                quantity_forwarders.append(dict(functionRva=root, edge=edge, callArguments=row,
                    namedCallers=inspection.paths_for(root), cfgConverged=trace['cfgConverged']))
    for rows in candidates.values():
        for row in rows:
            row['instructionDecodingComplete'] = inspection.trace(row['functionRva'])['instructionDecodingComplete']
    for row in quantity_forwarders:
        row['instructionDecodingComplete'] = inspection.trace(row['functionRva'])['instructionDecodingComplete']

    def resolution(rows):
        complete = bool(rows) and all(row['cfgConverged'] and row['instructionDecodingComplete']
                                      and row['originEvidenceComplete'] for row in rows)
        unique = len(rows) == 1 and complete
        return dict(candidateCount=len(rows), unique=unique,
                    status='unique-instruction-form' if unique else 'incomplete-instruction-or-origin-evidence' if rows and not complete
                    else 'ambiguous' if rows else 'unresolved')

    return dict(schemaVersion=1, kind='original-native-inventory-transaction-inspection', image=pe.identity(),
        descriptors=[d for d in inspection.descriptors if d['name'] in ROOT_NAMES],
        reflectedTypes=list(types.values()), candidates=candidates, quantityArgumentForwarders=quantity_forwarders,
        resolution={name: resolution(rows) for name, rows in candidates.items()},
        coverage=dict(maximumDirectCallDepth=maximum_depth, reachableProcedureGroups=len(roots),
                      decodedProcedureGroups=len(inspection.decoded), originTracedGroups=len(inspection.traces),
                      incompleteProcedureGroups=[root for root, code in inspection.decoded.items()
                                                 if any(i is None for i in code.values())]),
        nativeAbiProven=False, callable=False, liveVerified=False, processMemoryRead=False, processMemoryWritten=False,
        limitations=['Static instruction forms establish evidence candidates, not native-call authorization',
                     'Register origin sets are conservative and do not emulate engine state or journal commits',
                     'Unknown origins survive CFG joins and invalidate role confirmations which depend on them',
                     'A reflected field offset alone does not prove the type of an opaque native context',
                     'Only original direct CALL edges and compiler unwind groups are traversed',
                     'Partial-split subtraction and full removal must be validated independently in game'])


def read_live_code(pe, report, pid):
    """Compare bounded resolved procedure ranges through the existing read-only API."""
    from game_inspection.capture.read_process_memory import Process

    selected = defaultdict(list)
    for family, resolution in report['resolution'].items():
        if resolution['unique']:
            for candidate in report['candidates'][family]:
                selected[candidate['functionRva']].append(family)
    process = Process(pid, pe)
    rows, total = [], 0
    try:
        for root, families in sorted(selected.items()):
            ranges = [native for native in pe.functions if native['primaryBeginRva'] == root]
            size = sum(native['endRva'] - native['beginRva'] for native in ranges)
            if size > 65536 or total + size > 1024 * 1024:
                rows.append(dict(functionRva=root, families=families, matches=False,
                                 reason='bounded live readback budget exceeded'))
                continue
            fragments = []
            for native in ranges:
                begin, length = native['beginRva'], native['endRva'] - native['beginRva']
                expected = pe.read(begin, length)
                try:
                    actual = process.read(process.base + begin, length)
                    total += length
                    changed = [i for i, (left, right) in enumerate(zip(expected, actual)) if left != right]
                    fragments.append(dict(beginRva=begin, length=length, matches=not changed,
                        originalSha256=hashlib.sha256(expected).hexdigest(),
                        liveSha256=hashlib.sha256(actual).hexdigest(),
                        changedByteCount=len(changed), changedByteOffsets=changed[:32],
                        changedByteOffsetsTruncated=len(changed) > 32))
                except OSError as error:
                    fragments.append(dict(beginRva=begin, length=length, matches=False, reason=str(error)))
            rows.append(dict(functionRva=root, families=families, ranges=fragments,
                             matches=bool(fragments) and all(row['matches'] for row in fragments)))
        report.update(processMemoryRead=True,
                      liveCodeMatches=bool(rows) and all(row['matches'] for row in rows),
                      liveCodeReadback=dict(pid=pid, moduleBase=hex(process.base), bytesRead=total,
                                           procedures=rows, maximumProcedureBytes=65536,
                                           maximumTotalBytes=1024 * 1024, consistentSnapshot=False,
                                           scope='code identity only; no transaction or gameplay effects tested'))
    finally:
        process.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--capture', type=Path, help='directory containing reflection.json and optional functions.json')
    parser.add_argument('--reflection', type=Path, help='exact-image reflection capture')
    parser.add_argument('--functions', type=Path, help='exact-image function capture')
    parser.add_argument('--max-call-depth', type=int, choices=range(1, 9), default=5)
    parser.add_argument('--pid', type=int, help='optional read-only live code comparison against this exact executable')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    if args.capture and (args.reflection or args.functions):
        parser.error('--capture cannot be combined with --reflection or --functions')
    if args.pid is not None and args.pid <= 0:
        parser.error('--pid must be positive')
    pe = PE(args.executable)
    if args.capture:
        reflection, functions, _, _ = checked_reports(args.capture)
    else:
        reflection = json.loads(args.reflection.read_text(encoding='utf-8-sig')) if args.reflection else dict(image=pe.identity(), **extract(pe))
        functions = json.loads(args.functions.read_text(encoding='utf-8-sig')) if args.functions else None
    for report in (reflection, functions):
        if report is not None and report.get('image') != pe.identity():
            raise ValueError('capture belongs to another exact executable image')
    result = inspect(pe, reflection, functions, args.max_call_depth)
    if args.pid is not None:
        read_live_code(pe, result, args.pid)
    result['sources'] = dict(executable=str(pe.path), capture=str(args.capture) if args.capture else None,
                            reflection=str(args.reflection) if args.reflection else None,
                            functions=str(args.functions) if args.functions else None,
                            instructions='fresh Capstone disassembly of original executable bytes')
    write_result(result, args.out)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError, KeyError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
