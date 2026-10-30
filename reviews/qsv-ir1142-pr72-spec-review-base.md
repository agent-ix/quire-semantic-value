---
id: SR-16676
title: "IR-1142 QSV #72 structural identity specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value; spec/functional/FR-060-the-crate-mints-no-identity.md; spec/functional/FR-128-consume-structural-identities.md; context spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md"
review_set: subset
---
# IR-1142 QSV #72 structural identity specification review

## Summary

Independent review of PR #72 head f958b04 against QSV main and quire-exact #101 structural API. Examined FR-060-AC-5/6 and FR-128-AC-1 through AC-5, with FR-124-AC-4/5 as existing contract context. Quire validation passed; the two identity-handoff conflicts below leave incompatible acceptance criteria in the same repository.

## Verdict

**FAIL** — the spec cannot be implemented while retaining all existing FR-124 criteria. Reconcile the old digest-specific controls as structural controls in this PR, preserving their distinct-domain and refusal intent.

## Examined scope

- FR-060-AC-5 (examined): structural NodeKey constructor lint and no reminting.
- FR-060-AC-6 (examined): structural EffectiveId and PopulationId constructor lint and no fixed identity oracle.
- FR-128-AC-1 (examined): NodeKey component distinctness and missing-node refusal.
- FR-128-AC-2 (examined): EffectiveId component distinctness and typed refusal.
- FR-128-AC-3 (examined): declared/compound arms, canonical terms, two-step conversion and meter.
- FR-128-AC-4 (examined): no byte identity bridge or fixed digest oracle.
- FR-128-AC-5 (examined): structural refusal and charge order.
- FR-124-AC-4 (context_only): 16 MiB digest identity refusal.
- FR-124-AC-5 (context_only): equal digest bytes across distinct unit domains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-128 requires structural compound UnitId without digest derivation, but FR-124-AC-4 still requires a 16 MiB digest identity refusal; the exact structural UnitId API has no such fallible digest construction. Preserve the refusal boundary as an explicit independently owned control if still valid, or update the AC to the structural behavior. | spec/functional/FR-128-consume-structural-identities.md:49; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:121 |
| FND-002 | high | FR-128-AC-3 requires declared and compound structural arms, but FR-124-AC-5 still requires equal digest bytes and Behavior 6 orders domains before a digest; these oracles cannot be constructed with the exact structural identity API. Retain the cross-arm non-aliasing control using equal displayed components without digest bytes. | spec/functional/FR-128-consume-structural-identities.md:49; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:107,122 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | PR #72 head 5c05157 retains the 16 MiB InputBytes refusal on bounded structural term encoding before UnitId construction. FR-124-AC-4 and TC-923 step 4 keep the no-quantity/no-unit and unchanged UnitScope controls; bytes are not retained as identity. |
| FND-002 | fixed | FR-124-AC-5 and TC-923 step 5 compare Declared(r) with Compound([(r, 1)]), assert declared-before-compound ordering, and resolve the compound quantity to the compound unit. FR-128-AC-3 uses the same structural control. |
