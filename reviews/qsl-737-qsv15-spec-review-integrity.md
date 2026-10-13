---
id: SR-7892
title: "integrity review of FR-113 items 8-11"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@331febc30cf86e034b3b0f2af2553413ab9c9cd6; spec/functional/FR-113-composite-value-conformance.md"
review_set: subset
---

## Summary

Ticket: QSL-737. Integrity review of the FR-113 amendment for completeness and internal
consistency, checked against QSV main code. QSV's deferred construction path is missing
from the amendment, item 8 reads two ways on member descent, and item 10 contradicts
item 3 on unknown keys.

## Examined scope

- FR-113 item 8 (examined): admits decides a composite slot by conformance at Option payload, collection element, tuple position and record field. It descends through Option and collection values with a worklist and reuses the ancestor closure. A union payload follows later.
- FR-113 item 9 (examined): no covariance over type constructors. The container's declared type and kind match exactly.
- FR-113 item 10 (examined): nonconforming at any depth gives admits false; construction refuses TypeMismatch at field or position; an unknown key is nonconforming.
- FR-113 item 11 (examined): operand_value checks a composite operand by conformance under the given TypeEnvironment; EqualityOperandSourceNotAdmitted otherwise; equality conversion unchanged.
- FR-113 items 3-4 (context_only): reflexive composite_conforms; top-level composite admission.

## Verdict

**FINDINGS**. Items 9 and 11 are sound and match QSV code and FR-359. Items 9 and 11
agree with FR-359 item 5's no-covariance rule and its Reference-by-kind rule. Item 9's
exact container match agrees with the kernel `ValueType::admits` payload and element
type check. Item 11's claim that `operand_value` is a free function checking non-Reference
operands with exact `ValueType::admits` matches `declaration.rs:4140-4148`, and its new
`TypeEnvironment` parameter is a stated signature change. The ancestor closure is
transitive, so admission at nested depth is sound for grandchildren.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Items 8 and 10 cover record fields and tuple positions, but only for `record`/`tuple`. QSV's own `TypeEnvironment::evaluate_record` and `evaluate_tuple` admit each deferred member with the exact kernel `ValueType::admits` (`admitted`, declaration.rs:3409-3415). A conforming child at a `P` field there stops with the checked-invariant `DeferredResultNotAdmitted`. That contradicts FR-359 item 8, which says no conforming value reaches a checked-invariant fault. Name the deferred path: it admits by conformance and keeps `DeferredResultNotAdmitted` for a nonconforming member. | FR-113, FR-113-AC-4 |
| FND-002 | medium | Item 8 reads two ways on composite members. "a record or tuple whose member is typed `P`, a present ... member value is admitted" reads as `admits(Composite(R), r)` checking `r`'s slots. The next sentence descends only through `Option` and collection values, and today's `admits` checks only the key. State that the descent stops at a composite value and does not re-check its slots. Give the reason: members are admitted when the composite is built, by `fill_slots`/`tuple`, or by the caller before `from_admitted_slots`. That is the same caller-admitted basis that `from_admitted` and `form` rely on, so say why `Option` and collection contents are still re-checked (replay admits decoded containers, FR-359 item 7). | FR-113 |
| FND-003 | low | Item 10 calls "a composite key the environment does not admit" nonconforming. Item 3 and `composite_conforms` are reflexive for an identical key, admitted or not, so today `admits(Composite(X), x)` is true for an unknown `X`. Either restrict item 10 to a value key that differs from the declared key, or make item 3 require the key to be admitted. | FR-113 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | Item 11 says record and tuple construction and evaluation refuse a nonconforming member with ConstructionCause::TypeMismatch. Item 12, AC-4 and TC-913 expected result 3 say evaluate_record/evaluate_tuple stop with the checked-invariant DeferredResultNotAdmitted. Make item 11 say construction refuses with TypeMismatch and evaluation stops with DeferredResultNotAdmitted (item 12). | FR-113, FR-113-AC-4, TC-913 |
| FND-005 | low | Item 12 names evaluate_record and evaluate_tuple unqualified. quire-exact FR-378 item 4 and QSL FR-359 item 5 say the kernel's evaluate_record/evaluate_tuple stay exact and refuse a descendant with DeferredResultNotAdmitted. Item 12 means QSV's own TypeEnvironment::evaluate_record/evaluate_tuple, which sit above the kernel and build through from_admitted_slots (src/declaration.rs:2938-3004), so the two repos do not conflict in substance. Qualify the names as TypeEnvironment:: so a reader of the two specs does not see a contradiction. | FR-113 |

## Dispositions

Round 1, reviewed at 5870800cce45b74a6c549f1da4c855f47d535257.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | still-open | Item 9 now says the walk stops at a composite, but item 8 still lists 'tuple position' among what the walk checks, and a tuple is a composite value (kernel CompositeValue). AC-3 and TC-913 steps 1-2 expect admits to refuse 'the same containers holding a U', and that includes a tuple and a record built with from_admitted_slots. Under item 9, admits(Composite(R), r) returns true for such a record, because it checks only the key. Drop tuple position from item 8's walk, and drop tuple/record from AC-3's admits negatives and TC-913 step 2. Their refusal belongs to construction and evaluation, which AC-4 already covers. |
| FND-003 | fixed | 5870800cc |
