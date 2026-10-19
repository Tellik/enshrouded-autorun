import importlib.util
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'diagnostics-jobs'))
from game_inspection.analyze import find_runtime_type as runtime_type
from game_inspection.analyze.trace_function_anchors import trace_anchors
from game_inspection.inspect.read_pe_image import PE
from game_inspection.inspect.extract_executable_reflection import extract, fnv
from game_inspection.inspect.discover_executable_functions import discover, mask_rip_displacement
from game_inspection.inspect.read_system_descriptors import discover as discover_systems, inspect as inspect_systems


def fixture():
    """Independent tiny PE with two type descriptors and duplicate registry handles."""
    data = bytearray(0x1200)
    data[:2] = b'MZ'
    struct.pack_into('<I', data, 0x3c, 0x80)
    data[0x80:0x84] = b'PE\0\0'
    struct.pack_into('<HHIIIHH', data, 0x84, 0x8664, 1, 123, 0, 0, 240, 0)
    opt = 0x98
    struct.pack_into('<H', data, opt, 0x20b)
    struct.pack_into('<Q', data, opt + 24, 0x140000000)
    struct.pack_into('<II', data, opt + 56, 0x2000, 0x200)
    struct.pack_into('<8sIIIIIIHHI', data, opt + 240, b'.rdata', 0x1000, 0x1000,
                     0x1000, 0x200, 0, 0, 0, 0, 0x40000040)
    for name, text_rva, metadata, primitive, size in [
        (b'BlobString', 0x1001, 0x1100, 25, 16), (b'uint32', 0x1021, 0x1190, 6, 4)
    ]:
        raw = text_rva - 0xe00
        data[raw:raw + len(name)] = name
        struct.pack_into('<8Q', data, metadata - 0xe00,
                         0x140000000 + text_rva, len(name), 0x140000000 + text_rva, len(name),
                         0x140000000 + text_rva, len(name), 0, 0)
        struct.pack_into('<IHHIBB2xII', data, metadata - 0xe00 + 0x40,
                         size, 4, 4, 0, primitive, 0, fnv(name.decode()), 777)
    struct.pack_into('<QQ', data, 0x500, 0x140001100, 0x140001190)
    for raw in (0x540, 0x550):
        struct.pack_into('<QQ', data, raw, 0x140001300, 2)
    return data


class DiscoveryTest(unittest.TestCase):
    def load(self, data):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / 'game.exe'
        path.write_bytes(data)
        return PE(path)

    def test_extracts_registry_without_profile_and_preserves_collisions(self):
        result = extract(self.load(fixture()))
        self.assertEqual(result['headerRvas'], [0x1340, 0x1350])
        self.assertEqual(result['count'], 2)
        self.assertEqual(result['types'][1]['qualifiedName'], 'uint32')
        self.assertEqual(result['types'][1]['metadataRva'], 0x1190)
        self.assertEqual(result['hashCollisions']['internalHash'], {'777': [0, 1]})

    def test_malformed_type_reference_fails_instead_of_dropping_a_field(self):
        data = fixture()
        struct.pack_into('<Q', data, 0x300 + 0x38, 0x140001555)
        with self.assertRaisesRegex(ValueError, 'unresolved reflected type'):
            extract(self.load(data))

    def test_truncated_registry_cannot_be_reported_as_complete(self):
        data = fixture()
        for raw in (0x548, 0x558):
            struct.pack_into('<Q', data, raw, 100000)
        with self.assertRaisesRegex(ValueError, 'reflection registry'):
            extract(self.load(data))

    def test_file_bounds_and_preferred_base(self):
        pe = self.load(fixture())
        self.assertEqual(pe.rva(0x140001190), 0x1190)
        with self.assertRaises(ValueError):
            pe.read(0x1fff, 8)
        with self.assertRaises(ValueError):
            pe.rva(0x180001190)
        with self.assertRaises(ValueError):
            self.load(b'MZ')

    def test_hash_is_fnv1a_not_python_hash(self):
        self.assertEqual(fnv(''), 0x811c9dc5)
        self.assertEqual(fnv('hello'), 0x4f9f2cab)

    def test_system_descriptor_keeps_sixth_span_without_inventing_access_roles(self):
        data = fixture()
        name = b'fixture_system'
        dependency = b'InertialFrame'
        data[0x900:0x900 + len(name)] = name
        data[0x940:0x940 + len(dependency)] = dependency
        struct.pack_into('<QQQ', data, 0x700, 0x140001700, len(name), 0x140002000)
        struct.pack_into('<QQ', data, 0x700 + 104, 0x140001780, 1)
        struct.pack_into('<QQ', data, 0x980, 0x140001740, len(dependency))
        functions = dict(functionGroups=[dict(beginRva=0x2000)],
                         codePointers=[dict(targetRva=0x2000, slotRva=0x1510)])
        systems = discover_systems(self.load(data), functions)
        self.assertEqual(len(systems), 1)
        slots = systems[0]['dependencySlots']
        self.assertEqual([slot['offset'] for slot in slots], [24, 40, 56, 72, 88, 104])
        self.assertEqual(slots[5]['entries'], ['InertialFrame'])
        self.assertNotIn('access_mode', slots[5])

    def system_inspection_fixture(self):
        data = fixture()
        data.extend(b'\0' * 0x200)
        struct.pack_into('<H', data, 0x86, 2)
        struct.pack_into('<I', data, 0x98 + 56, 0x3000)
        struct.pack_into('<8sIIIIIIHHI', data, 0x98 + 280, b'.text', 0x200, 0x2000,
                         0x200, 0x1200, 0, 0, 0, 0, 0x60000020)
        struct.pack_into('<II', data, 0x98 + 136, 0x1380, 36)
        for ordinal, (begin, end, unwind) in enumerate(((0x2000, 0x2008, 0x13b0),
                (0x2010, 0x2011, 0x13c0), (0x2020, 0x2021, 0x13d0))):
            struct.pack_into('<III', data, 0x580 + ordinal * 12, begin, end, unwind)
            data[unwind - 0xe00] = 1
        # One direct call, one indirect call and an incomplete instruction.
        data[0x1200:0x1208] = b'\xe8' + struct.pack('<i', 0x2020 - 0x2005) + b'\xff\xd0\x0f'
        data[0x1210] = data[0x1220] = 0xc3
        for descriptor, name, name_raw, function in ((0x700, b'fixture_system', 0x900, 0x2000),
                (0x800, b'fixture_other', 0xb00, 0x2010)):
            data[name_raw:name_raw + len(name)] = name
            struct.pack_into('<QQQ', data, descriptor, 0x140000000 + name_raw + 0xe00, len(name),
                             0x140000000 + function)
            struct.pack_into('<QQ', data, descriptor + 24, 0x140001940, 1)
        struct.pack_into('<QQ', data, 0xb40, 0x140001021, 6)
        for offset, span_raw, text_raw, name in ((0x98, 0x9c0, 0xa40, b'ecs::ReadService'),
                (0xb0, 0x9e0, 0xa80, b'VoxelWorld')):
            data[text_raw:text_raw + len(name)] = name
            struct.pack_into('<QQ', data, 0x700 + offset, 0x140000000 + span_raw + 0xe00, 1)
            struct.pack_into('<QQ', data, span_raw, 0x140000000 + text_raw + 0xe00, len(name))
        pe = self.load(data)
        functions = dict(image=pe.identity(), functionGroups=[dict(beginRva=rva) for rva in (0x2000, 0x2010, 0x2020)],
                         codePointers=[dict(targetRva=0x2000, slotRva=0x1510), dict(targetRva=0x2010, slotRva=0x1610)])
        return pe, dict(image=pe.identity(), **extract(pe)), functions

    @unittest.skipUnless(importlib.util.find_spec('capstone'), 'optional disassembler dependency')
    def test_system_inspection_preserves_named_services_calls_and_decoding_gaps(self):
        pe, reflection, functions = self.system_inspection_fixture()
        result = inspect_systems(pe, reflection, functions, ['fixture_system'], [('uint32', ())])
        self.assertEqual(result['originalSystemCount'], 2)
        self.assertEqual([row['name'] for row in result['systems']], ['fixture_system'])
        row = result['systems'][0]
        self.assertEqual(row['services'], [dict(name='ecs::ReadService', access='read', ordinal=0),
                                           dict(name='VoxelWorld', access='write', ordinal=1)])
        self.assertEqual(row['directCalls'], [dict(instructionRva=0x2000, targetRva=0x2020,
            procedureRva=0x2020, bytes='e81b000000')])
        self.assertEqual(row['codeRanges'], [dict(beginRva=0x2000, endRva=0x2008, decodedEndRva=0x2007,
            decodingComplete=False, undecodedBytes=1)])
        self.assertFalse(row['instructionDecodingComplete'])
        self.assertEqual(result['reflectedLayouts'][0]['layout']['qualifiedName'], 'uint32')
        self.assertFalse(result['callable'] or result['nativeAbiProven'] or result['processMemoryRead'])

    @unittest.skipUnless(importlib.util.find_spec('capstone'), 'optional disassembler dependency')
    def test_system_inspection_rejects_partial_names_mixed_images_and_stale_pointers(self):
        pe, reflection, functions = self.system_inspection_fixture()
        with self.assertRaisesRegex(ValueError, 'system names were not found'):
            inspect_systems(pe, reflection, functions, ['fixture'])
        with self.assertRaisesRegex(ValueError, 'exact executable image'):
            inspect_systems(pe, dict(reflection, image={}), functions)
        functions['codePointers'][0]['targetRva'] += 1
        with self.assertRaisesRegex(ValueError, 'code pointer differs'):
            inspect_systems(pe, reflection, functions)

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_instruction_xrefs_and_address_normalization(self):
        data = fixture()
        data.extend(b'\0' * 0x200)
        struct.pack_into('<H', data, 0x86, 2)
        struct.pack_into('<I', data, 0x98 + 56, 0x3000)
        struct.pack_into('<8sIIIIIIHHI', data, 0x98 + 240 + 40, b'.text', 0x200, 0x2000,
                         0x200, 0x1200, 0, 0, 0, 0, 0x60000020)
        struct.pack_into('<II', data, 0x98 + 112 + 3 * 8, 0x1380, 12)
        struct.pack_into('<III', data, 0x580, 0x2000, 0x2012, 0x13a0)
        data[0x5a0] = 1
        # lea rcx,[rip -> descriptor 0]; mov eax,<qualified hash>; call 0x2100; ret
        code = b'\x48\x8d\x0d' + struct.pack('<i', 0x1100 - 0x2007)
        code += b'\xb8' + struct.pack('<I', fnv('uint32'))
        code += b'\xe8' + struct.pack('<i', 0x2100 - 0x2011) + b'\xc3'
        data[0x1200:0x1212] = code
        pe = self.load(data)
        result = discover(pe, extract(pe), progress=lambda _: None)
        function = result['functions'][0]
        self.assertEqual(function['metadataReferences'][0]['typeIndex'], 0)
        self.assertEqual(function['hashCandidates'][0]['value'], fnv('uint32'))
        self.assertEqual(function['directBranches'][0]['targetRva'], 0x2100)
        self.assertFalse(function['callable'])
        struct.pack_into('<i', data, 0x120d, 0x2200 - 0x2011)
        other = self.load(data)
        normalized = discover(other, extract(other), progress=lambda _: None)['functions'][0]
        self.assertEqual(function['relocatedCodeSha256'], normalized['relocatedCodeSha256'])
        self.assertNotEqual(function['codeSha256'], normalized['codeSha256'])

    @unittest.skipUnless(importlib.util.find_spec('capstone'), 'optional disassembler dependency')
    def test_prefixed_sse_displacement_masks_all_four_address_bytes(self):
        import capstone as cs
        engine = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
        engine.detail = True
        normalized = []
        for raw in ('66 0f 6f 0d da 8e ec 00', '66 0f 6f 0d da f3 98 00'):
            code = bytearray.fromhex(raw)
            instruction = next(engine.disasm(bytes(code), 0x2000))
            mask_rip_displacement(code, instruction, 0x2000, instruction.disp)
            normalized.append(code)
        self.assertEqual(normalized[0], normalized[1])

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_chained_unwind_fragments_have_one_primary_function(self):
        data = fixture()
        data.extend(b'\0' * 0x200)
        struct.pack_into('<H', data, 0x86, 2)
        struct.pack_into('<I', data, 0x98 + 56, 0x3000)
        struct.pack_into('<8sIIIIIIHHI', data, 0x98 + 280, b'.text', 0x200, 0x2000,
                         0x200, 0x1200, 0, 0, 0, 0, 0x60000020)
        struct.pack_into('<II', data, 0x98 + 136, 0x1380, 24)
        struct.pack_into('<6I', data, 0x580, 0x2000, 0x2004, 0x13a0, 0x2004, 0x2010, 0x13b0)
        data[0x5a0] = 1
        data[0x5b0] = 0x21
        struct.pack_into('<III', data, 0x5b4, 0x2000, 0x2004, 0x13a0)
        data[0x1200:0x1210] = b'\x90' * 15 + b'\xc3'
        pe = self.load(data)
        result = discover(pe, extract(pe), progress=lambda _: None)
        self.assertEqual(len(result['functionGroups']), 1)
        self.assertEqual(result['functionGroups'][0]['ranges'], [0x2000, 0x2004])
        self.assertEqual(result['functionGroups'][0]['codeBytes'], 16)
        struct.pack_into('<I', data, 0x5bc, 0x13b0)
        with self.assertRaisesRegex(ValueError, 'cyclic'):
            self.load(data)

    def test_lookup_retains_every_collision_and_rejects_mixed_builds(self):
        pe = self.load(fixture())
        directory = pe.path.parent
        (directory / 'reflection.json').write_text(json.dumps(dict(image=pe.identity(), **extract(pe))))
        result = runtime_type.lookup(directory, '777', 'internalHash')
        self.assertEqual(result['matchCount'], 2)
        (directory / 'functions.json').write_text(json.dumps(dict(image={'sha256': 'wrong'})))
        with self.assertRaisesRegex(ValueError, 'mixes executable identities'):
            runtime_type.checked_reports(directory)

    def anchor_fixture(self):
        data = fixture()
        data.extend(b'\0' * 0x200)
        struct.pack_into('<H', data, 0x86, 2)
        struct.pack_into('<I', data, 0x98 + 56, 0x3000)
        struct.pack_into('<8sIIIIIIHHI', data, 0x98 + 280, b'.text', 0x200, 0x2000,
                         0x200, 0x1200, 0, 0, 0, 0, 0x60000020)
        struct.pack_into('<II', data, 0x98 + 136, 0x1380, 48)
        for ordinal, (begin, end, unwind) in enumerate(((0x2000, 0x2004, 0x13c0),
                (0x2004, 0x2020, 0x13d0), (0x2020, 0x2030, 0x13e0), (0x2030, 0x2040, 0x13f0))):
            struct.pack_into('<III', data, 0x580 + ordinal * 12, begin, end, unwind)
            data[unwind - 0xe00] = 1
        data[0x5d0] = 0x21
        struct.pack_into('<III', data, 0x5d4, 0x2000, 0x2004, 0x13c0)
        data[0x1200:0x1240] = b'\x90' * 0x40
        for offset in (0x1204, 0x1230):
            data[offset:offset + 7] = bytes.fromhex('b8 10 20 30 40 90 c3')
        data[0x1220:0x1226] = b'\xe8' + struct.pack('<i', 0x2004 - 0x2025) + b'\xc3'
        pe = self.load(data)
        reflection = dict(image=pe.identity(), **extract(pe))
        functions = dict(image=pe.identity(), **discover(pe, reflection, progress=lambda _: None))
        group = next(g for g in functions['functionGroups'] if g['beginRva'] == 0x2020)
        group['engineDescriptors'] = [dict(name='fixture_caller', dependencySlots=[dict(offset=56, entries=['uint32'])])]
        return pe, reflection, functions

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_anchor_trace_rescans_retains_ambiguity_fragments_and_exact_call_edges(self):
        pe, reflection, functions = self.anchor_fixture()
        scan = dict(image=pe.identity(), features=[dict(name='effect', signature='B8 10 20 30 40 90 C3', matches=[1])])
        result = trace_anchors(pe, reflection, functions, scan, dict(entries=[dict(name='data handler')]))
        row = result['anchors'][0]
        self.assertEqual(row['matches'], [0x2004, 0x2030])
        self.assertEqual(row['status'], 'ambiguous-byte-matches')
        self.assertFalse(row['reportedMatchesAgree'])
        self.assertEqual(len(result['procedures'][str(0x2000)]['ranges']), 2)
        named = row['sites'][0]['callerTrace']['nearestNamedCallers'][0]
        self.assertEqual(named['names'], ['fixture_caller'])
        self.assertEqual(named['oneShortestCallPath'][0]['instructionRva'], 0x2020)
        self.assertEqual(named['oneShortestCallPath'][0]['targetRva'], 0x2004)
        self.assertEqual(result['callerFunctions'][str(0x2020)]['descriptorTypeReferences'][0]['reflectionCandidates'], [1])
        self.assertEqual([t['index'] for t in result['reflectedTypes']], [1])
        self.assertEqual(result['anchors'][1]['status'], 'unresolved-data-mechanism')
        self.assertFalse(result['nativeAbiProven'])

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_anchor_trace_rejects_cross_image_and_excessive_depth(self):
        pe, reflection, functions = self.anchor_fixture()
        scan = dict(image=dict(pe.identity(), sha256='another image'), features=[])
        with self.assertRaisesRegex(ValueError, 'exact executable image'):
            trace_anchors(pe, reflection, functions, scan)
        with self.assertRaisesRegex(ValueError, 'zero and six'):
            trace_anchors(pe, reflection, functions, dict(image=pe.identity(), features=[]), maximum_depth=7)

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_anchor_trace_marks_interior_byte_hits_and_invalid_signatures(self):
        pe, reflection, functions = self.anchor_fixture()
        scan = dict(image=pe.identity(), features=[dict(name='interior', signature='10 20 30 40'),
                                                  dict(name='invalid', signature='GG 00 11 22')])
        result = trace_anchors(pe, reflection, functions, scan,
                              dict(entries=[dict(name='interior', siteOffset=-1)]))
        self.assertTrue(result['anchors'][0]['sites'])
        self.assertTrue(all(not s['instructionBoundaryVerified'] for s in result['anchors'][0]['sites']))
        self.assertTrue(all(s['catalogSite']['instructionBoundaryVerified'] for s in result['anchors'][0]['sites']))
        self.assertEqual(result['anchors'][1]['status'], 'unresolved-invalid-signature')

    @unittest.skipUnless(importlib.util.find_spec('capstone') and importlib.util.find_spec('numpy'), 'optional disassembler dependencies')
    def test_anchor_trace_simple_patch_uses_captured_offset_instead_of_status(self):
        pe, reflection, functions = self.anchor_fixture()
        scan = dict(image=pe.identity(), features=[dict(name='simple', signature='B8 10 20 30 40 90 C3')])
        catalog = dict(entries=[dict(name='simple', kind20=0, value10=5000, siteOffset130=5,
                                     siteOffset178=0, replacementBytes=[0x90])])
        result = trace_anchors(pe, reflection, functions, scan, catalog)
        sites = result['anchors'][0]['sites']
        self.assertEqual([s['catalogSite']['rva'] for s in sites], [0x2009, 0x2035])
        self.assertTrue(all(s['catalogSite']['instructionBoundaryVerified'] for s in sites))
        self.assertEqual(result['anchors'][0]['catalogMetadata']['replacementBytes'], [0x90])


if __name__ == '__main__':
    unittest.main()
