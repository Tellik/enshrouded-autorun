"""Index captured procedure ranges, calls, and reflected type references."""
import bisect
from collections import defaultdict

class FunctionIndex:
    def __init__(self, reflection, report, components=None):
        self.groups = {f['beginRva']: f for f in report.get('functionGroups', [])}
        self.ranges = {f['beginRva']: f for f in report.get('functions', [])}
        self.starts = sorted(self.ranges)
        types = {t['index']: t['qualifiedName'] for t in reflection['types']}
        self.features = {}
        self.postings = {key: defaultdict(set) for key in ('digest', 'fragment', 'descriptor', 'metadata', 'hash', 'callback')}
        self.structures = {}
        self.calls = defaultdict(set)
        self.callers = defaultdict(set)
        for root, group in self.groups.items():
            features = dict(descriptor=set(), metadata=set(), hash=set(), callback=set())
            fragments, branches, hash_sites, external_targets = [], [], set(), {}
            for descriptor in group.get('engineDescriptors', []):
                features['descriptor'].add(descriptor['name'])
            for ordinal, rva in enumerate(group['ranges']):
                native = self.ranges[rva]
                length = native['endRva'] - rva
                digest = native.get('relocatedCodeSha256')
                fragments.append((rva - root, length, digest))
                if digest and length >= 16:
                    self.postings['fragment'][length, digest].add(root)
                features['metadata'].update(types[r['typeIndex']] for r in native.get('metadataReferences', [])
                                            if r['typeIndex'] in types)
                for immediate in native.get('hashCandidates', []):
                    # A colliding integer is not a resolved type reference.
                    candidates = immediate['candidates']
                    names = {types[c['typeIndex']] for c in candidates if c['typeIndex'] in types}
                    if len(names) == 1:
                        features['hash'].update((next(iter(names)), c['hashKind']) for c in candidates)
                        hash_sites.update((ordinal, immediate['instructionRva'] - rva, next(iter(names)), c['hashKind'])
                                          for c in candidates)
                for branch in native.get('directBranches', []):
                    target = branch['targetRva']
                    target_range = self.containing_range(target)
                    if target_range and target_range.get('primaryBeginRva') == root:
                        destination = ('internal', target - root)
                    else:
                        # Equality classes retain repeated callees without copying addresses.
                        destination = ('external', external_targets.setdefault(target, len(external_targets)))
                    branches.append((ordinal, branch['instructionRva'] - rva, branch['kind'], *destination))
                    if branch['kind'] == 'call':
                        callee = self.containing(branch['targetRva'])
                        if callee is not None and callee != root:
                            self.calls[root].add(callee)
                            self.callers[callee].add(root)
            self.features[root] = features
            self.structures[root] = dict(fragments=fragments, branches=tuple(branches), hashSites=hash_sites)
            for kind, values in features.items():
                for value in values:
                    self.postings[kind][value].add(root)
            digest = group.get('relocatedCodeSha256')
            if digest and group['codeBytes'] >= 32:
                self.postings['digest'][digest].add(root)
        self.callback_coverage = dict(stableRegistries=0, acceptedCallbacks=0, callbacksWithoutProcedureEntry=0)
        for registry in (components or {}).get('registries', []):
            if (not registry.get('stableReadback') or not isinstance(registry.get('layoutVersion'), int)
                    or not isinstance(registry.get('recordStride'), int)):
                continue
            self.callback_coverage['stableRegistries'] += 1
            for row in registry.get('rows', []):
                for callback in row.get('callbacks', []):
                    root = self.containing(callback.get('functionRva'))
                    # A pointer into a compiler range is not automatically its procedure entry.
                    if root is None or callback['functionRva'] != root:
                        self.callback_coverage['callbacksWithoutProcedureEntry'] += 1
                        continue
                    token = (registry['layoutVersion'], registry.get('recordStride'), row['qualifiedName'],
                             callback['origin'], callback['slotOffset'])
                    self.features[root]['callback'].add(token)
                    self.postings['callback'][token].add(root)
                    self.callback_coverage['acceptedCallbacks'] += 1

    def containing_range(self, rva):
        if not isinstance(rva, int):
            return None
        position = bisect.bisect_right(self.starts, rva) - 1
        if position >= 0:
            native = self.ranges[self.starts[position]]
            if rva < native['endRva']:
                return native
        return None

    def containing(self, rva):
        native = self.containing_range(rva)
        root = native.get('primaryBeginRva') if native else None
        return root if root in self.groups else None
