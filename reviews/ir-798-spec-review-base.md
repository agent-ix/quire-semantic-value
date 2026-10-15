---
id: SR-11150
title: "IR-798 base specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value; spec/functional/FR-118-bound-retained-decimal-type-links.md; spec/test-cases/TC-918-retained-decimal-type-link-admission.md"
review_set: subset
---
## Summary

Reviewed FR-118 and TC-918 against FR-109, the QSV supplied-membership implementation, the current exact-kernel contract, and the computed matrix. Two acceptance oracles need correction. FR-118-AC-1 through AC-4 are currently untagged because implementation and tests are planned; AC-5 is inspection-only. That status is consistent with the candidate's unrun declaration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-118-AC-3 requires pre-cancellation to retain spend through the initiating event, contradicting FR-109's poll-before-charge rule. A pre-cancelled call spends no new event; only cancellation observed after a permitted event includes it. Split those expected spends in the AC and TC. | spec/functional/FR-118-bound-retained-decimal-type-links.md:84; spec/test-cases/TC-918-retained-decimal-type-link-admission.md:30; src/supplied_membership.rs:588 |
| FND-002 | medium | FR-118-AC-1 and TC-918 use N/N-1 without defining N at the target Decimal link. If N counts the complete successful admission, N-1 may deny an event after Decimal equality and cannot prove helper non-entry. Define N as cumulative units through the target type-link event and assert that event and locus are denied at N-1. | spec/functional/FR-118-bound-retained-decimal-type-links.md:82; spec/test-cases/TC-918-retained-decimal-type-link-admission.md:23 |

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
| FND-001 | fixed | FR-118 behavior 1 and AC-3 now separate pre-event cancellation, which retains prior spend, from cancellation after a successful initiating event, which retains that event; TC-918 step 3 asserts both. |
| FND-002 | fixed | FR-118-AC-1 and TC-918 step 1 define N as cumulative spend immediately after the target Decimal link event and assert that N-1 denies that exact event and locus before helper entry. |
