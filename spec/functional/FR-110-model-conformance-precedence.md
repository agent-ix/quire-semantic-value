---
id: FR-110
title: "Defer model-owned redefinition conformance until QSL's verdict"
type: FR
relationships: []
---
# FR-110: Defer model-owned redefinition conformance until QSL's verdict

## Description

When QSL assembles object types from an admitted model, QSV SHALL check
structural declaration faults before QSL's model-conformance verdict and
defer only model-owned variance, multiplicity, and refinement decisions
until that verdict. QSV SHALL return a completed type environment only
after its own effective-attribute finalization. Direct QSV admission SHALL
retain strict declaration checking.

## Inputs

- One closed set of composite and object-type declarations, with the model
  object types identified by the QSL assembly caller.
- The existing `TypeEnvironmentLimits` and caller cancellation handle.
- QSL's source-backed checked-clause request, including its observation,
  for a declared member or an immediate-parent `self.f` projection while
  the environment is pending.
- QSL's lifecycle decision to continue or abandon staged admission after
  its completed model-conformance verdict over the same model declaration
  set. QSV does not receive QSL's typed conformance failure set.

## Outputs

- A completed `TypeEnvironment` only after all required checks succeed.
- A restricted pending declaration/parent view for QSL's checked-fact
  admission, with the actual `FieldRef`, declared type and presence, and
  the caller's unchanged source observation; it confers no
  completed-environment authority.
- Otherwise the original typed QSV declaration, limit, cancellation, or
  storage failure. On conformance refusal, QSL owns its returned result and
  abandons the pending QSV admission.

## Behavior

1. When QSL requests admission for model-derived object types, QSV SHALL
   check structural premises in the staged environment before QSL's later
   model-conformance verdict: declaration identity, same-declaration member
   duplicates, referenced member types, composite recursion, supertype
   existence and cycles, `redefines` target existence and proper ancestry,
   undominated redefinition conflicts, and duplicate names among distinct
   flattened fields that do not redefine one another. It SHALL check
   ancestor-step limits, work limits, storage failures, and cancellation at
   their existing checkpoints. It SHALL preserve the established typed
   cause, bound/counter or measured reservation request, and locus where
   one exists. An invalid or ambiguous member SHALL NOT become available
   through a staged query.
2. When QSL uses the model-assembly path, QSV SHALL defer only its
   `RedefinitionWidens` decision over a model field's value type or presence
   while QSL collects the corresponding model-owned variance,
   multiplicity, and refinement verdicts. QSV SHALL NOT report that
   widening cause before QSL's complete verdict. A missing target during
   transitive lineage construction, an undominated conflict, and a duplicate
   name on unrelated flattened fields SHALL retain their early structural
   QSV refusal, not a guessed QSL conformance cause. This path SHALL use
   the same staged environment and effective-attribute algorithm to reach
   those early results; it SHALL NOT introduce a second checker.
3. While admission is pending, QSV SHALL allow QSL's same checker to read
   only the declaration and immediate-parent view needed to type authored
   postconditions and derive FR-146 facts. QSV's pending `self.f` projection SHALL
   identify the actual parent `FieldRef`, carry the immediate parent's
   declared type and presence, and preserve QSL's supplied source observation;
   it SHALL NOT substitute the redefining child's narrowed type or presence. These
   reads SHALL use the same staged declaration graph, work budget, and
   cancellation handle. They SHALL refuse an unknown or ambiguous field
   with the established typed cause and SHALL NOT authorize general field
   lookup, effective-attribute exposure, or evaluation.
4. If QSL refuses model conformance and abandons staged admission, QSV SHALL
   expose no completed `TypeEnvironment`, public effective attribute set,
   general field lookup, or admitted reference operation from it. QSL owns the collection
   and return of its typed conformance failures; QSV SHALL NOT require their
   payload to abandon the pending admission.
5. If QSL accepts model conformance for that same declaration set, QSV SHALL
   finish effective-attribute construction under the existing work budget and
   cancellation handle before dispatch finalization or public checked-graph
   sealing can expose an admitted environment. QSL's completed verdict is
   the caller-side precondition: it follows all required variance and
   refinement checks after actual checked clause and fact admission, every
   owning writer lineage, the all-model conformance barrier, and full
   failure collection under the same meter. Any QSV
   declaration or resource failure reached during completion SHALL retain its
   original typed result. QSV SHALL NOT treat the QSL verdict as permission
   to skip effective-slot, lineage, name, or value-type safety checks.
   For a field redefining `Reference<T>` with `Reference<S>`, that final
   value-type safety check SHALL accept a proper `S` subtype of `T` in the
   admitted ancestry, because every value admitted for `Reference<S>` is
   also admitted for `Reference<T>`. It SHALL preserve the reference's
   most-specific object identity. QSV's final safety check SHALL refuse a
   reverse or unrelated reference type. QSL's separate refinement
   obligation for an exposed writer SHALL still require an actual checked
   postcondition fact.
6. QSV SHALL confine the intermediate state to its model-assembly contract.
   QSV SHALL NOT expose it as a completed `TypeEnvironment`, or use it for
   `attributes`, `attribute`, `admits`, general expression checking, or
   evaluation before completion. QSV's restricted pending reads in step 3 SHALL
   leave the state pending and SHALL NOT mint a proof or substitute a synthetic
   clause for QSL's checked source. The existing
   `new`, `bounded`, and `bounded_with_cancel` entry points SHALL retain
   strict admission and their observable refusal behavior for direct QSV
   callers. QSV SHALL use one admission implementation and one effective
   attribute algorithm for both paths. No public path from the pending
   state SHALL bypass QSV's final safety checks. QSV SHALL share the same
   structural admission and effective-attribute algorithms between strict
   and QSL-owned calls.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-110-AC-1 | Given a structurally valid model-derived redefinition whose complete QSL conformance stage refuses variance, multiplicity, or refinement after checked facts and all-model barriers, QSL returns its collected conformance refusal and abandons QSV's pending admission before QSV reports `RedefinitionWidens`; no completed QSV environment or attribute can be observed. QSV needs no QSL failure payload to abandon admission. | Test |
| FR-110-AC-2 | Given an unknown member type, a composite or supertype cycle, a missing supertype, an invalid `redefines` target, an undominated redefinition conflict, a duplicate name on unrelated flattened fields, exhausted `ancestor_steps` or `work_units`, or cancellation at an existing charge checkpoint, staged QSV admission stops with its established typed structural or resource failure before QSL's later conformance verdict. No ambiguous member or completed environment is exposed, and the exact bound/counter and cancellation handle are retained. | Test |
| FR-110-AC-3 | Given QSL acceptance for the identical model declaration set, effective attribute lookup and reference admission become available only after QSV completion and before dispatch finalization or public checked-graph sealing, with the same slot and lineage result and work accounting as strict admission of a valid set. If completion refuses or runs out of resources, no partial environment escapes. | Test |
| FR-110-AC-4 | Direct calls to `new`, `bounded`, and `bounded_with_cancel` still refuse an invalid redefinition target, undominated conflict, widening, and duplicate effective name with their existing QSV causes, and admit a valid redefinition with its existing effective attributes. No path from a pending result can expose a completed environment without QSV's final safety checks. | Test |
| FR-110-AC-5 | During pending admission, a source-backed `self.f` read for a redefining child returns the immediate parent's actual `FieldRef`, declared type and presence, and QSL's unchanged source observation to the same checked-fact graph under the original budget and cancellation handle. It never substitutes the child's narrowed declaration, invents a fact, or permits evaluation. An unknown or ambiguous member refuses before a fact can use it. | Test |
| FR-110-AC-6 | The QSL-owned and direct entry points share QSV's structural admission and effective-attribute algorithms; no second model redefinition checker, unchecked public `TypeEnvironment`, or alternate completion path is introduced. | Inspection |
| FR-110-AC-7 | Given a structurally valid model whose child field `Reference<S>` redefines a parent field `Reference<T>`, where `S` is a proper subtype of `T`, multiplicity and presence are unchanged, and no exposed writer leaves a refinement obligation, the complete QSL conformance verdict and QSV finalization succeed. The completed QSV environment admits an `S` reference through the `T` field without changing its most-specific identity. Reversing `S` and `T` or using unrelated object types refuses; when an exposed writer creates a narrowing obligation, QSL still requires an established checked postcondition fact. | Test |

## Dependencies

- [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) owns model
  variance, multiplicity, and refinement conformance, its refusal order,
  and the assembly caller's mapping of the
  final result. This QSV requirement defines only its own admission boundary.
  FR-082's shared-environment clause keeps duplicate names among unrelated
  inherited fields as a structural ambiguity, distinct from the model's
  later FR-151 axes.
  A selected model's existing normalization result is not the completed
  FR-151 conformance verdict: it does not include checked clause and fact
  admission, all writer lineages, or the all-model barrier.
- [QSV FR-108](./FR-108-report-registry-allocation-failure.md) owns typed
  storage failures during admission. This requirement preserves them.
- [QSpec FR-151](ix://agent-ix/quire-specification/FR-151) delegates value
  conformance to [QSpec FR-149](ix://agent-ix/quire-specification/FR-149),
  whose reference upcast keeps the most-specific identity. QSL FR-082's
  identical-reference wording is stale relative to these merged rules;
  QSL owns its normative alignment and consumer implementation.
- The completed QSV environment remains the only environment available to
  public expression checking and evaluation. QSL's internal checked-fact
  admission may read the restricted pending declaration/parent view above;
  QSL owns the checked source graph, FR-146 fact derivation, and observation.
