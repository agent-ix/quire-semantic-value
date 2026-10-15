---
id: SR-10865
title: "IR-792 gap analysis of bounded numeric value membership"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; PR #26 4e2020f over 8157909; FR-109, FR-117, QExact FR-380/TC-928, src/declaration/membership.rs, src/supplied_membership.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-792 gap analysis

## Summary

Ticket: IR-792. Planless, diff-scoped analysis of the computed matrix and the new bounded value path. Three new tests exercise all nine operand positions at B+1, B=0, exact-B membership, nonmembership, event denial, precancellation, two roots and cumulative spend. Existing tests cover the no-B path and other walker families. FR-117 retained numeric type-link criteria are pending a separate helper and are outside this value-only change.

## Verdict

**CONDITIONAL** — the mid-helper cancellation mapping added by this diff has no QSV test. This leaves a required edge of the typed result and spend contract unverified.

## Coverage

Plan completion: not assessed.

`quoin matrix --repo . --json` reports FR-109-AC-1/2/3 as tagged and FR-109-AC-7 as an inspection criterion with binders. FR-109-AC-6 and FR-117 criteria are untagged in the repository matrix, but they predate this value-only diff and are not attributed to it. The three new tests were run with `cargo test bounded_numeric --lib` (3 passed); the surrounding membership tests passed 23, with one existing ignored resource probe.

Examined: FR-380-AC-6 and TC-928 step 5 (caller integration); FR-109-AC-1/2/3/7; FR-117-AC-4 (context for unsupported retained links); QSpec FR-323-AC-15 (carrier context). No plan was supplied and no new source stub or unowned behavior was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No QSV test exercises cancellation inside the bounded numeric helper and asserts typed cause, cumulative spend, and rooted locus | src/declaration/membership.rs:269 |

The new mapping of `NumericMembershipFailure::Cancelled(cause)` returns `SuppliedAdmissionError::Cancelled` with a copied spent count, `ValueVisit` and the occurrence locus. The only new cancellation assertion pre-cancels before `budget.charge`, so it stops with spent zero and never enters this mapping. A defect in the kernel-cancellation arm could ship while all three new tests pass. Exercise a deterministic admitted comparison cancellation through the real QSV entry point and assert the original cause and charged spend at its rooted locus.
