---
id: SR-11521
title: "IR-801 relationship extents gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; PR #37 diff cca2bc26...09a3641b; FR-119-AC-1..AC-20 against TC-919 tests and src/relationship_extent.rs, src/object_closure.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-119
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-919
    type: references
---

# IR-801 relationship extents gap analysis

## Summary

Ticket: IR-801. PR: quire-semantic-value#37 at 09a3641ba396a628c981ed03df08435f73f453ea. The computed matrix (`quire matrix --scope <worktree> --format json`, quoin 0.28.3) shows all twenty FR-119 criteria tagged: AC-11 by a unit test, AC-15 by a unit and an integration test, the rest by tests/relationship_extents.rs. The focused targets pass (2 unit, 19 integration). Semantic intent-test-code review was run as part of this dispatch, scoped to the diff. Code with no owning requirement: none; every new public item maps to FR-119 Behavior 1-11.

## Verdict

**FAIL** — FR-119-AC-10 is tagged but its runtime zero-allocation clause has no oracle that can fail, so the matrix overstates coverage. A safe, falsifiable test is available under the crate's `unsafe_code = "forbid"` lint (FND-001). The FR-119/TC-919 status prose is accurate: it claims AC-1..AC-9 and AC-11..AC-20 and marks AC-10's allocation clause PLANNED/UNRUN. Four medium oracle gaps (shared with SR-11520) remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-119-AC-10 requires a runtime assertion of zero allocations on the four reads; the tagged test asserts none, so an allocating `targets`/`sources`/`complete`/`resolve` passes it. The stated blocker is wrong: a dev-dependency allocator (allocation-counter 0.8.1, MIT/Apache-2.0, thread-local counts) needs no unsafe in this package and was shown to compile and detect an allocation under `unsafe_code = "forbid"` | tests/relationship_extents.rs:525-581; spec/functional/FR-119-admit-and-read-relationship-extents.md |
| FND-002 | medium | FR-119-AC-5: no case with only the target end unique or both ends unique; ignoring `target_end.unique` passes | tests/relationship_extents.rs:304-311 |
| FND-003 | medium | FR-119-AC-15: the several-unknown row has caller order equal to EffectiveId order, so "first in caller order" is not distinguished from "smallest" as TC-919 step 6 requires | tests/relationship_extents.rs:679-686 |
| FND-004 | medium | FR-119-AC-4 and Behavior 2: the explicit membership check before `conforms` in extent admission has no failing test; it is observable only when the table and the store use different environments, which no test does | src/relationship_extent.rs:263 |
| FND-005 | low | FR-119-AC-10 repeated-read part compares counts only and runs `resolve` only on the Present path; Dangling and Undecidable reads are not repeated against clones | tests/relationship_extents.rs:540-549 |
| FND-006 | low | FR-119-AC-11/AC-15 Capacity is shown through the mapping helpers, not through admission | src/relationship_extent.rs:853-868, 905-920 |
| FND-007 | low | FR-119-AC-4 "remains available to resolve" and FR-119-AC-8 type-component ordering are not asserted directly | tests/relationship_extents.rs:281-297, 439-507 |

## AC-10 recommendation

Keep AC-10 as Test; do not move the allocation clause to Inspection. Add `allocation-counter = "=0.8.1"` as a dev-dependency (MIT/Apache-2.0, both on the deny.toml allow-list; its `#[global_allocator]` and `unsafe impl GlobalAlloc` live in the dependency, and it counts per thread, so parallel tests do not interfere). In tests/relationship_extents.rs wrap each read in `allocation_counter::measure(|| ...)`, draining the iterators inside the closure: `complete` (listed complete, incomplete, unlisted), `targets` and `sources` (hit, miss, unlisted), and `resolve` for Present, Dangling and Undecidable (including the environment-mismatch path). Assert `count_total == 0`. Add one control that allocates inside `measure` (for example cloning the store) and asserts `count_total > 0`, so the probe itself is shown live. This reviewer built the same setup in a scratch crate with `[lints.rust] unsafe_code = "forbid"` and a `#![no_std]` lib: the read assertion passed with 0 and an allocating control reported 1. Run `make deny` once for advisories. Only if the owner refuses the dev-dependency should the clause move to Inspection, and then with a falsifiable statement: the four read bodies call no `alloc` API (`Vec`, `Box`, `collect`, `to_vec`, `clone` of owned data, `format!`), return borrowed iterators over existing slices, and `K: Ord` comparison is the caller's.

## Coverage

- Criteria in scope: 20 (FR-119-AC-1..AC-20); tagged 20; untagged 0; ignored-only 0.
- Criteria with a hollow clause: 1 (AC-10 zero allocations).
- Reverse gaps (code without an owning requirement): 0.
- Stubs or tautologies: none; the Float NaN self-equality and signed-zero inequality can fail under numeric equality.
- Semantic review: run, scoped to the PR diff.
- Plan completion: not assessed

## Examined scope

```yaml
scope:
- id: FR-119-AC-4
  path: spec/functional/FR-119-admit-and-read-relationship-extents.md
  role: examined
  excerpt: At instance index 0, a Tank source reference against declared Sys, and separately a Tank target reference against declared Pump, return typed EndTypeMismatch with relationship, extent index, instance index, side, actual type and declared type even when the end object is missing from the closure.
- id: FR-119-AC-5
  path: spec/functional/FR-119-admit-and-read-relationship-extents.md
  role: examined
  excerpt: Repeating the same relationship/source/target triple at instance index 1 returns typed DuplicateTriple with relationship, extent index and instance index when both ends are unique or only one end is non-unique.
- id: FR-119-AC-10
  path: spec/functional/FR-119-admit-and-read-relationship-extents.md
  role: examined
  excerpt: "Test: repeating every read gives equal results, zero allocations and charges, and closure and store equal to their pre-read clones, including after incomplete and unlisted reads."
- id: FR-119-AC-15
  path: spec/functional/FR-119-admit-and-read-relationship-extents.md
  role: examined
  excerpt: from_enumerator rejects duplicate population keys before unknown covered type or reservation, and rejects unknown covered types with distinct typed ManifestFailure<P> causes, naming the first unknown in caller order when several occur in one row.
```

## Dispositions

Round 1, reviewed head 8445fa44b4338c222e26a0dfb009b14be0d4f18e (diff 09a3641..8445fa4). The FR-119 and TC-919 status now claim AC-10 passed; the claim was rerun and holds.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | allocation-counter 0.8.1 is a dev-dependency only (caret "0.8", lock adds only that crate, no transitive deps, MIT/Apache-2.0, cargo deny ok, no_std lib build ok). Fixtures are built outside measure and iterators are drained inside it. All four reads are covered: complete for listed complete, incomplete and unlisted relationships; targets and sources for hit, miss and unlisted; resolve for Present, Dangling, Undecidable and an environment mismatch. Each asserts count_total == 0, and an allocating control asserts > 0. A Vec-collecting targets mutation fails the test. |
| FND-002 | fixed | All four end-uniqueness combinations are tested. |
| FND-003 | fixed | Caller order now differs from EffectiveId order. |
| FND-004 | fixed | Membership across separate TypeEnvironments is tested, and the deletion mutation fails. |
| FND-005 | fixed | resolve is repeated on all three outcomes and a cloned manifest is compared; target and source reads assert counts over an immutable &self store, so equal results follow. |
| FND-006 | fixed | Capacity is now driven through admission via CapacityPolicy. |
| FND-007 | fixed | The AC-4 resolve assertion and the AC-8 type-order fixture were added. |
