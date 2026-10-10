---
id: SR-4946
title: "Coder Rust prehandoff review of IR-713 admission storage"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@ec6e801; src/declaration.rs; FR-108-AC-1 through FR-108-AC-5, TC-907"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: reviews
---
# Coder Rust prehandoff review of IR-713

## Summary

Reviewed the authored-declaration admission diff at ec6e801 using the repository conventions and dev-tools Rust review checklist. The reservation carrier and checked storage paths have no remaining infallible growth in the admission call graph; two narrow acceptance-test gaps were found and fixed afterward.

## Verdict

**CONDITIONAL** at the reviewed revision because the two test controls below were absent. Both are fixed at fd8781d. Full gate, QSL consumer compilation, and a union fixture remain for coordinated qualification; this coder prehandoff does not replace independent PR review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC2 had no direct test of a denied native byte request, so a bytes-to-elements reporting regression could pass the new controls. | src/declaration.rs:144 |
| FND-002 | low | AC4 had no public admission test for cancellation retaining its work-limit classification and counter. | src/declaration.rs:215 |

## Dispositions

| ID | Outcome |
| --- | --- |
| FND-001 | fixed fd8781d: `byte_reservation_reports_its_native_request` asserts the denied request and public failure both carry five bytes. |
| FND-002 | fixed fd8781d: `cancellation_keeps_the_work_limit_classification` asserts kind, configured bound, actual counter, and tripped cause. |

## Coverage

The admission audit followed `TypeEnvironment::bounded_with_reservations` through duplicate detection, member-type walks, recursion and generalization traversals, ancestry, field indexing, sealing, flattening, refusal construction, and key join. Tree insertion and clone sites in `contains_ieee` and `type_refusal` are public query paths; admission uses their separate fallible walkers. Construction and evaluation allocations elsewhere in `src/declaration.rs` are outside FR-108 admission. No unsafe code, production panic, compatibility layer, or copied source was added. The CI workflow diff is empty.

Focused evidence after fixes: `cargo test --lib declaration --locked` passed 20 tests; `cargo clippy --lib --tests --locked -- -D warnings`, `cargo fmt --check`, and `git diff --check` passed. The QSV base has Record and Tuple only, so TC-907's Union fixture is not yet evidenced. No QSL consumer compile has run.
