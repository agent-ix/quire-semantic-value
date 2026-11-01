---
id: SR-16685
title: "IR-1189 QSV gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/declaration.rs; tests/type_environment_reservation.rs"
review_set: subset
---

# IR-1189 QSV gap-analysis

## Summary

Ticket: IR-1189. PR quire-semantic-value#81. Candidate: cbe02bff17cdbe0ef4b8818fd1cbe241237682b0. Diff origin/main...HEAD against 3cc0b27e19a398f336152ada05972072053fa47e. Review scoped to changed behavior by dispatch. Rust lane folded into code-review.

The append fast path retains strict ordering, duplicate replacement, and reserve-before-mutation. All five reused DFS stacks drain by pop before the next root; early errors return admission without publishing state. No work-budget charges moved. Both scaling oracles reject the measured baseline, with comparison count and reservation count rather than timing as assertions. No implementation defect found.

## Examined scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| OrderedIndex::try_insert (src/declaration.rs) | examined | if self.entries.last().is_none_or(|(last, _)| last < &key) { |
| DFS path reuse (src/declaration.rs) | examined | let mut path: Vec<(&NodeKey, usize)> = Vec::new(); |
| FR-108-AC-1 (spec/functional/FR-108-report-registry-allocation-failure.md) | examined | At an actual fallible registry, index, or traversal reservation boundary after the storage redesign, an injected allocator denial returns the allocation member of `EnvironmentFailure` with the exact request amount and its byte or additional-element unit. No configured bound, setting, or declaration cause is present. The same declaration set admits when the denial is removed. |
| ascending_registry_insertion_uses_linear_key_comparisons (src/declaration.rs) | examined | ascending_registry_insertion_uses_linear_key_comparisons |
| record_environment_admission_scales_without_per_root_scratch_reservations (tests/type_environment_reservation.rs) | examined | record_environment_admission_scales_without_per_root_scratch_reservations |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | ascending_registry_insertion_uses_linear_key_comparisons is tagged FR-108-AC-1 but never injects allocator denial or asserts the typed failure request/unit and retry required by that criterion. Correct the trace binding to a criterion it actually verifies (or author an owned performance criterion). | src/declaration.rs:8737 |
| FND-002 | medium | record_environment_admission_scales_without_per_root_scratch_reservations is tagged FR-108-AC-1 but never injects allocator denial or asserts the typed failure request/unit and retry required by that criterion. Correct the trace binding to a criterion it actually verifies (or author an owned performance criterion). | tests/type_environment_reservation.rs:152 |

## Verdict

CONDITIONAL: two incorrect new test trace bindings.

## Coverage

Plan completion: not assessed

Computed quire matrix inspected for FR-108 AC-1 through AC-5. Existing admission storage behavior remains owned by FR-108; no new stub or unowned production behavior found in this diff. No applicable AssuranceProfile found. No spec/plan files changed, so spec-review and subanalyses do not apply. Repository-wide gap claims are outside the dispatched scope. Optional broad semantic audit not run; focused semantic/order/budget/allocation inspection explicitly requested.

No Cargo, full suites, or Kani run under the user's speed rule. Parent supplied passed focused fmt, all-target Clippy -D warnings, five public reservation tests and ascending-index scaling test; named ancestry/recursion checks pending with parent. Independently ran git diff --check (passed). Baseline counts supplied: ascending128 comparisons889 versus127; independent-record reservations2947/11779/47107 versus2439/9735/38919 at128/512/2048.
