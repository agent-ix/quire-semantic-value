---
id: SR-8689
title: "Gap analysis of quire-semantic-value PR #16: FR-113 items 8 to 14"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@abb629f5342f6ca4c695acf74f525e2bdc4adddf; FR-113 items 8-14, FR-113-AC-3..AC-6, TC-913, src/declaration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: reviews
---
# Gap analysis of quire-semantic-value PR #16

## Summary

Ticket: QSL-737. This is a planless audit of FR-113 items 8 to 14 and
FR-113-AC-3 to AC-6 against the computed matrix, which was produced with
`quire matrix --scope <repo> --format json` (quire 0.36.2, engine 0.50.2).
The matrix gives:

- AC-3 is tagged by 2 tests.
- AC-4 is tagged by 3 tests.
- AC-5 is tagged by 2 tests.
- AC-6 is `tagged-by-ignored-test`, from 1 `#[ignore]`d test.

All 7 tests carry `#[trace("TC-913", "FR-113-AC-N")]`, and every binding is
correct. There are no stubs and no `todo!`.

## Verdict

CONDITIONAL, with low findings only.

- Items 8 to 13 are implemented as specified.
- Item 14 holds: the result is a `bool`, there is no charge and no
  recursion.
- The union payload is correctly not implemented. Item 8 defers it until a
  union value exists.

The test-oracle strength analysis is in SR-8688. Every non-ignored TC-913
test goes red when its behaviour is reverted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Item 9's `Reference<T>` leaf reached inside the walk has no test, and no acceptance criterion names it. This is new behaviour. Before this PR, an `Option<Reference<T>>` or `Sequence<Reference<T>>` holding a reference of object type `S` was admitted by its recorded type alone. Now `conforms(S, T)` is required, and `operand_value` inherits the check for nested references. If the walk skipped or mis-checked a nested reference, all 7 TC-913 tests would stay green. Add a positive case (subtype object inside `Option<Reference<T>>`) and a negative case (unrelated object type), and an AC clause for them. | src/declaration.rs:1747-1751; spec/functional/FR-113-composite-value-conformance.md:74-76 |
| FND-002 | low | The code exceeds the spec. `CheckedEquality::evaluate` gains a public `types: &TypeEnvironment` parameter, but FR-113 item 13 names only `operand_value`. The change is justified: `run` must pass the environment to `operand_value`, and the capture-at-check alternative would retain descendant sets per equality node. Add one sentence to item 13 stating that `evaluate` takes the environment that checked the equality. | src/declaration.rs:3688-3696; spec/functional/FR-113-composite-value-conformance.md:101-110 |
| FND-003 | low | FR-113-AC-6 is backed only by the `#[ignore]`d `set_and_bag_of_parent_keep_equal_field_child_and_parent_distinct`, which waits on quire-exact FR-376 to FR-378 (quire-exact PR #39). The spec declares this dependency. The criterion stays unverified until the ignore is removed after quire-exact lands. | src/declaration.rs:5563-5586 |

## Coverage

- FR-113-AC-3: `container_of_parent_admits_conforming_descendants` and `container_of_parent_refuses_nonconforming_contents_and_covariance`. Bindings are correct.
- FR-113-AC-4: `nested_option_sequence_option_admits_by_conformance`, `evaluate_record_and_tuple_admit_members_by_conformance` and `construction_refuses_nonconforming_member_naming_the_component`. Bindings are correct.
- FR-113-AC-5: `operand_value_checks_composite_operands_by_conformance` and `operand_value_checks_reference_operand_by_kind_only`. Bindings are correct.
- FR-113-AC-6: `set_and_bag_of_parent_keep_equal_field_child_and_parent_distinct`, which is ignored (FND-003).
- Reverse gap: `CheckedEquality::evaluate` signature (FND-002). Nothing else is unowned.
- Semantic review: skipped as a separate pass. Intent and oracle were judged per test in SR-8688.

Plan completion: not assessed
