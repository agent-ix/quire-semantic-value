---
id: SR-11075
title: "Gap analysis of quire-semantic-value PR #12 at the ported union stages"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@dad364a5cc77ad999c0870ea6c8c0b310fba832d; FR-115-AC-1..AC-4, TC-915, tests/union_registry.rs, tests/union_staging.rs, src/declaration.rs unit tests"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---
# Gap analysis of quire-semantic-value PR #12 (ported head)

## Summary

Ticket: QSL-500. This is a planless audit. `quoin matrix` (quoin 0.28.3) on
the head reports FR-115-AC-1 as `tagged-by-ignored-test`, and AC-2, AC-3 and
AC-4 as `tagged`.

AC map:

- AC-1: tagged only by ignored tests
  (`native_union_positions_preserve_released_leaf_kinds_and_equality` and
  `resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed`).
  Some of its behaviour is exercised untagged by
  `ieee_and_named_escape_checks_run_before_any_binding_exists`,
  `named_union_edges_require_escape_and_report_exact_cycle` and
  `full_size_resolved_registry_can_be_sealed_without_replacing_topology`.
- AC-2: `invalid_seals_leave_every_descriptor_and_join_unchanged`,
  `final_key_collisions_cannot_alias_records_or_another_union` and
  `each_union_seal_destination_denies_before_any_binding_is_published`. All
  live.
- AC-3: the three union storage-denial unit tests in src/declaration.rs. All
  live.
- AC-4: N/N-1 is asserted exactly by
  `five_thousand_members_and_thousand_options_use_named_work_limit`
  (24,014 admits; 24,013 gives `WorkUnits`, bound 24,013, actual 24,014).

Ignored tests, spot-checked by reading:

- `native_union_positions_preserve_released_leaf_kinds_and_equality` first
  fails at `admits(Composite, union value)`, which is a genuine wait.
- `ordered_resolution_and_real_construction_validate_positions` first fails
  at line 155, `admits(union value)`, which is a genuine wait.
- `supplied_trusted_carriers_still_require_registry_admission` fails on the
  composite-carrier element. That is not a union-value refusal (SR-11074
  FND-001).

None of the ignored bodies was weakened against the earlier head. The removed
budget arguments are the only changes.

The seven removed tests exercised only the dropped seal, lookup and
construction budgets. No behaviour that survives lost its only test, except as
noted in FND-002.

## Verdict

**CONDITIONAL**: two medium findings.

Recorded gaps, waiting for the union arms in `admits_walk_core`: sealed union
value admission, checked equality over union values, union closure, and the
10,000-link small-stack union admission. All of these have only ignored
coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-115-AC-1's unsealed-refusal clauses have no live test, although they pass today. The first 44 lines of `resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed` do not depend on union value admission: name lookup before sealing, `union_key`/`runtime_type`/`union_member` returning `None`, equality returning `TypeMismatch`, `union()` refusing, and `evaluate_union` refusing without running any payload. They are ignored only because the test's later part needs the union arms. Split them into a live test tagged FR-115-AC-1. | tests/union_staging.rs:78-126 |
| FND-002 | medium | FR-115-AC-4's trace binding is vacuous, and its cancellation clause has no test. The only test tagged FR-115-AC-4, `full_size_resolved_registry_can_be_sealed_without_replacing_topology`, sets no work limit and no cancellation. The real N/N-1 test, `five_thousand_members_and_thousand_options_use_named_work_limit`, is tagged FR-321-AC-4 only. No test cancels union topology admission. Tag the N/N-1 test with FR-115-AC-4, and add a cancelled-handle `bounded` case over a union fixture. | tests/union_staging.rs:557-560; tests/union_registry.rs:360-363 |
