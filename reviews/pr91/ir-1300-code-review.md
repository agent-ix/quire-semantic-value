---
id: SR-20261102072336
title: "Code review of FR-131 Rust implementation"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/enumeration.rs, src/reader_admission.rs, tests/admitted_equality.rs; spec/functional/FR-131-compare-enum-declarations-by-typed-structure.md"
review_set: subset
---

## Summary

PR #91 at `ea350f30e2c94ca22027f623a8c8bceff53735eb`. The direct enum comparison retains and compares exact typed structure. The reader materialization path can select the wrong declaration when descriptors contain ambiguous same-key declarations.

## Examined units

- `FR-131` (examined): When `compare_enum` receives two admitted `EnumValue` operands, `quire-semantic-value` SHALL refuse `DistinctEnumDeclarations` unless their caller-admitted declaration `NodeKey`, ordered flag, and complete declaration member lists in canonical order are equal by typed comparison. `quire-semantic-value` SHALL retain in each value the complete member list from the `EnumDeclaration` that admitted it, including members other than the selected case.
- `FR-131-AC-1` (examined): Two values made from declarations with the same admitted key and ordered flag, but member lists `['A', 'B']` and `['A', 'C']`, yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-2` (examined): Two ordered declarations with the same admitted key and members `['A', 'B']` and `['B', 'A']` yield `DistinctEnumDeclarations` for `=` even when both selected cases are `A`; the meter has no new charges.
- `FR-131-AC-3` (examined): Two otherwise identical declarations with different caller-admitted, owner-sensitive typed `NodeKey` values yield `DistinctEnumDeclarations` for `=`; the meter has no new charges.
- `FR-131-AC-4` (examined): Two declarations with equal admitted keys, ordered flags, and exact ordered member lists admit comparison: equal selected member keys make `=` true, and the existing ordering behavior and charge schedule apply. A difference only in the ordered flag yields `DistinctEnumDeclarations` before ordering refusal or charges.

## Verdict

**CONDITIONAL** — One medium reader materialization defect needs resolution. Full gates and builds were excluded by the review brief.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Reader first-match lookup can attach the wrong complete member list when two admitted declarations share key, ordered flag, and selected case/position but differ in another member; it should preserve the member's actual declaration or reject ambiguity. | src/reader_admission.rs:443 |

## Evidence

For declarations `(K, true, [A,B])` and `(K, true, [A,C])`, an `A` member at position 0 matches both predicates. `Iterator::find` selects the first declaration regardless of which declaration the member row came from. The resulting `EnumValue` can therefore retain `[A,B]` when its actual declaration was `[A,C]`. The reader descriptor currently has no unambiguous association field; `declaration.key()` also clones each candidate during the unmetered linear scan. The unchanged integration test has only one declaration and cannot expose this case.

Direct `EnumDeclaration::member` shares one `Arc<[String]>` with its declaration, and `compare_enum` checks key, ordered flag and list before charges. The focused test's content, order, key, flag, positive equality and ordering assertions can fail under relevant mutations.
