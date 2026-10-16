---
id: SR-11610
title: "Code and Rust review of IR-714 budgeted 10k supplied-admission tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; tests/union_staging.rs; FR-109-AC-1; TC-908; QSL FR-321-AC-4; ticket IR-714; branch code/ir-714-budgeted-10k-test at 9f767240d997b776f8328de19ba93dc5a5cd4310 vs origin/main 98091ee525ff10e24f991fd8e15a36c4f79c0700"
review_set: subset
---

## Summary

Ticket: IR-714. Reviewed `git diff origin/main...9f76724`, one commit that changes only `tests/union_staging.rs` (+130/-1): a shared `sealed_tree_10k_supplied_fixture`, an exact-N completion test and an N-1 denial test, both on a 512 KiB thread stack. The Rust review is folded into this artifact. There is no change to `src/`, `Cargo.toml`, CI or spec, and no new `#[allow]`. The `#[allow(clippy::disallowed_methods)]` near the FR-321-AC-2 test is on `origin/main` (line 717 there).

## Verdict

**PASS with low findings.** The reviewer derived N independently from `admits_walk_core` / `admit_occurrence` / `same_type_walk` / `Frontier::schedule` (`src/declaration/membership.rs`) and `SuppliedMembershipBudget::charge` (`src/supplied_membership.rs`). Each `Next` link costs 6 charged events: ValueVisit(union), DescriptorQuery(union), ChildScheduling(Option payload slot), ValueVisit(Option), TypeLinkComparison(Composite(40) vs Composite(1), Equal after one pair), ChildScheduling(nested union). `End` costs ValueVisit and DescriptorQuery, and it has no payload, so it makes no reserve and no schedule. `reserve` and the `AfterHelper` boundaries poll and do not charge. That gives N = 10,000 × 6 + 2 = 60,002, which matches the test.

At ceiling N-1, the 60,001 successful events end before the `End` DescriptorQuery. The limit then reports ceiling 60,001, spent 60,001 and requested 1, at locus argument 7, `Descriptor(0)`, 2 steps × 10,000 links = 20,000 steps from the leaf. Every assertion is an exact typed value, with no `is_err` and no skip guard. All 14 count, event, locus and budget mutants were killed. The walk is an explicit worklist (`membership.rs:564`), and the value nests 20,001 levels deep on a 512 KiB stack (about 26 B per level), so a recursive walk could not pass by luck. `cargo fmt --check` and `cargo clippy --test union_staging -D warnings` are clean.

## Scope examined

- `FR-109-AC-1` (examined): A source-enumerated supported membership fixture requiring N logical events completes at N and fails at N-1 before the last event, retaining supplied-admission phase, configured N-1, successful spend N-1, next amount one and the actual denied event/locus.
- `TC-908` step 1 (examined): Independently enumerate event order and N; run N-1 and N and inspect the actual event performed at the refusal boundary.
- QSL `FR-321-AC-4` (context_only): A supplied Tree 10,000 levels deep is admitted, converted and compared equal without host stack overflow; with the value-occurrence limit one below the occurrence count, admission returns that limit by name.
- `tests/union_staging.rs:715-842` (examined): fixture, both new tests, import change.
- `src/declaration/membership.rs:72-131,193-279,541-804` (examined): Frontier reserve/schedule, same_type_walk, admits_walk_core, admit_occurrence.
- `src/declaration.rs:2721-2788` (examined): `admit_supplied` routes `Event` to `budget.charge` and `AfterHelper` to `check_cancel`.
- `src/supplied_membership.rs:555-629` (examined): `phase()`, `SuppliedMembershipBudget::charge` (`next <= ceiling`, denied event leaves spent unchanged).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `assert_eq!(stop.phase(), SuppliedAdmissionPhase::Membership)` cannot fail: `SuppliedAdmissionError::phase` is a `const fn` returning `Membership` for every variant, so the assertion and its new import add no oracle strength | tests/union_staging.rs:830 |
| FND-002 | low | N = 60_002 and the 20_000 step count are literal constants repeated per test, not computed from a single link-count constant shared with the fixture's `0..10_000`; the derivation lives only in the fixture doc comment (a fixture-size change still fails loudly) | tests/union_staging.rs:755 |
| FND-003 | low | The new fixture duplicates the pre-existing FR-321-AC-4 test's Tree construction verbatim instead of that test reusing `sealed_tree_10k_supplied_fixture`; two copies of the same 10k Tree can drift | tests/union_staging.rs:719 |

## Evidence

Mutants were applied in a throwaway worktree and reverted after each run. "Killed" means at least one of the two new tests failed.

| Mutant | Exact-N | N-1 | Result |
| --- | --- | --- | --- |
| M1 child scheduling undercharged (Event→AfterHelper) | fail | fail | killed |
| M2 ValueVisit overcharged (charged twice) | fail | fail | killed |
| M3 type comparison charge skipped | fail | fail | killed |
| M4a End leaf extra charge | fail | pass | killed |
| M4b End leaf DescriptorQuery skipped | fail | fail | killed |
| M5a union descriptor named ValueVisit | pass | fail | killed |
| M5b union descriptor locus Value instead of Descriptor(0) | pass | fail | killed |
| M6a `next < ceiling` | fail | fail | killed |
| M6b `next <= ceiling + 1` | pass | fail | killed |
| M7a reported spent − 1 | pass | fail | killed |
| M7b requested 2 | pass | fail | killed |
| M7c denied event spent (reported and stored) | pass | fail | killed |
| M7d denied event spent silently (stored only) | pass | fail | killed |
| M8 Next descriptor relabelled TypeLinkComparison (count-preserving, non-final) | pass | pass | survives these tests; killed by existing `union_payload_worklist_denial_is_typed_and_retry_succeeds` |

M8 is expected. AC-1 asks for the denied event, not the full sequence, and the existing unit tests enumerate full sequences.

## Dispositions

Round 1, checked against `git diff 9f76724..abc8f1f` (head `abc8f1f37292bc5a3ce19e2706e050ceef974d4d`). The same 14 mutants were re-run against the refactored tests. All 13 count/event/locus/budget mutants are killed by the new tests. The count-preserving relabel is still killed only by the existing unit test `union_payload_worklist_denial_is_typed_and_retry_succeeds`. Changing `TREE_LINK_COUNT` to 9,999 fails the existing test's literal `20_001` occurrence assertion.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The `stop @` binding, the `stop.phase()` assertion and the `SuppliedAdmissionPhase` import are removed. The let-else now matches `Err(SuppliedAdmissionError::Limit { .. })` directly. |
| FND-002 | fixed | `TREE_LINK_COUNT`, `TREE_EVENTS_PER_LINK`, `TREE_END_EVENTS` and `TREE_STEPS_PER_LINK` now derive `TREE_EVENT_COUNT` (60,002) and `TREE_STEP_COUNT` (20,000). The fixture loop uses `TREE_LINK_COUNT` and asserts `occ() == TREE_STEP_COUNT + 1`. |
| FND-003 | fixed | One `sealed_tree_10k_supplied_fixture` now returns a builder closure that both the existing FR-321-AC-4 test and the new tests use. The existing test's body from `let left = supplied();` to its end is byte-identical to 9f76724. |
