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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Narrowed FR-122-AC-1 says "each reachable equality operand failure", but "reachable" is undefined. quire-exact FR-369 calls every conversion-shape, admission, unresolved-unit, nonintegral and unexpected-placement failure a checked-program internal fault, so a reader could treat every operand row as unreachable and excluded. Name the single exclusion instead, e.g. "each equality operand failure other than the non-exact placement row in AC-4" | FR-122-AC-1, spec/functional/FR-122-supplied-membership-owner-obligations.md:37 |

## Dispositions

Round 1, at head 7f0e1bea702453aec8c2be3a9d7352c9c1a7ce2b. An earlier copy of this file ended with stray tool-call text; this version removes it.

`quire matrix` now has 107 criteria: tagged 93, untagged 2 (FR-109-AC-6, FR-121-AC-6), method-without-symbol 12. Against 5e9ab39, FR-122-AC-4 is the new row and is method-without-symbol (Inspection, no symbol); everything else is unchanged.

- FR-122-AC-1 has nine tagged tests.
- `quire matrix --strict` exits 1; the two untagged rows predate this PR.
- `quire validate` over spec/ and reviews/ passes.
- `make fmt-check` and `make lint` pass, and `cargo test --lib` passes 132 (1 ignored).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The AC-1 tuple test now asserts a failed position. The `evaluate_tuple`-skips-`admitted()` mutant now fails `deferred_record_and_tuple_evaluation_preserve_admission_and_prior_refusal` |
| FND-002 | fixed | `ExpectedIntegerPlacement` is asserted behaviorally. `EqualityQuantityNonExactPlacement` moved out of AC-1 into the Inspection criterion FR-122-AC-4, which carries the configurable-target note. AC-4's facts hold on the tree: `operand_value` hard-codes `&QuantityTarget::Exact` (src/declaration.rs:5665), `convert_quantity` maps it to `Target::Exact` (src/quantity.rs:436), and `convert` returns only `ConvertedValue::Exact` for it. Together, AC-1 and AC-4 carry every QSV row of the quire-exact FR-369 table on main. All cause-swap mutants die under AC-1-tagged tests |
| FND-003 | fixed | TC-906 step 2 no longer exercises the nonexact placement. Step 4, Expected Results and Status describe the AC-4 inspection consistently |
| FND-004 | still-open | New this round. Replace "reachable" with the explicit AC-4 exclusion |
