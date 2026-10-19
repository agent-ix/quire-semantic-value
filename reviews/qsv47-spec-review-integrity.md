---
id: SR-13041
title: "IR-878 QSV PR 47 spec-review integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value; spec/functional/FR-109-bound-supplied-value-traversal.md, spec/functional/FR-122-supplied-membership-owner-obligations.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-878 QSV PR 47 spec-review integrity

## Summary

Ticket: IR-878. Integrity lens over head aea435e16a77c862b7f52345afb253521ce92df1. It checks whether the re-homing leaves QSV's own spec set consistent: the TC documents that verify FR-109 and FR-369-AC-9 today, the FR-122 statement, and the FR-109 provenance notes. The unchanged TC-906 and TC-908 were read because the diff changes what they must verify.

## Reviewed scope

- `FR-122` — examined: Description claims it "carries quire-exact FR-369-AC-9, FR-372-AC-5 and the QSV portion of FR-380-AC-6".
- `FR-109-AC-6` — examined: QSV-scoped supplied path; no evaluator meter or conversion obligations.
- `FR-109-AC-8` — examined: evaluator-charge clause removed.
- `TC-906` — examined: scope and status bind QSV tests to `FR-369-AC-9`.
- `TC-908` — examined: step 6 (external evaluator meter, conversion-initiated helper), step 8 ("or evaluator charge") and Expected Results ("evaluation accounting is unchanged").
- `FR-109` — context_only: Dependencies note on QSL FR-391.

## Verdict

**FAIL.** The FR text is internally consistent. The QSV test-case documents were not moved with it: one still binds to a criterion quire-exact #56 deletes, and one still assigns QSL-owned evaluator controls to QSV.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-906 still scopes and binds its tests to quire-exact `FR-369-AC-9`, which quire-exact PR #56 deletes. No TC verifies FR-122, and TC-906 has no relationship to FR-122. After #56 merges, QSV's verifying TC names a criterion that no longer exists. | spec/test-cases/TC-906-qsv-checked-invariant-causes.md:14-21, spec/test-cases/TC-906-qsv-checked-invariant-causes.md:68-71 |
| FND-002 | medium | TC-908 (verifies FR-109) still tells QSV to observe an external evaluator meter, assert no evaluated FunctionCall, run a conversion-initiated shared helper (step 6), check "no evaluator charge" (step 8) and confirm "evaluation accounting is unchanged". The PR moves these obligations to QSL FR-391-AC-1/AC-3. TC-908 now contradicts the narrowed FR-109-AC-6/AC-8, and QSV has no conversion phase to exercise. | spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:43-47, spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:57-66, spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:83 |
| FND-003 | low | The FR-122 Description says it carries FR-372-AC-5 whole. Its "evaluator meter change" clause went to QSL FR-391-AC-3, as quire-exact #56's own note states. Only FR-380-AC-6 is labelled "the QSV portion". | spec/functional/FR-122-supplied-membership-owner-obligations.md:19 |
| FND-004 | low | The FR-109 Dependencies note records only the FR-109-AC-6 relocation. The removed FR-109-AC-8 "evaluator charge" clause, now QSL FR-391-AC-3 per the PR body, has no provenance note in FR-109. | spec/functional/FR-109-bound-supplied-value-traversal.md:135-137, spec/functional/FR-109-bound-supplied-value-traversal.md:121 |

## Coverage

There is no `spec/tests.md` in this repo, so no matrix rows or counts were affected. The TC findings above are the traceability gap.

Plan completion: not assessed
