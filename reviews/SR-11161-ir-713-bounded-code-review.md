---
id: SR-11161
title: "Code and Rust review of IR-713 bounded reservation admission"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration/membership.rs; tests/bounded_numeric_reservation.rs; FR-109-AC-5; TC-908; ticket IR-713"
review_set: subset
---

## Summary

Reviewed frozen head `f0218bf8606e5554f001af7e125be14c44224931` against `38040e55`, including the Rust review. The bounded public entry point feeds the caller's reservation callback through the existing production walk while the original entry point retains its global policy.

## Verdict

**PASS** — No code or Rust defect was found in the changed path.

## Scope examined

- `FR-109-AC-5` (examined): A worklist reservation denial retains its measured request and native unit, remains separate from checked capacity overflow, and retry uses the same membership rules.
- `TC-908` step 5 (examined): Inject a denial at an actual traversal reservation, compare the request and retry without denial.
- `src/declaration/membership.rs` (examined): Both public bounded entry points, shared `BoundedNumericControls`, reservation forwarding, event charging, and existing Decimal typed error projection.
- `src/declaration.rs` reservation policy and fallible vector reservation (examined): The callback receives `StorageRequest` only for required capacity growth, after checked-size validation.
- `src/supplied_membership.rs` path growth and error types (examined): Child path reservation forwards the callback; `Admission::Allocation` and `Admission::Capacity` remain distinct from `DecimalAllocation` and `DecimalCapacity`.
- `tests/bounded_numeric_reservation.rs` (examined): Public Option/Decimal denial and retry, exact additional-element request, child locus, and cumulative budget assertions.
- `FR-109-AC-6` (context_only): The computed matrix's one existing untagged criterion is outside this reservation-only diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

`Frontier::reserve` and `MembershipPathArena::append_child` both use the forwarded policy. The test rejects the first Option payload reservation with `AdditionalElements(1)`, checks the rooted `OptionPayload` child part and spend of two, then permits both required growth requests and reaches `Ok(())` with spend four. The existing `decimal_value_result` mapping and its unit tests retain independent helper allocation/capacity variants. `cargo fmt --all -- --check` and `git diff --check` passed. Full build gates belong to the PR owner.
