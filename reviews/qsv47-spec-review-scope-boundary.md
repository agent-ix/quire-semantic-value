---
id: SR-13042
title: "IR-878 QSV PR 47 spec-review scope-boundary"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-semantic-value; spec/functional/FR-109-bound-supplied-value-traversal.md, spec/functional/FR-122-supplied-membership-owner-obligations.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-878 QSV PR 47 spec-review scope-boundary

## Summary

Ticket: IR-878. Scope-boundary lens over head aea435e16a77c862b7f52345afb253521ce92df1, applying the binding planner ruling: a criterion belongs to the repo that owns the code path it names, and criterion text is not weakened. Ownership was checked by grep over QSV `src/`. QSV has no conversion phase (`SuppliedAdmissionPhase` has only `Membership`) and no evaluator meter. The checked-invariant producers (src/declaration.rs, src/quantity.rs, src/enumeration.rs) and the supplied caller (src/supplied_membership.rs, src/declaration/membership.rs) are in QSV.

## Reviewed scope

- `FR-122-AC-1` — examined: QSV producer paths; all named causes are produced in QSV src.
- `FR-122-AC-2` — examined: QSV integrated Decimal caller and projection; QSV-owned.
- `FR-122-AC-3` — examined: QSV supplied caller B handling, plus a clause on "the exact helper".
- `FR-109-AC-6` — examined: old vs new. Old text (one initiating owner per event/helper, evaluator meter unchanged, conversion work not charged as membership, no FunctionCall) moved verbatim to QSL FR-391-AC-1. The new QSV text keeps every part QSV can execute: one charge per event to the supplied budget, original budget and handle through helpers, no conversion debit, no meter or FunctionCall. It adds the denied-event spend/locus. Not weakened.
- `FR-109-AC-8` — examined: only the evaluator clause was removed; the QSV path text is intact.

## Verdict

**PASS WITH FINDINGS.** The QSV/QSL split of FR-109-AC-6, FR-109-AC-8, FR-372-AC-5 and FR-380-AC-6 follows the ownership rule, and no QSV-executable obligation was dropped. One clause is still allocated to a non-owner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-122-AC-3 asserts that "the exact helper ... do[es] not default B". That names quire-exact's code path, not QSV's. QSL FR-391-AC-2 (PR #995) asserts the same clause. quire-exact #56 deletes FR-380-AC-6 and keeps this only as FR-380 Behavior prose with no AC. So the obligation sits in two non-owner repos and in no owner criterion. | spec/functional/FR-122-supplied-membership-owner-obligations.md:39 |

## Coverage

Not applicable to this lens; see SR-13044.

Plan completion: not assessed

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1. FR-122-AC-3 now asserts only the direct QSV caller's no-default and absent-B result. It points the exact-helper no-default control to quire-exact FR-380-AC-7, which exists on quire-exact #56 head c58ab92d. The leftover copy of that clause in QSL FR-394-AC-2 is a defect in QSL #995, flagged by the author for the QSL owner. It is not in QSV #47. |
