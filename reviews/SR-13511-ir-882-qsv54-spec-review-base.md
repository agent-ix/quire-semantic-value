---
id: SR-13511
title: "IR-882 quire-semantic-value PR 54 base spec review of FR-125 and TC-924"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@90a659dc5c32aa0cea79fb1cda2b89c0765523f6 vs origin/main 9270383; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---

## Summary

Ticket: IR-882. Base spec review of FR-125 and TC-924 against QSpec FR-272 (main 240faeb), QSL's
admission code (main 469f8b4), the retiring RT copy (`quire-contract-runtime` main
`src/exact/definition.rs`, which has six causes including `revision-mismatch` and two digest causes) and
the existing QSV requirements.

What holds:

- The Description correctly says FR-125 'restates the subset that definition-closure admission raises'.
  It does not claim the three causes are the whole `invalid_package` row.
- No QSV requirement already covers `definition.rs`. A grep of `spec/` for the type names, the
  selection codes and `invalid_package` finds only the two new files. So there is no overlap. FR-060
  (no `NodeKey` minting) does not conflict with FR-125.
- Retiring RT's six-variant copy (Linear IR-882 R-3) leaves no gap: QSV's three causes are exactly
  what QSL raises.

## Examined scope

- FR-125 (examined): `quire-semantic-value` SHALL own the refusal vocabulary that definition-closure admission reports: the lock's eight `selection_refusal_codes` (`SelectionRefusalCode`), the one diagnostic code `invalid_package` (`PackageRefusalCode::InvalidPackage`) and the three causes it pairs with here (`PackageCause`). A consumer such as the runtime consumes these types and keeps no copy of the vocabulary. QSpec FR-272 ("typed diagnostic cause") is the upstream authority for the spellings and for which cause pairs with which code. This requirement restates the subset that definition-closure admission raises
- FR-125#use-case (examined): The runtime admits a package's `div`, `rem` and `mod` definitions and must name the refusal exactly as the QSpec catalog does. It matches on `PackageCause` without a wildcard arm, so a cause added later is a compile error at every consumer, not a silent fallthrough.
- FR-125#behavior-1 (examined): `PackageCause` SHALL have exactly three variants: `MissingMember`, `ConflictingDefinition` and `IncompatibleDefinition`.
- FR-125#behavior-2 (examined): `PackageCause::as_str` SHALL return `missing-member`, `conflicting-definition` and `incompatible-definition` respectively, the spellings of FR-272's `invalid_package` row, byte for byte.
- FR-125#behavior-3 (examined): `PackageRefusalCode` SHALL have exactly one variant, `InvalidPackage`, whose `as_str` is `invalid_package`.
- FR-125#behavior-4 (examined): `PackageRefusal::invalid_package` SHALL pair every `PackageCause` with `PackageRefusalCode::InvalidPackage`, and the crate SHALL offer no constructor that pairs a `PackageCause` with another code.
- FR-125#behavior-5 (examined): The crate SHALL NOT declare a cause of any other code family, including the `stale_dependency` and `source_digest_mismatch` causes `byte-digest-mismatch`, `digest-domain-mismatch` and `content-mismatch`; the QSpec package-read and replay layers own them.
- FR-125#behavior-6 (examined): `PackageCause` SHALL be a closed enumeration.
- FR-125#behavior-7 (examined): `PackageCause` SHALL NOT be marked `#[non_exhaustive]`, because the set is the three causes that definition-closure admission raises, a consumer's exhaustive `match` is the check that it handles all of them, and a new cause is a change to FR-272 and to this requirement together.
- FR-125#behavior-8 (examined): `SelectionRefusalCode::ALL` SHALL list the eight codes in normative check order: `selection_unknown_trigger`, `selection_duplicate_trigger`, `selection_unknown_role`, `selection_duplicate_role`, `selection_required_missing`, `selection_alternative_conflict`, `selection_trigger_unsatisfied` and `selection_untriggered_profile`.
- FR-125#behavior-9 (examined): `SelectionRefusalCode::as_str` SHALL return each code's spelling, and `from_code` SHALL return the code for exactly those eight spellings and `None` for any other string, `invalid_package` included.
- FR-125-AC-1 (examined): `PackageCause::as_str` returns `missing-member`, `conflicting-definition` and `incompatible-definition`, each byte-equal to the spelling in QSpec FR-272's `invalid_package` row, and `PackageRefusalCode::InvalidPackage.as_str()` is `invalid_package`.
- FR-125-AC-2 (examined): `PackageRefusal::invalid_package(cause)` returns `code` `InvalidPackage` and the given `cause` for each of the three causes, and no public item pairs a `PackageCause` with another code.
- FR-125-AC-3 (examined): `PackageCause` has exactly three variants and is not `#[non_exhaustive]`: an exhaustive `match` over it with no wildcard arm compiles, and none of `byte-digest-mismatch`, `digest-domain-mismatch` or `content-mismatch` is the spelling of any variant.
- FR-125-AC-4 (examined): `SelectionRefusalCode::ALL` holds the eight codes in the order above, `as_str` returns their spellings, `from_code` round-trips each, and `from_code` returns `None` for `invalid_package`, an empty string and a differently cased spelling.
- TC-924 (examined): Verify [FR-125](../functional/FR-125-definition-closure-refusal-vocabulary.md) through the public `quire_semantic_value::definition` API: the spellings, the pairing with `invalid_package`, the closed cause set and the selection codes.

## Verdict

**PASS with findings (0 medium, 2 low).** The requirement is coherent and true of the code. Two
explanatory sentences misstate who raises the causes and what a new cause would change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The reason given for not using `#[non_exhaustive]` ends 'a new cause is a change to FR-272 and to this requirement together'. That is false for most likely new causes. FR-272's `invalid_package` row already lists 17 causes besides these three (for example `definition-cycle`, `duplicate-member` and `feature-set-mismatch`). If definition-closure admission started raising one of them, only FR-125 and `PackageCause` would change; FR-272 would not. Say instead that a new cause is a change to this requirement, and that it must be a cause FR-272's `invalid_package` row already lists, or one FR-272 adds first. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:61-64 |
| FND-002 | low | The Use case says the three causes come from admitting 'a package's `div`, `rem` and `mod` definitions'. But QSL also raises all three in `DefinitionLock::admit_ieee_profile`: `incompatible-definition` for a wrong profile identity, `missing-member` and `conflicting-definition` for zero or several profiles, and `conflicting-definition` for a user declaration bound to a reserved IEEE intrinsic identity (FR-148). The retiring RT copy documents the same. The source's own doc comment says 'division admission'. Behavior 7 relies on 'the three causes that definition-closure admission raises', so name both closures, the integer-division closure and the IEEE profile closure, so that a reader checking that set looks at both raise sites. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:29-32 |

## Evidence

FR-272 copy: QSpec `git show 240faeb:spec/objects/foundation/FR-272-typed-diagnostic-cause.md` (invalid_package row at line 74). QSL raise sites: `qsl-semantics/src/value/definition.rs` at 469f8b4 (admit_selection, admit_integer_division, resolve_division, admit_ieee_profile).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The Description's closing clause contradicts the crate. It says 'the crate declares no cause, and no code family, that FR-272 does not list'. But the same requirement makes the crate declare `SelectionRefusalCode`, a family of eight `selection_*` codes. FR-272 lists none of them, and no QSpec `spec/` artifact does: `git grep selection_unknown_trigger` at QSpec 240faeb finds them only under `proposals/` and in the tests. The round-1 edit made this sharper by naming value-complete.md as their separate authority two sentences earlier. Fix: limit the clause to what FR-272 governs. For example: 'the crate declares no `invalid_package` cause, and no diagnostic code, that FR-272 does not list; the selection codes are value-complete.md's.' This was in the round-0 text and missed in the review pass. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:20-25 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1 head d5a01ca5e: d5a01ca5e: the rationale now reads 'A new cause is a change to this requirement, and it is a cause FR-272's `invalid_package` row already lists or one FR-272 adds first.' |
| FND-002 | fixed | round 1 head d5a01ca5e: d5a01ca5e: the Use case names both closures, the integer-division closure and `admit_ieee_profile`, as the raise sites, and Behavior 7 now says 'the three causes that the two closures raise'. |
| FND-003 | fixed | round 2 head bde40e374: bde40e374: the clause now reads 'the crate declares no `invalid_package` cause and no diagnostic code that FR-272 does not list. The eight selection codes are not FR-272's; they come from `value-complete.md`.' This no longer contradicts `SelectionRefusalCode`, and it agrees with Behavior 6 and with Dependencies. |
