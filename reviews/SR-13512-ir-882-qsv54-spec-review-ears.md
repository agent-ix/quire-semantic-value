---
id: SR-13512
title: "IR-882 quire-semantic-value PR 54 EARS review of FR-125"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-semantic-value@90a659dc5c32aa0cea79fb1cda2b89c0765523f6 vs origin/main 9270383; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---

## Summary

Ticket: IR-882. EARS requirement-grammar review of FR-125's Description statement and its nine
Behavior statements. All are ubiquitous `SHALL` / `SHALL NOT` statements whose subject is the crate or one
of its types, which suits a static vocabulary with no trigger. Behaviors 1, 2, 3, 8 and 9 each make one
checkable claim. `quire validate` reports no grammar warnings on FR-125 or TC-924. The tree has 3
warnings, all on FR-110, the same as on main.

## Examined scope

- FR-125 (examined): `quire-semantic-value` SHALL own the refusal vocabulary that definition-closure admission reports: the lock's eight `selection_refusal_codes` (`SelectionRefusalCode`), the one diagnostic code `invalid_package` (`PackageRefusalCode::InvalidPackage`) and the three causes it pairs with here (`PackageCause`). A consumer such as the runtime consumes these types and keeps no copy of the vocabulary. QSpec FR-272 ("typed diagnostic cause") is the upstream authority for the spellings and for which cause pairs with which code. This requirement restates the subset that definition-closure admission raises
- FR-125#behavior-1 (examined): `PackageCause` SHALL have exactly three variants: `MissingMember`, `ConflictingDefinition` and `IncompatibleDefinition`.
- FR-125#behavior-2 (examined): `PackageCause::as_str` SHALL return `missing-member`, `conflicting-definition` and `incompatible-definition` respectively, the spellings of FR-272's `invalid_package` row, byte for byte.
- FR-125#behavior-3 (examined): `PackageRefusalCode` SHALL have exactly one variant, `InvalidPackage`, whose `as_str` is `invalid_package`.
- FR-125#behavior-4 (examined): `PackageRefusal::invalid_package` SHALL pair every `PackageCause` with `PackageRefusalCode::InvalidPackage`, and the crate SHALL offer no constructor that pairs a `PackageCause` with another code.
- FR-125#behavior-5 (examined): The crate SHALL NOT declare a cause of any other code family, including the `stale_dependency` and `source_digest_mismatch` causes `byte-digest-mismatch`, `digest-domain-mismatch` and `content-mismatch`; the QSpec package-read and replay layers own them.
- FR-125#behavior-6 (examined): `PackageCause` SHALL be a closed enumeration.
- FR-125#behavior-7 (examined): `PackageCause` SHALL NOT be marked `#[non_exhaustive]`, because the set is the three causes that definition-closure admission raises, a consumer's exhaustive `match` is the check that it handles all of them, and a new cause is a change to FR-272 and to this requirement together.
- FR-125#behavior-8 (examined): `SelectionRefusalCode::ALL` SHALL list the eight codes in normative check order: `selection_unknown_trigger`, `selection_duplicate_trigger`, `selection_unknown_role`, `selection_duplicate_role`, `selection_required_missing`, `selection_alternative_conflict`, `selection_trigger_unsatisfied` and `selection_untriggered_profile`.
- FR-125#behavior-9 (examined): `SelectionRefusalCode::as_str` SHALL return each code's spelling, and `from_code` SHALL return the code for exactly those eight spellings and `None` for any other string, `invalid_package` included.

## Verdict

**PASS with findings (0 medium, 2 low).** One statement has a clause that cannot fail, and two
statements say the same thing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 4 joins two obligations, and the second cannot fail. 'the crate SHALL offer no constructor that pairs a `PackageCause` with another code' always holds while Behavior 3 holds, because `PackageRefusalCode` has only one variant, so there is no other code to pair with. It also counts as a constructor the public struct literal `PackageRefusal { code, cause }`, which the Outputs list. Keep the first clause (`invalid_package` pairs every cause with `InvalidPackage`). Replace the second with the real invariant: 'every `PackageRefusal` carries `PackageRefusalCode::InvalidPackage`', which follows from Behavior 3. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:53-55 |
| FND-002 | low | Behavior 6, '`PackageCause` SHALL be a closed enumeration', has no test of its own. Its meaning is covered by Behavior 1 (exactly three variants) and Behavior 7 (not `#[non_exhaustive]`), and 'closed' is not defined. Drop it, or merge it into Behavior 7 as that statement's lead clause. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:60 |

## Evidence

quire 0.36.2 `quire validate --scope . 'spec/**/*.md'`: exit 0; 3 warnings, all FR-110, both at the head and on main.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1 head d5a01ca5e: d5a01ca5e: Behavior 4 now holds only the pairing. The cannot-fail clause is replaced by its own Behavior 5, 'Every `PackageRefusal` SHALL carry `PackageRefusalCode::InvalidPackage`'. |
| FND-002 | fixed | round 1 head d5a01ca5e: d5a01ca5e: 'SHALL be a closed enumeration' is deleted. Behaviors 1 and 7 (exactly three variants, not `#[non_exhaustive]`) carry the meaning. |
