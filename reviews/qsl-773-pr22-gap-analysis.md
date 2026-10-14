---
id: SR-9118
title: "Gap analysis of quire-semantic-value PR #22: typed-event membership seam"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@205f26d54f129021e26ba8b3bcf00195dc9aeded; FR-109, TC-908, src/declaration/membership.rs, src/declaration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# Gap analysis of quire-semantic-value PR #22

## Summary

Ticket: QSL-773. This is a planless audit scoped to the two seam commits. The
matrix was computed with `quire matrix --scope .` (quire 0.36.2) on the
worktree at the reviewed sha. The PR covers the seam only and says it does
not claim integrated FR-109: IR-714 owns the budget and the public API.

- FR-109-AC-1: `event_stream_for_a_nested_value_is_exact`,
  `initial_type_pair_has_its_own_event`,
  `denied_event_is_not_performed_and_its_error_is_unchanged` (denial at each
  of the 11 positions, with the error returned unchanged),
  `limit_stop_is_returned_unchanged` and `children_are_scheduled_one_event_each`.
  The seam half of the AC (each event once per actual step, a denied event not
  performed) is asserted with typed fields. The budget half (configured N-1,
  spend) belongs to IR-714.
- FR-109-AC-3: the denial test covers unchanged propagation of a callback
  cancellation. The original-handle half belongs to IR-714.
- FR-109-AC-4: `invalid_membership_carries_distinct_cause_and_locus` (leaf,
  payload type link, collection kind),
  `conformance_and_union_failures_have_distinct_causes` (composite, reference,
  union, each with its descriptor locus) and
  `foreign_arena_locus_is_refused_without_events`. All causes are typed and
  distinct.
- FR-109-AC-5: `worklist_reservation_failures_are_distinct_and_precede_child_events`
  covers a denied reservation with its exact request, before any child event.
  The Capacity half is covered through the `storage` mapping and the existing
  `reserve_elements` overflow test.
- Red evidence: all 8 new tests failed against a stub walker, each for the
  right reason. No test is vacuous: each one asserts exact typed values.
- No stubs, `todo!` or ignored tests.

## Verdict

**PASS** for this seam slice. No blocking gaps. FND-001 and FND-002 are
recorded, non-blocking.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Recorded. `event_stream_for_a_nested_value_is_exact` is tagged FR-109-AC-2, but it asserts one argument's event stream. Nothing in it shares a budget across two arguments, so the AC-2 binding inflates coverage. Leave the tag for IR-714's integrated test. | src/declaration/membership.rs:375 |
| FND-002 | low | Recorded. The AC-5 clause "retry after removing the denial yields the same membership outcome" has no test. The walker never produces Capacity in any test, which is covered only through the `storage` helper. | src/declaration/membership.rs:619-649 |
