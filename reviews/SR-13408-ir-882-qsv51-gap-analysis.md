---
id: SR-13408
title: "IR-882 quire-semantic-value PR 51 gap analysis: CheckedEquality plan arm"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@c7c38d4e1f87c24d0bfe3eb5a7965133b5342537 vs origin/main 9fd1375; src/declaration.rs, Cargo.lock"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: references
---

## Summary

Ticket: IR-882. This is a planless, diff-scoped audit; plan completion was not assessed.
`quoin matrix --json` (quoin 0.28.3) was run at the PR head. The production change is one call
site in `CheckedEquality::run`'s `EqualitySchedule::Plan` arm. It is behaviour-preserving:
SR-13407 shows verdicts and charge schedules byte-identical, old call against new call. No AC text,
public item or spec file changes. The owning requirement is FR-113 Behavior 13, under which
composite equality "belongs to the kernel's composite equality".

The matrix rows whose tagged tests run this arm all stay `tagged` and pass at head:

- FR-115-AC-1 and FR-115-AC-2: `native_union_positions_preserve_released_leaf_kinds_and_equality`.
- `registry_binding_drives_identifier_key_order_and_checked_equality`. Its `FR-323-AC-1` tag
  names a cross-repo id that is not a QSV matrix row, but the test runs this arm on unions.
- FR-113-AC-5 and FR-113-AC-6: the `operand_value` conformance tests and the set/bag equality test.

No new criterion is needed, and no test needs a retag. No production code lacks an owning
requirement, and there are no stubs. Spec-review sub-analyses were skipped because the diff touches
no `spec/` or `plan/` file.

## Reviewed units

- FR-113 Behavior 13 (examined): "`operand_value` SHALL check a composite equality operand by conformance ... whether two composites of different declarations are equal belongs to the kernel's composite equality. `CheckedEquality::evaluate` takes the `TypeEnvironment` that checked the equality and passes it to `operand_value`."
- FR-115-AC-1 (examined): tagged by `tests/union_registry.rs::native_union_positions_preserve_released_leaf_kinds_and_equality`, which asserts plan-arm `Completed(true)`; passes.
- tests/union_registry.rs::registry_binding_drives_identifier_key_order_and_checked_equality (examined): tagged FR-323-AC-1, a cross-repo id that is not a QSV matrix row; it asserts plan-arm true/false verdicts on unions; passes.
- FR-113-AC-6 (context_only): "A `Set<P>` or `Bag<P>` holding a `C` and a `P` of equal shared fields has two elements and is admitted."

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. The changed call site sits under an existing requirement, its tests stay tagged and green,
and the change adds no behaviour that needs new criteria. Mergeable.
