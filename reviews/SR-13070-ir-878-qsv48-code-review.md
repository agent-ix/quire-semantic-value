---
id: SR-13070
title: "IR-878 quire-semantic-value PR 48 code review: FR-122-AC-2/AC-3 trace tags"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@5f83bbef5b45b10b0144f5a778eb9e676efcbb12 vs origin/main; src/declaration/membership.rs, spec/functional/FR-122-supplied-membership-owner-obligations.md, spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-908
    type: reviews
---

## Summary

Ticket: IR-878. The review covers code-review with the rust-review lane folded in. The diff adds `#[trace]` ids FR-122-AC-2 and FR-122-AC-3 to seven existing tests in `src/declaration/membership.rs` and edits two Status/prose paragraphs. No production code changes. Each tag was audited clause by clause against the AC text as merged on main (QSV #47). Mutants were applied to `src` in a throwaway worktree and the membership test module was re-run. Results:

- M1, which polls the original handle and enters the helper before the charge: killed only incidentally, by the cancellation poll-count tests.
- M1b, which enters `DecimalType::try_contains` before the ValueVisit charge on a fresh handle: SURVIVED.
- M2, which records Cancelled `spent` per call rather than cumulatively: SURVIVED.
- M3, which drops the cancellation locus: killed.
- M4, which collapses allocation into capacity: killed.
- M5, which drops the Lower endpoint size check: killed.
- M6, where a Decimal type link ignores B: killed.
- M7, where an Integer value ignores B: killed, but only by tests not tagged FR-122-AC-3.
- M8, where an absent B admits Integer: killed.
- M9, which adds an extra Decimal helper event: killed.

`make fmt-check lint` passes at head.

## Reviewed units

- FR-122-AC-2 (examined): An integrated supplied Decimal fixture with independently enumerated N owning logical events succeeds at N and denies its actual next event at N-1 before entering that event's helper. Reachable membership, nonmembership and original-handle cancellation retain the owning supplied-admission locus and already-spent budget across arguments, with no additional helper event, per-limb debit or partial descriptor. A separate QSV test passes constructed typed kernel allocation and capacity failures through its production error projection and retains their distinct causes, measured fields and rooted locus.
- FR-122-AC-3 (examined): The QSV supplied caller documents and tests the authenticated finite B for value plus every endpoint, invokes the qualified helper after charging one initiating logical event, and preserves original cancellation and locus. The direct QSV caller does not default B; absent B returns `SuppliedAdmissionError::UnsupportedHelper { family: UnsupportedHelperFamily::NumericDomain, .. }`.
- FR-122-AC-1 (context_only): Untouched; it stays PLANNED and its TC-906 tests keep their FR-369-AC-9 tags.
- quire-exact FR-380 (context_only): "Bounded cancellable Integer and Rational supplied membership"; its former AC-6 is the source of FR-122-AC-3.
- quire-exact FR-372 (context_only): Its former AC-5, the supplied Decimal control, is now allocated to QSV FR-122-AC-2.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-122-AC-3 is bound to the wrong tests. AC-3 carries quire-exact FR-380-AC-6, which covers bounded Integer and Rational membership, and it requires an absent B to return `NumericDomain`. The PR tags six Decimal tests with AC-3. One of them, `supplied_api_reports_unqualified_decimal_before_the_helper`, asserts `UnsupportedHelperFamily::DecimalDomain`, which contradicts the AC's literal family. The Integer/Rational tests that do assert B on the value and every endpoint carry no AC-3 tag: `bounded_numeric_value_checks_all_three_integer_and_six_rational_operands`, `bounded_numeric_type_links_check_expected_then_actual_endpoints`, `bounded_numeric_respects_event_order_zero_and_original_cancel` and `bounded_numeric_maps_cancellation_during_kernel_comparison`. Mutant M7, in which an Integer value ignores B, is killed only by those untagged tests. Fix: move FR-122-AC-3 to the Integer/Rational tests and drop it from the Decimal tests. If the owner intends AC-3 to cover Decimal as well, amend AC-3 to name `DecimalDomain` for Decimal. | src/declaration/membership.rs:2925,3367,3456,3524-3532,3625-3632,3791-3798 |
| FND-002 | medium | The FR-122-AC-2 clause "denies its actual next event at N-1 before entering that event's helper" is not asserted. `bounded_decimal_uses_cumulative_exact_event_boundary` checks only the `Stop::Limit` value and the spent count, which come out the same when the helper ran first. Mutant M1b calls `try_contains` before the ValueVisit charge on a fresh handle, and it survives the whole membership module. Fix: assert that the helper is not entered on the denied event, for example with an observing `Cancel` whose poll count stays 0, or with a `with_reservation`/probe seam. | src/declaration/membership.rs:3572-3604 |
| FND-003 | medium | The FR-122-AC-2 clause "nonmembership and original-handle cancellation retain ... already-spent budget across arguments" is not asserted. `bounded_decimal_retains_size_and_nonmembership_locus` and `bounded_decimal_maps_original_handle_cancellation_during_helper` each use a single argument with a fresh budget, where spent=1 is the same per call and cumulatively. Mutant M2 makes Cancelled `spent` per-call and survives. Fix: run a prior admitted argument on the same budget and arena, then assert nonmembership and cancellation at the second root with spent=2. | src/declaration/membership.rs:3720-3736,3836-3856 |
| FND-004 | low | The PR leaves the dangling `FR-372-AC-5` tag on all four retagged Decimal tests next to its successor `FR-122-AC-2`. quire-exact main removed that row in #56; FR-372 now says "formerly FR-372-AC-5 is allocated to QSV FR-122-AC-2". Strict coverage lists these tags only as "matches no declared row", which it prints for every cross-repo id, valid ones such as FR-369-AC-5 included. It does not count them, so nothing will flag the dangle. Drop `FR-372-AC-5` in this PR. | src/declaration/membership.rs:3529,3628,3743,3795 |
| FND-005 | low | The seven retagged tests keep deprecated `/// Trace:` doc lines that now disagree with their `#[trace]` attributes: the lines omit FR-122-AC-2/3, and the attribute still carries FR-372-AC-5. quire reads both forms (`rust-trace-line`). Delete the doc lines on the touched tests. | src/declaration/membership.rs:2924,3366,3455,3523,3624,3742,3790 |

## Verdict

AC-2 is partly backed. These clauses are asserted and mutation-checked:

- N/N-1/zero budgets with the denied event's locus.
- No extra helper or per-limb event (M9).
- The size and nonmembership loci.
- Cancellation cause and locus (M3).
- Distinct allocation and capacity causes, with measured fields and the rooted locus (M4).

The "before entering the helper" and "across arguments" clauses are not asserted (FND-002, FND-003). AC-3's tags sit on Decimal tests. The tests that actually exercise the FR-380 Integer/Rational contract are untagged, and one AC-3-tagged test asserts a different family (FND-001). The Status prose "FR-122-AC-2 and FR-122-AC-3 are backed" overstates the evidence until these are fixed. fmt and clippy pass. Not mergeable as-is.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-122-AC-3 now tags the Integer/Rational value, endpoint, event-order, cancellation, and absent-B NumericDomain tests. All Decimal tests dropped AC-3. |
| FND-002 | fixed | The denied N-1 Decimal event has an observing original handle and asserts exactly one budget-check poll, with no helper polls. A premature `try_contains` mutant failed with nine polls. |
| FND-003 | fixed | Nonmembership and kernel cancellation now follow an admitted argument on the same budget and arena, and assert cumulative spend 2. Per-call cancellation-spend and budget-spend mutants both failed. |
| FND-004 | fixed | The four Decimal tests dropped the deleted FR-372-AC-5 tag. |
| FND-005 | fixed | The seven originally retagged tests dropped their deprecated `/// Trace:` lines; the four newly tagged numeric tests also dropped theirs. |

## Follow-up verdict

The membership module passed (41 tests, one existing ignored resource probe); `make fmt-check` and one `make lint` pass passed. Strict coverage remains 95/106 backed, 10 unbacked, 0 contradicted. FR-122-AC-2 and FR-122-AC-3 remain tagged with evidence for their complete criteria.
