---
id: SR-13110
title: "Code review of quire-semantic-value PR #49: FR-122-AC-1 tests and retag"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@5e9ab39b1cf29428774d36dc6946e80b832ba002; PR #49 diff origin/main...HEAD (src/declaration.rs checked_invariant_tests, spec/functional/FR-122, spec/test-cases/TC-906, TC-908); rust-review lane folded in"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: references
---
# Code review of quire-semantic-value PR #49

## Summary

Ticket: IR-886 (parent IR-878). PR: quire-semantic-value#49 at 5e9ab39. The rust-review lane is
folded in. The diff retags eight TC-906 tests from the deleted FR-369-AC-9 to FR-122-AC-1. It
uses `#[trace]` only and removes every `/// Trace:` line it touched. It also adds two
`include_str!` source-text checks for the defensive `EqualityQuantityNonExactPlacement` and
`ExpectedIntegerPlacement` arms.

Gates run at the PR head in a throwaway worktree:

- `make fmt-check` and `make lint`, both clippy invocations: pass.
- `cargo test --lib checked_invariant_tests`: 10/10 pass.

Mutation results against the ten AC-1-tagged tests:

- Killed:
  - DeferredResultNotAdmitted → CollectionElementNotAdmitted
  - admission refusal dropped
  - Source↔Target swap
  - NonIntegral → ShapeMismatch
  - conversion UnitUnresolved → ScheduleMismatch
  - conversion `IllTypedCause` replaced
  - NonExact → ShapeMismatch (killed only by the source-text check)
  - ShapeMismatch → TargetNotAdmitted
  - TargetNotAdmitted → SourceNotAdmitted
  - EnumVariant → UnitUnresolved
  - comparison UnitUnresolved → EnumVariant
  - ScheduleMismatch → ShapeMismatch
  - ScheduledComparisonRefused → ConversionRejected
  - quantity.rs ExpectedIntegerPlacement → ValueKindMismatch (killed only by the source-text check)
  - scale-one integer placement (killed by the behavioral success test)
- Survived: `evaluate_tuple` bypassing `admitted`. This is recorded in the gap analysis
  (SR-13111 FND-001).
- Equivalent refactor: reordering the NonExact match pattern (`Integer { .. } | Decimal(_)`)
  fails `exact_equality_conversion_names_unexpected_nonexact_placement`. That is a false
  positive.

## Verdict

**CONDITIONAL**: the retag is correct and the eight behavioral tests are sound. The two new
checks are not vacuous, because they fail on a wrong cause. They do assert source spelling
rather than behavior, though: they are formatting-coupled, they self-reference, and one of
them replaces a behavioral test that is feasible.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `scale_zero_integer_conversion_names_noninteger_placement` greps `quantity.rs` text instead of exercising the branch. The branch is reachable in-crate: a `#[cfg(test)]` module in `quantity.rs` calling private `convert` with `Target::Integer { placement: <max_scale 1>, .. }` returns `CheckedInvariant { ExpectedIntegerPlacement }` (reviewer reproduced, test passed). A behavioral test is feasible and should replace the source check | src/declaration.rs:6073-6090, src/quantity.rs:789-802 |
| FND-002 | medium | `exact_equality_conversion_names_unexpected_nonexact_placement` asserts exact whitespace and pattern order of `declaration.rs` via `include_str!` of its own file. A semantically equivalent reorder of the or-pattern fails it: `split_once` then matches the test's own string literal (panic at :6068). It checks spelling, not behavior, and breaks under rustfmt or refactor | src/declaration.rs:6054-6071, src/declaration.rs:5677-5681 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The committed `reviews/qsv49-code-review.md` and `reviews/qsv49-gap-analysis.md` carry a coder-authored `## Dispositions` section. Its outcomes ("fixed; need decision", "partially fixed; need decision") are outside the outcome set, and dispositions belong to the reviewer. Replace both files with the reviewer's dispositioned SR files before merge | reviews/qsv49-code-review.md:64-70, reviews/qsv49-gap-analysis.md:51-58 |

## Dispositions

Round 1, at head 7f0e1bea702453aec8c2be3a9d7352c9c1a7ce2b (fix commits e7a0e8f, a186dc9, 7f0e1be). An earlier copy of this file ended with stray tool-call text; this version removes it. The committed copy had already dropped it.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The source check is deleted. `quantity::checked_invariant_tests::integer_conversion_rejects_an_admitted_noninteger_placement` (src/quantity.rs:879-929, tagged FR-122-AC-1) calls the private `convert()` with an admitted scale-one `Target::Integer` placement and asserts `ExpectedIntegerPlacement`. The reviewer re-ran three mutants and all died: the cause swapped to `ValueKindMismatch`, the refusal replaced by `unwrap_or_else`, and the production placement given scale 1 |
| FND-002 | fixed | The self-reading `include_str!` check is deleted, and no `include_str!` or `include_bytes!` remains in src/ or tests/. Reordering the or-pattern to `Integer { .. } \| Decimal(_)` now leaves every AC-1 test passing. The unreachable row moved to the Inspection criterion FR-122-AC-4 |
| FND-003 | still-open | Open until the reviewer's SR files replace the committed copies under reviews/ |
