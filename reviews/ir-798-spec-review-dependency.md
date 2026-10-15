---
id: SR-11152
title: "IR-798 specification dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-semantic-value; spec/functional/FR-118-bound-retained-decimal-type-links.md; spec/test-cases/TC-918-retained-decimal-type-link-admission.md"
review_set: subset
---
## Summary

FR-109 and FR-117 are valid contextual links. Decimal type equality has no named exact-kernel requirement or delivered helper in the locked dependency; the draft correctly says implementation waits, but the prerequisite edge is not traceable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The dependency section says the exact kernel must specify and deliver Decimal type equality without naming a kernel FR or explicit dependency edge. The locked quire-exact revision 9a3411d has DecimalType::try_contains and numeric type equality for Integer/Rational, but no DecimalType bounded equality contract or operation. Link a concrete exact-kernel requirement once authored and make the prerequisite for implementation explicit. | spec/functional/FR-118-bound-retained-decimal-type-links.md:95; spec/test-cases/TC-918-retained-decimal-type-link-admission.md:15 |

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
| FND-001 | deferred | The exact-kernel Decimal type-equality FR has not yet been allocated or authored. FR-118 dependencies now explicitly make that requirement, its delivered helper, and a later typed frontmatter edge blocking prerequisites to implementation and acceptance; TC-918 status agrees. Add the edge once the owner creates the real artifact. |
| FND-001 | deferred | IR-799 now exists and the structured Linear relation confirms IR-798 is blocked by IR-799. FR-118 and TC-918 name that blocker and keep implementation and acceptance pending. The exact FR artifact is still unpublished, so its typed frontmatter edge must be added once authored; this does not prevent opening the explicitly blocked spec-only PR. |
| FND-001 | fixed | FR-382 is now published in quire-exact main d18927e8 with `DecimalType::try_eq_bounded`; FR-118 carries a typed `depends_on` edge to that real artifact and cites its measured borrowed-limb, cancellation and allocation-free contract. TC-918 names the published helper. The QSV implementation and acceptance tests remain unrun and are separate work. |
