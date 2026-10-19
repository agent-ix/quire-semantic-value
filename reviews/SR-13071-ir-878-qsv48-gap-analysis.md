---
id: SR-13071
title: "IR-878 quire-semantic-value PR 48 gap analysis: FR-122 backing and dangling owner tags"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@5f83bbef5b45b10b0144f5a778eb9e676efcbb12 vs origin/main; src/declaration/membership.rs, src/declaration.rs, spec/functional/FR-122-supplied-membership-owner-obligations.md, spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
---

## Summary

Ticket: IR-878. This is a planless, diff-scoped audit. Plan completion: not assessed. `quire coverage --scope . --strict` (quire 0.36.2, engine 0.50.2) was run on origin/main 0b26020 and on PR head 5f83bbe. The spec-review sub-analyses were skipped: the spec edits are a Status paragraph and one TC-908 sentence, and no requirement or AC text changed.

## Reviewed units

- FR-122-AC-2 (examined): An integrated supplied Decimal fixture with independently enumerated N owning logical events succeeds at N and denies its actual next event at N-1 before entering that event's helper; membership, nonmembership and cancellation retain locus and already-spent budget across arguments; constructed allocation and capacity failures keep distinct causes, measured fields and rooted locus.
- FR-122-AC-3 (examined): The QSV supplied caller documents and tests the authenticated finite B for value plus every endpoint; absent B returns `UnsupportedHelper { family: NumericDomain }`.
- FR-122-AC-1 (examined): Deferred result, enum/unit, schedule, comparator, equality operand rows, scale-zero `ExpectedIntegerPlacement`, and retained `IllTypedCause`; it is still unbacked and correctly left untagged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Strict coverage moves FR-122-AC-3 to backed only through tags on Decimal tests. AC-3 is FR-380-AC-6, the Integer/Rational helper contract with absent B returning `NumericDomain`. One tagged test asserts `DecimalDomain`, and the Integer/Rational B tests are untagged. The coverage gain for AC-3 is not established. See SR-13070 FND-001. | src/declaration/membership.rs:3456-3518 |
| FND-002 | medium | Strict coverage moves FR-122-AC-2 to backed, but two of its clauses are not asserted by any tagged test. The helper-not-entered-on-denial clause is shown by surviving mutant M1b, and the spent-across-arguments clause for nonmembership and cancellation by surviving mutant M2. See SR-13070 FND-002 and FND-003. | src/declaration/membership.rs:3572-3604,3720-3736,3836-3856 |
| FND-003 | low | Strict coverage cannot see the dangling ids FR-369-AC-9 (8 TC-906 tests in `src/declaration.rs`) and FR-372-AC-5 (4 tests in `src/declaration/membership.rs`). quire-exact main deleted both rows in #56. Coverage prints them in the same "matches no declared row" advisory as valid cross-repo ids such as FR-369-AC-5, leaves them out of the unbacked/contradicted count, and exits on unbacked rows only. Remove FR-372-AC-5 here, since FR-122-AC-2 supersedes it in the same tests. FR-369-AC-9 should go in a follow-up FR-122-AC-1 batch that adds the missing `ExpectedIntegerPlacement` (scale-zero noninteger) and retained `IllTypedCause` assertions and retags those 8 tests. | src/declaration.rs; src/declaration/membership.rs:3529,3628,3743,3795 |

## Verdict

Strict coverage delta, origin/main to PR head:

- Rows backed: 93/106 to 95/106.
- Unbacked rows: 12 to 10. Only FR-122-AC-2 and FR-122-AC-3 leave the unbacked list.
- FR-122: 0/3 to 2/3.
- Contradicted statuses: 0 to 0.

This matches the author's claim (12 to 10, 0 contradicted). The numeric delta is real, but the backing behind it is not. AC-3's tags are on the wrong family, and AC-2 has two unasserted clauses. The author's skip of FR-122-AC-1 is correct: the TC-906 tests do not assert `ExpectedIntegerPlacement` or every equality-operand row. The FR-109-AC-6 skip was not re-examined beyond confirming that the PR does not tag it. Not mergeable as-is.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-122-AC-3 tags moved from Decimal to the bounded Integer/Rational and absent-B NumericDomain tests. |
| FND-002 | fixed | A denied-event poll assertion kills premature helper entry; prior admitted arguments and cumulative spend 2 kill per-call spend mutants for nonmembership and cancellation. |
| FND-003 | deferred: FR-369-AC-9 follow-up | The four deleted FR-372-AC-5 tags were removed. The eight TC-906 FR-369-AC-9 tags remain for a separate FR-122-AC-1 batch that adds noninteger `ExpectedIntegerPlacement` and retained `IllTypedCause` assertions before retagging. |

## Follow-up verdict

Strict coverage is 95/106 backed, 10 unbacked, 0 contradicted, preserving the main-to-PR delta of +2 backed and -2 unbacked. FR-122 is 2/3 backed: AC-2 and AC-3 are tagged, while AC-1 stays unbacked. Plan completion: not assessed.
