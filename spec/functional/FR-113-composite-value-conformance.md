---
id: FR-113
title: "Admit composite values under declared composite supertypes"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-208
    type: references
---
# FR-113: Admit composite values under declared composite supertypes

## Description

`quire-semantic-value` SHALL represent the direct supertype relation between
record and tuple declarations and SHALL admit a composite value under any of
its declared composite supertypes. The value retains its most-specific
declaration identity; subtype admission does not rewrite the value or change
composite equality.

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

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-113-AC-1 | Given a three-level composite chain, the direct child admits at its parent and grandparent type boundaries. A parent value is not admitted as the child, and an unrelated composite value is not admitted as the parent. The admitted child's `CompositeValue` retains its exact child `NodeKey`; equality conversion between distinct composite keys remains unavailable. | Test (TC-912) |
| FR-113-AC-2 | A missing composite supertype and a composite supertype cycle are refused with their typed declaration causes. A chain exceeding `ancestor_steps` and closure work exceeding `work_units` stop at the configured typed limit; no partial environment is returned. | Test (TC-912) |

## Dependencies

- [QSpec FR-208](ix://agent-ix/quire-specification/FR-208) defines the
  record/event specialization semantics whose normalized ancestor keys are
  supplied by QSL's consumer layer.
