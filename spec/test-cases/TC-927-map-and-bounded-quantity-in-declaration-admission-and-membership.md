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

Verify FR-127-AC-1 to AC-11 and AC-13 to AC-17 (and pin AC-12) for `ValueType::Map` and
`ValueType::BoundedQuantity` through the public `TypeEnvironment`,
`admits_equality_conversion`, `check_equality_in` and supplied-membership API, as integration tests
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
   cycle edge; call `admits_equality_conversion` for a bounded type against
   itself (AC-6).
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
   `[5, 3]`, and the magnitude 1/2 against a `[1, 100]` control, recording the
   events and the cause (AC-9).
10. Admit `Option` values whose retained payload type is an equal `Map`, a
    `Map` with a different bound, a `Map` with bound `None` against
    `Some(0, max)`, a different value type, and a `Collection` of the same
    element; repeat with an `Int` value type under a configured numeric bound
    and under none, recording the `TypeLinkComparison` events (AC-10).
11. Admit `Option` values whose retained payload type is a `BoundedQuantity`
    against equal, unit-only, minimum-only, maximum-only, presence-only
    different and `Quantity(U)` expected types (AC-11).
12. Inspect the matches named in AC-12 for explicit `Map` and
    `BoundedQuantity` arms (AC-12).
13. Call `admits_equality_conversion` with `BoundedQuantity(U, [0, 10])`,
    `[None, None]` and inverted `[5, 3]` sources against `Quantity(U)`, a
    convertible and a non-convertible `Quantity(V)`, an unresolved unit, an
    equal bounded type, a same-unit bounded type with other bounds, and with a
    `Quantity(U)` source against a bounded target (AC-13).
14. Call `check_equality_in` for equal `Map<Text, Integer>` operands,
    `Option<Map<Text, Boolean>>`, `Sequence<Map<Text, Boolean>>`, a record with
    a map field, and a conversion target holding a map (AC-14).
15. Call `check_equality_in` for bounded against bounded (equal, wider,
    disjoint, inverted) and against `Quantity(U)`, a same-dimension other unit,
    another dimension and an unresolved unit; evaluate `50 U` against `3 U`;
    check `Option<BoundedQuantity(U, ..)>` against `Option<Quantity(U)>` and
    against `Option<BoundedQuantity(V, ..)>` (AC-15).
16. Seal a union and call `runtime_type` on `Map<Text, Union>` and
    `Option<Map<Text, Union>[1, 3]>`, and on an unsealed one; call `same_type`
    on maps with different union handles of one sealed union and on maps
    differing in bound, bound presence and value type (AC-16).
17. Admit `Option` values whose retained payload type is a `BoundedQuantity`
    under a configured `B`: equal bounds; minimum, maximum, presence and unit
    differing (with an oversized bound beside a unit or presence difference);
    bounds of exactly `B` bits and one bit over `B` on each of the four
    endpoints, one at a time and several together; pre-cancellation with an
    oversized bound; cancellation between bound pairs; no configured `B`; a
    denied event at N-1 and N, recording the events and cumulative spend
    (AC-17).

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
- Step 12: each named match has an explicit arm, or is pinned by Steps 8 and 9.
- Step 13: bounded to `Quantity(U)` admitted for all three bound shapes, to
  `Quantity(V)` exactly as `Quantity(U)` is, refused for the unresolved unit;
  equal bounded type admitted, other-bounds bounded type refused, `Quantity(U)`
  to bounded refused; the same rows for bounds no value satisfies.
- Step 14: `OperatorIneligible` for every shape, with no schedule selected.
- Step 15: `EqualitySchedule::Quantity` for bounded/bounded and bounded/plain,
  `DistinctUnits`, `IncompatibleDimensions`, `TypeMismatch` as named; `50 U`
  against `3 U` compares exact values; the `Option` forms are accepted for the
  same unit and a mismatch for another.
- Step 16: sealed key resolved with bound kept, `None` when unsealed; `same_type`
  true for the joined handles and false for each differing map.
- Step 17: equal admitted; each differing pair a mismatch with no measurement
  for unit and presence; `B` bits complete; one over `B` returns typed unsupported
  size naming side and bound, first in the stable order; pre-cancellation
  returns the original cause first; no `B` returns `NumericDomain` after one
  event; the denied event at N-1 prevents any measurement; spend is the
  enumerated events plus the initiating event.
