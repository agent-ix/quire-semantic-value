---
id: SR-7893
title: "failure-domain review of FR-113 items 8-11"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-semantic-value@331febc30cf86e034b3b0f2af2553413ab9c9cd6; spec/functional/FR-113-composite-value-conformance.md"
review_set: subset
---

## Summary

Ticket: QSL-737. Failure-domain review of the nested descent in FR-113 item 8: leaf
handling, work bounds and the union note.

## Examined scope

- FR-113 item 8 (examined): worklist descent through Option and collection values; composite leaf decided by the ancestor closure; union payload follows later.
- FR-113 items 5-7 (context_only): work_units and ancestor_steps charge only graph validation and closure.

## Verdict

**FINDINGS**. The descent itself uses a worklist, which matches the kernel's
no-recursion rule. Its handling of non-composite leaves, the cost of the walk, and union
members are not stated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Item 8 states the rule only for composite leaves. It does not say what the descent does at a non-composite payload or element: check it with the exact `ValueType::admits`, check a `Reference<T>` by `conforms` (as top-level `admits` does), or trust it. Two implementations will diverge. For example, an `Option<Reference<T>>` holding a subtype reference is refused under the exact check, but the top-level rule accepts the same reference. | FR-113 |
| FND-002 | medium | Item 8 changes `admits` from a check of the outermost value (constant time, as the kernel documents) into a walk over every Option payload and collection element. The walk is not charged to `work_units`, and a `bool` result cannot report a limit. `fill_slots`, `tuple` and the caller's per-occurrence admission repeat it at each enclosing layer. State the bound, or charge the walk. | FR-113 |
| FND-003 | low | The union note says a member payload "follows the same rule". With ancestor admission, a child `C` conforms to two related members (`P` and `M`) of one union, so admission by conformance does not pick a member. Say that the union carrier's own member tag picks the member and conformance is checked only against that member. | FR-113 |

## Dispositions

Round 1, reviewed at 5870800cce45b74a6c549f1da4c855f47d535257.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | fixed | 5870800cc |
| FND-003 | fixed | 5870800cc |
