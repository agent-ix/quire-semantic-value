---
id: SR-20261102072337
title: "Gap analysis of FR-131 Rust implementation"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/enumeration.rs, src/reader_admission.rs, tests/admitted_equality.rs; spec/functional/FR-131-compare-enum-declarations-by-typed-structure.md"
review_set: subset
---

## Summary

PR #91 at `ea350f30e2c94ca22027f623a8c8bceff53735eb`. The computed matrix binds all four FR-131 criteria to the focused unit test. No reverse requirement gap or stub was found in the PR diff.

## Examined units

- `FR-131` (examined): When `compare_enum` receives two admitted `EnumValue` operands, `quire-semantic-value` SHALL refuse `DistinctEnumDeclarations` unless their caller-admitted declaration `NodeKey`, ordered flag, and complete declaration member lists in canonical order are equal by typed comparison. `quire-semantic-value` SHALL retain in each value the complete member list from the `EnumDeclaration` that admitted it, including members other than the selected case.
- `FR-131-AC-1` (examined): Two values made from declarations with the same admitted key and ordered flag, but member lists `['A', 'B']` and `['A', 'C']`, yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-2` (examined): Two ordered declarations with the same admitted key and members `['A', 'B']` and `['B', 'A']` yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-3` (examined): Two otherwise identical declarations with different caller-admitted, owner-sensitive typed `NodeKey` values yield `DistinctEnumDeclarations` for `=`; the meter has no new charges.
- `FR-131-AC-4` (examined): Two declarations with equal admitted keys, ordered flags, and exact ordered member lists admit comparison: equal selected member keys make `=` true, and the existing ordering behavior and charge schedule apply. A difference only in the ordered flag yields `DistinctEnumDeclarations` before ordering refusal or charges.

## Verdict

**PASS** — FR-131 AC-1 through AC-4 are tagged and backed by mutation-sensitive assertions. The reader ambiguity is recorded in the code-review artifact.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

`quire matrix --scope . --format json` reports FR-131-AC-1 through AC-4 as tagged to `tests::enum_comparison_requires_equal_admitted_declaration_structure` at `src/enumeration.rs:498`. Scope is the PR diff and its FR-131 evidence, not a full repository assurance assertion. The reader integration assertion at `tests/admitted_equality.rs:219` verifies one retained list. Optional semantic review was not invoked; the PR brief specifically requested assertion inspection, which was performed manually.
