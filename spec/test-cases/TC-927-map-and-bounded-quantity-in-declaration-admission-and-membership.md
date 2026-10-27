---
id: TC-927
title: "Map and BoundedQuantity in declaration admission and membership through the public API"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-127
    type: verifies
---
# TC-927: Map and BoundedQuantity in declaration admission and membership through the public API

## Description

Verify FR-127-AC-1 to AC-11 (and pin AC-12) for `ValueType::Map` and
`ValueType::BoundedQuantity` through the public `TypeEnvironment`,
`admits_equality_conversion` and supplied-membership API, as integration tests
of the crate (`tests/map_and_bounded_quantity.rs`). Every control is a
negative or adverse case that fails if a walk's wildcard or leaf arm handles a
new variant as it handled the old ones.

## Test Procedure

1. Build `Map<Text, Float64>`, `Option<Map<Text, Float32>>`,
   `Map<Text, Option<Sequence<Float64>>>`, `Map<Text, Integer>`,
   `Map<Text, BoundedQuantity>` and a float-free record map; call
   `contains_ieee` and the metered form on each, and compare the metered charge
   of a `Map` link with the charge of the corresponding `Collection` link
   (AC-1).
2. Declare record fields of `Map<Text, Float64>`, `Option<Map<Text, Float32>>`,
   `Sequence<Map<Text, Float64>>` and `Set<Map<Text, Float64>>` through plain
   and metered admission (AC-2).
3. Declare fields whose map value is `Population`, `Option<Population>`,
   `Sequence<Population>`, an unregistered `Composite` and a `Reference` to a
   non-object type; repeat each inside `Option` and `Collection`; compare plain
   and metered causes (AC-3).
4. Declare union members whose positions are `Map<Text, Integer>`,
   `Option<Map<Text, Composite>>` and `Map<Text, Map<Text, Boolean>>`; run with
   exactly the needed work and with one unit fewer (AC-4).
5. Declare self-referential records through `Map<Text, Self>`,
   `Map<Text, Self>[0, 4]`, `Map<Text, Self>[1, 4]`, `Sequence<Self>[1, 4]` and
   `Option<Map<Text, Self>[1, 4]>`, with and without a named field edge (AC-5).
6. Declare `BoundedQuantity` (bounded, unbounded, inverted) as field, option
   payload, collection element and map value; call `contains_ieee`; check no
   cycle edge; call `admits_equality_conversion` for `Int`, `Quantity(U)` and
   `BoundedQuantity` sources against `BoundedQuantity`, `Quantity(U)` and `Map`
   targets, and for a type equal to itself (AC-6).
7. Call `integer_source_admits` through `admits_equality_conversion` with an
   `Int[0, 10]` source against `Map<Text, Integer>` and `BoundedQuantity`
   targets, and against the existing targets (AC-7).
8. Run supplied and structural membership of a collection, text, option,
   composite and quantity value against `Map<Text, Boolean>` and
   `Map<Text, Option<Boolean>>[1, 3]`, recording the events and the cause
   (AC-8).
9. Run membership of quantities of magnitude -1, 0, 50, 100, 101 and 1/2 in
   `U`, one in another unit, and a text, boolean and option value, against
   `BoundedQuantity` `[0, 100]`, `[None, 100]`, `[0, None]` and inverted
   `[5, 3]`, recording the events and the cause (AC-9).
10. Admit `Option` values whose retained payload type is an equal `Map`, a
    `Map` with a different bound, a `Map` with bound `None` against
    `Some(0, max)`, a different value type, and a `Collection` of the same
    element; repeat with an `Int` value type under a configured numeric bound
    and under none, recording the `TypeLinkComparison` events (AC-10).
11. Admit `Option` values whose retained payload type is a `BoundedQuantity`
    against equal, unit-only, minimum-only, maximum-only, presence-only
    different and `Quantity(U)` expected types (AC-11).
12. Inspect the ten matches named in AC-12 for explicit `Map` and
    `BoundedQuantity` arms (AC-12).

## Expected Results

- Step 1: true for the first three types, false for the last three; the `Map`
  charge equals the `Collection` charge.
- Step 2: the first three declarations are accepted; the `Set` declaration is
  refused `OperatorIneligible`.
- Step 3: `OperatorIneligible` for the three population shapes,
  `UnknownDeclaration` naming the key, `Type(TypeMismatch)` for the reference;
  identical in the plain and metered walks and inside `Option` and
  `Collection`.
- Step 4: all three walk to the leaf at the independently counted charge; one
  fewer unit stops with the established work limit.
- Step 5: accepted for the unbounded map, `[0, 4]` map and the `Option` form;
  refused as a non-escaping cycle for `Map [1, 4]` and for `Sequence [1, 4]`;
  the named edge keeps its name.
- Step 6: every declaration is accepted, `contains_ieee` is false, no edge is
  added; only the type-equal-to-itself conversion is admitted.
- Step 7: false for both new targets; existing targets unchanged.
- Step 8: invalid membership `Kind` at the value locus after one `ValueVisit`
  event for every value; no panic.
- Step 9: admitted for 0, 50 and 100 at `[0, 100]`; -1 and 101 refuse
  `Payload`, 1/2 refuses `Payload` for a `[1, 100]` control; other unit refuses
  `Type`; non-quantities refuse `Kind`; open sides admit any magnitude on that
  side; inverted refuses everything; one `ValueVisit` each.
- Step 10: the equal map is admitted; each differing map or collection is
  invalid type at the compared link; the `Int` inside the map takes one event
  per pair and returns `UnsupportedHelper::NumericDomain` with no configured
  bound.
- Step 11: only the equal type is admitted; every differing type is invalid
  type.
- Step 12: each of the ten matches has an explicit arm, or is pinned by Steps 8
  and 9.
