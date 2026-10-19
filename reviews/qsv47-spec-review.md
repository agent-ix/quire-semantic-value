---
id: SR-13040
title: "IR-878 QSV PR 47 spec-review base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value; spec/functional/FR-109-bound-supplied-value-traversal.md, spec/functional/FR-122-supplied-membership-owner-obligations.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-878 QSV PR 47 spec-review base

## Summary

Ticket: IR-878 (item 4, owner allocation of unbacked strict-coverage criteria). Independent non-author review of frozen head aea435e16a77c862b7f52345afb253521ce92df1, scoped to `git diff origin/main...HEAD` (two spec files, spec-only). Review set: subset (base, integrity, scope-boundary, ears-conformance) plus gap-analysis in a separate artifact. No applicable AssuranceProfile was found. `quire validate --scope . 'spec/**/*.md'` (quire 0.36.2, engine 0.50.2) exits 0 with no warnings on the changed files. Cross-references were resolved against quire-exact main and PR #56 head 159f170d, and QSL main and PR #995 head 85e6ccf1.

## Reviewed scope

- `FR-122-AC-1` — examined: verbatim copy of quire-exact FR-369-AC-9 (QSV deferred field/tuple result `DeferredResultNotAdmitted`, distinct enum-variant/unit rows, equality operand rows, `ExpectedIntegerPlacement`, retained `IllTypedCause`).
- `FR-122-AC-2` — examined: quire-exact FR-372-AC-5 with the "evaluator meter change" clause removed and "AC-3" qualified as "quire-exact FR-372-AC-3".
- `FR-122-AC-3` — examined: quire-exact FR-380-AC-6 with "leaves evaluator charges unchanged" and the QSL omitted-B default sentence removed.
- `FR-109-AC-6` — examined: new QSV-scoped text naming `src/supplied_membership.rs`; old text moved verbatim to QSL FR-391-AC-1 on PR #995.
- `FR-109-AC-8` — examined: "evaluator charge" clause removed; QSL FR-391-AC-3 on PR #995 carries it.
- `FR-122` — examined: frontmatter, relationships, Description, Behavior, Dependencies, Status.
- `FR-109` — context_only: Dependencies note added for QSL FR-391.

## Verdict

**FAIL (one blocker).** Moved text is faithful: FR-122-AC-1 is verbatim, AC-2 and AC-3 drop only evaluator/QSL clauses that reappear in QSL FR-391-AC-1..3, and old FR-109-AC-6 reappears verbatim as FR-391-AC-1. The QSV-owned identifiers `DeferredResultNotAdmitted`, `ExpectedIntegerPlacement`, `IllTypedCause` and `UnsupportedHelperFamily::NumericDomain` exist in QSV `src/`. IDs and AC table format are correct. QSV has no `spec/tests.md`; there are no trace rows or counts to update. The blocker is that the QSL target id is already taken on QSL main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `ix://agent-ix/quire-spec-language/FR-391` already resolves on QSL main to a different requirement, "Name a domain-package population as the operand of allInstances and lookup in a state clause" (#982). PR #995's merge base predates it. A trial merge of #995 into QSL main yields two FR-391 files. So both QSV links point at the wrong requirement now, and will keep doing so unless #995 renumbers, which would then break these links. | spec/functional/FR-109-bound-supplied-value-traversal.md:135, spec/functional/FR-122-supplied-membership-owner-obligations.md:44 |
| FND-002 | low | FR-122-AC-3 names `UnsupportedHelper::NumericDomain`, which is not a QSV identifier. The QSV result is `SuppliedAdmissionError::UnsupportedHelper { family: UnsupportedHelperFamily::NumericDomain, .. }`. The inherited shorthand leaves the asserted result open to reading. | spec/functional/FR-122-supplied-membership-owner-obligations.md:39, src/supplied_membership.rs:410-440 |

## Coverage

`quire coverage --scope . --strict`: origin/main 93/103 rows backed with 9 unbacked; head 93/106 rows backed with 12 unbacked (+3: FR-122-AC-1..3, PLANNED as expected). 0 contradicted statuses on both. FR-109-AC-6 is unbacked on both. See the gap-analysis artifact.

Plan completion: not assessed

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (head df8a3dae). QSL #995 (head 6862c2f0, rebased on QSL main) renumbered the requirement to FR-394. QSL main has FR-390..393 only, and no other open QSL PR adds FR-394. QSV FR-109:135 and FR-122:44 now link FR-394. |
| FND-002 | fixed | Round 1. FR-122-AC-3 now names `SuppliedAdmissionError::UnsupportedHelper { family: UnsupportedHelperFamily::NumericDomain, .. }`. |
