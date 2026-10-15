---
id: SR-11074
title: "Code review of quire-semantic-value PR #12 at the ported union stages"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@dad364a5cc77ad999c0870ea6c8c0b310fba832d; src/declaration.rs, src/containment.rs, tests/union_registry.rs, tests/union_staging.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---
# Code review of quire-semantic-value PR #12 (ported head)

## Summary

Ticket: QSL-500. The code-review and rust-review lanes ran together over
`git diff origin/main...HEAD`, where the merge base is QSV main ecc6e06 (this
includes IR-792/793 at 0e12429 and IR-719). `src/declaration/membership.rs`
and `src/supplied_membership.rs` are untouched: their diff is empty. This head
drops the PR's own admission walk and the caller-budget API. Union payloads are
admitted through main's `admits_checked`. Review is limited to issues that give
a wrong result, a panic, a failed AC, a lost or vacuous test, a spec
contradiction or a broken gate.

## Verdict

**CONDITIONAL**: one high finding.

What holds:

- **Dropped API.** `WorkBudget` is private on main too (`struct WorkBudget`,
  no `pub`), so the public `WorkBudget` from the earlier head never reached
  main. No README, spec or doc test names it. quire-spec-language has no use of
  `WorkBudget`, `seal_union*`, `UnionMemberDeclaration`, `CompositeShape::Union`
  or `bounded_with_budget`. QSL matches `CompositeShape` and `GraphNode` only
  through constructors and `let ... else`, so the new variants break no
  exhaustive match there.
- **Sealing atomicity.** Validation runs first. All three destinations then
  reserve their full growth before any mutation, and the unit test denies each
  one with the exact request, an unchanged environment and a deterministic
  retry.
- **Supertype exclusion.** `check_composite_supertypes` now refuses a union as
  subtype or supertype, which fixes the earlier head's composite-conformance
  wrong answer.
- **Non-union charge points.** Union-only charges are guarded by shape checks,
  and Record/Tuple member checks pass `metered = false`. A non-union package is
  charged the same as on main.
- **`check_equality_in` scope change.** The new `type_refusal` pre-check
  refuses only operands that were already ill-typed: an unknown declaration, a
  `Reference` target that is not an object type, and a nested `Population`.
  Before, these reached the `l == r` Plan schedule and could only fail at
  evaluation. No valid non-union equality changes its result. This is recorded,
  not blocking.
- **Panic surface.** The new library code has no `unwrap`, `expect` or
  `unsafe`. `path.len() - start + 1` cannot underflow. `runtime_type`'s
  `_ => return None` arm cannot be reached.
- **Published shape.** All line references in the PR body match the head
  (:933, :964, :972, :980, :993, :2415, :2457, :2462, :2560, :2588, :2636,
  :4109, :4136, and membership.rs:562). Together they are enough to write the
  union arm.

Recorded (medium/low, not blocking): `check_recursion` adds an `active`
index reservation for every declaration, which changes Traversal-phase
reservation counts for Record and Tuple. No test covers union supertype
refusal. The `check_equality_in` scope change described above.

Gates were not re-run (reviewer-only). The coder reports `make ci` EXIT 0 on
dad364a.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A Record or Tuple carrier is admitted at a union type. Main's walk admits `(Composite(declared), Value::Composite(c))` when `composite_conforms(c.declaration(), declared)` holds, and that is `sub == sup` with no shape check. So `admits(Composite(U), from_admitted_slots(U, []))` returns `true` for a union `U`. This PR is what makes union declarations reachable, so it exposes the wrong admission result. The ignored test `supplied_trusted_carriers_still_require_registry_admission` asserts `false` for this carrier and fails on it today. Its ignore reason, "fails only because main refuses union values", is therefore false. Fix: refuse a union-shaped key in `composite_conforms` (declaration.rs, in this PR), and add a live test. Otherwise, hand the case to the union-arm work explicitly and correct the ignore reason. | src/declaration.rs:2447-2453; src/declaration/membership.rs:534-546; tests/union_registry.rs:183-214 |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 458565f. `composite_conforms` now returns false when either key has a union shape, by handle or by final key. membership.rs is untouched. The tests `record_and_tuple_carriers_are_not_admitted_at_a_union_type` and `sealed_union_final_key_never_admits_a_record_or_tuple_carrier` are live, and both would fail on the old `sub == sup`. Non-union paths are unchanged, because prepare refuses a union as a supertype in either direction. `supplied_trusted_carriers_still_require_registry_admission` is live. |
