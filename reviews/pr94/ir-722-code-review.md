---
id: SR-20261102072338
title: "IR-722 QSV path arena capacity code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value PR94 cd644c08; src/supplied_membership.rs, src/supplied_stop.rs, FR-109-AC-9, TC-908"
review_set: subset
---

## Summary

Scoped Rust/code review of PR94 at cd644c08: the private counter path exercises the same arena allocation and supplied-run error mapping as production. No finding in the changed code or tests; aggregate gate evidence remains the code lead's responsibility.

## Verdict

**PASS** — no concrete defect in the reviewed diff. The process-global counter remains untouched by test injection, and no public API or compatibility path was added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
  - {id: FR-109-AC-9, path: spec/functional/FR-109-bound-supplied-value-traversal.md, role: examined, excerpt: "Last issuable arena ID succeeds; later requests return one-unit ArenaIdentifiers capacity; supplied run stops before parameter, event, locus, or work."}
  - {id: TC-908-step-10, path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md, role: examined, excerpt: "Private counter drives constructor boundary and production supplied-run mapping without changing process counter."}
  - {id: MembershipPathArena::new, path: src/supplied_membership.rs, role: examined, excerpt: "Self::with_counter(&NEXT_ARENA_ID)"}
  - {id: MembershipPathArena::with_counter, path: src/supplied_membership.rs, role: examined, excerpt: "next_arena_id(counter)?; return arena with that ID"}
  - {id: MembershipPathArena::new_with_test_counter, path: src/supplied_membership.rs, role: examined, excerpt: "cfg(test) crate-private wrapper around with_counter"}
  - {id: SuppliedRun::new, path: src/supplied_stop.rs, role: examined, excerpt: "Self::with_arena(limits, cancel, MembershipPathArena::new())"}
  - {id: SuppliedRun::with_arena, path: src/supplied_stop.rs, role: examined, excerpt: "Maps capacity to admission phase, no parameter/event/locus, zero consumed, Capacity(ArenaIdentifiers)."}
  - {id: arena_constructor_exhaustion_reports_one_native_unit_without_reusing_an_id, path: src/supplied_membership.rs, role: examined, excerpt: "Issues MAX-1 then asserts repeated one-unit capacity errors and unchanged MAX counter."}
  - {id: scoped_arena_takes_no_process_id_and_shares_none_with_issued_arenas, path: src/supplied_membership.rs, role: examined, excerpt: "Asserts scoped and issued tokens resolve only in their owning arenas."}
  - {id: exhausted_path_arena_stops_before_a_supplied_run_or_locus_exists, path: src/supplied_stop.rs, role: examined, excerpt: "Asserts complete typed SuppliedStop at exhausted private counter."}
```

## Review evidence

No vendored or copied artifact, lockfile edit, CI diff, unsafe block, new panic path, fallback, or public seam is in the PR diff. The atomic fetch_update remains the single ID allocator; Relaxed ordering is sufficient for atomic uniqueness because the ID does not publish other data. Each new test reaches production constructor or mapping code and has exact assertions capable of failing. Quire 0.38.3 matrix binds FR-109-AC-9 to all three tests and leaves FR-109-AC-5 tagged by its own tests. The two edited spec status paragraphs are truthful to those tests and preserve the no-downstream-trigger limit. Quire validation of both edited spec files passed. Focused test, fmt and Clippy results were reported by the coder; final aggregate gates are pending with the lead.
