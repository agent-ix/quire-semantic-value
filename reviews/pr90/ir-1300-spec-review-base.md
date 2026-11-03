---
id: SR-20261102072330
title: base review of FR-131
type: SpecReview
analysis: base
scope: agent-ix/quire-semantic-value; spec/functional/FR-131-compare-enum-declarations-by-typed-structure.md
review_set: subset
---

## Summary

The requirement and all four acceptance criteria are specific, testable examples of declaration identity and exact ordered structure. The QSL nominal path invariant and QSV caller boundary remain intact.

## Verdict

**PASS** — No defects in the PR diff for this method.

## Examined units

- `FR-131` (examined): When `compare_enum` receives two admitted `EnumValue` operands, `quire-semantic-value` SHALL refuse `DistinctEnumDeclarations` unless their caller-admitted declaration `NodeKey`, ordered flag, and complete declaration member lists in canonical order are equal by typed comparison. `quire-semantic-value` SHALL retain in each value the complete member list from the `EnumDeclaration` that admitted it, including members other than the selected case.
- `FR-131-AC-1` (examined): Two values made from declarations with the same admitted key and ordered flag, but member lists `['A', 'B']` and `['A', 'C']`, yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-2` (examined): Two ordered declarations with the same admitted key and members `['A', 'B']` and `['B', 'A']` yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-3` (examined): Two otherwise identical declarations with different caller-admitted, owner-sensitive typed `NodeKey` values yield `DistinctEnumDeclarations` for `=`; the meter has no new charges.
- `FR-131-AC-4` (examined): Two declarations with equal admitted keys, ordered flags, and exact ordered member lists admit comparison: equal selected member keys make `=` true, and the existing ordering behavior and charge schedule apply. A difference only in the ordered flag yields `DistinctEnumDeclarations` before ordering refusal or charges.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
