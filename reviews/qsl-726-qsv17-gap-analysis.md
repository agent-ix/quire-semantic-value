---
id: SR-8696
title: "Gap analysis of quire-semantic-value PR #17: FR-114 and TC-914"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@b90ac1fc50fef599a4ad59c4c1554bd717da6df1; FR-114, FR-114-AC-1, TC-914, src/location.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-114
    type: reviews
---
# Gap analysis of quire-semantic-value PR #17

## Summary

Ticket: QSL-726. This is a planless audit of FR-114 and FR-114-AC-1 against
the computed matrix (`quoin matrix --repo .`, quoin 0.28.3). FR-114-AC-1 is
`tagged` by one test, `a_model_operation_clause_origin_is_its_own`, which
carries `#[trace("TC-914", "FR-114-AC-1")]`. The binding is correct. There are
no stubs and no `todo!`.

## Verdict

PASS, with one low finding.

- FR-114 Behavior items 1 to 3 are implemented by the variant and the derived
  traits.
- Nothing in the code is unowned. `OperationClauseKind` is named by Behavior
  item 1.
- The "then path" clause of the ordering is the existing `Location` order. It
  is covered by the existing path tests in `src/location.rs` and does not
  depend on the origin.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The test does not run TC-914 step 4 as written. The TC compares a `ModelOperationClause` for operation `Q` at position 2 with `Origin::StateClause { clause: "Q", index: 2 }`. The test compares `place`/`Requires`/0 with `StateClause { clause: "place", index: 0 }`. The assertion is equivalent, but the procedure and the test disagree. A reader checking the TC against the test finds different vectors. Make one match the other. | src/location.rs:365-369; spec/test-cases/TC-914-model-operation-clause-origin.md:27-28 |

## Coverage

- FR-114-AC-1: `a_model_operation_clause_origin_is_its_own`. The binding is correct.
- Reverse gap: none.
- Semantic review: intent, test and code agree. The oracle-strength analysis is in SR-8695.

Plan completion: not assessed
