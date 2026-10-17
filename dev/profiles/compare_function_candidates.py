"""Rank captured native procedures without authorizing engine calls."""
from collections import defaultdict
from function_index import FunctionIndex

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
