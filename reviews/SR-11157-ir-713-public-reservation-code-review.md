---
id: SR-11157
title: "Code review of IR-713 public supplied reservation seam"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration.rs; src/declaration/membership.rs; src/supplied_membership.rs; tests/union_staging.rs; FR-108-AC-1; FR-109-AC-4; FR-109-AC-5; TC-908; ticket IR-713"
review_set: subset
---

## Summary

Reviewed the frozen `594f956` diff against `542bc097`. The public callback forwards to the existing supplied walk and reaches actual worklist and path-arena growth. The new integration test claims two criteria that its assertions do not exercise.

## Verdict

**CONDITIONAL** — One medium trace-binding defect remains. The focused public test passes.

## Scope examined

- `FR-108-AC-1` (examined): At an actual fallible registry, index, or traversal reservation boundary after the storage redesign, an injected allocator denial returns the allocation member of `EnvironmentFailure` with the exact request amount and its byte or additional-element unit. No configured bound, setting, or declaration cause is present. The same declaration set admits when the denial is removed.
- `FR-109-AC-4` (examined): Invalid kind/type and member/payload failures permitted by the owning supported-family contracts return their typed membership cause and locus, never a partial admission. Released nested construction custody is preserved; a helper cannot manufacture extra payload visits.
- `FR-109-AC-5` (examined): An actual fallible supplied-worklist reservation denial reports its measured request and native unit; checked-size overflow has its own classification. Neither becomes a traversal limit, declaration-storage failure or invalid value. Retry after removing the denial yields the same membership outcome.
- `TC-908` (examined): Deny an actual fallible traversal-worklist reservation without replacing membership logic. Compare the reported allocation amount/unit with the real reservation request. Separately trigger checked representation/size overflow. Retry without denial and compare membership with the fresh run.
- `src/declaration.rs` (examined): New `Fn(StorageRequest) -> bool` policy adapter and public `admit_supplied_with_reservation` forwarder; production default retains `GlobalReservations`.
- `src/declaration/membership.rs` (examined): Frontier growth calls `reserve_elements`; child path growth forwards the policy to `MembershipPathArena::append_child`.
- `src/supplied_membership.rs` (examined): Path arena growth returns typed allocation or capacity with its request and locus; the budget charges events independently.
- `tests/union_staging.rs` (examined): Public denial fixture asserts a measured additional-elements request, rooted argument and retained spend, then succeeds through the default path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public supplied-reservation test traces to registry allocation and invalid membership instead of the supplied-reservation criterion | tests/union_staging.rs:259 |

## Evidence

`public_supplied_reservation_denial_retains_request_and_locus` observes `SuppliedAdmissionError::Allocation`, so it does not exercise `EnvironmentFailure::Allocation` required by `FR-108-AC-1`. It supplies a valid union value, so it does not exercise an invalid cause required by `FR-109-AC-4`. Its denial, request and retry are the public evidence for `FR-109-AC-5`; tag the test to that criterion and remove the false bindings. The incorrect tags can make an acceptance audit credit these two unrelated criteria from a test that would still pass if their behavior regressed.

The focused command `cargo test --test union_staging public_supplied_reservation_denial_retains_request_and_locus` passed: one test, zero failures. No workflow file changed; no full gate was run in this review.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At follow-up head `dd3e2ad709f0854588d0ecbf80e5a8a5c492f74b`, the test's doc trace and `#[trace]` now name only `TC-908` and `FR-109-AC-5`. The unrelated `FR-108-AC-1`, `FR-109-AC-4`, and `TC-907` claims are removed. |

## Follow-up verdict

**PASS** — The only finding is fixed. The follow-up changes only this test's trace tags and locus assertions; the focused test and `git diff --check` pass.
