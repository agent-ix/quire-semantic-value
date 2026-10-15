---
id: SR-11073
title: "IR-719 gap analysis of supplied Decimal admission"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; PR #29 6d892044 over 0e12429; FR-109, TC-908, src/declaration/membership.rs, src/supplied_membership.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-719 gap analysis

## Summary

Ticket: IR-719. Planless, diff-scoped comparison of the proposed FR-109-AC-8 and TC-908 step 8 against the three added Decimal tests and the changed supplied-admission path. The tests exercise event budgets, upper-endpoint size refusal, nonmembership and late cancellation; two required refusal/size branches remain untested at QSV's entry point.

## Verdict

**CONDITIONAL** — two medium acceptance-test gaps. The production mapping was inspected separately in SR-11072 and has no identified defect.

## Coverage

Plan completion: not assessed.

`quire matrix --scope . --format json` on the code branch shows existing FR-109-AC-1 through AC-5 tagged, AC-6 untagged, and AC-7 as an inspection criterion. Proposed AC-8 is absent because spec PR #28 has not merged; the new tests cannot yet produce an AC-8 matrix binding. Reviewed exact-kernel FR-372-AC-5 as the cross-repo acceptance context. Examined TC-908 steps 7 and 8 and the new test bindings to FR-109-AC-1 through AC-4. No new source stub or unowned production algorithm was found. Optional repository-wide semantic review was not run; the changed path's test oracles were inspected manually.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No QSV test induces actual Decimal scratch allocation denial or checked capacity and verifies rooted typed mapping and retry | src/declaration/membership.rs:298 |
| FND-002 | medium | Bit-ceiling tests cover only the upper endpoint, leaving coefficient and lower operand selection unverified | src/declaration/membership.rs:275 |

The pending FR-109-AC-8 and TC-908 step 8 require an actual kernel scratch denial and checked-capacity failure to remain distinct supplied-admission causes at the Decimal locus, with no partial result and a successful retry. The new QSV tests only exercise upper-size, nonmembership and cancellation. A regression mapping `DecimalMembershipFailure::Allocation` to `Capacity` or losing the locus would pass them. Exercise the actual production helper under a denial seam, or provide a source-grounded equivalent that reaches each new QSV mapping; assert the measured native request and retry.

The caller's B premise must cover `bits(c)`, `bits(lo)` and `bits(hi)`. The sole new size case makes `hi` oversized. An implementation that accidentally skips the normalized coefficient or lower endpoint passes it. Add separate direct QSV cases with only `c` and only `lo` above B, asserting `DecimalSize` operand, measured bits, B, charged spend and rooted locus. A boundary case at exact B would verify inclusive acceptance.

## Dispositions

Round 1 on reviewed head `c507024d7faf51bea788048936497f8a6ba95cb2` against merged QSV spec `1140d400`. The changed projection test passed as one focused library test. The original finding rows above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Production `decimal_value_result` now maps both constructed typed kernel failures through the same function used by `admit_supplied_bounded_numeric`; the test asserts distinct measured capacity/allocation fields, `Err` rather than partial success, and the resolved rooted locus. Merged FR-109-AC-8 and TC-908 step 9 allocate actual allocator/capacity mechanism testing to exact-kernel TC-920, which already owns the kernel tests. |
| FND-002 | fixed | Separate QSV calls now put only the normalized coefficient and only the lower endpoint over B, asserting first operand, measured bits, limit, locus and spent value; the prior upper case remains. |
| FND-003 | still-open | The newly merged FR-109-AC-8 is untagged in the computed matrix; this round found the gap and no fix is on the reviewed head. |
| FND-003 | fixed | Round 2: AC-8 now has meaningful resource-projection and cancellation binders on `ca7bdca0`; the round-1 untagged state is superseded. |
| FND-004 | still-open | Round 2: AC-8's direct N/N-1 and nonmembership clauses are asserted but their tests lack this criterion's trace tag. |
| FND-004 | fixed | Round 3: the direct Decimal N/N-1 and nonmembership tests now carry FR-109-AC-8 tags; the computed matrix lists all four meaningful binders. |

After excerpt for FND-001: `return decimal_value_result::<Cause>(domain.try_contains(decimal, cancel), locus, spent.get());` and `Err(DecimalMembershipFailure::Allocation { additional_elements: 3, element_width: 4, layout_bytes: 12 })` flows to `BoundedSuppliedAdmissionError::DecimalAllocation` in the focused test.

After excerpt for FND-002: `operand: DecimalMembershipOperand::Coefficient, measured_bits: 8, limit_bits: 7, locus: at` and `operand: DecimalMembershipOperand::Lower, measured_bits: 7, limit_bits: 6, locus: at` are asserted by distinct supplied calls.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Merged FR-109-AC-8 is untagged despite the new integrated Decimal and projection tests | src/declaration/membership.rs:2664 |

`quire matrix --scope . --format json` reports FR-109-AC-8 as `untagged` on this exact head. The resource projection test carries only `#[trace("TC-908", "FR-372-AC-5")]`; the other three Decimal tests bind existing FR-109 ACs but not AC-8. The newly merged AC-8 is thus invisible to the computed matrix and its acceptance evidence cannot be found by criterion. Bind the direct N/N-1, size/nonmembership/cancellation, and resource-projection assertions to FR-109-AC-8 where each actually tests its clause; then verify the computed binding. This is a changed-scope trace gap, not a request for additional exhaustive tests.

Current verdict: **CONDITIONAL** solely for FND-003. FND-001 and FND-002 are fixed on this head.

## Round 2 evidence

Reviewed `ca7bdca0c6f48f5462447841b4b415f643da692b` against `c507024d7faf51bea788048936497f8a6ba95cb2`. The changed four lines add FR-109-AC-8 to the resource-projection and cancellation test traces. `quire matrix --scope . --format json` now reports FR-109-AC-8 as `tagged` with those two test binders. No code path changed.

After excerpt for FND-003: `#[trace("TC-908", "FR-109-AC-8", "FR-372-AC-5")]` on `decimal_kernel_resource_failures_project_without_partial_admission` and `#[trace("TC-908", "FR-109-AC-3", "FR-109-AC-8", "FR-372-AC-5")]` on `bounded_decimal_maps_original_handle_cancellation_during_helper`.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | AC-8's N/N-1 and nonmembership assertions exist but are not bound to that criterion | src/declaration/membership.rs:2459 |

FR-109-AC-8 requires an integrated Decimal call to preserve N/N-1 admission and inclusive membership/nonmembership under the caller's bounds. `bounded_decimal_uses_cumulative_exact_event_boundary` asserts the N/N-1 and inclusive cases, and `bounded_decimal_retains_size_and_nonmembership_locus` asserts an out-of-range Decimal refusal; neither test tags AC-8. The two new AC-8 binders only cover constructed resource projection and cancellation. A reviewer following the computed AC-8 matrix row would miss the tests for its direct membership clauses and could not distinguish complete coverage from a partial tag. Add AC-8 trace tags to those existing tests, without expanding their assertions, and verify the computed binders list.

Current verdict: **CONDITIONAL** solely for FND-004. No production code defect was found.

## Round 3 evidence

Reviewed pushed head `a113677459ba5d3407897cb43970a65855cfdf40`. The diff changes only Trace comments and attributes on two existing Decimal tests. `quire matrix --scope . --format json` reports FR-109-AC-8 `tagged` with four binders, matching the four tests below.

| AC-8 clause | Evidence checked |
| --- | --- |
| Owning value visit, cumulative budget, N/N-1 and zero, inclusive upper result, no helper debit | `bounded_decimal_uses_cumulative_exact_event_boundary`: two calls at budget 2 return `Ok(())` with spend 2; the same calls at budget 1 deny the second `ValueVisit` with spend 1 and rooted second argument; zero denies the first. The `(1,0)` fixture against `[0,100]` scale 2 completes at the inclusive upper bound. |
| B over coefficient and both endpoints, nonmembership and locus | `bounded_decimal_retains_size_and_nonmembership_locus`: three distinct oversized positions preserve measured bits, B, spend and rooted locus; the out-of-range value returns typed `Invalid(Payload)` at that locus. |
| Constructed capacity and allocation projection, no partial result | `decimal_kernel_resource_failures_project_without_partial_admission`: constructed exact failures pass through production `decimal_value_result` and emerge as distinct `Err` values with native measurements and the same resolved root. Actual scratch mechanism tests belong to exact TC-920 per merged FR-109-AC-8. |
| Original Cancel and admission phase | `bounded_decimal_maps_original_handle_cancellation_during_helper`: observed helper polling returns `Cancelled(Deadline)` with spent 1, event `ValueVisit`, original handle tripped, and root locus. |
| Missing-bound/default refusal | The pre-existing `supplied_api_reports_unqualified_decimal_before_the_helper` tests the ordinary API with no B and receives `UnsupportedHelper(DecimalDomain)` before helper completion, for direct Decimal and retained type link. The bounded API requires B explicitly; S is the admitted well-formed Decimal type's nonnegative `min_scale`, which bounds the actual alignment shift. No default B or S is selected. |

The QSV path has no evaluator meter or descriptor output to partly publish; no changed code adds either. The review claims the changed QSV caller and matrix, not downstream QSL integration. Current verdict: **PASS** for this PR diff; FND-001 through FND-004 have final fixed dispositions and no new findings emerged in the full AC-8 audit.
