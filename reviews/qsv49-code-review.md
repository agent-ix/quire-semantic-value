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

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Replaced the source-reading check with an in-module `convert()` test using an admitted scale-one placement; the wrong-cause mutant failed. |
| FND-002 | fixed; need decision | Deleted the source-reading check. `Target::Exact` can only return `ConvertedValue::Exact`, so the nonexact defensive row has no behavioral assertion; Inspection disposition or a test seam needs owner choice. |
