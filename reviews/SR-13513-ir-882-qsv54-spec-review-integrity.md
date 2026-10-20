---
id: SR-13513
title: "IR-882 quire-semantic-value PR 54 integrity review of FR-125 and TC-924"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@90a659dc5c32aa0cea79fb1cda2b89c0765523f6 vs origin/main 9270383; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---

## Summary

Ticket: IR-882. Integrity review: do FR-125's Behaviors map onto its ACs, do TC-924's steps
follow them, and can each AC fail?

- Behaviors 1-3 map to AC-1 and AC-3, Behavior 4 to AC-2, Behaviors 5-7 to AC-3 and Behaviors 8-9 to
  AC-4. TC-924 steps 1-4 map one-to-one onto AC-1..AC-4, and its Expected Results match.
- AC-1, AC-2 and AC-4 can fail against plausible regressions. The scratch probe (SR-13510) exercised
  each of them against main.
- Numbering: FR-125 and TC-924 are the next free ids after FR-124 and TC-923. No other file in the
  tree, and nothing in the one other open PR (#53, the FR-124 implementation), uses either id. The
  section layout and the `verifies` edge follow FR-124 and TC-923.
- `quoin matrix` was not run: no test is tagged yet, which is expected for a spec-only PR.

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
- FR-125-AC-1 (examined): `PackageCause::as_str` returns `missing-member`, `conflicting-definition` and `incompatible-definition`, each byte-equal to the spelling in QSpec FR-272's `invalid_package` row, and `PackageRefusalCode::InvalidPackage.as_str()` is `invalid_package`.
- FR-125-AC-2 (examined): `PackageRefusal::invalid_package(cause)` returns `code` `InvalidPackage` and the given `cause` for each of the three causes, and no public item pairs a `PackageCause` with another code.
- FR-125-AC-3 (examined): `PackageCause` has exactly three variants and is not `#[non_exhaustive]`: an exhaustive `match` over it with no wildcard arm compiles, and none of `byte-digest-mismatch`, `digest-domain-mismatch` or `content-mismatch` is the spelling of any variant.
- FR-125-AC-4 (examined): `SelectionRefusalCode::ALL` holds the eight codes in the order above, `as_str` returns their spellings, `from_code` round-trips each, and `from_code` returns `None` for `invalid_package`, an empty string and a differently cased spelling.
- TC-924 (examined): Verify [FR-125](../functional/FR-125-definition-closure-refusal-vocabulary.md) through the public `quire_semantic_value::definition` API: the spellings, the pairing with `invalid_package`, the closed cause set and the selection codes.
- TC-924#step-1 (examined): Compare each `PackageCause::as_str` and `PackageRefusalCode::as_str` with the spellings in QSpec FR-272's `invalid_package` row (FR-125-AC-1).
- TC-924#step-2 (examined): Build `PackageRefusal::invalid_package` for each cause and read `code` and `cause`; inspect the public items for another pairing (FR-125-AC-2).
- TC-924#step-3 (examined): Compile an exhaustive `match` over `PackageCause` with no wildcard arm, and compare the three spellings against `byte-digest-mismatch`, `digest-domain-mismatch` and `content-mismatch` (FR-125-AC-3).
- TC-924#step-4 (examined): Compare `SelectionRefusalCode::ALL` with the eight codes in order, round-trip `from_code`, and call it with `invalid_package`, `""` and an upper-cased spelling (FR-125-AC-4).
- FR-125#references (examined): - Linear IR-882 (R-3): the runtime retires its six-variant copy of the vocabulary and consumes this crate's.

## Verdict

**PASS with findings (1 medium, 3 low).** The medium finding is AC-3's compile check. Written in
the crate's own unit tests, it cannot fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-3's check, 'an exhaustive `match` over it with no wildcard arm compiles', only catches `#[non_exhaustive]` when the match is compiled in a crate other than `quire-semantic-value`. Inside the defining crate, `#[non_exhaustive]` has no effect. Probe: with `#[non_exhaustive]` added to `PackageCause`, an exhaustive no-wildcard match in a `#[cfg(test)]` module of `src/definition.rs` compiled and passed, and the same match in `tests/probe_fr125.rs` failed with E0004. Neither AC-3 nor TC-924 step 3 says where the match goes. Fix: require the match to be in an integration test under `tests/` (a separate crate), or in a downstream crate. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:80 |
| FND-002 | low | AC-2's Verification is 'Test (TC-924)', but its second clause, 'no public item pairs a `PackageCause` with another code', can only be checked by reading the API, and TC-924 step 2 says 'inspect the public items'. Use 'Test and Inspection (TC-924)', as AC-3 does, or drop the clause (see SR-13512 FND-001). | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:79 |
| FND-003 | low | The References line, 'Linear IR-882 (R-3): the runtime retires its six-variant copy of the vocabulary and consumes this crate's', records one consumer's migration task. It does not describe this system. When the RT work lands it becomes a stale changelog entry. The lasting point, that the runtime consumes these types and keeps no copy, is already in the Description. Drop the line, or reduce it to a bare ticket id. | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:90-93 |
| FND-004 | low | FR-125 has no `## Status` section, but 16 of the 21 QSV FRs have one, including FR-123 and FR-124. The specified code is already on main and only the tests are missing, so the reader cannot tell whether FR-125 is planned or implemented. Add `## Status` (for example 'Implemented; TC-924 tests pending'). | spec/functional/FR-125-definition-closure-refusal-vocabulary.md:11-93 |

## Evidence

Probe (non_exhaustive added to PackageCause in a scratch copy): the in-crate unit test with an exhaustive no-wildcard match passed; the same match in tests/probe_fr125.rs failed with E0004 'PackageCause is marked as non-exhaustive'. Copy at /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv54/probe.
