---
id: SR-7891
title: "base review of FR-113 items 8-11 and TC-913"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@331febc30cf86e034b3b0f2af2553413ab9c9cd6; spec/functional/FR-113-composite-value-conformance.md; spec/test-cases/TC-913-nested-composite-conformance-admission.md"
review_set: subset
---

## Summary

Ticket: QSL-737. Base checklist review of quire-semantic-value PR #15, which amends
FR-113 (items 8-11, AC-3..AC-5) and adds TC-913 for nested composite conformance
admission (QSL FR-359 cross-repo requirement 1). The IDs are well formed and unique:
TC-913 exists on no other remote branch or open PR. Draft PR #12 collides with main on
FR-112 and TC-911, which predates this PR. AC-3..AC-5 trace to TC-913, and TC-913
`verifies` FR-113. Two findings concern the test vectors.

## Examined scope

- FR-113 (examined): Description and Behavior items 8-11.
- FR-113-AC-3 (examined): C admitted at Option/collection/tuple/record P slots; P refused at C/M slots; U refused; Option<C>/Sequence<C> refused at Option<P>/Sequence<P>; collection of P keeps C and P.
- FR-113-AC-4 (examined): nested Option<Sequence<Option<P>>>; one nonconforming element refuses the whole; construction TypeMismatch at field/position.
- FR-113-AC-5 (examined): operand_value admits C/grandchild and Option<P>/Sequence<P> holding one; refuses U and P at C with EqualityOperandSourceNotAdmitted; Reference<T> by kind.
- TC-913 (examined): procedure steps 1-4 and expected results 1-4.

## Verdict

**FINDINGS**. The ID format, traceability, uniqueness and leak checks pass. No
hash, digest or pin is introduced, and the diff has no private-repo text, local paths or
external protected material (CONTENT_RIGHTS.md). TC-913 does not say how its values are
built, and AC-5 uses an ambiguous term.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-913 and AC-3/AC-4 do not say how the nested values are built. The kernel's checked constructors (`OptionValue::present`, `construct_collection`) stay exact (quire-exact FR-378), so the positive vectors cannot be built with them. The discriminating negatives (a `U` inside an `Option<P>`/`Sequence<P>` value, a `P` inside a `C`-typed container) can only be built through `OptionValue::from_admitted`, `form` or `from_admitted_slots`. Today's `admits` already accepts every top-level positive, because kernel `ValueType::admits` compares only the container's declared payload or element type. So only the nested-negative vectors exercise item 8. Name the caller-admitted constructors for each vector. | FR-113-AC-3, FR-113-AC-4, TC-913 |
| FND-002 | low | AC-5 and TC-913 step 4 test "a C or a grandchild" at source type P. In the P/M/C chain of AC-3, C is already P's grandchild, and nothing below C exists. Name the second value (M, or a declared grandchild of C). | FR-113-AC-5, TC-913 |

## Evidence

The review compares the PR diff with QSV main `src/declaration.rs` (`admits` 1734-1744,
`composite_conforms` 1722-1728, `fill_slots` 672-700, `tuple` 2913-2931, `operand_value`
4127-4148), quire-exact main `src/value.rs` (`ValueType::admits`, `OptionValue::present`,
`OptionValue::from_admitted`) and quire-exact FR-378 (PR #38).

## Dispositions

Round 1, reviewed at 5870800cce45b74a6c549f1da4c855f47d535257.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | fixed | 5870800cc |
