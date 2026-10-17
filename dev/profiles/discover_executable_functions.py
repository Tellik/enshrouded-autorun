"""Instruction-boundary xrefs, direct calls, and explicit unresolved semantics."""
import bisect
import hashlib
import re
import json
import struct


def mask_rip_displacement(code, instruction, begin, displacement):
    # In 64-bit address mode RIP-relative displacements are always signed disp32.
    # Capstone 5.0.7 reports disp_size=2 for e.g. 66 0f 6f 0d <disp32> (movdqa).
    # Validate against decoded bytes instead of trusting that encoding-size field.
    offset = instruction.address - begin + instruction.disp_offset
    if (instruction.disp_offset + 4 > instruction.size or
            struct.unpack_from('<i', code, offset)[0] != displacement):
        raise ValueError(f'inconsistent RIP displacement decoding at RVA 0x{instruction.address:x}')
    code[offset:offset + 4] = b'\0' * 4


def generate_signature(pe, rva, min_bytes=16, max_bytes=96):
    """Find the shortest unique instruction-aligned AOB within one native range.

    ``unique`` proves only a byte match in this image. It never verifies a hook,
    calling convention, patch offset, or the semantics of a candidate function.
    Functions without a containing PE RUNTIME_FUNCTION range are left unresolved.
    Invalid arguments raise ValueError. Missing dependencies, undecodable bytes,
    short ranges and ambiguous patterns instead return an explicit ``reason``.
    """
    if (not isinstance(rva, int) or isinstance(rva, bool) or rva < 0 or
            not isinstance(min_bytes, int) or isinstance(min_bytes, bool) or
            not isinstance(max_bytes, int) or isinstance(max_bytes, bool) or
            not 1 <= min_bytes <= max_bytes):
        raise ValueError('expected a nonnegative integer RVA and 1 <= min_bytes <= max_bytes')
    result = dict(rva=rva, signature=None, originalBytes=[], length=0,
                  matchCount=0, matches=[], unique=False, callable=False,
                  semanticStatus='unverified', reason=None, minBytes=min_bytes,
                  maxBytes=max_bytes, instructionAlignmentVerified=False,
                  maskRanges=[], maskedByteCount=0)
    index = pe.containing_function(rva)
    if index is None:
        result['reason'] = 'no-containing-function'
        return result
    native = pe.functions[index]
    begin, end = native['beginRva'], native['endRva']
    result['functionRange'] = dict(beginRva=begin, endRva=end)
    if not pe.executable(rva):
        result['reason'] = 'non-executable-rva'
        return result
    try:
        code = pe.read(begin, end - begin)
    except ValueError as error:
        result.update(reason='function-bytes-unavailable', detail=str(error))
        return result
    try:
        import capstone as cs
        from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_REG_RIP
    except ImportError:
        result.update(reason='disassembler-unavailable', dependency='capstone')
        return result

    engine = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
    engine.detail = True
    normalized = bytearray(code)
    masked, mask_ranges = set(), []
    executable_sections = None
    length = 0

    def snapshot():
        nonlocal executable_sections
        if not length:
            return
        original = code[rva - begin:rva - begin + length]
        tokens = ['??' if i in masked else f'{value:02X}' for i, value in enumerate(original)]
        result.update(signature=' '.join(tokens), originalBytes=list(original), length=length,
                      maskRanges=[row for row in mask_ranges if row['offset'] + row['size'] <= length],
                      maskedByteCount=sum(i < length for i in masked))
        if executable_sections is None:
            # Read every executable section. Omitting an unreadable section could
            # turn an ambiguous signature into a false unique match.
            executable_sections = [(section['rva'], pe.read(section['rva'], section['rawSize']))
                                   for section in pe.sections if section['flags'] & 0x20000000]
        needle = b''.join(b'.' if token == '??' else re.escape(bytes([value]))
                          for token, value in zip(tokens, original))
        regex = re.compile(b'(?=(' + needle + b'))', re.DOTALL)
        matches = sorted({section_rva + match.start() for section_rva, data in executable_sections
                          for match in regex.finditer(data)})
        result.update(matches=matches, matchCount=len(matches))

    def record_mask(instruction, offset, size, kind, target_rva):
        relative = instruction.address - rva + offset
        row = dict(instructionRva=instruction.address, offset=relative, size=size,
                   kind=kind, targetRva=target_rva)
        if row not in mask_ranges:
            mask_ranges.append(row)
        masked.update(range(relative, relative + size))

    def mask_encoding(instruction, offset, size, value, kind, target_rva):
        if offset < 0 or size <= 0 or offset + size > instruction.size:
            raise ValueError(f'invalid {kind} encoding at RVA 0x{instruction.address:x}')
        position = instruction.address - begin + offset
        encoded = int.from_bytes(code[position:position + size], 'little')
        if encoded != (value & ((1 << (size * 8)) - 1)):
            raise ValueError(f'inconsistent {kind} decoding at RVA 0x{instruction.address:x}')
        normalized[position:position + size] = b'\0' * size
        record_mask(instruction, offset, size, kind, target_rva)

    def image_address(value):
        # Operand bytes and a mapped preferred-image address are both required.
        # Numeric equality to an image address still does not prove semantics.
        address = value & ((1 << 64) - 1)
        target = address - pe.base
        if not 0 <= target < pe.image_size:
            return None
        if target < pe.headers_size or any(section['rva'] <= target < section['rva'] +
                                          max(section['size'], section['rawSize'])
                                          for section in pe.sections):
            return target
        return None

    cursor, stop_reason = begin, None
    for instruction in engine.disasm(code, begin):
        instruction_end = instruction.address + instruction.size
        if instruction.address < rva:
            if instruction_end > rva:
                result['reason'] = 'not-instruction-boundary'
                return result
            cursor = instruction_end
            continue
        result['instructionAlignmentVerified'] = True
        if instruction_end - rva > max_bytes:
            stop_reason = 'max-bytes-reached'
            break
        try:
            for operand in instruction.operands:
                if operand.type == X86_OP_MEM and operand.mem.base == X86_REG_RIP:
                    # Reuse the disp32 validation, including Capstone's known
                    # incorrect disp_size for operand-size-prefixed instructions.
                    if instruction.disp_offset < 0:
                        raise ValueError(f'invalid RIP displacement offset at RVA 0x{instruction.address:x}')
                    if instruction.address - rva + instruction.disp_offset not in masked:
                        mask_rip_displacement(normalized, instruction, begin, operand.mem.disp)
                    record_mask(instruction, instruction.disp_offset, 4, 'rip-relative-displacement',
                                instruction_end + operand.mem.disp)
                elif operand.type == X86_OP_IMM:
                    if (instruction.group(cs.CS_GRP_BRANCH_RELATIVE) or
                            instruction.group(cs.CS_GRP_JUMP) or instruction.group(cs.CS_GRP_CALL)):
                        offset, size = instruction.imm_offset, instruction.imm_size
                        position = instruction.address - begin + offset
                        if offset < 0 or size not in (1, 2, 4) or offset + size > instruction.size:
                            raise ValueError(f'unsupported relative branch encoding at RVA 0x{instruction.address:x}')
                        displacement = int.from_bytes(code[position:position + size], 'little', signed=True)
                        if ((instruction_end + displacement) & ((1 << 64) - 1)) != (operand.imm & ((1 << 64) - 1)):
                            raise ValueError(f'inconsistent relative branch decoding at RVA 0x{instruction.address:x}')
                        mask_encoding(instruction, offset, size, displacement,
                                      'relative-branch-immediate', operand.imm)
                    else:
                        target = image_address(operand.imm)
                        if target is not None:
                            mask_encoding(instruction, instruction.imm_offset, instruction.imm_size,
                                          operand.imm, 'absolute-image-immediate', target)
                elif (operand.type == X86_OP_MEM and not operand.mem.base and not operand.mem.index
                      and not operand.mem.segment):
                    target = image_address(operand.mem.disp)
                    if target is not None:
                        mask_encoding(instruction, instruction.disp_offset, instruction.disp_size,
                                      operand.mem.disp, 'absolute-image-displacement', target)
        except ValueError as error:
            snapshot()
            result.update(reason='inconsistent-instruction-encoding', detail=str(error))
            return result
        cursor = instruction_end
        length = cursor - rva
        if length >= min_bytes:
            snapshot()
            if result['matches'] == [rva]:
                result.update(unique=True, reason='unique-byte-match')
                return result
        if length == max_bytes:
            stop_reason = 'max-bytes-reached'
            break
    if cursor < rva:
        result.update(reason='undecoded-before-rva', undecodedRva=cursor)
        return result
    if stop_reason is None:
        stop_reason = 'function-range-end' if cursor == end else 'undecoded-instructions'
    snapshot()
    result['stopReason'] = stop_reason
    if stop_reason == 'undecoded-instructions':
        result.update(reason='undecoded-instructions', undecodedRva=cursor)
    elif length < min_bytes:
        result['reason'] = 'insufficient-bytes'
    else:
        result['reason'] = 'ambiguous-signature'
    return result


def group_functions(pe, functions):
    """Join UNW_FLAG_CHAININFO fragments before comparing whole native procedures.

    https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64
    """
    rows = {f['beginRva']: f for f in functions}
    groups = {}
    for native in pe.functions:
        root = native['primaryBeginRva']
        rows[native['beginRva']]['primaryBeginRva'] = root
        if root is not None:
            groups.setdefault(root, []).append(rows[native['beginRva']])
    result = []
    for root, members in sorted(groups.items()):
        members.sort(key=lambda f: f['beginRva'])
        digests = [(m['endRva'] - m['beginRva'], m.get('relocatedCodeSha256')) for m in members]
        digest = hashlib.sha256(json.dumps(digests).encode('ascii')).hexdigest() if all(d for _, d in digests) else None
        result.append(dict(beginRva=root, ranges=[m['beginRva'] for m in members],
                           codeBytes=sum(length for length, _ in digests),
                           relocatedCodeSha256=digest, semanticStatus='unverified', callable=False))
    return result


def profile_audit(pe, profile):
    if profile is None:
        return None
    image = profile['image']
    identity_matches = (profile['target'].lower() == pe.path.name.lower() and
                        image['timestamp'] == pe.timestamp and image['size'] == pe.image_size and
                        (not image.get('sha256') or image['sha256'] == pe.sha256))
    records = []

    def scan(pattern):
        tokens = pattern.split()
        if not tokens:
            return []
        needle = b''.join(b'.' if t in ('?', '??') else re.escape(bytes([int(t, 16)])) for t in tokens)
        # Lookahead retains overlapping occurrences; DOTALL includes 0x0a wildcards.
        regex = re.compile(b'(?=(' + needle + b'))', re.DOTALL)
        return [section['rva'] + match.start() for section in pe.sections if section['flags'] & 0x20000000
                for match in regex.finditer(pe.read(section['rva'], section['rawSize']))]

    for name, hook in profile.get('hooks', {}).items():
        matches = scan(hook['signature'])
        records.append(dict(id='hook.' + name, kind='hook', matches=matches,
                            status='unique-byte-match' if len(matches) == 1 else 'unresolved'))
    for name, operation in profile.get('worldOperations', {}).items():
        guard = operation.get('guardBytes', [])
        rva = operation.get('functionRva')
        guard_rva = operation.get('guardRva', rva)
        try:
            matches = bool(guard) and guard_rva is not None and pe.read(guard_rva, len(guard)) == bytes(guard)
        except ValueError:
            matches = False
        row = dict(id=name, kind='world-operation', functionRva=rva, guardRva=guard_rva,
                   guardMatches=matches, abi=operation.get('abi'), thread=operation.get('thread'),
                   status='guard-match' if identity_matches and matches and operation.get('validated', True) else 'unresolved',
                   unresolvedReason=operation.get('unresolvedReason'))
        if rva is not None:
            idx = pe.containing_function(rva)
            if idx is not None:
                row['containingFunctionRva'] = pe.functions[idx]['beginRva']
                row['offsetWithinFunction'] = rva - pe.functions[idx]['beginRva']
        records.append(row)
    for name, patch in profile.get('runtimePatches', {}).items():
        matches = scan(patch['signature'])
        records.append(dict(id=name, kind='patch', matches=matches,
                            status='unique-byte-match' if len(matches) == 1 else 'unresolved'))
    return dict(id=profile['id'], imageMatches=identity_matches,
                identityStrength='sha256' if image.get('sha256') else 'timestamp-and-size',
                operations=records, semanticsVerifiedByDiscovery=False)


def discover(pe, reflection, progress=print):
    import capstone as cs
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_REG_RIP
    import numpy as np

    engine = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
    engine.detail = True
    types = reflection['types']
    metadata = {t['metadataRva']: t['index'] for t in types}
    metadata_starts = sorted(metadata)
    metadata_slots = {}
    for section in pe.sections:
        if section['flags'] & 0x20000000:
            continue
        prefix = (-section['rva']) % 8
        data = pe.read(section['rva'], section['rawSize'])
        values = np.frombuffer(data, dtype='<u8', offset=prefix, count=(len(data) - prefix) // 8)
        candidates = np.flatnonzero((values >= pe.base + metadata_starts[0]) &
                                   (values <= pe.base + metadata_starts[-1]))
        for slot in candidates.tolist():
            ty = metadata.get(int(values[slot]) - pe.base)
            if ty is not None:
                metadata_slots[section['rva'] + prefix + slot * 8] = ty
    hash_types = {}
    for t in types:
        for kind in ('qualifiedHash', 'internalHash', 'nameHash', 'impactHash'):
            hash_types.setdefault(t[kind], []).append(dict(typeIndex=t['index'], hashKind=kind))
    functions, undecoded = [], []
    decoded_bytes = 0
    for ordinal, native in enumerate(pe.functions):
        begin, end = native['beginRva'], native['endRva']
        code = pe.read(begin, end - begin)
        calls, refs, hashes = [], [], []
        normalized = bytearray(code)
        cursor = begin
        for ins in engine.disasm(code, begin):
            cursor = ins.address + ins.size
            for operand in ins.operands:
                if operand.type == X86_OP_MEM and operand.mem.base == X86_REG_RIP:
                    target = cursor + operand.mem.disp
                    mask_rip_displacement(normalized, ins, begin, operand.mem.disp)
                    if target in metadata_slots:
                        refs.append(dict(instructionRva=ins.address, targetRva=target,
                                         typeIndex=metadata_slots[target], kind='metadata-pointer-slot',
                                         mnemonic=ins.mnemonic))
                    position = bisect.bisect_right(metadata_starts, target) - 1
                    if position >= 0 and target < metadata_starts[position] + 0x90:
                        descriptor = metadata_starts[position]
                        refs.append(dict(instructionRva=ins.address, targetRva=target,
                                         typeIndex=metadata[descriptor], metadataOffset=target - descriptor,
                                         mnemonic=ins.mnemonic))
                elif operand.type == X86_OP_IMM:
                    if ins.group(cs.CS_GRP_JUMP) or ins.group(cs.CS_GRP_CALL):
                        offset = ins.address - begin + ins.imm_offset
                        normalized[offset:offset + ins.imm_size] = b'\0' * ins.imm_size
                    if ins.id in (cs.x86.X86_INS_CALL, cs.x86.X86_INS_JMP):
                        calls.append(dict(instructionRva=ins.address, targetRva=operand.imm,
                                          kind='call' if ins.id == cs.x86.X86_INS_CALL else 'jump'))
                    elif (operand.imm & 0xffffffff) in hash_types and -(1 << 31) <= operand.imm <= 0xffffffff:
                        value = operand.imm & 0xffffffff
                        hashes.append(dict(instructionRva=ins.address, value=value, candidates=hash_types[value],
                                           status='immediate-equality-only'))
        decoded_bytes += cursor - begin
        if cursor != end:
            undecoded.append(dict(beginRva=cursor, endRva=end))
        functions.append(dict(native, id=f'rva.{begin:08x}', codeSha256=hashlib.sha256(code).hexdigest(),
                              relocatedCodeSha256=hashlib.sha256(normalized).hexdigest() if cursor == end else None,
                              decodedBytes=cursor - begin, directBranches=calls, metadataReferences=refs,
                              hashCandidates=hashes, semanticStatus='unverified', callable=False))
        if ordinal and ordinal % 10000 == 0:
            progress(f'disassembled {ordinal}/{len(pe.functions)} compiler ranges')
    pointers = []
    for section in pe.sections:
        if section['flags'] & 0x20000000:
            continue
        prefix = (-section['rva']) % 8
        data = pe.read(section['rva'], section['rawSize'])
        values = np.frombuffer(data, dtype='<u8', offset=prefix, count=(len(data) - prefix) // 8)
        offsets = np.flatnonzero((values >= pe.base) & (values < pe.base + pe.image_size))
        for slot in offsets.tolist():
            rva = int(values[slot]) - pe.base
            if pe.executable(rva):
                idx = pe.containing_function(rva)
                pointers.append(dict(slotRva=section['rva'] + prefix + slot * 8, targetRva=rva,
                                     containingFunctionRva=pe.functions[idx]['beginRva'] if idx is not None else None))
    covered = sum(f['endRva'] - f['beginRva'] for f in pe.functions)
    groups = group_functions(pe, functions)
    return dict(functions=functions, functionGroups=groups, codePointers=pointers, undecodedRanges=undecoded,
                unknownUnwindVersions=pe.unknown_unwind_versions,
                metadataPointerSlots=[dict(slotRva=rva, typeIndex=index) for rva, index in sorted(metadata_slots.items())],
                coverage=dict(compilerRanges=len(functions), rangeBytes=covered, decodedBytes=decoded_bytes,
                              chainedFunctionGroups=len(groups),
                              functionsWithMetadataReferences=sum(bool(f['metadataReferences']) for f in functions),
                              functionsWithHashCandidates=sum(bool(f['hashCandidates']) for f in functions),
                              semanticBindingsProven=0, allEngineFunctionsEnumerated=False),
                limitations=['PE exception ranges may be fragments and omit leaf/inlined functions',
                             'metadata references and matching hash immediates do not prove function arguments or semantics',
                             'indirect calls and runtime-generated code require additional analysis'])
