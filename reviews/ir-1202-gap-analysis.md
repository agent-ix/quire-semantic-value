---
id: SR-16683
title: "Gap analysis of QSV PR #80 AbsenceMode placement"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/absence.rs, src/lib.rs; QSL FR-400 plain-value placement"
review_set: subset
---
# Gap analysis of QSV PR #80

## Summary

Ticket: IR-1202. Planless, PR-diff-scoped audit of the new value, its test, and the governing QSL FR-400 placement rule. QSL-1375 tracks the downstream cleanup, which is not yet complete. Plan completion was not assessed.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-400 | examined | QSL spec/functional/FR-400-separate-checked-terms-from-the-checker.md | A plain value type that a moved record carries and that `qsl-foundation` or `qsl-forms` defines moves down into SV in the same change and leaves no copy or re-export behind. |
| src/absence.rs:33 | examined | src/absence.rs | `fn each_absence_mode_spells_its_authored_keyword() {` |
| FR-400-AC-5 | context_only | QSL spec/functional/FR-400-separate-checked-terms-from-the-checker.md | No `qsl_semantics::` path to a moved item resolves: `qsl_semantics::check::CheckedBody`, `qsl_semantics::check::CheckedGraph`, and `qsl_semantics::model::key::DeclarationKey` each fail to compile from another crate, and no F or layer-2 path names a value that moved to SV. |

## Verdict

**FAIL** — the new source has a governing external requirement and a real three-mode unit test, but the cross-repository removal and consumer repointing required by that requirement are absent at this reviewed head. QSL-1375 tracks the removal but does not itself remove the duplicate. This is the same ownership defect recorded in the code review, not a separate code defect.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-400 move is incomplete at this head: QSL still defines and uses `qsl_foundation::absence::AbsenceMode`, so the new SV type is not yet the single source that checked terms and parsed forms share. | src/absence.rs:9; QSL qsl-foundation/src/absence.rs:11; QSL FR-400 Closure rule 2 |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.29.1) and manual PR-diff criterion-to-test inspection. FR-400 is external to this repository, so QSV's computed matrix cannot bind it.
- Plan completion: not assessed
- New three-mode spelling test: 3 exact assertions, all execute and pass.
- Untraced behaviors / stubs in PR diff: 0; cross-repository ownership gap: 1.
- Semantic review: skipped as an optional repository-wide pass; the requested three-mode assertion was inspected manually.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Round 1: Peter approved this temporary duplicate for the staged AbsenceMode move, expiring when QSL-1375 merges. The cross-repository gap remains until QSL repoints consumers and deletes its old definition, but this explicit owner exception permits the QSV prerequisite PR to merge without a code change. |
