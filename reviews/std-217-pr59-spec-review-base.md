---
id: SR-13640
title: 'Spec review of QSV PR #59: FR-123 StepRowRef order and hypotheses wording, base checklist'
type: SpecReview
analysis: base
scope: 'agent-ix/quire-semantic-value@695c9f24d; PR #59 diff origin/main...HEAD: spec/functional/FR-123-step-simulation-record-and-obligation.md; ticket STD-217'
review_set: subset
date: '2026-10-21'
relationships:
- target: ix://agent-ix/quire-semantic-value/FR-123
  type: reviews
---

## Summary

Ticket: STD-217. PR quire-semantic-value#59, head 695c9f24db298a43c7d63fc85244f93ed13cff01.

Base checklist over the FR-123 wording change. `quire validate --scope . spec/functional/FR-123-step-simulation-record-and-obligation.md` exits 0. The hypotheses wording matches QSL ADR-020 RE-2 ('invariant clauses whose context is an object type of the concrete model'); the 'not declaration order' correction matches FR-375's sort of `steps`. No AC, encoding or API text changes; the pre-existing Status section is outside the diff.

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| FR-123 StepRowRef | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| FR-123 Use case | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| FR-123 API hypotheses | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| QSpec FR-375 Checked-package node order | quire-specification spec/functional/analysis/FR-375-declare-model-refinement.md | context_only |
| QSpec PR #311 FR-375 tie order | quire-specification spec/functional/analysis/FR-375-declare-model-refinement.md (PR #311 head) | context_only |
| QSpec FR-468 Initial query | quire-specification spec/functional/analysis/FR-468-certify-per-step-refinement-obligations.md (PR #312 head) | context_only |
| QSL ADR-020 RE-2 | quire-spec-language spec/decisions/ADR-020-refinement-mappings-between-qsl-models.md | context_only |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-123 restates FR-375's order, which the PR body says it would not do, and the restatement is already incomplete. The body says the order is 'described by reference to QSpec FR-375 rather than restated, so a later tie-break rule there ... needs no edit here'. But FR-123 adds the parenthetical '(ascending by the node-key digest of each row's first named declaration, then by member name)'. That leaves out FR-375's 'as UTF-8 bytes', and it leaves out the tie-break that QSpec PR #311 (same ticket) adds: protocol `steps` rows that tie (`duplicate`/`lose`, `spawn`/`retire`, and `memory` rows, which name no declaration) are ordered by their alias-free class spelling. Once #311 merges, the parenthetical gives no total order for protocol subjects. The doc comment in src/step_simulation.rs refers to FR-375 and does not restate it. Fix: drop the parenthetical so FR-123 matches the doc comment. | spec/functional/FR-123-step-simulation-record-and-obligation.md:33-35 |
| FND-002 | medium | The sentence this PR rewrites says that the `Initial` record, as well as each `Step` record, lists the invariant clauses as hypotheses ('one `Initial` record and one `Step` record per step row, each listing as hypotheses ...'). The new `StepSimulationRecord` doc says the same for every obligation ('the invariants ... that it may assume'). QSpec FR-468 (PR #312, same ticket) says the initial query 'has no hypotheses beyond the declared types', which is right for a base case: initial states have to establish the invariants, not assume them. As written, the two STD-217 PRs give different `Initial` records, and since `hypotheses` is part of the record's encoding, they key different `operation-contract` identities. Fix: resolve this together with SR-13635 FND-002 on PR #312, then say in FR-123 whether an `Initial` record carries hypotheses (empty, or the clauses) and match the doc comment to it. The type does not need to change. | spec/functional/FR-123-step-simulation-record-and-obligation.md:41-43 |

## Verdict

Not mergeable as is: FND-001 is a one-line text fix in this PR. FND-002 must be settled together with QSpec PR #312's initial-query hypotheses (SR-13635 FND-002) so the two STD-217 PRs agree on the `Initial` record.

## Dispositions

Round 1, head 89764225.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fixed in 89764225: A `StepRowRef` names one step row of the refinement by its zero-based position in the refinement's `steps` array, in the order QSpec FR-375 fixes for it, not the order the rows were written in the source. A refinement has exactly one row per concrete step class (QSL ADR-020 RM-5), so a refinement key and a row position name one row. The position is a name, not an index QSV dereferences. ## Use case QSL checks `CasRef |
| FND-002 | fixed | Fixed in 89764225: QSL checks `CasRefinesCounter` and writes one `Initial` record and one `Step` record per step row, each listing as hypotheses the invariant clauses whose context is an object type of the concrete model. The `Initial` record carries the same hypotheses as every `Step` record, as QSpec FR-468 states for the initial query. CG reads the same values to route them to the SMT backend, and QSL uses their canonical encodings  |
