"""Inspect original named ECS systems, dependencies and direct calls offline.

Descriptor discovery preserves engine spellings and raw dependency slots. The
optional inspection reads service names, compiler ranges and reflected layouts.
It never opens a process, invokes engine code or establishes a callable ABI.
"""
import argparse
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
import re
import sys


def discover(pe, functions):
    roots = {f['beginRva'] for f in functions['functionGroups']}

    def span(pointer, length):
        if not 1 <= length <= 256:
            raise ValueError('invalid string span')
        value = pe.read(pe.rva(pointer), length).rstrip(b'\0')
        if not value or any(c < 32 or c >= 127 for c in value):
            raise ValueError('invalid descriptor string')
        return value.decode('ascii')

    records = {}
    for pointer in functions['codePointers']:
        root, slot = pointer['targetRva'], pointer['slotRva']
        if root not in roots or slot < 16:
            continue
        try:
            name = span(*pe.u('<QQ', slot - 16))
            if not re.fullmatch(r'[a-z][a-z0-9_]{2,95}', name):
                continue
            arrays = []
            for offset in (24, 40, 56, 72, 88, 104):
                data, count = pe.u('<QQ', slot - 16 + offset)
                if count > 256 or bool(data) != bool(count):
                    raise ValueError('invalid descriptor array')
                values = [span(*pe.u('<QQ', pe.rva(data) + i * 16)) for i in range(count)]
                arrays.append(dict(offset=offset, entries=values))
            if not any(a['entries'] for a in arrays):
                continue
            records[slot - 16] = dict(name=name, functionRva=root, descriptorRva=slot - 16,
                                     nameSource='engine-execution-descriptor', dependencySlots=arrays)
        except (ValueError, UnicodeError):
            continue
    return sorted(records.values(), key=lambda r: r['descriptorRva'])


def reflected_layout(reflection, name, path):
    """Resolve an exact qualified type and nested field names from reflection."""
    matches = [ty for ty in reflection['types'] if ty['qualifiedName'] == name]
    if len(matches) != 1:
        raise ValueError(f'reflected type must resolve uniquely: {name}')
    ty, offset = matches[0], 0
    for field in path:
        entry = ty['structFields'].get(field)
        if entry is None:
            raise ValueError(f'unknown reflected field {ty["qualifiedName"]}.{field}')
        offset += entry['dataOffset']
        ty = reflection['types'][entry['type']]
    return dict(component=name, path=list(path), offset=offset,
                layout={key: value for key, value in ty.items() if key != 'defaultValue'})


def inspect(pe, reflection, functions, system_names=None, component_paths=None):
    """Inspect all descriptors or an exact-name selection without changing discover()."""
    from capstone import Cs, CS_ARCH_X86, CS_MODE_64
    from capstone.x86_const import X86_OP_IMM

    identity = pe.identity()
    if reflection.get('image') != identity or functions.get('image') != identity:
        raise ValueError('reflection and function captures must belong to the exact executable image')
    for pointer in functions['codePointers']:
        if pe.u('<Q', pointer['slotRva'])[0] != pe.base + pointer['targetRva']:
            raise ValueError('captured code pointer differs from the original executable')
    descriptors = discover(pe, functions)
    requested = set(system_names or ())
    missing = requested - {row['name'] for row in descriptors}
    if missing:
        raise ValueError('original system names were not found: ' + ', '.join(sorted(missing)))
    groups = defaultdict(list)
    for native in pe.functions:
        if native['primaryBeginRva'] is not None:
            groups[native['primaryBeginRva']].append(native)
    decoder = Cs(CS_ARCH_X86, CS_MODE_64)
    decoder.detail = True
    decoder.skipdata = False
    selected = []
    for descriptor in descriptors:
        if requested and descriptor['name'] not in requested:
            continue
        row = dict(descriptor, services=[], directCalls=[], codeRanges=[])
        for offset, access in ((0x98, 'read'), (0xb0, 'write')):
            pointer, count = pe.u('<QQ', descriptor['descriptorRva'] + offset)
            if count > 64 or bool(pointer) != bool(count):
                raise ValueError('invalid original system service span')
            for index in range(count):
                name_pointer, length = pe.u('<QQ', pe.rva(pointer) + index * 16)
                if not 1 <= length <= 256:
                    raise ValueError('invalid original system service name span')
                name = pe.text(name_pointer, length).rstrip('\0')
                if not name or any(ord(c) < 32 or ord(c) >= 127 for c in name):
                    raise ValueError('invalid original system service name')
                row['services'].append(dict(name=name, access=access, ordinal=len(row['services'])))
        for native in groups[descriptor['functionRva']]:
            begin, end = native['beginRva'], native['endRva']
            cursor = begin
            for instruction in decoder.disasm(pe.read(begin, end - begin), begin):
                cursor = instruction.address + instruction.size
                if (instruction.mnemonic == 'call' and len(instruction.operands) == 1
                        and instruction.operands[0].type == X86_OP_IMM):
                    target = instruction.operands[0].imm
                    index = pe.containing_function(target)
                    row['directCalls'].append(dict(instructionRva=instruction.address, targetRva=target,
                        procedureRva=pe.functions[index]['primaryBeginRva'] if index is not None else None,
                        bytes=instruction.bytes.hex()))
            row['codeRanges'].append(dict(beginRva=begin, endRva=end, decodedEndRva=cursor,
                decodingComplete=cursor == end, undecodedBytes=end - cursor))
        row['instructionDecodingComplete'] = bool(row['codeRanges']) and all(
            part['decodingComplete'] for part in row['codeRanges'])
        selected.append(row)
    layouts = [reflected_layout(reflection, name, path) for name, path in (component_paths or ())]
    return dict(schemaVersion=1, kind='original-ecs-system-inspection',
        capturedAt=datetime.now(timezone.utc).isoformat(), image=identity,
        originalSystemCount=len(descriptors), selectedSystemNames=sorted(requested),
        systems=selected, reflectedLayouts=layouts, callable=False, nativeAbiProven=False,
        processMemoryRead=False, processMemoryWritten=False, worldChanged=False,
        limitations=[
            'Original names, dependency spans and direct calls do not prove callable native ABIs',
            'Compiler ranges may omit leaf functions and inlined or generated code',
            'Decoding stops at the first undecodable byte of each compiler range without resynchronization',
            'Indirect calls are not resolved and no native arguments or commit contracts are inferred',
            'Reflected field layouts do not prove local player identity or authoritative write permission',
            'No live service binding, component membership, code identity or gameplay effects are observed'])


def main():
    sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
    from game_inspection.analyze.read_capture_reports import checked_reports
    from game_inspection.analyze.write_inspection_result import write_result
    from game_inspection.inspect.read_pe_image import PE

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('executable', type=Path)
    parser.add_argument('--capture', type=Path, required=True,
                        help='exact-image directory containing reflection.json and functions.json')
    parser.add_argument('--system', action='append', default=[], metavar='EXACT_NAME',
                        help='full original system name, repeat to select several. Default: all systems')
    parser.add_argument('--layout', action='append', nargs='+', default=[], metavar='TYPE_OR_FIELD',
                        help='qualified reflected type followed by optional nested field names, repeatable')
    parser.add_argument('--out', type=Path)
    args = parser.parse_args()
    pe = PE(args.executable)
    reflection, functions, _, _ = checked_reports(args.capture)
    if not functions:
        raise ValueError('an exact-image function capture is required')
    layouts = [(row[0], row[1:]) for row in args.layout]
    result = inspect(pe, reflection, functions, args.system, layouts)
    result['sources'] = dict(executable=str(pe.path), capture=str(args.capture),
                             instructions='fresh Capstone disassembly of original executable bytes')
    write_result(result, args.out)


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, ImportError, KeyError) as error:
        print(f'inspection failed: {error}', file=sys.stderr)
        sys.exit(2)
