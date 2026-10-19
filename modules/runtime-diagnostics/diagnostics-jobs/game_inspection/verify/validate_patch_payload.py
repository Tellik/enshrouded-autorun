"""Validate the actual patch payload and its inline references."""
import struct

def validate_payload(spec):
    payload = bytes(spec['payload'])
    if spec['kind'] == 'bytes':
        assert len(payload) == spec['overwriteBytes']
    else:
        offset = spec['returnRel32Offset']
        assert payload[offset - 1] == 0xe9 and offset + 4 <= len(payload)
    for ref in spec.get('inlineReferences', []):
        offset, next_, data = (ref[k] for k in ['displacementOffset', 'nextInstructionOffset', 'dataOffset'])
        assert offset + 4 == next_ and data >= spec['returnRel32Offset'] + 4
        assert 0 < ref['dataSize'] <= len(payload) - data
        assert next_ + struct.unpack_from('<i', payload, offset)[0] == data, 'RIP-relative load does not address its inline constant'
        assert abs(struct.unpack_from('<f', payload, data)[0] - (-1.57)) < 1e-6
    if spec['modifier']['id'] == 'preserve_health_on_fall':
        assert payload[:4] == b'\x90' * 4, 'Health preservation must suppress the store, not add the computed value'
        assert payload[4:7] == bytes.fromhex('48 8b cb')
    return True
