---
id: SR-11158
title: "Gap analysis of IR-713 public supplied reservation evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/declaration.rs; src/declaration/membership.rs; src/supplied_membership.rs; tests/union_staging.rs; FR-109-AC-5; TC-908; ticket IR-713"
review_set: subset
---

## Summary

The public denial test covers the measured request and retained budget, but its rooted-locus assertion is too weak to detect loss of the child location. The trace-binding issue is recorded separately in SR-11157.

## Verdict

**CONDITIONAL** — One medium acceptance-evidence gap remains.

## Scope examined

- `FR-109-AC-5` (examined): An actual fallible supplied-worklist reservation denial reports its measured request and native unit; checked-size overflow has its own classification. Neither becomes a traversal limit, declaration-storage failure or invalid value. Retry after removing the denial yields the same membership outcome.
- `TC-908` (examined): Deny an actual fallible traversal-worklist reservation without replacing membership logic. Compare the reported allocation amount/unit with the real reservation request. Separately trigger checked representation/size overflow. Retry without denial and compare membership with the fresh run.
- `src/declaration.rs` (examined): The public callback forwards to the supplied walk; the default method still uses `GlobalReservations`.
- `src/declaration/membership.rs` (examined): A two-child union payload reserves the pending worklist before reverse-order scheduling; the failure locus is `Child(UnionPayload(1))` at the rooted argument.
- `src/supplied_membership.rs` (examined): A path append has its own measured allocation and capacity variants and retains the child locus.
- `tests/union_staging.rs` (examined): The new test observes a request of two additional elements and spend of two, then retries with the default admission method.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Public denial test checks only the locus argument, so returning the root or a wrong child locus would pass | tests/union_staging.rs:297 |

## Coverage

Plan completion: not assessed.

`quire matrix --scope . --strict --format json` found one repository-wide untagged criterion outside this diff; existing internal tests already tag `FR-109-AC-5`. The new public test currently tags `FR-108-AC-1` and `FR-109-AC-4`, as recorded in SR-11157. It checks `arena.resolve(locus).unwrap().argument() == 7`, but never asserts the `MembershipPart` or path steps. A change that reports the root value locus for a failed first child reservation would retain argument 7 and pass. Assert the actual child part and steps so the public API preserves the failure location that the walk supplies.

The focused public test passed (one test, zero failures). Checked capacity and cancellation are implemented below this forwarding seam and covered by existing internal tests; this slice adds no public capacity or cancellation fixture. QSL consumer evidence for zero evaluation charge remains outside this QSV diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At follow-up head `dd3e2ad709f0854588d0ecbf80e5a8a5c492f74b`, the test resolves the returned locus and asserts argument 7, `MembershipPart::Child(PathStep::UnionPayload(1))`, and zero ancestor steps. Returning the root or another child now fails the test. |

## Follow-up verdict

**PASS** — The only finding is fixed. The focused public test passes, and no production code changed in the follow-up diff.
