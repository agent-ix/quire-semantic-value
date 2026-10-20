---
id: SR-13411
title: "IR-831 quire-semantic-value PR 52 EARS review of FR-124"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-semantic-value@f0cc3120be0a6f15eac62c9e2ca45e8b3630aca0 vs origin/main 9f1453a; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: reviews
---

## Summary

Ticket: IR-831. EARS requirement-grammar review of FR-124's Description statement and its seven
Behavior statements. Behaviors 1-3 are event-driven `When ..., identified_quantity SHALL ...` statements,
each with one trigger and one response. Behaviors 4, 6 and 7 are ubiquitous `SHALL` statements whose
subject is the method or the system. The Description's `SHALL turn ...` is ubiquitous and atomic.
`quire validate` reports no grammar warnings on FR-124 or TC-923.

## Examined scope

- FR-124 (examined): `quire-semantic-value` SHALL turn a completed exact quantity conversion into the pair a later quantity operation needs: a kernel `Quantity` and the `IdentifiedUnit` that carries the quantity's unit id. `convert_quantity` returns a `Conversion` that holds the converted value and a bare `QuantityUnit`. A later operation reads a `Quantity` through `UnitScope::resolve`, which needs the quantity's `UnitId` and the unit retained under it. Without the handoff a consumer would compute a compound unit's id and keep its own table, which the runtime's single-core rule forbids: RT FR-275-AC-23 gives compo
- FR-124#behavior-1 (examined): When the conversion's value is `ConvertedValue::Exact(v)`, `identified_quantity` SHALL return the quantity with magnitude `v` and the unit id of the conversion's target unit, paired with the `IdentifiedUnit` of that unit, whose `unit()` equals the conversion's `unit()`. It SHALL compute the unit's id once.
- FR-124#behavior-2 (examined): When the conversion's value is a decimal or an integer representation, `identified_quantity` SHALL return `HandoffRefusal::NotExact` and form no quantity: such a value is a number in the target representation, not a quantity of the unit.
- FR-124#behavior-3 (examined): When the target is a compound unit whose id the identity byte limit refuses, `identified_quantity` SHALL return `HandoffRefusal::Identity` with the originating `IdentityRefusal` and return no quantity and no unit.
- FR-124#behavior-4 (examined): `identified_quantity` SHALL charge no meter and read no clock or environment: the conversion has charged its work, and the result is a function of the conversion.
- FR-124#behavior-5 (examined): A caller that records the unit with `UnitScope::form` SHALL get back the id the quantity carries, and `UnitScope::resolve` of the quantity SHALL then return the same `QuantityUnit` as the conversion's `unit()`.
- FR-124#behavior-6 (examined): The declared and compound arms of `UnitId` SHALL stay distinct even when their digest bytes are equal, so a compound quantity never resolves to a declared unit.
- FR-124#behavior-7 (examined): The method SHALL use only the crate's existing dependencies, call no identity-minting method of `NodeKey`, and add no re-export (FR-060).

## Verdict

**PASS with one low finding.** One statement has the wrong subject.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 5's subject is 'A caller', not the system: 'A caller that records the unit with `UnitScope::form` SHALL get back the id ...'. EARS puts the obligation on the system. Recast it, for example 'When a caller records the returned `IdentifiedUnit` with `UnitScope::form`, `UnitScope::form` SHALL return the quantity's unit id and `UnitScope::resolve` of the quantity SHALL return the conversion's `unit()`'. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:91-93 |

## Evidence

quire validate output: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/specval.log (only the 3 FR-110 warnings that are also on main).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1 head aa62a43e1 |
