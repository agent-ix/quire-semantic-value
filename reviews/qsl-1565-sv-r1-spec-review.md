---
id: SR-20261102072321
title: "Spec review of quire-semantic-value PR #87: FR-106 Behaviour item 4, FR-106-AC-12, TC-904 rows 9-10"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@b3616d478; spec/functional/FR-106-object-closure-admission.md; spec/test-cases/TC-904-object-closure-refuses-duplicates-unknown-types-and-ambiguous-find.md; ticket QSL-1565"
review_set: subset
date: '2026-11-03'
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-904
    type: references
---

# Spec review of quire-semantic-value PR #87

## Summary

Ticket: QSL-1565. This review covers the FR-106 description sentence, Behaviour item 4, FR-106-AC-12, and TC-904's scope, rows 9 and 10, expected results and status. I ran the base review together with the EARS and integrity checks.

Examined, clean:

- The new description sentence ("It SHALL record each admitted object type's attribute names in its slot order, find a slot by attribute name, and iterate every admitted object with its named slots") is a well-formed ubiquitous requirement. It matches the code.
- Behaviour item 4 matches the implementation:
  - names are recorded in the admitting slot order;
  - `attribute` resolves the field under the reader's environment and then looks it up by name;
  - `object` and `iter` return `(name, slot)` pairs, in reference order.
- AC-12 does not contradict AC-10 (admission and find) or AC-11 (dangling references). It adds a disjoint read-side behaviour.
- TC-904 rows 9 and 10 restate AC-12's clauses with concrete fixtures. Each fixture matches a test that asserts it (see SR-20261102072320).
- The expected-results and status lines now bind rows 9 and 10 to AC-12.
- `quire validate --scope .` on both changed files exits 0. The output is only module-level warnings, all DuplicateArchetype/DuplicateInverseEdge from spec-artifacts-process being contributed twice, and none of them concern these files.

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| FR-106 Description | spec/functional/FR-106-object-closure-admission.md | examined |
| FR-106 Behaviour item 4 | spec/functional/FR-106-object-closure-admission.md | examined |
| FR-106-AC-12 | spec/functional/FR-106-object-closure-admission.md | examined |
| FR-106-AC-10 | spec/functional/FR-106-object-closure-admission.md | context_only |
| FR-106-AC-11 | spec/functional/FR-106-object-closure-admission.md | context_only |
| TC-904 rows 9-10, Expected Results, Status | spec/test-cases/TC-904-object-closure-refuses-duplicates-unknown-types-and-ambiguous-find.md | examined |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-106-AC-12 states a single instance rather than the rule. It only covers object type `A`, two same-typed attributes, `{b, a}` read as `{a, b}`, which is TC-904 row 9's fixture. By contrast, AC-10 and AC-11 state general rules. The general rule ("`attribute` returns the slot whose admitting attribute name is the name the reader's environment resolves `field` to") appears only in Behaviour item 4, which no AC states. So AC-12 does not require name lookup for differently typed, more than two, or inherited attributes. | spec/functional/FR-106-object-closure-admission.md:30 |

## Verdict

The spec edits are consistent with the code and with AC-10 and AC-11, and TC-904 rows 9 and 10 are genuine. FND-001 is a wording improvement: state the general by-name rule in AC-12 and keep the `{b, a}`/`{a, b}` case as the TC row.

## Dispositions

- FND-001: fixed. FR-106-AC-12 now states the general rule: `attribute` returns the slot whose admitting attribute name is the name the reader resolves the field to, whatever position either environment gives it, for attributes of any type and number, including attributes inherited from supertypes listed in a different order. New TC-904 row 11 (`attribute_reads_mixed_and_inherited_attributes_by_name`) covers mixed types, five attributes and two supertypes whose order is swapped. Positional lookup misreads `p` and `q` in that test.
