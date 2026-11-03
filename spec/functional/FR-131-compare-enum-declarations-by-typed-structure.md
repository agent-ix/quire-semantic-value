---
id: FR-131
title: "Compare enum declarations by admitted identity and exact structure"
type: FR
relationships:
  - target: "ix://agent-ix/quire-semantic-value/FR-128"
    type: "depends_on"
  - target: "ix://agent-ix/quire-specification/FR-141"
    type: "depends_on"
---
# FR-131: Compare enum declarations by admitted identity and exact structure

## Description

When `compare_enum` receives two admitted `EnumValue` operands, `quire-semantic-value` SHALL
refuse `DistinctEnumDeclarations` unless their caller-admitted declaration
`NodeKey`, ordered flag, and complete declaration member lists in canonical order are
equal by typed comparison. `quire-semantic-value` SHALL retain in each value the
complete member list from the `EnumDeclaration` that admitted it, including members
other than the selected case.

## Inputs

- Two enum values built from admitted declarations, each with a caller-supplied
  declaration key, ordered flag, and complete ordered member list.
- A comparison operator and meter.

## Behavior

1. `quire-semantic-value` SHALL retain its declaration's complete member list
   through `EnumDeclaration::member`
   in the resulting `EnumValue`, in the same order and with exact case names.
2. Before performing the member comparison or charging the meter,
   `quire-semantic-value` SHALL compare through `compare_enum` the operands'
   admitted declaration keys, ordered flags, and
   complete member lists. If any component differs, it SHALL return
   `IllTypedCause::DistinctEnumDeclarations`.
3. When those components are equal, `quire-semantic-value` SHALL apply the existing
   unordered-enum ordering refusal and member-comparison schedule.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-131-AC-1 | Two values made from declarations with the same admitted key and ordered flag, but member lists `['A', 'B']` and `['A', 'C']`, yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges. | Test |
| FR-131-AC-2 | Two ordered declarations with the same admitted key and members `['A', 'B']` and `['B', 'A']` yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges. | Test |
| FR-131-AC-3 | Two otherwise identical declarations with different caller-admitted, owner-sensitive typed `NodeKey` values yield `DistinctEnumDeclarations` for `=`; the meter has no new charges. | Test |
| FR-131-AC-4 | Two declarations with equal admitted keys, ordered flags, and exact ordered member lists admit comparison: equal selected member keys make `=` true, and the existing ordering behavior and charge schedule apply. A difference only in the ordered flag yields `DistinctEnumDeclarations` before ordering refusal or charges. | Test |

## Dependencies

- [FR-128](./FR-128-consume-structural-identities.md) governs typed identity
  consumption.
- `ix://agent-ix/quire-specification/FR-141` defines the enum comparison schedule.
- The caller admits the declaration `NodeKey` and owns its owner-sensitive
  identity contract. QSV accepts that key as supplied; it cannot distinguish
  two owners represented by an identical supplied key.

## Status

Planned for IR-1300.
