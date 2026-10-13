---
id: TC-913
title: "Nested composite conformance admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: verifies
---
# TC-913: Nested composite conformance admission

## Description

Verify that the QSV type environment and equality operand check admit a
composite value by conformance at every nested depth, without covariance over
type constructors, and refuse nonconforming values with typed causes.

## Test Procedure

1. Declare parent `P`, intermediate `M` (supertype `P`), child `C` (supertype
   `M`, so a grandchild of `P`) and unrelated `U`, each a record with distinct
   keys, and one value of each. The kernel's checked constructors are exact, so
   build every container holding a `C`, `M` or `U` at a `P`-typed slot with the
   trusted `from_admitted`, `form` and `from_admitted_slots` paths. Query
   `admits` for those holding a `C` or `M`: `Option<P>`, `Sequence`, `Set`,
   `Bag` and `OrderedSet` of `P`, and `Option<Sequence<Option<P>>>` with `C` innermost. A top-level `C`
   at `P` already passes today.
2. Query the negative set, of which the nested cases alone test this
   requirement: the same `Option` and collection containers holding a `U`; a nested value with one `U`
   among conforming elements; `P` at `C`-typed and `M`-typed slots; an
   `Option<C>` and a `Sequence<C>` value at declared `Option<P>` and
   `Sequence<P>`.
3. Run `TypeEnvironment::evaluate_record` and `TypeEnvironment::evaluate_tuple` with a member that completes to
   a `C` and to a `U` at a `P` member. Construct a record and a tuple holding
   a `U`.
4. Call `operand_value` with source type `P` and values `C`, `M`, and
   `Option<P>` and `Sequence<P>` holding a `C`; then with `U`, and with a `P`
   value at source type `C`; then with a `Reference<T>` operand.
5. Query `admits` for an `Option<Reference<T>>` and a `Sequence<Reference<T>>`
   holding a reference whose object type is a subtype of `T`, one of `T`, an
   unrelated object type, and (at a subtype slot) a supertype object.
6. After quire-exact FR-376 to FR-378 land, build a `Set<P>` and a `Bag<P>`
   holding a `C` and a `P` of equal shared fields and admit them.

## Expected Results

1. Every positive query is admitted and keeps its key.
2. Every negative query returns `false`.
3. The `C` member evaluates with the child kept; the `U` member stops with
   `DeferredResultNotAdmitted`; construction with `U` is refused with
   `TypeMismatch` naming the field or position.
4. Conforming operands return unchanged with their own keys. `U` and the
   reverse case return `EqualityOperandSourceNotAdmitted`. The `Reference<T>`
   operand is still checked by kind only.
5. The subtype and identical object types are admitted; the unrelated and
   supertype-at-subtype cases return `false`.
6. Both collections are admitted with two elements.
