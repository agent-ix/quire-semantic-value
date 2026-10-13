---
id: SR-7894
title: "dependency review of FR-113 amendment and TC-913"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-semantic-value@331febc30cf86e034b3b0f2af2553413ab9c9cd6; spec/functional/FR-113-composite-value-conformance.md; spec/test-cases/TC-913-nested-composite-conformance-admission.md"
review_set: subset
---

## Summary

Ticket: QSL-737. Dependency review of the new FR-359 `references` edge and of what
AC-3..AC-5 and TC-913 need from other repositories.

## Examined scope

- FR-113 relationships and Dependencies (examined): new `references` edge to QSL FR-359, cross-repo requirement 1.
- FR-113-AC-3 (examined): a collection of P holding a C and a P of equal shared fields keeps both elements.
- TC-913 step 2 and expected result 2 (examined): a Set of P holding a C and a P holds both.

## Verdict

**FINDINGS**. The amendment covers FR-359 cross-repo requirement 1. Admission is
at every depth, and `operand_value` checks a composite operand by conformance. The
amendment matches quire-exact PR #38: `ValueType::admits` and the checked constructors
stay exact in the kernel, and admission is QSV's. Two dependency gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-3 says a collection of `P` holding a `C` and a `P` keeps both elements, and TC-913 builds a `Set` of `P` from them. That is kernel behavior from quire-exact FR-376/FR-377, merged in PR #38 as spec only. On quire-exact main, `equality.rs` still returns `ValueKindMismatch` for composites of different declarations. FR-113 lists no quire-exact dependency, so TC-913 cannot pass until the kernel code lands. Add the dependency, or move the Set vector to the kernel's TC-924/925. | FR-113-AC-3, TC-913 |
| FND-002 | low | The new `ix://agent-ix/quire-spec-language/FR-359` reference targets a document that is not on quire-spec-language `main`. PR #800 (013170a08) merged into `impl/business-model-meanings`, so the edge does not resolve against `main` until that branch lands. | FR-113 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The frontmatter adds depends_on only for quire-exact FR-376. The Dependencies section and AC-6 name FR-376 to FR-378, and AC-6's Set/Bag vector needs FR-377 (canonical key order across declarations in form) and FR-378 (the caller-admitted form path). Add depends_on edges for FR-377 and FR-378. | FR-113, FR-113-AC-6 |

## Dispositions

Round 1, reviewed at 5870800cce45b74a6c549f1da4c855f47d535257.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | deferred | The FR-359 references edge still does not resolve against QSL main. It resolves when impl/business-model-meanings merges to QSL main. FR-113 Dependencies now states that the document lives on that branch. quire validate passes, and naming a branch of a public repo leaks nothing. |
