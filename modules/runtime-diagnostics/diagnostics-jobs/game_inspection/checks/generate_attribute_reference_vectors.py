"""Generate differential-test vectors by emulating the ORIGINAL game interpreters.

The PE is data in Unicorn, never loaded/executed by Windows. Only private scratch
and stack pages are writable; imports, invalid accesses and timeouts fail closed.
No game process is opened. RVAs below are independently disassembled build facts.
"""
import argparse
import json
import random
import struct
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from game_inspection.inspect.read_pe_image import PE
from unicorn import Uc, UC_ARCH_X86, UC_MODE_64, UC_PROT_READ, UC_PROT_EXEC, UC_PROT_ALL
from unicorn.x86_const import (UC_X86_REG_RCX, UC_X86_REG_RDX, UC_X86_REG_R8,
                              UC_X86_REG_R9, UC_X86_REG_RSP, UC_X86_REG_RIP, UC_X86_REG_MXCSR)

BUILDS = {
    'af2f5a1227911d8aa06b3908d6bd0211838211cae14ea91099cb57d0df990781':
        {'interpreters': {'sint32': 0x1e1b00, 'uint32': 0x214090, 'float32': 0x1f3040},
         'resultValueOffset': 4},
    '001c1b40ed091d8c1aee583adde3800d7c858ae2c7f4dff54fca2938b2be1637':
        {'interpreters': {'sint32': 0x2c310, 'uint32': 0x4e1c0, 'float32': 0x3d4f0},
         'resultValueOffset': 4},
    # Current KFC4 client. Each unwind-bounded interpreter writes success at +0,
    # auxiliary metadata at +4 and the actual scalar result at +8.
    '4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a':
        {'interpreters': {'sint32': 0x21c510, 'uint32': 0x24d6d0, 'float32': 0x2316c0},
         'resultValueOffset': 8},
    # Exact client build 26082781. The three unwind-bounded roots and the +8
    # scalar result were rederived from this executable before vector generation.
    'd64966e23b1a458ee6f147bcbceac0b577fa7e39a212f371287d8cc119817882':
        {'interpreters': {'sint32': 0x21c9a0, 'uint32': 0x24db80, 'float32': 0x231b70},
         'resultValueOffset': 8},
    # Exact server build 26083076. Interpreter roots were located in the exact
    # executable by unwind-bounded disassembly and matched to the current client
    # interpreter structure before differential vector generation.
    'd7baa768eb00aa895d9f5a1cd8747ccf66858c7b542f50983092b2e3261588ee':
        {'interpreters': {'sint32': 0x3a550, 'uint32': 0x61230, 'float32': 0x4f240},
         'resultValueOffset': 8},
}
SCALARS = {2556031774: 'sint32', 848563180: 'uint32', 3902764048: 'float32'}
INTEGER_EDGES = (0, 1, 2, 3, 0x7f, 0xffff, 0x10000, 0xffffff, 0x1000000,
                 0x1000001, 0x7ffffffe, 0x7fffffff, 0x80000000, 0x80000001,
                 0xfeffffff, 0xff000000, 0xff000001, 0xfffffffd, 0xfffffffe, 0xffffffff)
# Keep products finite. Rejecting nonfinite results is an intentional safety
# restriction of the owned model, not an assertion about the native VM.
FLOAT_EDGES = tuple(struct.unpack('<I', struct.pack('<f', x))[0] for x in
                    (0.0, -0.0, 1e-20, -1e-20, 0.1, -0.1, 1.0, -1.0,
                     16777215.0, 16777216.0, 16777218.0, -16777216.0, 1e18, -1e18))


def sample_values(rng, kind, count, sample, edge=False):
    if edge:
        choices = FLOAT_EDGES if kind == 'float32' else INTEGER_EDGES
        return [rng.choice(choices) for _ in range(count)]
    if sample < 3:
        value = ([0, 1, 0x7fffffff][sample] if kind != 'float32' else
                 struct.unpack('<I', struct.pack('<f', float(sample)))[0])
        return [value] * count
    if kind == 'float32':
        return [struct.unpack('<I', struct.pack('<f', rng.uniform(-100, 100)))[0]
                for _ in range(count)]
    return [rng.randint(-10000 if kind == 'sint32' else 0, 10000) & 0xffffffff
            for _ in range(count)]


class Oracle:
    SCRATCH = 0x40000000
    STACK = 0x50000000
    STOP = 0x60000000

    def __init__(self, pe):
        self.pe = pe
        build = BUILDS[pe.sha256]
        self.roots = build['interpreters']
        self.result_value_offset = build['resultValueOffset']
        self.uc = Uc(UC_ARCH_X86, UC_MODE_64)
        self.uc.mem_map(pe.base, (pe.image_size + 4095) & ~4095, UC_PROT_READ | UC_PROT_EXEC)
        self.uc.mem_write(pe.base, pe.data[:pe.headers_size])
        for s in pe.sections:
            self.uc.mem_write(pe.base + s['rva'], pe.data[s['raw']:s['raw'] + s['rawSize']])
        self.uc.mem_map(self.SCRATCH, 0x10000, UC_PROT_ALL)
        self.uc.mem_map(self.STACK, 0x10000, UC_PROT_ALL)
        self.uc.mem_map(self.STOP, 4096, UC_PROT_READ | UC_PROT_EXEC)

    def execute(self, kind, values, words):
        if len(values) > 1024 or len(words) > 1024:
            raise ValueError('oversized test vector')
        u = self.uc
        u.mem_write(self.SCRATCH, bytes(0x10000))
        u.mem_write(self.STACK, bytes(0x10000))
        u.mem_write(self.SCRATCH + 0x1000, struct.pack('<' + 'I' * len(values), *values))
        u.mem_write(self.SCRATCH + 0x3000, struct.pack('<' + 'I' * len(words), *words))
        u.mem_write(self.SCRATCH + 0x100, struct.pack('<QQ', self.SCRATCH + 0x1000, len(values)))
        u.mem_write(self.SCRATCH + 0x200, struct.pack('<QQ', self.SCRATCH + 0x3000, len(words)))
        rsp = self.STACK + 0xff08
        u.mem_write(rsp, struct.pack('<Q', self.STOP))
        for r, value in [(UC_X86_REG_RCX, self.SCRATCH), (UC_X86_REG_RDX, self.SCRATCH + 0x100),
                         (UC_X86_REG_R8, self.SCRATCH + 0x200), (UC_X86_REG_R9, 0),
                         (UC_X86_REG_RSP, rsp), (UC_X86_REG_MXCSR, 0x1f80)]:
            u.reg_write(r, value)
        u.emu_start(self.pe.base + self.roots[kind], self.STOP, timeout=100000, count=100000)
        if u.reg_read(UC_X86_REG_RIP) != self.STOP:
            raise ValueError('interpreter exceeded execution budget')
        status = u.mem_read(self.SCRATCH, 1)[0]
        result = struct.unpack('<I', u.mem_read(self.SCRATCH + self.result_value_offset, 4))[0]
        after = list(struct.unpack('<' + 'I' * len(values), u.mem_read(self.SCRATCH + 0x1000, len(values) * 4)))
        return dict(status=status, result=result, after=after)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', required=True)
    parser.add_argument('--resources', required=True)
    parser.add_argument('--out', required=True)
    parser.add_argument('--samples', type=int, default=32)
    parser.add_argument('--edge-samples', type=int, default=0,
                        help='additional finite boundary vectors for each nonempty program')
    parser.add_argument('--scale-samples', type=int, default=0,
                        help='additional integer boundary vectors for LoadRef/ScaleToNewMax programs')
    parser.add_argument('--root-samples', type=int, default=0,
                        help='complete descending root recomputations using original program results')
    args = parser.parse_args()
    if any(not 0 <= n <= 4096 for n in
           (args.samples, args.edge_samples, args.scale_samples, args.root_samples)):
        parser.error('sample counts must be in 0..4096')
    pe = PE(args.exe)
    oracle = Oracle(pe)
    resources = json.loads(Path(args.resources).read_text())
    roots = next(r['value']['attributes'] for r in resources['resources']
                 if r['qualifiedType'] == 'keen::AttributeContainerResource')
    rng = random.Random(20261009)
    vectors = []
    coverage = []
    for root in roots:
        kind = SCALARS[root['type']['value']]
        count = len(root['ids'])
        root_id = root['ids'][0]['value']
        coverage.append(dict(rootId=root_id, kind=kind, definitions=count,
                             programs=sum(bool(row['calculation']) for row in root['structure'])))
        for i, row in enumerate(root['structure']):
            words = row['calculation']
            if not words:
                continue
            scale = any(word >> 16 == 13 for word in words)
            total = args.samples + args.edge_samples + (args.scale_samples if scale else 0)
            for sample in range(total):
                values = sample_values(rng, kind, count, sample, edge=sample >= args.samples)
                native = oracle.execute(kind, values, words)
                vectors.append(dict(name=root['debugNames'][i], kind=kind, values=values,
                                    words=words, rootId=root_id, slot=i,
                                    case='edge' if sample >= args.samples else 'sample', **native))
        # The original signed adder starts at root.count-1, skips empty programs,
        # writes the returned scalar at that slot, then decrements the row index.
        # Each captured nonempty step is checked by the existing Rust oracle test.
        for sample in range(args.root_samples):
            values = sample_values(rng, kind, count, sample, edge=sample >= 3)
            for i in range(count - 1, -1, -1):
                words = root['structure'][i]['calculation']
                if not words:
                    continue
                before = list(values)
                native = oracle.execute(kind, before, words)
                vectors.append(dict(name=root['debugNames'][i], kind=kind, values=before,
                                    words=words, rootId=root_id, slot=i, case='root-walk',
                                    rootSample=sample, **native))
                if native['status']:
                    raise ValueError(f'original root recomputation failed: {root_id:#x}, slot {i}')
                values = list(native['after'])
                values[i] = native['result']
    output = Path(args.out)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(dict(executableSha256=pe.sha256, interpreters=oracle.roots,
        resultValueOffset=oracle.result_value_offset,
        coverage=dict(roots=coverage, definitions=sum(r['definitions'] for r in coverage),
                      programs=sum(r['programs'] for r in coverage),
                      rootSamples=args.root_samples,
                      rootWalkOrder='descending; empty programs leave the stored slot unchanged'),
        environment='Unicorn x86_64; isolated read-only PE; no host native execution', vectors=vectors)))
    print(json.dumps(dict(executableSha256=pe.sha256, roots=len(coverage),
                          definitions=sum(r['definitions'] for r in coverage),
                          programs=sum(r['programs'] for r in coverage),
                          samples=len(vectors), nativeStatuses=sorted(set(v['status'] for v in vectors)))))


if __name__ == '__main__':
    main()
