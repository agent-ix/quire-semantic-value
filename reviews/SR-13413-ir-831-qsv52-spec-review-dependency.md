---
id: SR-13413
title: "IR-831 quire-semantic-value PR 52 dependency review of FR-124 and TC-923"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-semantic-value@f0cc3120be0a6f15eac62c9e2ca45e8b3630aca0 vs origin/main 9f1453a; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: reviews
---

## Summary

Ticket: IR-831. Dependency review of the new `relationships` edges. FR-124 references QSV FR-122
and FR-060, QSpec FR-142 and RT FR-275, and TC-923 `verifies` FR-124. All five targets resolve:

- QSV `spec/functional/FR-122-...` and `FR-060-...`.
- QSpec `spec/functional/type-model/FR-142-evaluate-quantities-and-units.md`.
- RT `spec/exact/functional/FR-275-single-exact-kernel.md`.

`references` is the right edge type for the consumer criterion (RT FR-275) and for the owner of
conversion (QSpec FR-142). This is enablement work for RT AD-004 step 4. FR-124 depends on nothing
unbuilt: every item it composes exists at the head.

## Examined scope

- FR-124 (examined): `quire-semantic-value` SHALL turn a completed exact quantity conversion into the pair a later quantity operation needs: a kernel `Quantity` and the `IdentifiedUnit` that carries the quantity's unit id. `convert_quantity` returns a `Conversion` that holds the converted value and a bare `QuantityUnit`. A later operation reads a `Quantity` through `UnitScope::resolve`, which needs the quantity's `UnitId` and the unit retained under it. Without the handoff a consumer would compute a compound unit's id and keep its own table, which the runtime's single-core rule forbids: RT FR-275-AC-23 gives compo
- FR-124#behavior-6 (examined): The declared and compound arms of `UnitId` SHALL stay distinct even when their digest bytes are equal, so a compound quantity never resolves to a declared unit.
- FR-124#behavior-7 (examined): The method SHALL use only the crate's existing dependencies, call no identity-minting method of `NodeKey`, and add no re-export (FR-060).
- FR-124-AC-5 (examined): A declared unit and a compound unit whose digest bytes are equal produce distinct unit ids from `identified_quantity`, and resolving the compound quantity returns the compound unit.
- TC-923 (examined): Verify [FR-124](../functional/FR-124-identified-quantity-from-an-exact-conversion.md) through the public `quire_semantic_value::quantity` API: the pair that `Conversion::identified_quantity` returns, its retention and resolution in a `UnitScope`, its refusals, and that it adds no charge. Steps 1 to 5 are executable tests; step 6 is the implementing PR's inspection and build gate. The fixtures are RT TC-187 U25's units (a declared unit converted into a compound target, then into joules).

## Verdict

**PASS with one low finding.** The edges are correct. One kernel property is restated without an
edge to its owner.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 6 gives QSV a SHALL of its own: 'the declared and compound arms of `UnitId` SHALL stay distinct even when their digest bytes are equal'. That guarantee belongs to quire-exact, whose `UnitId` equality and order compare the domain label first (`src/identity.rs`, FR-088 and TC-411 'quantity unit id two domains'). QSV cannot change it. The unified-core rule wants one authoritative statement. Recast Behavior 6 as a consequence of quire-exact FR-088 and add a `references` edge to `ix://agent-ix/quire-exact/FR-088`. AC-5 then checks that the handoff picks the right arm, not the kernel's equality. Optionally, also add an edge to QSpec TC-187, the U25 fixture source. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:94-96 |

## Evidence

Targets checked in /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/qsv, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/rt, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/qspec; quire-exact spec at ~/dev/quire-exact.
