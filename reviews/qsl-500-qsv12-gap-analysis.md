---
id: SR-9034
title: "Gap analysis of quire-semantic-value PR #12: FR-115 and union admission"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@52786ce8ae19383a052e4d3a9a28804ecaf776b1; FR-115-AC-1..AC-4, TC-915, FR-113 items 8-14, FR-108, FR-109, tests/union_registry.rs, tests/union_staging.rs, src/declaration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---
# Gap analysis of quire-semantic-value PR #12

## Summary

Ticket: QSL-500. This is a planless audit scoped to the PR. The matrix was
computed on a read-only worktree with `quire matrix --scope <repo> --format
json` (quire 0.36.2, engine 0.50.2). The matrix gives:

- FR-115-AC-1 is tagged by 2 tests, AC-2 by 4, AC-3 by 3 and AC-4 by 2.
- FR-108-AC-1 to AC-4 are tagged. AC-5 is `method-without-symbol`.
- FR-109-AC-1 to AC-6 are untagged and AC-7 is `method-without-symbol`.
  FR-109 is a draft, so this is unchanged from main.
- FR-110 is untagged, as on main.

`quire validate` on every spec file passes. FR-115 and TC-915 produce no
warnings; the only warnings are the existing ones in FR-110.

FR-115 and TC-915 are unique across every remote branch and every open PR.
Only `task/qsl-500-sv-union` carries them. Main's FR-112 and TC-911 (native
String) keep their ids. The `FR-112-AC-*` tags in `src/declaration.rs` are
main's own String tests, not stale references to the old union numbering.

The quire-exact lock rev d41a1da is on quire-exact main and includes FR-379
(the union carrier) and FR-376 to FR-378. FR-113-AC-6 is no longer ignored.

## Verdict

**CONDITIONAL**: two medium findings.

The FR-115 bindings that exist are correct. Oracle strength was checked on
the tagged tests:

- `each_union_seal_destination_denies_before_any_binding_is_published` asserts
  the exact request, an unchanged environment, retained spend and a
  deterministic retry for each destination.
- `topology_sealing_and_member_search_share_a_cumulative_budget` asserts exact
  N and N+1 totals at each stage.
- `resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed`
  asserts that unsealed lookup, construction and admission refuse, and that
  sealed ones succeed.

Each of these goes red if its behaviour is reverted.

The deeper evidence for FR-115 is in tests that the matrix cannot see (FND-001).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 22 of the 28 integration tests are tagged only with `FR-321-AC-*` or `FR-323-AC-3`. These are quire-spec-language and QSpec criterion ids, and no criterion in this repository matches them, so the QSV matrix ignores the tests. They hold the evidence that FR-115's criteria name. AC-1 (named escaping recursion, all-member IEEE): `named_union_edges_require_escape_and_report_exact_cycle`, `ieee_and_named_escape_checks_run_before_any_binding_exists`, `unused_ieee_member_denies_all_keyed_kinds_but_sequence_admits`. AC-2 (collisions, duplicates): `final_key_collisions_cannot_alias_records_or_another_union`, `duplicate_names_keys_foreign_bindings_and_unknown_types_refuse`. AC-4 (5,000/1,000/10,000 fixtures, N/N-1, cancellation): `five_thousand_members_and_thousand_options_use_named_work_limit`, both `ten_thousand_*` tests, `seal_work_limits_are_exact_and_do_not_attach_a_prefix`, `bounded_construction_keeps_work_stops_separate_from_prior_runtime_stop`. `zero_member_seal_polls_cancellation_without_attaching_a_final_key` has no tag. Add the FR-115-AC-n and TC-915 tags these tests verify. | tests/union_registry.rs:148-1101; tests/union_staging.rs:118-1316 |
| FND-002 | medium | Union payload admission has no owning criterion in QSV. FR-113 item 8's union clause ("a union admits a payload when any one declared member admits it") describes an untagged union. The implementation checks only the value's own carried member, and FR-113-AC-1 to AC-7 say nothing about unions. FR-115's criteria cover staging and sealing, not payload admission. `nested_union_admission_traverses_record_tuple_option_and_collection` and `supplied_trusted_carriers_still_require_registry_admission` have nothing in QSV to bind to. A union value at a record-typed or tuple-typed slot is refused, but that rule is stated nowhere. Rewrite item 8's union clause for the tagged member carrier and add an FR-113 or FR-115 criterion with these cases. | spec/functional/FR-113-composite-value-conformance.md:61-65; src/declaration.rs:2640-2655 |

## Coverage

- FR-115-AC-1: `native_union_positions_preserve_released_leaf_kinds_and_equality`, `resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed`. Bindings are correct.
- FR-115-AC-2: `each_union_seal_destination_denies_before_any_binding_is_published`, `native_union_positions_preserve_released_leaf_kinds_and_equality`, `resolved_topology_is_visible_and_runtime_union_is_denied_until_sealed`, `invalid_seals_leave_every_descriptor_and_join_unchanged`. Bindings are correct.
- FR-115-AC-3: `union_topology_storage_denial_and_retry_use_the_production_boundary`, `each_union_seal_destination_denies_before_any_binding_is_published`, `union_binding_validation_byte_failure_retains_its_native_request`. Bindings are correct. The capacity control is the generic `size_overflow_is_distinct_from_allocator_denial`, which is tagged FR-108-AC-2 only.
- FR-115-AC-4: `topology_sealing_and_member_search_share_a_cumulative_budget`, `full_size_resolved_registry_can_be_sealed_without_replacing_topology`. Bindings are correct. The fixtures that the criterion names are untagged (FND-001).
- FR-113 items 8 to 14: implemented with the deviations in SR-9033 FND-001, FND-002 and FND-006.
- FR-109: its carrier is reserved to QSpec FR-323 and QSL-681 (Dependencies, last three bullets), and its status is SPEC DRAFT. The infallible supplied-phase worklists are an accepted external dependency, and growth is bounded by the charged `WorkBudget` (SR-9033). `check_type` and `contains_ieee` are authored parameter and result type checks, not supplied membership. Neither FR-108 (environment admission) nor FR-109 covers their storage, so the PR body files them under the wrong owner.
- Semantic review: not run as a separate pass. Intent and oracle were judged by hand above.

Plan completion: not assessed
