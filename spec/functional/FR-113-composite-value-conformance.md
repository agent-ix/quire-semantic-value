---
id: FR-113
title: "Admit composite values under declared composite supertypes"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-359
    type: references
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
8. `TypeEnvironment::admits` SHALL decide a composite slot by conformance at
   every depth at which a declared type names a composite key: an `Option`
   payload, a `Sequence`, `Set`, `Bag` or `OrderedSet` element, a tuple
   position and a record field. Given a declared `Option<P>`, a collection of
   `P`, or a record or tuple whose member is typed `P`, a present payload,
   element or member value is admitted when it is a composite value whose
   declaration key conforms to `P` under item 3, and it keeps its own key. The
   check descends through `Option` and collection values with an explicit
   worklist and reuses the environment's admitted ancestor closure; it adds
   no ancestor walk of its own. A union member payload follows the same rule
   once the shared semantic value carries a union value.
9. Admission SHALL NOT be covariant over type constructors. A value is
   admitted at `Option<P>` or a collection of `P` element by element, and the
   container's own declared type and kind must match exactly: a collection value
   whose element type is a descendant `S` is not admitted as a collection of
   `P`, an `Option<S>` is not admitted as `Option<P>`, and a collection kind
   never converts to another. A child value is admitted into the slot as a
   value, never by retyping the container. Admission never rewrites a value:
   the admitted payload, element or member is the same composite value with
   its own key.
10. A value whose composite key does not conform to the declared key, at any
    depth, SHALL NOT be admitted. `admits` returns `false` and carries no
    cause. Record and tuple construction SHALL refuse the member that holds
    it with `ConstructionCause::TypeMismatch` naming the field or position,
    exactly as for any other nonconforming member. A reverse, grandparent-to-
    child, sibling or unrelated composite is nonconforming, and so is a
    composite key the environment does not admit.
11. `operand_value` SHALL check a composite equality operand by conformance
    under the `TypeEnvironment` it is given, as it already checks a
    `Reference<T>` operand by kind: an operand whose source type is composite
    key `P` and whose value is a composite conforming to `P`, directly or
    through an `Option` payload or collection element, is admitted and keeps
    its own key. A nonconforming operand returns the established
    `EqualityOperandSourceNotAdmitted` checked-invariant cause. Admission
    changes nothing about equality conversion: `admits_equality_conversion`
    between distinct composite keys stays unavailable, and deciding whether
    two composite values of different declarations are equal belongs to the
    kernel's composite equality.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-113-AC-1 | Given a three-level composite chain, the direct child admits at its parent and grandparent type boundaries. A parent value is not admitted as the child, and an unrelated composite value is not admitted as the parent. The admitted child's `CompositeValue` retains its exact child `NodeKey`; equality conversion between distinct composite keys remains unavailable. | Test (TC-912) |
| FR-113-AC-2 | A missing composite supertype and a composite supertype cycle are refused with their typed declaration causes. A chain exceeding `ancestor_steps` and closure work exceeding `work_units` stop at the configured typed limit; no partial environment is returned. | Test (TC-912) |
| FR-113-AC-3 | With a three-level chain parent `P`, intermediate `M`, child `C` and an unrelated record `U`, a `C` value is admitted at `Option<P>`, `Option<M>`, a `Sequence`, `Set`, `Bag` and `OrderedSet` of `P`, a tuple position typed `P` and a record field typed `P`, each keeping key `C`. A `P` value is refused at the `C` and `M` versions of those slots, `U` is refused at the `P` versions, and `Option<C>` or `Sequence<C>` as a whole is refused at `Option<P>` or `Sequence<P>`. A collection of `P` holding a `C` and a `P` of equal shared fields keeps both elements. | Test (TC-913) |
| FR-113-AC-4 | A nested case admits at every level: `Option<Sequence<Option<P>>>` admits a `C` at its innermost payload, and one nonconforming element among conforming ones refuses the whole value. Record and tuple construction holding a nonconforming nested value refuses with `TypeMismatch` at the field or position, and a conforming one constructs with the child kept. | Test (TC-913) |
| FR-113-AC-5 | `operand_value` admits a composite equality operand of source type `P` holding a `C` or a grandchild, unchanged and with its own key, and an `Option<P>` or `Sequence<P>` operand holding one. It refuses a `P` operand holding `U` or a `P` value at source type `C` with `EqualityOperandSourceNotAdmitted`. A `Reference<T>` operand is still checked by kind only. | Test (TC-913) |

## Dependencies

- [QSpec FR-208](ix://agent-ix/quire-specification/FR-208) defines the
  record/event specialization semantics whose normalized ancestor keys are
  supplied by QSL's consumer layer.
- [QSL FR-359](ix://agent-ix/quire-spec-language/FR-359) states the slot, child
  and refusal rules whose admission half this requirement provides
  (cross-repo requirement 1).
