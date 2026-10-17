"""Verify unique original world-hook signatures and complete instruction boundaries."""
import re
from capstone import Cs, CS_ARCH_X86, CS_MODE_64

def verify_runtime_hook_origins(pe, profile):
    # Hooks are target-specific executable signatures, never copied client
    # addresses. Prove uniqueness and complete overwritten instructions.
    for name, hook in profile['hooks'].items():
        if not name.startswith('world_'): continue
        tokens = hook['signature'].split()
        pattern = re.compile(b''.join(b'.' if t in ('?', '??') else re.escape(bytes([int(t,16)])) for t in tokens), re.DOTALL)
        matches = []
        for section in pe.sections:
            if section['flags'] & 0x20000000:
                data = pe.data[section['raw']:section['raw']+section['rawSize']]
                matches += [section['rva']+m.start() for m in pattern.finditer(data)]
        assert len(matches) == 1, f'{name}: expected one executable signature, got {matches}'
        assert pe.read(matches[0], len(hook['original'])) == bytes(hook['original'])
        instructions = list(Cs(CS_ARCH_X86,CS_MODE_64).disasm(bytes(hook['original']), matches[0]))
        assert sum(i.size for i in instructions) == len(hook['original']), f'{name}: partial instruction overwrite'
    return True
