"""Rank cross-build native procedure candidates without authorizing engine calls."""
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


def unique_anchors(source, target):
    """Use reciprocal unique code/descriptor/registration evidence, retaining conflicts."""
    claims, reverse = defaultdict(set), defaultdict(set)
    methods = defaultdict(set)
    for kind in ('digest', 'descriptor', 'callback'):
        for value, left in source.postings[kind].items():
            right = target.postings[kind].get(value, set())
            if len(left) == len(right) == 1:
                a, b = next(iter(left)), next(iter(right))
                claims[a].add(b)
                reverse[b].add(a)
                methods[a, b].add(kind)
    anchors = {a: next(iter(bs)) for a, bs in claims.items() if len(bs) == 1
               and len(reverse[next(iter(bs))]) == 1}
    return anchors, methods, len(claims) - len(anchors)


def operations(audit, profile=None):
    records = {row['id']: dict(row) for row in (audit or {}).get('operations', [])}
    if profile is None:
        if audit and not audit.get('imageMatches', False):
            for row in records.values():
                # These are offsets copied from the audited profile, not discovered code.
                row.pop('functionRva', None)
                row.pop('sourceSiteRva', None)
        return list(records.values())
    # An explicit current source profile can add operations created after the capture.
    selected = {}
    for name in profile.get('hooks', {}):
        key = 'hook.' + name
        selected[key] = dict(records.get(key, {}), id=key, kind='hook')
    for name, value in profile.get('worldOperations', {}).items():
        selected[name] = dict(records.get(name, {}), id=name, kind='world-operation',
                              functionRva=value.get('functionRva'), guardRva=value.get('guardRva'),
                              sourceGuardBytes=value.get('guardBytes', []))
    for name, value in profile.get('runtimePatches', {}).items():
        selected[name] = dict(records.get(name, {}), id=name, kind='patch')
        extent = value.get('function', {})
        if 'beginRva' in extent and 'targetOffset' in extent:
            selected[name]['sourceSiteRva'] = extent['beginRva'] + extent['targetOffset']
    return list(selected.values())


def source_site(operation):
    if operation.get('sourceSiteRva') is not None:
        return operation['sourceSiteRva'], 'exact-source-profile-patch-site'
    if operation.get('functionRva') is not None:
        return operation['functionRva'], 'source-profile-function-rva'
    matches = operation.get('matches', [])
    if len(matches) == 1:
        return matches[0], 'unique-source-audit-byte-match'
    return None, 'missing-or-ambiguous-source-anchor'


def structural_evidence(source, root, target, candidate):
    """Compare only structure actually preserved in the capture, never inferred bytes."""
    left, right = source.structures[root], target.structures[candidate]
    fragments, fragment_score, unique_fragments, used_targets = [], 0, 0, set()
    for source_ordinal, (offset, length, digest) in enumerate(left['fragments']):
        if not digest or length < 16:
            continue
        for target_ordinal, (other_offset, other_length, other_digest) in enumerate(right['fragments']):
            if target_ordinal in used_targets or length != other_length or digest != other_digest:
                continue
            used_targets.add(target_ordinal)
            source_owners = len(source.postings['fragment'][length, digest])
            target_owners = len(target.postings['fragment'][length, digest])
            unique = source_owners == target_owners == 1 and length >= 32
            unique_fragments += unique
            fragment_score += 35 if unique else 12 if length >= 32 else 3
            fragments.append(dict(sourceRangeRva=root + offset, targetRangeRva=candidate + other_offset,
                sourceRangeOrdinal=source_ordinal, targetRangeOrdinal=target_ordinal, bytes=length,
                sourceOwnerCount=source_owners, targetOwnerCount=target_owners,
                reciprocalUniqueFragment=unique, normalizedSha256=digest))
            break
    branch_equal = bool(left['branches']) and left['branches'] == right['branches']
    matching_hash_sites = sorted(left['hashSites'] & right['hashSites'])
    evidence = []
    if fragments:
        evidence.append(dict(kind='partial-normalized-unwind-fragments', fragments=fragments,
                             establishesWholeProcedureEquality=False))
    if branch_equal:
        evidence.append(dict(kind='relative-direct-branch-structure',
            branches=[dict(rangeOrdinal=ordinal, instructionOffset=offset, kind=kind,
                           targetKind=destination, targetOffsetOrEqualityClass=value)
                      for ordinal, offset, kind, destination, value in left['branches']],
            capturesConditionalBranches=False, nativeArgumentsProven=False))
    if matching_hash_sites:
        evidence.append(dict(kind='same-relative-type-hash-sites',
            sites=[dict(rangeOrdinal=ordinal, instructionOffset=offset, name=name, hashKind=kind)
                   for ordinal, offset, name, kind in matching_hash_sites], semanticsProven=False))
    score = min(fragment_score, 60) + (24 if len(left['branches']) >= 2 else 6) * branch_equal
    score += 6 * min(len(matching_hash_sites), 3)
    return dict(evidence=evidence, score=score, uniqueFragments=unique_fragments,
                matchingFragmentBytes=sum(row['bytes'] for row in fragments),
                branchStructureEqual=branch_equal, branchCount=len(left['branches']),
                matchingHashSiteCount=len(matching_hash_sites))


def rank_operation(operation, source, target, anchors, anchor_methods, target_sites, verified_profile_guard=False):
    site, anchor_source = source_site(operation)
    root = source.containing(site)
    result = dict(id=operation['id'], kind=operation.get('kind'), sourceRva=site,
                  sourceAnchor=anchor_source, sourceFunctionRva=root,
                  targetRvas=[], callGraphCandidates=[], rankedCandidates=[],
                  method='cross-build-native-evidence', status='unresolved',
                  semanticStatus='unverified', callable=False, patchSiteResolved=False)
    if root is None:
        result['unresolvedReason'] = ('source-anchor-not-in-compiler-described-procedure' if site is not None
                                      else 'missing-or-ambiguous-source-anchor')
        if verified_profile_guard and site is not None and target_sites:
            result['targetSiteCandidates'] = [dict(targetRva=rva, scope='byte-site-without-known-function-extent',
                evidence='exact-source-profile-guard-and-target-scan-signature-and-original',
                sourceGuardBytes=operation['sourceGuardBytes'], nativeArgumentsProven=False,
                semanticStatus='unverified', callable=False) for rva in sorted(set(target_sites))]
            if len(result['targetSiteCandidates']) == 1:
                result.update(status='unique-byte-site-candidate', candidateSiteRva=target_sites[0],
                              unresolvedReason='procedure-extent-and-native-abi-not-established')
            else:
                result.update(status='ambiguous-byte-site-candidates',
                              unresolvedReason='multiple-exact-guard-sites-and-unknown-procedure-extents')
        return result
    group = source.groups[root]
    features = source.features[root]
    result.update(sourceCodeBytes=group['codeBytes'], sourceRanges=group['ranges'],
                  sourceOffsetWithinFunction=site - root,
                  sourceEngineDescriptors=sorted(features['descriptor']))
    digest = group.get('relocatedCodeSha256')
    exact = target.postings['digest'].get(digest, set()) if digest and group['codeBytes'] >= 32 else set()
    result['targetRvas'] = sorted(exact)
    pool = set(exact)
    for kind in ('descriptor', 'metadata', 'hash', 'callback'):
        for value in features[kind]:
            owners = target.postings[kind].get(value, set())
            # Common primitive types and hashes must not turn this into an all-pairs scan.
            if kind in ('descriptor', 'callback') or len(owners) <= 64:
                pool.update(owners)
    for _, length, fragment in source.structures[root]['fragments']:
        owners = target.postings['fragment'].get((length, fragment), set())
        if fragment and length >= 16 and len(owners) <= 64:
            pool.update(owners)
    mapped_callees = {anchors[rva] for rva in source.calls[root] if rva in anchors}
    mapped_callers = {anchors[rva] for rva in source.callers[root] if rva in anchors}
    for callee in mapped_callees:
        pool.update(target.callers[callee])
    for caller in mapped_callers:
        pool.update(target.calls[caller])
    matched_sites = defaultdict(list)
    for target_site in target_sites:
        candidate = target.containing(target_site)
        if candidate is not None:
            pool.add(candidate)
            matched_sites[candidate].append(target_site)
    if root in anchors:
        pool.add(anchors[root])
    ranked = []
    for candidate in pool:
        other = target.features[candidate]
        descriptors = sorted(features['descriptor'] & other['descriptor'])
        metadata = sorted(features['metadata'] & other['metadata'])
        hashes = sorted(features['hash'] & other['hash'])
        callbacks = sorted(features['callback'] & other['callback'])
        structure = structural_evidence(source, root, target, candidate)
        callees = sorted(mapped_callees & target.calls[candidate])
        callers = sorted(mapped_callers & target.callers[candidate])
        exact_code = candidate in exact
        descriptor_conflict = bool(features['descriptor'] and other['descriptor'] and not descriptors)
        score = (200 * (anchors.get(root) == candidate) + 100 * exact_code
                 + 80 * min(len(descriptors), 2) + 12 * min(len(callees), 8)
                 + 15 * min(len(callers), 4) + 10 * min(len(metadata), 3) + 2 * min(len(hashes), 3)
                 + 80 * min(len(callbacks), 2) + structure['score']
                 + 8 * bool(matched_sites[candidate]) - 80 * descriptor_conflict)
        evidence = list(structure['evidence'])
        if exact_code:
            evidence.append(dict(kind='address-masked-code-equality', bytes=group['codeBytes']))
        if descriptors:
            evidence.append(dict(kind='engine-descriptor-name', names=descriptors))
        if callbacks:
            evidence.append(dict(kind='stable-component-callback-owner-and-slot',
                owners=[dict(layoutVersion=version, recordStride=stride, qualifiedName=name,
                             origin=origin, slotOffset=offset) for version, stride, name, origin, offset in callbacks],
                nativeAbiProven=False))
        if metadata:
            evidence.append(dict(kind='reflected-metadata-reference', names=metadata))
        if hashes:
            evidence.append(dict(kind='noncolliding-type-hash-immediate', values=[dict(name=n, hashKind=k) for n, k in hashes],
                                 semanticsProven=False))
        if callees or callers:
            evidence.append(dict(kind='reciprocal-unique-code-descriptor-or-callback-neighbors', callees=callees, callers=callers))
        if matched_sites[candidate]:
            evidence.append(dict(kind='target-byte-pattern-match', siteRvas=sorted(matched_sites[candidate])))
        if anchors.get(root) == candidate:
            evidence.append(dict(kind='reciprocal-unique-anchor', methods=sorted(anchor_methods[root, candidate])))
        # Size only breaks ties. It is not evidence for identity or native arguments.
        difference = abs(target.groups[candidate]['codeBytes'] - group['codeBytes'])
        ranked.append(dict(targetRva=candidate, score=score, codeBytes=target.groups[candidate]['codeBytes'],
                           codeSizeDifference=difference, matchedCallees=len(callees), matchedCallers=len(callers),
                           exactNormalizedCode=exact_code, engineDescriptorNames=descriptors,
                           matchingFragmentBytes=structure['matchingFragmentBytes'],
                           reciprocalUniqueFragments=structure['uniqueFragments'],
                           branchStructureEqual=structure['branchStructureEqual'],
                           directBranchCount=structure['branchCount'],
                           matchingHashSiteCount=structure['matchingHashSiteCount'],
                           descriptorConflict=descriptor_conflict, evidence=evidence,
                           semanticStatus='unverified', callable=False))
    ranked.sort(key=lambda row: (-row['score'], row['codeSizeDifference'], row['targetRva']))
    result.update(candidateCount=len(ranked), candidatesTruncated=len(ranked) > 10,
                  rankedCandidates=ranked[:10],
                  callGraphCandidates=[dict(targetRva=row['targetRva'], matchedCallees=row['matchedCallees'],
                       matchedCallers=row['matchedCallers'], translatableCallees=len(mapped_callees),
                       codeBytes=row['codeBytes'], method='unique-neighbor-evidence', semanticStatus='unverified')
                       for row in ranked[:10] if row['matchedCallees'] or row['matchedCallers']])
    if not ranked:
        result['unresolvedReason'] = 'no-cross-build-evidence'
        return result
    best = ranked[0]
    margin = best['score'] - ranked[1]['score'] if len(ranked) > 1 else None
    result['scoreMargin'] = margin
    if best['descriptorConflict']:
        result.update(status='conflicting-evidence', unresolvedReason='best-candidate-has-different-engine-descriptor')
    elif anchors.get(root) == best['targetRva']:
        result.update(status='unique-static-evidence-candidate', candidateFunctionRva=best['targetRva'])
    elif (best['matchedCallees'] + best['matchedCallers'] >= 2 and
          (margin is None or margin >= 12)):
        result.update(status='unique-callgraph-candidate', candidateFunctionRva=best['targetRva'])
    elif (best['branchStructureEqual'] and (margin is None or margin >= 12) and
          ((best['reciprocalUniqueFragments'] and
            (best['matchingHashSiteCount'] or best['matchedCallees'] or best['matchedCallers'] or
             matched_sites[best['targetRva']])) or
           (best['directBranchCount'] >= 3 and best['matchingHashSiteCount']))):
        result.update(status='unique-structural-evidence-candidate', candidateFunctionRva=best['targetRva'])
    elif len(ranked) == 1 and (best['exactNormalizedCode'] or best['engineDescriptorNames']):
        result.update(status='unique-static-evidence-candidate', candidateFunctionRva=best['targetRva'])
    else:
        result.update(status='ambiguous' if margin == 0 else 'ranked-candidates',
                      unresolvedReason='no-unique-independent-function-evidence')
    # Entry-point equality never relocates a hook/patch instruction or its capture offsets.
    result['siteMapping'] = 'unresolved-requires-instruction-and-context-validation'
    return result


def compare_functions(left_reflection, left_functions, left_audit,
                      right_reflection, right_functions, right_audit, profile=None, target_scan=None,
                      source_components=None, target_components=None):
    rows = operations((left_audit or {}).get('audit'), profile)
    if not left_functions or not right_functions:
        return dict(operationCandidates=[dict(id=row['id'], status='unresolved', callable=False,
                    semanticStatus='unverified', unresolvedReason='source-or-target-function-capture-missing') for row in rows],
                    functionMatching=dict(available=False, semanticsVerified=False))
    source = FunctionIndex(left_reflection, left_functions, source_components)
    target = FunctionIndex(right_reflection, right_functions, target_components)
    anchors, methods, conflicts = unique_anchors(source, target)
    target_operations = {row['id']: row.get('matches', [])
                         for row in ((right_audit or {}).get('audit') or {}).get('operations', [])}
    scans = {row['id']: row for row in (target_scan or {}).get('functions', [])}
    target_operations.update({key: row.get('matches', []) for key, row in scans.items()})
    candidates = []
    for row in rows:
        scan = scans.get(row['id'], {})
        guard = row.get('sourceGuardBytes', [])
        expected = [f'{value:02X}' for value in guard]
        verified_guard = bool(profile and guard and len(guard) >= 7 and
            row.get('functionRva') is not None and row.get('functionRva') == row.get('guardRva') and
            scan.get('signature', '').upper().split() == expected and scan.get('original') == guard and
            scan.get('originalBytesMatch') is True)
        candidates.append(rank_operation(row, source, target, anchors, methods,
                          target_operations.get(row['id'], []), verified_guard))
    return dict(operationCandidates=candidates,
                functionMatching=dict(available=True, sourceFunctions=len(source.groups), targetFunctions=len(target.groups),
                    reciprocalUniqueAnchors=len(anchors), conflictingAnchorClaims=conflicts,
                    sourceComponentCallbacks=source.callback_coverage, targetComponentCallbacks=target.callback_coverage,
                    semanticsVerified=False, nativeArgumentsInferred=False,
                    limitations=['Masked code equality and named descriptors identify static candidates only',
                        'Hash immediates and callgraph rankings do not prove native arguments or gameplay effects',
                        'Old hook/patch instruction offsets, capture offsets and guard bytes are not transferred',
                        'Branch-structure evidence covers captured direct CALL/JMP, not conditional or indirect branches',
                        'Matching fragments and relative branch/hash sites are partial static evidence, not whole-code equality',
                        'Compiler changes, inlining, leaf functions and indirect calls can leave operations unresolved']))


def trace_anchors(pe, reflection, functions, scan, catalog=None, maximum_depth=6):
    """Trace every fresh byte match to original procedures and named callers.

    The caller graph retains instruction RVAs and all equally short incoming
    edges. It establishes static code relationships, never an ABI or effect.
    """
    import re
    import capstone as cs
    from collections import deque
    from discovery.functions import profile_audit

    identity = pe.identity()
    if not 0 <= maximum_depth <= 6:
        raise ValueError('caller depth must be between zero and six')
    if scan.get('image') != identity or reflection.get('image') != identity or functions.get('image') != identity:
        raise ValueError('anchor scan and captures must belong to the exact executable image')
    index = FunctionIndex(reflection, functions)
    features = scan.get('features')
    if not isinstance(features, list):
        raise ValueError('anchor scan requires a features array')
    if any(not isinstance(row, dict) for row in features):
        raise ValueError('anchor features must be objects')
    rows = [dict(row) for row in features]
    catalog_entries = {row.get('name'): row for row in (catalog or {}).get('entries', [])}
    present = {row.get('name') for row in rows}
    for row in (catalog or {}).get('entries', []):
        if row.get('name') not in present:
            rows.append(dict(row))
            present.add(row.get('name'))
    valid, invalid = {}, {}
    for ordinal, row in enumerate(rows):
        tokens = str(row.get('signature') or '').split()
        if not tokens:
            continue
        if (len(tokens) > 4096 or any(not re.fullmatch(r'[0-9a-fA-F]{2}|\?\??', t) for t in tokens)
                or sum(t not in ('?', '??') for t in tokens) < 4):
            invalid[ordinal] = 'invalid-or-insufficiently-specific-byte-signature'
        else:
            valid[str(ordinal)] = dict(signature=' '.join(tokens))
    audited = profile_audit(pe, dict(id='trace-anchors', target=identity['target'], image=identity, hooks=valid))
    matches = {int(row['id'][5:]): row['matches'] for row in audited['operations']}
    edges = defaultdict(list)
    for caller, group in index.groups.items():
        for fragment in group['ranges']:
            for branch in index.ranges[fragment].get('directBranches', []):
                callee = index.containing(branch['targetRva'])
                if branch['kind'] == 'call' and callee is not None and caller != callee:
                    edges[callee].append(dict(callerFunctionRva=caller, calleeFunctionRva=callee,
                                             instructionRva=branch['instructionRva'], targetRva=branch['targetRva']))
    decoder = cs.Cs(cs.CS_ARCH_X86, cs.CS_MODE_64)
    procedures, caller_functions, referenced_types, anchors = {}, {}, set(), []
    type_names = defaultdict(list)
    for ty in reflection['types']:
        for name in {ty.get('name'), ty['qualifiedName']} - {None}:
            type_names[name].append(ty['index'])

    def descriptor_types(group):
        dependencies = [dict(descriptor=d['name'], slotOffset=s['offset'], name=name,
                             reflectionCandidates=type_names[name], associationUnique=len(type_names[name]) == 1)
                        for d in group.get('engineDescriptors', [])
                        for s in d.get('dependencySlots', []) for name in s.get('entries', [])]
        referenced_types.update(t for row in dependencies for t in row['reflectionCandidates'])
        return dependencies

    def procedure(root):
        key = str(root)
        if key not in procedures:
            group = index.groups[root]
            ranges = []
            for rva in group['ranges']:
                native = index.ranges[rva]
                code = pe.read(rva, native['endRva'] - rva)
                instructions = [dict(rva=i.address, bytes=i.bytes.hex(), asm=i.mnemonic + ' ' + i.op_str)
                                for i in decoder.disasm(code, rva)]
                ranges.append(dict(native, originalBytes=code.hex(), instructions=instructions))
                referenced_types.update(r['typeIndex'] for r in native.get('metadataReferences', []))
                referenced_types.update(c['typeIndex'] for h in native.get('hashCandidates', []) for c in h['candidates'])
            procedures[key] = dict(group, ranges=ranges, callable=False, nativeAbiProven=False,
                                   gameplayEffectsProven=False, descriptorTypeReferences=descriptor_types(group))
        return procedures[key]

    def callers(root):
        queue, distance, paths, found, graph = deque([root]), {root: 0}, {root: []}, [], {}
        while queue:
            current = queue.popleft()
            names = sorted(index.features[current]['descriptor'])
            if names:
                found.append(dict(functionRva=current, names=names, depth=distance[current],
                                  oneShortestCallPath=paths[current]))
                continue
            if distance[current] == maximum_depth:
                continue
            for edge in sorted(edges[current], key=lambda e: (e['callerFunctionRva'], e['instructionRva'])):
                caller, depth = edge['callerFunctionRva'], distance[current] + 1
                if caller == root or distance.get(caller, depth) < depth:
                    continue
                graph[edge['instructionRva']] = edge
                caller_functions[str(caller)] = dict(index.groups[caller], callable=False, nativeAbiProven=False,
                    descriptorTypeReferences=descriptor_types(index.groups[caller]))
                if caller not in distance:
                    distance[caller] = depth
                    paths[caller] = [edge, *paths[current]]
                    queue.append(caller)
        return dict(nearestNamedCallers=found, directCallEdges=list(graph.values()), maximumDepth=maximum_depth,
                    pathPolicy='one shortest path per named procedure; graph retains alternative shortest edges',
                    includesIndirectCalls=False, nativeArgumentsProven=False)

    for ordinal, row in enumerate(rows):
        sites = matches.get(ordinal, [])
        result = dict(name=row.get('name'), signature=row.get('signature'), reportedMatches=row.get('matches'),
                      matches=sites, matchCount=len(sites), sites=[], callable=False, nativeAbiProven=False)
        metadata = catalog_entries.get(row.get('name'), {})
        result['catalogMetadata'] = metadata or None
        if ordinal in invalid:
            result.update(status='unresolved-invalid-signature', reason=invalid[ordinal])
        elif not row.get('signature'):
            result.update(status='unresolved-data-mechanism', reason='no-byte-signature-supplied')
        else:
            result['status'] = 'unresolved-no-match' if not sites else 'ambiguous-byte-matches' if len(sites) > 1 else 'unique-byte-match'
        for site in sites:
            root = index.containing(site)
            detail = dict(siteRva=site, primaryFunctionRva=root, callable=False)
            if root is None:
                detail.update(instructionBoundaryVerified=False, reason='no-compiler-described-procedure')
            else:
                group = procedure(root)
                instructions = [i for part in group['ranges'] for i in part['instructions']]
                position = next((n for n, i in enumerate(instructions) if i['rva'] == site), None)
                detail.update(instructionBoundaryVerified=position is not None,
                              instructionContext=instructions[max(0, position - 6):position + 10] if position is not None else [],
                              callerTrace=callers(root))
            offset = metadata.get('siteOffset', metadata.get('siteOffset130') if metadata.get('kind20') == 0
                                  else metadata.get('siteOffset178'))
            if isinstance(offset, int) and not isinstance(offset, bool):
                target, target_root = site + offset, index.containing(site + offset)
                instructions = [i for part in procedure(target_root)['ranges'] for i in part['instructions']] if target_root is not None else []
                position = next((n for n, i in enumerate(instructions) if i['rva'] == target), None)
                detail['catalogSite'] = dict(offset=offset, rva=target, primaryFunctionRva=target_root,
                    instructionBoundaryVerified=position is not None,
                    instructionContext=instructions[max(0, position - 6):position + 10] if position is not None else [],
                    evidence='catalog-declared offset, independently checked instruction boundary', callable=False)
            result['sites'].append(detail)
        result['reportedMatchesAgree'] = sorted(row['matches']) == sites if 'matches' in row else None
        anchors.append(result)
    return dict(schemaVersion=1, kind='exact-image-original-anchor-trace', image=identity, anchors=anchors,
                coverage=dict(anchors=len(anchors), signatureAnchors=len(valid), matchedAnchors=sum(bool(a['matches']) for a in anchors),
                              ambiguousAnchors=sum(len(a['matches']) > 1 for a in anchors), procedureGroups=len(procedures)),
                procedures=procedures, callerFunctions=caller_functions,
                reflectedTypes=[t for t in reflection['types'] if t['index'] in referenced_types],
                callable=False, nativeAbiProven=False, gameplayEffectsProven=False,
                limitations=['Byte matches and type-hash immediates do not establish function arguments or effects',
                             'All byte matches are retained, including non-instruction-boundary matches',
                             'PE unwind groups omit some leaf functions, inlining and generated code',
                             'Caller traces use captured direct CALL instructions only'])
