---
id: SR-13510
title: "IR-882 quire-semantic-value PR 54 code review: FR-125 grounding in src/definition.rs"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@90a659dc5c32aa0cea79fb1cda2b89c0765523f6 vs origin/main 9270383; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---

## Summary

Ticket: IR-882. Spec-only PR: `git diff origin/main...90a659dc5` adds FR-125 and TC-924 and changes
no source. This review checks every FR-125 claim against `src/definition.rs` on origin/main 9270383
(unchanged at the head) and against its consumer, QSL `qsl-semantics/src/value/definition.rs` (QSL main
469f8b4).

Source checks:

- `PackageCause` has exactly `MissingMember`, `ConflictingDefinition` and `IncompatibleDefinition`,
  derives `Clone, Copy, Debug, Eq, Hash, PartialEq`, and has no `#[non_exhaustive]`. `as_str` gives
  `missing-member`, `conflicting-definition` and `incompatible-definition`.
- `PackageRefusalCode` has the single variant `InvalidPackage`, spelled `invalid_package`, and is not
  `#[non_exhaustive]` either.
- `PackageRefusal { pub code, pub cause }` has one constructor, `invalid_package(cause)`. Its fields
  are public, so a struct literal is a second way to build one, but it can only use `InvalidPackage`,
  the code's single variant.
- `SelectionRefusalCode::ALL` lists the eight codes in the order FR-125 states, which is the order in
  QSpec `proposals/quire-v1/definitions/value-complete.md`. `from_code` is an exact byte comparison
  over `ALL`.
- Who raises what, in QSL: `DefinitionLock::admit_selection` raises all eight selection codes in that
  order. `admit_integer_division` and `resolve_division` raise all three causes, and so does
  `admit_ieee_profile`. Nothing in QSL or QSV raises any other `invalid_package` cause through this
  type.
- FR-272 (QSpec main 240faeb): the three spellings are byte-equal to entries of the `invalid_package`
  row. The row lists 20 causes, so these three are a subset of it. `byte-digest-mismatch`,
  `digest-domain-mismatch` and `content-mismatch` belong to the `stale_dependency` /
  `source_digest_mismatch` row, as FR-125 says.

A scratch probe (`tests/probe_fr125.rs` in a copy of the tree, not in the PR) passed. It checked the
eight `from_code` round trips, `None` for `invalid_package`, `""`, `Selection_unknown_trigger`, every
upper-cased spelling and every spelling with a trailing space, the `ALL` order, and the
`invalid_package` pairing for each cause. These are the `from_code` negatives the author did not check.

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

## Verdict

**PASS (no findings).** Every claim FR-125 makes about the code is true on main, and the
author's unchecked `from_code` negatives hold. The findings in the spec-review artifacts (SR-13511 to
SR-13514) are about how the requirement is worded and traced, not about the code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

Probe: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv54/probe/tests/probe_fr125.rs (copy of the tree at /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv54/probe; cargo target /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv54/target).
