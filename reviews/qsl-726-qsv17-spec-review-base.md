---
id: SR-8697
title: "base review of FR-114 and TC-914"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@b90ac1fc50fef599a4ad59c4c1554bd717da6df1; spec/functional/FR-114-locate-model-operation-clauses.md; spec/test-cases/TC-914-model-operation-clause-origin.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-114
    type: reviews
---
# Base review of FR-114 and TC-914

## Summary

Ticket: QSL-726. Base checklist review of quire-semantic-value PR #17. The PR
adds FR-114, which gives `Origin` a variant for a model operation's
`requires` or `ensures` clause, and TC-914, which verifies it.

The IDs are well formed. FR-114 and TC-914 are unique: no other remote branch
or PR head has either id, including open draft #12. TC-914 `verifies` FR-114,
and FR-114-AC-1 is tagged by one test.

## Examined scope

- FR-114 (examined): Description, Use case, and Behavior items 1 to 3.
- FR-114-AC-1 (examined): equality by operation, kind, position and path;
  order by operation, kind (`Requires` before `Ensures`), position and path;
  never equal to a `StateClause` origin.
- TC-914 (examined): procedure steps 1 to 4 and expected results 1 to 4.

## Verdict

**FINDINGS**. The target design is stated. The origin names the operation,
the clause kind and the zero-based position, and it is distinct from
`StateClause`. That is what the QSL consumer needs to map the origin to
`Operation.pre[k]` or `Operation.post[k]`. That mapping is QSL's behaviour,
and FR-114's Dependencies section rightly leaves it to
`agent-ix/quire-spec-language`.

The hash, digest and pin checks pass, and `Cargo.toml` is untouched. The text
has no private-repo content or external protected material
(CONTENT_RIGHTS.md).

`quire validate` rejects FR-114, because AC-1 bundles three obligations. Two
wording findings and one checklist finding follow.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `quire validate --scope . spec/functional/FR-114-...md` fails with `[ac:non-singular] criterion bundles more than one obligation` at line 39. It also warns `[ac:non-canonical-shape]`. The sibling FR-112 and FR-113 validate clean. AC-1 holds three obligations: equality, ordering and distinctness from `StateClause`. Split them into AC-1 to AC-3, mirroring Behavior items 1 to 3. Update TC-914's Scope line and the test's `#[trace]` tags to match. | FR-114-AC-1; spec/functional/FR-114-locate-model-operation-clauses.md:39 |
| FND-002 | medium | The Description's last sentence puts an obligation on the caller, not on this crate: "A checking refusal, an evaluation outcome or a loss record located in such a clause carries that origin and no other." QSV builds no such refusal, outcome or record for model operation clauses. No AC or test covers the sentence, and none could in this repo. Restate it as the variant's purpose, or move it to QSL's spec. | FR-114; spec/functional/FR-114-locate-model-operation-clauses.md:13-15 |
| FND-003 | low | Behavior item 1 says `operation` holds "the operation's identifying text". It does not say that the text must identify the operation uniquely within the package. The Use case example uses the bare name `place`. Two models with an operation `place` would then share origins and mislocate refusals. The QSL consumer actually passes the operation's graph node key. State that the text is opaque to QSV and must identify one operation within the package. | FR-114; spec/functional/FR-114-locate-model-operation-clauses.md:26-28 |
| FND-004 | low | The `Verification` cell of FR-114-AC-1 reads `Test (TC-914)`. The checklist requires the cell to name a method only, with no `TC-` id. The TC link is already carried by TC-914's `verifies` relationship and the test's trace tag. | FR-114-AC-1; spec/functional/FR-114-locate-model-operation-clauses.md:39 |
