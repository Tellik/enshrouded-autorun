"""Verify the exact-image addresses and guards of world operations."""

def verify_world_operation_origins(pe, profile):
    world = []
    for name, operation in profile['worldOperations'].items():
        actor_context = operation['abi'] == 'actor-world-context'
        address = operation.get('functionRva', operation.get('globalRva', operation.get('guardRva', 0) if actor_context else 0))
        guards = bytes(operation.get('guardBytes', []))
        try:
            guard_matches = pe.read(operation.get('guardRva', 0), len(guards)) == guards if guards else None
        except ValueError:
            guard_matches = False
        if operation.get('validated', True):
            assert 0 < address < pe.image_size, f'{name}: address outside target image'
            assert guard_matches is not False, f'{name}: code guard mismatch'
        world.append(dict(id=name, address_in_image=0 < address < pe.image_size, guard_matches=guard_matches,
                          native_call_executed=False, context_source='actor-frame' if actor_context else operation['context'],
                          abi=operation['abi'], context=operation['context']))
    return world
