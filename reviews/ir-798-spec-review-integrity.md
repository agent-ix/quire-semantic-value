---
id: SR-11151
title: "IR-798 specification integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value; spec/functional/FR-118-bound-retained-decimal-type-links.md; spec/test-cases/TC-918-retained-decimal-type-link-admission.md"
review_set: subset
---
## Summary

The identity fields and four-endpoint precedence are internally consistent with DecimalType and QSpec FR-140. The finite helper-cost and storage qualification is incomplete.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-118 calls Decimal type equality bounded but gives no finite comparison-step bound, cancellation response gap, or temporary-storage bound for this specific helper. FR-109 behavior 5 requires these source-backed premises before QSV can claim support; Decimal value membership FR-372 and Integer/Rational type equality FR-381 do not prove this helper's costs. State measurable bounds in the owning kernel contract and cite them here; TC-918 should verify the reachable bound and storage route. | spec/functional/FR-118-bound-retained-decimal-type-links.md:58; spec/test-cases/TC-918-retained-decimal-type-link-admission.md:30 |

## Examined scope

FR-118 description, inputs, outputs, behavior, dependencies and TC-918 procedure/expected results were examined. All five FR-118 criteria were examined:

- FR-118-AC-1: Public supplied admission compares equal and unequal Decimal links nested in Option and Collection, charging one type-link event per compared pair with the independently enumerated zero-based ordinal and argument-rooted locus. At N-1, event denial prevents helper entry; at N, the helper may enter.
- FR-118-AC-2: With configured B, all four coefficient bounds at B complete. Each bound at B+1 yields typed unsupported size naming expected/actual and lower/upper, measured bits, B and rooted locus. Simultaneously oversized bounds refuse in expected-lower, expected-upper, actual-lower, actual-upper order. Zero counts as one bit, so B=1 admits it and B=0 refuses at expected lower. A scale or rounding-mode difference yields invalid type only after all four bounds pass.
- FR-118-AC-3: Pre-cancellation and cancellation during a longest admitted coefficient comparison retain the original cause, rooted link locus and spend through the initiating event, with no next comparison or evaluator charge. Typed kernel resource stops, if its implementation can produce them, retain their request and locus.
- FR-118-AC-4: With no configured B, a permitted paired Decimal link returns `UnsupportedHelper::DecimalDomain` after its event without helper entry. Event denial and cancellation from the caller's original Cancel retain their established precedence.
- FR-118-AC-5: Inspection confirms one authentic B, expected-first borrowed operands and original Cancel reach the authoritative exact-kernel type-equality operation. No unbounded `ValueType` equality or QSV-owned duplicate comparator can decide paired Decimal links.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-118 behavior 3 and AC-5 now state the proposed 2ceil(B/32) limb-pair ceiling, constant auxiliary storage, bounded cancellation gaps, and a conditional measured scratch contract. TC-918 step 3 verifies these against the delivered production helper; implementation cannot claim support before source qualification. |
