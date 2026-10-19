---
id: SR-13111
title: "Gap analysis of quire-semantic-value PR #49: FR-122-AC-1 backing"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@5e9ab39b1cf29428774d36dc6946e80b832ba002; PR #49 diff origin/main...HEAD (FR-122-AC-1, TC-906, TC-908 status, src/declaration.rs checked_invariant_tests, src/quantity.rs:796-802)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: references
---
# Gap analysis of quire-semantic-value PR #49

## Summary

Ticket: IR-886 (parent IR-878). Repository-driven audit scoped to the PR diff; plan completion
was not assessed. FR-122-AC-1 moves from untagged to tagged with ten tests. Each row of the
FR-369 QSV cause table and each clause of the AC was checked against an asserting test.
Rows asserted behaviorally: DeferredResultNotAdmitted for direct admission and record field,
EqualityEnumVariantUnresolved, EqualityUnitUnresolved (comparison and conversion),
EqualityScheduleMismatch, ScheduledComparisonRefused with its `IllTypedCause`,
EqualityOperandSourceNotAdmitted, EqualityOperandNonIntegralDecimal,
EqualityQuantityConversionRejected with its `IllTypedCause`, EqualityConversionShapeMismatch,
EqualityOperandTargetNotAdmitted, prior non-result (Undefined, Incomplete, prior Refused) and
charge stops on the comparison and conversion paths, and successful integer placement.
No FR-369-AC-9 or FR-372-AC-5 tag remains in `src/` or `spec/`.

## Verdict

**CONDITIONAL** — no untagged criterion and no high finding. However, the tuple
`DeferredResultNotAdmitted` clause has no assertion under the AC-1 tag. Two table rows are
backed only by source-text inspection that is tagged as Test evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No FR-122-AC-1-tagged test asserts that a failed deferred tuple result returns `DeferredResultNotAdmitted`; the tuple test covers only success and a prior refusal. Mutating `evaluate_tuple` to skip `admitted` survives all ten AC-1 tests (killed only by FR-113-AC-4's `evaluate_record_and_tuple_admit_members_by_conformance`) | FR-122-AC-1, src/declaration.rs:5893-5965, src/declaration.rs:4278-4293 |
| FND-002 | medium | The `EqualityQuantityNonExactPlacement` and `ExpectedIntegerPlacement` rows are backed only by `include_str!` source-text checks tagged as Test evidence of a Verification: Test AC. That overstates the strict coverage. `ExpectedIntegerPlacement` can be tested behaviorally at the private `convert` seam (reproduced). The NonExact row needs a seam, or an honest Inspection disposition | FR-122-AC-1, src/declaration.rs:6054-6090 |
| FND-003 | low | TC-906 Test Procedure step 2 still says to "exercise" an unexpected nonexact placement and assert its payload, while its new Status says that row is only inspected. Procedure and Status disagree | TC-906, spec/test-cases/TC-906-qsv-checked-invariant-causes.md:31-41,67-74 |

## Coverage

- Reconciliation: quire matrix (quire 0.36.2, engine 0.50.2); quoin matrix (quoin 0.28.3) read with no run evidence
- Plan completion: not assessed
- Criteria (106): base ffdeffd tagged 92, untagged 3, method-without-symbol 11; PR tagged 93, untagged 2 (FR-109-AC-6, FR-121-AC-6), method-without-symbol 11. FR-122-AC-1 is untagged on base and tagged on the PR. The author's "95/106 -> 96/106 backed, 10 -> 9 unbacked" figures could not be reproduced from either matrix, but the +1 delta agrees
- Evidence: no run evidence
- Untraced behaviors / stubs: 0 new; two source-text checks (FND-002)
- Semantic review: ran over FR-122-AC-1 (dispatch brief), with 16 mutations plus one equivalent refactor
- Spec status wording: FR-122 Status is honest about which rows are inspected rather than exercised. TC-908's FR-109-AC-6 "remains planned" note is honest and matches the matrix (untagged)

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-1 tuple test now asserts failed deferred admission returns `DeferredResultNotAdmitted`; bypassing tuple admission failed the test. |
| FND-002 | partially fixed; need decision | `ExpectedIntegerPlacement` is asserted behaviorally and its wrong-cause mutant failed. Both source-reading tests are gone. The unreachable `EqualityQuantityNonExactPlacement` row needs an Inspection disposition or test seam. |
| FND-003 | fixed | TC-906 step 2 no longer claims to exercise the unreachable nonexact placement and its Status names the unverified row. |
