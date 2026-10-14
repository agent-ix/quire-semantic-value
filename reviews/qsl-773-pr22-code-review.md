---
id: SR-9117
title: "Code review of quire-semantic-value PR #22: typed-event membership seam"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@205f26d54f129021e26ba8b3bcf00195dc9aeded; commits f82cb1e and 205f26d over carrier 46a8c85; src/declaration/membership.rs, src/declaration.rs, src/supplied_membership.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# Code review of quire-semantic-value PR #22

## Summary

Ticket: QSL-773. Code review with the rust-review lane folded in. Scope is the
two commits on top of luna's carrier (`git diff 46a8c85..HEAD`); the carrier
`src/supplied_membership.rs` was read as context only.

- Behaviour preservation: checked by a differential derived from the code,
  arm by arm, against `origin/main` `TypeEnvironment::admits`. Reference
  conformance, Composite conformance then stop, Composite-vs-Union refusal and
  the kernel leaf fallthrough are the same. The Option arm is the same: the
  kernel's `option.payload_type() == payload` is a link-by-link loop over
  `shallow_eq` (quire-exact d41a1da `value_type.rs:178`), and `same_type_walk`
  makes the same comparisons: Option links match by variant, Collection links
  by kind and bound, every other link by `==`, and it descends through the
  same children. The Collection arm is also the same: kernel
  `collection_type() == declared` is kind, bound, then the element chain,
  which is TypeLink(0) and then `same_type_walk` from TypeLink(1). Visit order
  changed (old code popped the last element first; new code visits forward).
  That cannot change a conjunction of answers. No arm differs.
- Event contract: one event before each ValueVisit, TypeLinkComparison
  (initial pair included), actual ChildScheduling and DescriptorQuery. A
  callback `Err` returns unchanged through `?` before the action is taken. The
  arena node is appended only after the scheduling event passes. No batched
  or entry charge. The 11-event stream for `Some([1,2])` is right: the Option
  arm compares the payload chain (Sequence link, then Integer link), and the
  Collection arm compares its own link at TypeLink(0) and the element chain
  from TypeLink(1). That matches the kernel's link count, so "each pair, the
  initial one included" holds.
- Storage: the worklist is reserved once per batch, before the first
  ChildScheduling event of that batch. Pushes then fill the reserved capacity
  and never reallocate. Allocation and Capacity stay distinct. The walker has
  no `vec!`, `extend`, `unwrap`, `expect`, indexing or `unreachable!` in
  non-test code. A foreign starting locus gives `Invalid{ForeignPath}` with no
  event fired.
- Gate evidence (author logs `/home/peter/dev/worktrees/logs/qsv773-*.log`):
  the red log shows the 8 new tests failing against the stub, each for the
  right reason (no events, or `Ok` where a typed stop was due). The gate log
  shows fmt, clippy -D warnings on all targets and on thumbv7em, 86 tests,
  the thumbv7em build, deny and docs all passing. This reviewer ran no build.

## Verdict

**FAIL: one blocking finding (FND-001).** The walker itself is correct and
keeps main's answers. The defect is in how the plain `admits` path gets its
arena. Recorded, non-blocking: FND-002 to FND-005 (medium/low).

Merge condition, not a finding: QSL main calls `types.admits(..)` as a `bool`
at `qsl-eval/src/value/expression/mod.rs:191`,
`qsl-replay/src/result/wire/value.rs:100,109` and
`qsl-replay/src/spine/call.rs:738`. QSL tracks QSV `branch=main`, so this PR
lands together with the QSL adaptation PR (the PR body says it is separate).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Every plain `admits_checked` call (public `admits`, `fill_slots` per field, `tuple` per position, `admitted`, and `operand_value` twice per equality) draws a fresh id from the process-wide `usize` arena counter, although no token from this path ever escapes. On thumbv7em (a gated target) the counter is exhausted after about 4.29e9 calls; at monitor rates this is hours to days. From then on, every call fails for the life of the process: `admits` returns `Err(Capacity)`, construction refuses with `StorageCapacity`, and `admitted`/`operand_value` report the false checked invariants `DeferredResultNotAdmitted`/`EqualityOperandSourceNotAdmitted` for valid values. These are wrong results where the old code could not fail. The plain path should use an arena that takes no process id. | src/declaration.rs:2150-2155; src/declaration.rs:3868; src/declaration.rs:4746; src/supplied_membership.rs:126-153 |
| FND-002 | medium | Recorded. Carrier `append_child_owned` grows the arena with `try_reserve_exact(1)` per node, and this PR now runs it once for every child on main's hot admission path. On an allocator that cannot grow a block in place (a no_std target), large collections cost quadratic copying. Carrier code, owned by luna:IR. | src/supplied_membership.rs:204-210 |
| FND-003 | medium | Recorded. `admitted` and `operand_value` report a storage failure as the not-admitted checked invariant. This is documented in the PR, and under `GlobalReservations` it is reachable only on a real allocator failure, where the old code aborted. A proper fix needs a storage channel in the quire-exact `Stop`. | src/declaration.rs:3865-3871; src/declaration.rs:4744-4746 |
| FND-004 | low | Recorded. `admits_checked` maps `Limit`/`Cancelled` to a fabricated `Capacity(0)`. Neither can fire: the callback is a no-op, and `append_child` produces only Allocation or Capacity. A total type-level match would need a carrier error type without those variants. Arena-id exhaustion is also labelled with unit `AdditionalElements` instead of `ArenaIdentifiers`. | src/declaration.rs:2150-2155; src/declaration.rs:2176-2185 |
| FND-005 | low | Recorded. The `_` leaf arm also gets kind mismatches such as an `Option` type against an `Integer` value, so these are labelled `LeafMembership`. The bool answer is correct. FR-109 Dependencies still says "`TypeEnvironment::admits` is bool-only". | src/declaration/membership.rs:229-233; spec/functional/FR-109-bound-supplied-value-traversal.md |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | Recorded. The `leaf_cause` mapping added in 1210842 has no test for its Payload, Type and Member arms; only Kind is asserted. A `Population` pair has matching kinds, yet the kernel always refuses it and the mapping labels it `Kind`. Every bool answer is still correct. | src/declaration/membership.rs:82-96 |

## Dispositions

Round 1, reviewed at 12108429e38885b4d85b4cf07e2b4d356dfca8d3.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 1210842: `admits_checked` walks in `MembershipPathArena::scoped()` (id `usize::MAX`), which the counter never issues. `fetch_update` with `checked_add` returns `usize::MAX - 1` last, then `Err`, and never wraps. `scoped` is `pub(crate)`, `admits_walk` is `pub(super)`, and only `StorageFailure` (no locus) leaves the plain path, so a scoped token cannot escape. |
| FND-002 | deferred | Carrier `append_child_owned` growth is unchanged. It is luna:IR's carrier code and stays with its owner. |
| FND-003 | fixed | 1210842: `admitted`/`operand_value` call `admits_or_abort`, which aborts through `handle_alloc_error` on a storage failure (main's infallible worklist aborted too). A valid value is never reported as not admitted. |
| FND-004 | accepted-no-change | The unit mislabel is gone because `new()` is no longer called. The `Limit`/`Cancelled` arm stays and cannot fire (no-op callback; `append_child` yields only Allocation or Capacity). Low and recorded. |
| FND-005 | accepted-no-change | `LeafMembership` is replaced by the kernel-derived causes in 1210842. FR-109 Dependencies line 140 still says "bool-only". That is a low spec-text item, recorded. |
| FND-006 | accepted-no-change | Low, recorded in the last review round. The cause labels do not change any admission answer. |
