---
id: SR-13514
title: "IR-882 quire-semantic-value PR 54 dependency review of FR-125 and TC-924"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-semantic-value@90a659dc5c32aa0cea79fb1cda2b89c0765523f6 vs origin/main 9270383; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---

## Summary

Ticket: IR-882. Dependency review of the new `relationships` edges. FR-125 `references` QSpec
FR-272 and QSV FR-060, and TC-924 `verifies` FR-125. All three targets resolve. The FR-272 edge is right
for the three causes and the `invalid_package` code. Note that FR-272's cause table says of itself that
it is 'proposed, not adopted'. Calling it the upstream authority is consistent with how QSL and RT
already cite it.

## Examined scope

- FR-125 (examined): `quire-semantic-value` SHALL own the refusal vocabulary that definition-closure admission reports: the lock's eight `selection_refusal_codes` (`SelectionRefusalCode`), the one diagnostic code `invalid_package` (`PackageRefusalCode::InvalidPackage`) and the three causes it pairs with here (`PackageCause`). A consumer such as the runtime consumes these types and keeps no copy of the vocabulary. QSpec FR-272 ("typed diagnostic cause") is the upstream authority for the spellings and for which cause pairs with which code. This requirement restates the subset that definition-closure admission raises
- FR-125#dependencies (examined): - QSpec FR-272 (spec/objects/foundation/FR-272-typed-diagnostic-cause.md) owns the causes and their pairing with diagnostic codes. - [FR-060](FR-060-the-crate-mints-no-identity.md) keeps the crate's identity rule; this requirement adds no identity.
- FR-125#behavior-8 (examined): `SelectionRefusalCode::ALL` SHALL list the eight codes in normative check order: `selection_unknown_trigger`, `selection_duplicate_trigger`, `selection_unknown_role`, `selection_duplicate_role`, `selection_required_missing`, `selection_alternative_conflict`, `selection_trigger_unsatisfied` and `selection_untriggered_profile`.
- FR-125-AC-4 (examined): `SelectionRefusalCode::ALL` holds the eight codes in the order above, `as_str` returns their spellings, `from_code` round-trips each, and `from_code` returns `None` for `invalid_package`, an empty string and a differently cased spelling.
- TC-924 (examined): Verify [FR-125](../functional/FR-125-definition-closure-refusal-vocabulary.md) through the public `quire_semantic_value::definition` API: the spellings, the pairing with `invalid_package`, the closed cause set and the selection codes.

## Verdict

**PASS with findings (1 medium, 1 low).** The eight selection codes, half the vocabulary,
have no upstream edge. One edge points at a requirement that is not related.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The eight selection codes have no upstream source. The Description names QSpec FR-272 as 'the upstream authority for the spellings', and Dependencies lists only FR-272 and FR-060. But FR-272 has no `selection_*` code. Their spellings and the 'normative check order' that Behavior 8 and AC-4 rely on come from QSpec `proposals/quire-v1/definitions/value-complete.md` ('A refused selection reports the first failing check in this order ...') and the lock's `selection_refusal_codes` in `complete-value-lock.json`. That array is sorted alphabetically, not in check order. A reader cannot check AC-4's order against anything. Fix: name `value-complete.md` as the authority for the selection codes and their order, in the Description and in Dependencies, and add a `references` edge to it if the module allows one. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:83-88 |
| FND-002 | low | The `references` edge to FR-060 has no basis in the requirement. FR-060 forbids minting `NodeKey`, `EffectiveId` and `PopulationId`, and this vocabulary has no identity, so the Dependencies line 'adds no identity' states something no one would check here. The edge makes FR-060's blast radius include FR-125 for no reason. Drop the edge and the Dependencies line. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:8-9 |

## Evidence

QSpec main 240faeb: `git grep selection_refusal_codes` finds only proposals/quire-v1/definitions/complete-value-lock.json, complete-model-lock.json and value-complete.md (plus QSpec tests); FR-272 has no selection_* code.
