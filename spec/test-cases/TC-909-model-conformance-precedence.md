---
id: TC-909
title: "Structural admission precedes model conformance and widening completion"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-110
    type: verifies
---
# TC-909: Structural admission precedes model conformance and widening completion

## Description

Verify [FR-110](../functional/FR-110-model-conformance-precedence.md)
through QSV's strict entry points and its QSL-owned model-assembly boundary.

## Test Procedure

1. Supply a structurally valid model field redefinition whose complete QSL
   FR-151 verdict rejects variance, multiplicity, or refinement after
   checked facts, every owning writer lineage and the all-model barrier.
   Run staged QSV admission and abandon it on QSL refusal. Assert QSV has
   reported no `RedefinitionWidens`. Check the public QSV surface at compile
   time: pending admission offers only restricted declaration/parent reads,
   has no completed-environment or evaluation API, and cannot become a
   completed environment without QSV finalization.
2. In separate staged runs, use an unknown member type, composite cycle,
   supertype cycle, missing supertype, invalid `redefines` target,
   undominated redefinition conflict, unrelated inherited fields with a
   duplicate flattened name, an N-1/N `ancestor_steps` or `work_units`
   limit, and cancellation at an observed charge point. Assert the original
   QSV structural or resource result before QSL's later conformance stage,
   with its typed payload and no ambiguous field exposed. The duplicate
   flattened-name case must fail without a second checker.
3. Before the verdict, read a source-backed postcondition `self.f` through
   the pending declaration/parent view, with a child that narrows its
   immediate parent's field type or presence. Assert the returned
   `FieldRef`, declared type and presence identify the parent and the
   observation identifies the actual source; they must not identify the
   child's narrowed field or a synthetic clause. Feed that result to the
   same QSL checked-fact graph under the original budget and cancellation
   handle. An unknown or ambiguous `f` must refuse without a usable fact.
   Assert this restricted read cannot call evaluation or expose an effective
   attribute set.
4. Supply a conforming model with a valid redefinition. Complete the full QSL
   verdict and QSV admission using the original budget and cancellation
   handle. Compare effective attributes, lineage, reference admission and
   work spend with strict QSV admission of the same valid declarations.
   Deny completion at a real work or storage boundary and inspect the
   failure and lack of partial output. Assert this precedes dispatch
   finalization and public checked-graph sealing.
5. Use a child field of `Reference<S>` redefining `Reference<T>` with
   `S` a proper subtype of `T`, unchanged multiplicity and presence, and
   no exposed writer with an outstanding refinement obligation. After
   the complete QSL verdict, assert QSV finalization succeeds and the
   completed environment admits an `S` object reference through the `T`
   field without changing the reference's most-specific type or identity.
   Reverse the types and then use unrelated types; assert refusal in each
   run. Add an exposed writer that narrows the field and assert QSL requires
   an actual established postcondition fact before the verdict can succeed.
6. Call `new`, `bounded`, and `bounded_with_cancel` directly with invalid
   target, undominated conflict, widening and duplicate effective-name
   fixtures. Compare each refusal with its established QSV cause. Inspect
   the QSL-owned interface for a public path from pending state to checking
   or evaluation without completed conformance and effective construction.

## Expected Results

QSL returns its model refusal and abandons QSV admission before QSV's
model-derived widening decision. Structural target, conflict and duplicate
name refusals retain their original QSV causes; other structural and resource
failures retain their original details.
Only successful conformance followed by QSV completion yields an effective
environment. Direct QSV callers remain strict. Pending state allows the
restricted declaration/parent reads needed for checked FR-146 facts, while
public expression checking and evaluation remain unavailable.
Conforming subtype references survive finalization with their original
identity; reverse and unrelated references refuse, and a writer's
refinement obligation still requires a checked fact.

## Status

Planned for IR-720 implementation. QSL owns its consumer call-order and
refusal-mapping tests; this QSV test case does not claim to verify QSL's
model-conformance algorithm.
