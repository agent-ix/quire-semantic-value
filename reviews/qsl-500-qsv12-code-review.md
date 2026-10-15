---
id: SR-9033
title: "Code review of quire-semantic-value PR #12: shared Union stages"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@52786ce8ae19383a052e4d3a9a28804ecaf776b1; src/declaration.rs, src/object_closure.rs, src/containment.rs, tests/union_registry.rs, tests/union_staging.rs, Cargo.lock"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: reviews
---
# Code review of quire-semantic-value PR #12

## Summary

Ticket: QSL-500 (QSL-737 program). The code-review and rust-review lanes ran
together over the diff from main 5ab440a to head 52786ce. The PR adds resolved
and sealed union declarations to `TypeEnvironment`, a fallible sealing path,
caller-owned `WorkBudget` parameters on the union, type, equality,
construction, graph-build and object-closure APIs, and a union arm in
`admits` and `check_closed`. The `Cargo.lock` change moves only the
quire-exact source line to d41a1da, which is on quire-exact main and contains
the union carrier (#17).

## Verdict

**CONDITIONAL**: two high, two medium and three low findings.

What holds:

- **Sealing atomicity.** `seal_union_with_reservations` validates every
  binding, then reserves the member index, forward join and inverse join
  before any mutation. The later `try_insert` calls fit the reserved capacity
  and never reach the policy, so a denial leaves the environment unchanged.
  The unit test denies each of the three destinations and checks the exact
  request, an unchanged environment, retained spend and a deterministic retry.
- **Union admission.** The union arm in `admits` checks the final-key join,
  the member binding, the arity and then each position. A union value at a
  record-typed or tuple-typed slot is refused, because `union_key` of a record
  key is `None`. This also covers what QSV #19's explicit refusing arm adds.
- **Charge before growth.** Every worklist push in `admits_walk`,
  `contains_ieee_walk`, `type_refusal_walk`, `runtime_type_walk`, `build`,
  `evaluate_union_walk` and `ObjectClosure::check_closed` is paid for by a
  charge made before the push. Worklist size therefore never exceeds the
  caller's `WorkBudget`. The coder's table matches the code.
- **Test-only `Unlimited` trait.** It is `#[cfg(test)]` and private, so it
  does not leak into the public API. Each `*_u` method calls the real
  production method with a fresh `DEFAULT_WORK_UNITS` budget. It does not
  weaken any oracle: the adapted main tests still assert their original
  outcomes, and budget behaviour has its own N/N-1 tests. The name is
  inaccurate (the budget is 16,777,216 units, not unlimited), but that is not
  a defect.
- **Panic surface in the diff.** No new `unwrap`, `expect`, indexing panic or
  `unsafe` in library code. `path.len() - start + 1` in `check_recursion`
  cannot underflow, because `start` is an index into `path`.
- **Public-repo hygiene.** No local paths and no private QSpec text appear in
  the diff. No vendored files and no SHA or version tracking were added.

Gates were not re-run (reviewer-only). `quire validate` and `quire matrix`
were run on a read-only worktree, see SR-9034 and SR-9035.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `TypeEnvironment::admits` now takes `&mut WorkBudget`, charges work units and returns `Result<bool, EnvironmentLimit>`. FR-113 item 14 on main says `admits` takes no `work_units` charge, reports no limit, and returns a `bool`. The PR does not amend FR-113, so the code violates a requirement it leaves in force. Either amend item 14 in this PR, stating which budget and which typed result supplied membership uses (see SR-9035 FND-001), or keep the bool-only uncharged `admits` and add a separate budgeted entry point. | src/declaration.rs:2608-2620 |
| FND-002 | high | `admits_walk` diverges from FR-113 items 9 and 11 in two ways. (a) Item 9 says the walk SHALL stop at any composite value. The PR descends into a composite whose key is exactly the declared key, and stops only at a descendant. A trusted-built `P` with a nonconforming member is now refused at `P`, while a `C` with the same bad member is still admitted at `P`, which is asymmetric. (b) Item 11 says an identical key conforms even when the environment does not admit it. The new `type_refusal_walk` pre-check returns `UnknownDeclaration` for that key, so `admits(Composite(K), v_K)` with `K` unknown now returns `false`. No test asserts either change. Amend items 9 and 11 to the new rule, or restore the old behaviour for record and tuple composites. | src/declaration.rs:2628-2630, 2656-2697 |
| FND-003 | medium | The PR breaks the public API of a crate consumed on `branch = "main"`. `admits`, `check_type`, `contains_ieee`, `record`, `tuple`, `evaluate_record`, `evaluate_tuple`, `check_equality`, `build` and `ObjectClosure::new` all gain a budget parameter and a nested `Result`. quire-spec-language takes QSV at `branch = "main"`, so its next lock update stops compiling. The PR body puts "QSL-502/503/504 caller migration" outside the PR. Merging #12 therefore needs the paired QSL adaptation PR to land with it. | src/declaration.rs:2613, 2848, 2778; src/object_closure.rs:75; src/containment.rs:136 |
| FND-004 | medium | Composite supertype edges to or from a union declaration are not refused. quire-exact FR-379 Behavior 5 says a union declaration has no descendant. `check_composite_supertypes` accepts a record that names a union as a supertype, and a union that names supertypes. Then `composite_conforms(record, union)` returns `true`, which is a wrong answer from a public query. `admits` still refuses the value only because the exact-key `match` falls through to the `Union` arm. Refuse both edge directions at admission with a typed cause. | src/declaration.rs:3314-3379, 2266-2272 |
| FND-005 | low | The merge did not fix the orphan doc comments; it stacked them. The `contains_ieee` doc opens with `check_type`'s summary ("Check a type named outside a declaration..."), and `check_type` carries `type_refusal`'s summary ("The first refusal of one type's named declarations"). `union`, `evaluate_union`, `seal_union_verified`, `same_type`, `runtime_type`, `union_member_named` and `admits` each have two or three stacked summaries that partly repeat each other. Keep one summary per item. | src/declaration.rs:1808-1810, 1850-1853, 2285-2294, 2385-2387, 2446-2451, 2579-2580, 2608-2612, 2769-2777, 2845-2847 |
| FND-006 | low | `operand_value` calls `admits_walk` with a no-op charge. Its worklist memory is bounded by the operand's occurrences, because each push is a distinct occurrence and a union payload's length equals the declared arity, which is checked before the push. Its time is not linear, though. Every `Option` and `Collection` occurrence runs `same_type_walk` over the rest of its type chain, so a nest of depth d costs O(d²) uncharged steps. FR-113 item 14 claims "time linear in the value's occurrences". Either charge type links to the meter or bound them, or state the O(occurrences x declared type depth) cost in item 14. | src/declaration.rs:5476-5483, 2708-2724 |
| FND-007 | low | This finding is outside the diff: `compute_composite_ancestors` on main (from #14) has a production `expect("validated composite key")`. The surrounding validation makes it unreachable in practice, but it is still panic surface in library code. Replace it with a typed refusal or a let-else that returns a checked-invariant failure. | src/declaration.rs:3398 |
