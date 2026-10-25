"""Emulate both native hook branches, including a clobbering Windows x64 callback.

Requires unicorn==2.1.4 and capstone==5.0.7. Pass the export-hook-code executable.
This tests generated instructions; it does not establish live game semantics.
"""
import pathlib
import struct
import subprocess
import sys
import tempfile
from capstone import Cs, CS_ARCH_X86, CS_MODE_64
from unicorn import Uc, UC_ARCH_X86, UC_MODE_64, UC_HOOK_CODE
from unicorn.x86_const import *


def main():
    with tempfile.TemporaryDirectory() as directory:
        path = pathlib.Path(directory) / 'hook.bin'
        jumps = [int(x) for x in subprocess.check_output([sys.argv[1], str(path)], text=True).split()]
        code = bytearray(path.read_bytes())
    base, callback, charge, skip = 0x100000, 0x300000, 0x400000, 0x400100
    for offset, destination in zip(jumps, [charge, skip], strict=True):
        struct.pack_into('<i', code, offset + 1, destination - (base + offset + 5))
    instructions = list(Cs(CS_ARCH_X86, CS_MODE_64).disasm(code, base))
    assert sum(i.size for i in instructions) == len(code), 'undecoded instructions'
    registers = [UC_X86_REG_RAX, UC_X86_REG_RBX, UC_X86_REG_RCX, UC_X86_REG_RDX,
                 UC_X86_REG_RBP, UC_X86_REG_RSI, UC_X86_REG_RDI, UC_X86_REG_R8,
                 UC_X86_REG_R9, UC_X86_REG_R10, UC_X86_REG_R11, UC_X86_REG_R12,
                 UC_X86_REG_R13, UC_X86_REG_R14, UC_X86_REG_R15]
    volatile = [UC_X86_REG_RAX, UC_X86_REG_RCX, UC_X86_REG_RDX, UC_X86_REG_R8,
                UC_X86_REG_R9, UC_X86_REG_R10, UC_X86_REG_R11]
    for decision in [False, True]:
        emu = Uc(UC_ARCH_X86, UC_MODE_64)
        for address in [base, callback, charge, 0x500000]:
            emu.mem_map(address, 0x10000)
        emu.mem_write(base, bytes(code))
        emu.mem_write(callback, b'\xc3')
        expected = {reg: 0x123456789000 + n * 0x101 for n, reg in enumerate(registers)}
        expected[UC_X86_REG_RBP] = 0x50a000
        expected[UC_X86_REG_RAX] = 0x50b000
        for reg, value in expected.items(): emu.reg_write(reg, value)
        emu.mem_write(0x50a390, struct.pack('<Q', 0xabcdef012345))
        emu.mem_write(0x50b000, struct.pack('<I', 42))
        rsp = 0x508000
        emu.reg_write(UC_X86_REG_RSP, rsp)
        emu.reg_write(UC_X86_REG_EFLAGS, 0x647)
        xmm = {UC_X86_REG_XMM0 + i: 0x123456789abcdef00000000000000000 + i for i in range(16)}
        for reg, value in xmm.items(): emu.reg_write(reg, value)
        reached = []

        def step(uc, address, size, _):
            if address == callback:
                assert uc.reg_read(UC_X86_REG_RSP) % 16 == 8, 'callback stack misaligned'
                assert uc.reg_read(UC_X86_REG_RCX) == 0xabcdef012345
                assert uc.reg_read(UC_X86_REG_RDX) == expected[UC_X86_REG_RBX]
                assert uc.reg_read(UC_X86_REG_R8) == 42
                assert not uc.reg_read(UC_X86_REG_EFLAGS) & 0x400, 'DF not cleared'
                # A callee may overwrite shadow space, volatile GPRs/XMM0..5,
                # and flags. The hook must restore all original live state.
                uc.mem_write(uc.reg_read(UC_X86_REG_RSP) + 8, b'\xa5' * 32)
                for reg in volatile: uc.reg_write(reg, 0xbadbadbad)
                for i in range(6): uc.reg_write(UC_X86_REG_XMM0 + i, 0xbadbadbad)
                uc.reg_write(UC_X86_REG_EFLAGS, 2)
                uc.reg_write(UC_X86_REG_RAX, int(decision))
            if address in (charge, skip):
                reached.append(address)
                uc.emu_stop()

        emu.hook_add(UC_HOOK_CODE, step)
        emu.emu_start(base, base + len(code), count=1000)
        assert reached == [skip if decision else charge], reached
        if not decision: expected[UC_X86_REG_R8] = 0xffffffff
        for reg, value in expected.items(): assert emu.reg_read(reg) == value, (decision, reg)
        for reg, value in xmm.items(): assert emu.reg_read(reg) == value, (decision, reg)
        assert emu.reg_read(UC_X86_REG_RSP) == rsp
        assert emu.reg_read(UC_X86_REG_EFLAGS) == 0x647
    print('Native hook branches preserve stack, flags, GPRs and all XMM registers')


if __name__ == '__main__':
    main()
