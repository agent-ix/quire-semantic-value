---
id: FR-113
title: "Admit composite values under declared composite supertypes"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-359
    type: references
  - target: ix://agent-ix/quire-exact/FR-376
    type: depends_on
  - target: ix://agent-ix/quire-exact/FR-377
    type: depends_on
  - target: ix://agent-ix/quire-exact/FR-378
    type: depends_on
---
# FR-113: Admit composite values under declared composite supertypes

## Description

`quire-semantic-value` SHALL represent the direct supertype relation between
record and tuple declarations and SHALL admit a composite value under any of
its declared composite supertypes. The value retains its most-specific
declaration identity; subtype admission does not rewrite the value or change
composite equality. The same conformance applies at every depth at which a
slot names a composite type, and a composite equality operand is checked by
conformance.

## Behavior

1. A composite declaration MAY name zero or more direct composite supertypes by
   their producer-assigned `NodeKey`. Every named key SHALL identify another
   declaration admitted in the same `TypeEnvironment`.
2. Composite supertype graphs SHALL be acyclic. Admission SHALL reject a
   missing supertype and any cycle before returning a `TypeEnvironment`.
3. `TypeEnvironment::composite_conforms(sub, sup)` SHALL be reflexive for an
   identical key and SHALL otherwise succeed only when `sup` is in the
   admitted transitive supertype closure of `sub`.
4. `TypeEnvironment::admits(ValueType::Composite(sup), value)` SHALL accept a
   composite value declared as `sub` when `sub` conforms to `sup`. It SHALL
   continue to reject reverse and unrelated declarations. The accepted value
   retains `sub` as its declaration key, and equality conversion remains based
   on exact composite type identity.
5. QSV SHALL charge composite graph validation and ancestor-closure work
   against the environment's `work_units` ceiling.
6. QSV SHALL return the established typed limit failure whenever a composite
   ancestry walk or closure exceeds `ancestor_steps` or `work_units`.
7. QSV SHALL validate and close composite ancestry with an explicit worklist
   rather than native recursion.
8. `TypeEnvironment::admits` SHALL check the contents of a value against its
   declared type, not only the declared type of its container. The kernel's
   `ValueType::admits` compares an `Option` or collection value's recorded
   payload or element type with the declared one and never looks inside, so an
   `Option<P>` or `Sequence<P>` whose payload type is `P` is already admitted
   when it holds a nonconforming composite that a trusted constructor
   (`from_admitted`, `form`, `from_admitted_slots`) put there, and a replayed
   container is decoded then admitted. `admits` SHALL walk the value with an
   explicit worklist and require, for every `Option` payload and collection
   element reached, that the present value is admitted at
   the declared payload or element type, with a composite admitted by
   `composite_conforms` item 3. A conforming child keeps its own key. A union
   member payload follows the same rule once the shared semantic value carries
   a union value; a union admits a payload when any one declared member admits
   it, so a child conforming to two related members is admitted without
   choosing between them and keeps its own key.
9. The walk SHALL stop at a composite value. A composite's own members were
   admitted against its own declaration when it was built (item 12 and
   the constructors), and an effective member type of a descendant conforms to
   the ancestor's, so re-walking them at the ancestor's slot would test the
   wrong declaration. A record field or tuple position typed `P` is therefore
   admitted when `admits` is applied at that member's declared type, by the
   constructors and `TypeEnvironment::evaluate_record` and
   `TypeEnvironment::evaluate_tuple` (item 12).
   Every non-composite leaf in the walk SHALL be admitted by the exact kernel
   `ValueType::admits`, except `Reference<T>`, which is admitted by
   `conforms` on the reference's object type as today.
10. Admission SHALL NOT be covariant over type constructors. The container's
    own declared type and kind must match exactly: a collection value whose
    element type is a descendant `S` is not admitted as a collection of `P`,
    an `Option<S>` is not admitted as `Option<P>`, and a collection kind never
    converts to another. A child is admitted into the slot as a value, never
    by retyping the container. Admission never rewrites a value.
11. A value whose composite key does not conform to the declared key, at any
    depth reached by the walk, SHALL NOT be admitted. Conformance (item 3)
    is reflexive on an identical key, so an identical key conforms even when
    the environment does not admit it, and an unequal key the environment does
    not admit never conforms. `admits` returns `false` and carries no cause;
    the typed refusal belongs to the caller: record and tuple construction
    refuses the member with `ConstructionCause::TypeMismatch` naming the field
    or position, `TypeEnvironment::evaluate_record` and
    `TypeEnvironment::evaluate_tuple` stop with `DeferredResultNotAdmitted`,
    and `operand_value` returns
    `EqualityOperandSourceNotAdmitted`. Reverse, grandparent-to-child, sibling
    and unrelated composites are nonconforming.
12. `TypeEnvironment::evaluate_record` and `TypeEnvironment::evaluate_tuple`
    (not the kernel functions of the same name, which stay exact) SHALL admit each completed member
    value through `TypeEnvironment::admits` and not through the exact kernel
    `ValueType::admits`, so a conforming child member does not stop with
    `DeferredResultNotAdmitted`. A nonconforming member still stops with that
    checked-invariant cause.
13. `operand_value` SHALL check a composite equality operand by conformance
    under the `TypeEnvironment` it is given, as it already checks a
    `Reference<T>` operand by kind: an operand whose source type is composite
    key `P` and whose value is a composite conforming to `P`, directly or
    through an `Option` payload or collection element, is admitted and keeps
    its own key. A nonconforming operand returns the established
    `EqualityOperandSourceNotAdmitted` cause. Equality conversion is
    unchanged: `admits_equality_conversion` between distinct composite keys
    stays unavailable, and whether two composites of different declarations
    are equal belongs to the kernel's composite equality. `CheckedEquality::evaluate`
    takes the `TypeEnvironment` that checked the equality and passes it to
    `operand_value`.
14. `admits` takes no `work_units` charge and reports no limit. It walks a
    value the caller already holds, once, in time linear in the value's
    occurrences, with an explicit worklist and no recursion, and the
    environment's ceiling bounds admission of declarations, not of supplied
    values. Charging the occurrences of a built or decoded value is the
    caller's, through the value-accounting charge it already makes at
    construction and replay; the answer stays a `bool`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-113-AC-1 | Given a three-level composite chain, the direct child admits at its parent and grandparent type boundaries. A parent value is not admitted as the child, and an unrelated composite value is not admitted as the parent. The admitted child's `CompositeValue` retains its exact child `NodeKey`; equality conversion between distinct composite keys remains unavailable. | Test (TC-912) |
| FR-113-AC-2 | A missing composite supertype and a composite supertype cycle are refused with their typed declaration causes. A chain exceeding `ancestor_steps` and closure work exceeding `work_units` stop at the configured typed limit; no partial environment is returned. | Test (TC-912) |
| FR-113-AC-3 | With a chain parent `P`, intermediate `M`, child `C` (a grandchild of `P`) and an unrelated record `U`, built with the trusted constructors (`from_admitted`, `form`, `from_admitted_slots`) because the kernel's checked constructors are exact, `admits` accepts an `Option<P>`, a `Sequence`, `Set`, `Bag` or `OrderedSet` of `P`, each keeping its key; the top-level `C` at `P` and `M` already passes. It refuses the same `Option` and collection containers holding a `U`, a `P` at a `C`-typed or `M`-typed version, and `Option<C>` or `Sequence<C>` at `Option<P>` or `Sequence<P>` as a declared type. The refusals for a nonconforming element inside a container whose recorded element type is `P` are the cases only this requirement adds. | Test (TC-913) |
| FR-113-AC-4 | `Option<Sequence<Option<P>>>` holding a `C` innermost is admitted, and one `U` among conforming elements refuses the whole value. `TypeEnvironment::evaluate_record` and `TypeEnvironment::evaluate_tuple` with a member that completes to a `C` at a `P` member succeed with the child kept, and with a `U` stop with `DeferredResultNotAdmitted`. Record and tuple construction holding a `U` refuses with `TypeMismatch` at the field or position. | Test (TC-913) |
| FR-113-AC-5 | `operand_value` admits a composite operand of source type `P` holding a `C` or an `M` unchanged and with its own key, and an `Option<P>` or `Sequence<P>` operand holding one. It refuses a `P` operand holding `U`, and a `P` value at source type `C`, with `EqualityOperandSourceNotAdmitted`. A `Reference<T>` operand is still checked by kind only. | Test (TC-913) |
| FR-113-AC-6 | A `Set<P>` or `Bag<P>` holding a `C` and a `P` of equal shared fields has two elements and is admitted. This vector requires the quire-exact collection and equality changes of FR-376 to FR-378 and cannot pass before they land. | Test (TC-913) |
| FR-113-AC-7 | A `Reference<T>` inside an `Option` payload or collection element is admitted only when the reference's object type is `T` or conforms to `T`: an `Option<Reference<T>>` or `Sequence<Reference<T>>` holding a reference to a subtype object is admitted, and one holding a reference to an unrelated object type, or a supertype object at a subtype `T`, is refused although the recorded payload or element type matches. | Test (TC-913) |

## Dependencies

- [QSpec FR-208](ix://agent-ix/quire-specification/FR-208) defines the
  record/event specialization semantics whose normalized ancestor keys are
  supplied by QSL's consumer layer.
- QSL FR-359 (agent-ix/quire-spec-language, on its integration branch
  `impl/business-model-meanings`, not yet on its `main`) states the slot, child
  and refusal rules whose admission half this requirement provides
  (cross-repo requirement 1).
- quire-exact FR-376 to FR-378 define the kernel collection, canonical-key and
  equality behavior for composites of distinct declarations that FR-113-AC-6
  needs; they are specified in quire-exact and not yet implemented.
