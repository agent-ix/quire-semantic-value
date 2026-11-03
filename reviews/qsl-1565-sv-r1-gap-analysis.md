---
id: SR-20261102072320
title: "Gap analysis of quire-semantic-value PR #87: FR-106-AC-12 named ObjectClosure slots"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@b3616d478; spec/functional/FR-106-object-closure-admission.md, spec/test-cases/TC-904-object-closure-refuses-duplicates-unknown-types-and-ambiguous-find.md, src/object_closure.rs; ticket QSL-1565"
review_set: subset
date: '2026-11-03'
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-904
    type: references
---

# Gap analysis of quire-semantic-value PR #87

## Summary

Ticket: QSL-1565. This analysis is planless, so plan completion is not assessed.

`quoin matrix --repo .` at the PR head lists FR-106-AC-10, AC-11 and AC-12 as `Test` and `tagged`, with no run evidence. That is the same state AC-10 and AC-11 had on main.

FR-106-AC-12 is tagged by two tests:

- `attribute_reads_by_name_across_reordered_environments` (TC-904 row 9) asserts `a`→1 and `b`→2 when a closure admitted under `{b, a}` is read under `{a, b}`. It asserts `None` for an undeclared `c` and for an unadmitted object.
- `objects_iterate_as_named_slots` (TC-904 row 10) asserts that `iter` yields reference order with the lower reference first, even though it was admitted second. It asserts `(name, slot)` pairs in admitting order with the omitted `note` read as `Absent`, `get` hits and misses, and `object` of an unadmitted reference returning `None`.

Every clause of AC-12 has an assertion. Mutating `attribute` back to positional lookup fails row 9. Dropping a name in `ObjectAttributes::iter`, or in `new`'s recording, fails row 10.

Code with no owning criterion: the new `self.names == other.names` term in `PartialEq` is the only one (FND-001).

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| FR-106-AC-12 | spec/functional/FR-106-object-closure-admission.md | examined |
| TC-904 rows 9-10 | spec/test-cases/TC-904-object-closure-refuses-duplicates-unknown-types-and-ambiguous-find.md | examined |
| attribute_reads_by_name_across_reordered_environments | src/object_closure.rs | examined |
| objects_iterate_as_named_slots | src/object_closure.rs | examined |
| impl PartialEq for ObjectClosure | src/object_closure.rs | examined |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `PartialEq` now also compares `names`, but no test and no AC covers it. Replacing `self.names == other.names` with `true` leaves the full `cargo test --lib` suite green (156 passed). The behaviour is real: two closures from reordered environments whose slot vectors coincide were previously equal and are now unequal. So it needs one assertion, for example in row 10, or the term should go. | src/object_closure.rs:279-289 |

## Verdict

FR-106-AC-12 is fully backed by genuine, mutation-sensitive tests. One small gap remains: the untested `PartialEq` change (FND-001).

## Dispositions

- FND-001: fixed. The `names` term stays, because it is real. Two closures whose slot vectors coincide under reordered names read different values through `attribute`, so they are different closures. TC-904 row 9 (`attribute_reads_by_name_across_reordered_environments`) now asserts `assert_ne!` against such a closure and `assert_eq!` against a rebuilt one. FR-106-AC-12 states the inequality.
