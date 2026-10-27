---
id: FR-127
title: "Map and BoundedQuantity in declaration admission and membership"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-111
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-117
    type: references
  - target: ix://agent-ix/quire-exact/FR-386
    type: references
  - target: ix://agent-ix/quire-exact/FR-387
    type: references
  - target: ix://agent-ix/quire-exact/FR-381
    type: references
  - target: ix://agent-ix/quire-specification/FR-142
    type: references
  - target: ix://agent-ix/quire-specification/FR-149
    type: references
  - target: ix://agent-ix/quire-specification/FR-143
    type: references
  - target: ix://agent-ix/quire-specification/FR-153
    type: references
  - target: ix://agent-ix/quire-specification/FR-154
    type: references
---
# FR-127: Map and BoundedQuantity in declaration admission and membership

## Description

`quire-exact` added `ValueType::Map` (QExact FR-386, `Map<Text, V>[min, max]`,
a type with no kernel values) and `ValueType::BoundedQuantity` (QExact FR-387,
`Quantity<U>[min, max]`, admitting only in-range quantities of `U`).
`quire-semantic-value` SHALL treat both explicitly in every walk over
`ValueType` that it owns, so that neither is silently handled by a wildcard or
leaf arm that was written before they existed. This requirement fixes, for each
walk, how a `Map` type and a `BoundedQuantity` type behave. It adds no type
identity and no value; both belong to QExact.

## Inputs

- A declared or retained `ValueType` that is, or contains at any depth, a
  `Map` or `BoundedQuantity` link.

## Outputs

- The same results, refusals and events that the walk gives for the existing
  links, as fixed by the Behavior below.

## Behavior

1. **Map in the IEEE containment walk.** `contains_ieee`, in its plain and its
   metered form, SHALL descend a `Map` link to its value type `V`, so
   `contains_ieee` is true for a `Float32` or `Float64` reached through any
   number of `Option`, `Collection` and `Map` links, including
   `Option<Map<Text, Float64>>`. The metered form charges and reserves for a
   `Map` link exactly as for a `Collection` link.
2. **Map is not refused for IEEE content.** `quire-semantic-value` SHALL NOT
   refuse in its declaration type-refusal walk a `Map` whose value type contains `Float32` or `Float64`.
   The existing refusal of IEEE content under `Set`, `Bag` and other
   non-`Sequence` collection kinds exists because those kinds deduplicate,
   order or key their elements by element equality; a map's keys are `Text`,
   its values are neither deduplicated, ordered nor keyed, and a map therefore
   behaves like a `Sequence` here. `Map<Text, Float64>` is a valid declared
   type. A `Set` or `Bag` whose element is a map over floats remains refused,
   because its element `contains_ieee` (Behavior 1) and the existing rule
   applies to the collection, not to the map.
3. **Map in the declaration type-refusal walk.** `quire-semantic-value` SHALL,
   in its type-refusal walk, plain and metered, push a `Map` link's value type and apply every refusal to
   it that it applies inside `Option` and `Collection`: a `Population` inside a
   map's value type is refused `OperatorIneligible`; an unknown `Composite` is
   refused `UnknownDeclaration`; a `Reference` whose target is not a model
   object type is refused `Type(TypeMismatch)`. `quire-semantic-value` SHALL give the plain and
   metered walks the same cause and the same first cause.
4. **Map in the union-link and cycle-edge walks.** The union-member position
   walk SHALL descend a `Map` link to its value type, charging one unit per
   link as for `Option` and `Collection`. The cycle-edge walk SHALL descend a
   `Map` link to its value type and keep every `Composite` reached through it
   as a recursion edge. A `Map` link's escape follows its bound as a
   `Collection` link's does: the descent is escaping when the inherited
   escape is already true, or when the map's bound is absent, or when its
   bound's minimum is `0`; it is not escaping when the bound is present with
   a minimum of at least `1` and the inherited escape is false. A recursive
   declaration whose only cycle passes through a `Map<Text, Self>` that can be
   empty is therefore accepted, and one whose only cycle passes through
   `Map<Text, Self>[1, n]` is refused as a non-escaping cycle.
5. **BoundedQuantity is a leaf in every declaration walk.** `contains_ieee`
   (plain and metered), the type-refusal walk (plain and metered), the
   union-link walk and the cycle-edge walk SHALL treat `BoundedQuantity` as a
   leaf that has no nested type, is not IEEE, is never refused, and adds no
   edge, exactly as `Quantity(U)`. A unit named by a `BoundedQuantity` is not
   resolved by these walks.
6. **Equality-conversion admission.** `quire-semantic-value` SHALL make
   `integer_source_admits` return `false` when the target is `Map` or
   `BoundedQuantity`, so an `Int` source is never convertible to either.
   `admits_equality_conversion` SHALL read a `BoundedQuantity` source over `U`
   as `Quantity(U)` (the QSpec FR-154 refinement read: a bounded value is read
   as a `Quantity<u>` by conversion, comparison and equality) and then apply
   the existing closed `Quantity(from)` to `Quantity(to)` row, so
   `BoundedQuantity(U, bounds)` to `Quantity(U)` is admitted for every bound
   shape (inverted included), and to `Quantity(V)` exactly when
   `Quantity(U)` to `Quantity(V)` is admitted. The conversion SHALL NOT check
   the source's bounds: a bound is a run-time invariant of QSpec FR-154, not a
   conversion precondition, and the converted value is a plain quantity.
   The only conversion row that admits a `BoundedQuantity` target is the
   identity row (`source == target`); no QSpec row converts into a bounded
   type, and the flow of a plain `Quantity(U)` into a bounded position is the
   checker's assignment admission left to the run-time invariant (QSpec
   FR-154), not an equality-conversion row. A `Map` source or target is
   admitted only by the identity row; identity is an admission row and not an
   equality relation (Behavior 10).
7. **Membership of a Map type admits no value.** Supplied and structural
   membership SHALL admit no value at a `Map` expected type, because QExact
   FR-386 provides no map value. The expected type is reached by the generic
   leaf arm; the result is the typed invalid membership of cause `Kind` at the
   value's locus, for a value of every variant, after the permitted
   `ValueVisit` event and its after-helper boundary, as for any other kind
   mismatch. It is never a panic and never an accept.
8. **Membership of a BoundedQuantity type.** At a `BoundedQuantity` expected
   type over `U`, membership SHALL admit exactly a `Value::Quantity` in unit
   `U` whose magnitude satisfies the present bounds inclusively (QExact FR-387
   Behavior 7), decided by the kernel's `ValueType::admits` at the leaf arm,
   with one `ValueVisit` event as for `Quantity(U)`. A refused value SHALL
   carry the invalid membership cause: `Type` for a `Value::Quantity` in a unit
   other than `U` (as for `Quantity(U)`), `Payload` for a `Value::Quantity` in
   `U` whose magnitude is outside the bounds (as for an out-of-domain `Int`),
   and `Kind` for every non-quantity value. A type whose bounds are inverted
   (`min > max`) refuses every value, by the same causes. The causes follow
   from the pair (expected type, value), never from the arm that happened to
   match.
9. **Retained type links in supplied membership.** When `same_type_walk`
   compares retained type links, a pair of `Map` links SHALL be a mismatch when
   their bounds differ (presence included) and otherwise continue to the pair
   of value types as one further compared link, as a pair of `Collection`
   links does, each pair taking its own `TypeLinkComparison` event and
   zero-based ordinal under [FR-109](./FR-109-bound-supplied-value-traversal.md)
   and [FR-117](./FR-117-bound-retained-numeric-type-links.md), so that a
   numeric link inside a map's value type is compared by the bounded numeric
   operations and never by unbounded `ValueType` equality. A pair of
   `BoundedQuantity` links is compared by Behavior 12. A `Map` link against a
   non-`Map` link, or a `BoundedQuantity` link against any other link, is a
   mismatch.
10. **Equality checking.** `check_equality_in` SHALL refuse with
    `OperatorIneligible`, before unit resolution and schedule selection, an
    operand whose source type or conversion target contains a `Map` link at any
    depth, through `Option`, `Collection`, `Map` links and record, tuple and
    union declarations, as the IEEE containment walk descends. This holds for
    two equal `Map` types, because QSpec FR-149 states no map equality relation
    and QExact FR-386 provides no map value to compare; the identity row of
    Behavior 6 does not make a map comparable. For an operand whose comparison
    type is a `BoundedQuantity` over `U`, `check_equality_in` SHALL resolve
    `U` in the unit scope (an unresolved unit is `TypeMismatch`), apply the
    `Quantity` dimension and distinct-unit refusals (`IncompatibleDimensions`,
    `DistinctUnits`) to the pair, and select `EqualitySchedule::Quantity`, for a
    bounded operand against a bounded operand of any bounds (equal, wider,
    narrower, disjoint or inverted) and against a plain `Quantity(U)`; the
    bounds play no part in the selection or in the comparison, which is exact
    value equality in the identical unit. A `BoundedQuantity` link nested
    inside any other type (`Option`, `Collection`, `Map`, record, tuple or
    union positions) is not read as `Quantity(U)`: QSpec FR-149 admits only
    explicit conversion and no `Option` or composite equality conversion, and
    QExact FR-387 gives `BoundedQuantity` its own type identity, so
    `Option<BoundedQuantity(U, bounds)>` against `Option<Quantity(U)>`, and
    against `Option<BoundedQuantity(U, other bounds)>` or another unit, are
    different compared types and `check_equality_in` refuses them as it
    refuses any other pair of different types, with `TypeMismatch`. Only the
    top-level operand read above applies the refinement read.
11. **Runtime-type resolution and same-type.** `runtime_type` SHALL descend a
    `Map` link as it descends a `Collection` link, resolving union handles in
    the value type through verified final-key joins and rebuilding the `Map`
    with its original bound, and returning `None` when an unsealed union is
    reached through it. `same_type` SHALL compare two `Map` links by bound
    (presence included) and then by value type, joining union handles as it
    does under `Option` and `Collection`, and SHALL NOT fall to unbounded
    `ValueType` equality, which cannot see that two handles name one sealed
    union. A `BoundedQuantity` link is a leaf of both walks, compared by QExact
    FR-387 type identity (unit and both bounds), so `same_type` is false for a
    bounded link against `Quantity(U)` or against a bounded link of other
    bounds. The supplied-membership form of that comparison is Behavior 12.
12. **Retained bounded-quantity type-link comparison.** For a compared pair
    of retained links of which either is a `BoundedQuantity`,
    `quire-semantic-value` SHALL, under the caller's authenticated finite
    magnitude ceiling `B` and original `Cancel` of
    [FR-117](./FR-117-bound-retained-numeric-type-links.md), proceed in this
    order and no other. (a) It requests exactly one `TypeLinkComparison` event
    before any comparison work, so a denied event prevents it. (b) A pair in
    which only one link is a `BoundedQuantity` is a completed mismatch. (c) A
    pair of units that differ, or of bounds whose presence differs on either
    side, is a completed mismatch with no magnitude measured. (d) When the
    units and the presence of both bounds are equal and a present bound pair
    remains, and `B` is not configured, it retains
    `UnsupportedHelper::NumericDomain` at the link locus, after the event and
    before any measurement, and never invents a default. (e) With `B`, it polls
    `Cancel`, then measures the magnitude bit length of each present bound with
    the kernel's public `Rational` part-bit measure, in the stable order
    expected minimum, expected maximum, actual minimum, actual maximum; the
    first measure above `B` produces typed unsupported size naming the side,
    the bound (minimum or maximum), the measured bits, `B` and the rooted
    locus, distinct from a mismatch and from supplied work-budget exhaustion.
    (f) Only when every measure is within `B` does it compare the bounds by
    exact `Rational` equality, which is then finite work of at most `B` bits
    per operand, polling `Cancel` before each bound pair and before publishing
    the result; a completed `true` continues the walk and a completed `false`
    returns the established invalid-type cause at the link locus. Cancellation
    keeps the original cause. The comparison never falls through to unmeasured
    `ValueType` equality, charges the initiating event once, adds no evaluator
    debit, and adds no logical event for a measure or comparison step.
13. **Wildcards are not an implementation.** Every `ValueType` match this
    requirement names SHALL list `Map` and `BoundedQuantity` explicitly where
    the behaviour above is not that of an existing arm, so that a future
    variant fails to compile at each site rather than inheriting a default.
    Where the existing generic arm already gives the behaviour above
    (Behaviors 7 and 8 admission, and the `Kind` cause for a `Map`), the
    behaviour SHALL be pinned by a test and not left implicit.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-127-AC-1 | `contains_ieee` and its metered form return true for `Map<Text, Float64>`, `Option<Map<Text, Float32>>`, `Map<Text, Option<Sequence<Float64>>>` and a record field of such a type, and false for `Map<Text, Integer>`, `Map<Text, BoundedQuantity>` and a map of a record without a float. The metered form's charge for a `Map` link equals its charge for the corresponding `Collection` link. | Test (TC-927) |
| FR-127-AC-2 | Declaring a record field of type `Map<Text, Float64>`, `Option<Map<Text, Float32>>` and `Sequence<Map<Text, Float64>>` is accepted in plain and metered declaration admission, while `Set<Map<Text, Float64>>` is refused `OperatorIneligible` by the existing non-`Sequence` rule. | Test (TC-927) |
| FR-127-AC-3 | A `Map` whose value type is `Population`, `Option<Population>` or `Sequence<Population>` is refused `OperatorIneligible`; one whose value type is an unregistered `Composite` is refused `UnknownDeclaration` with that key; one whose value type is a `Reference` to a non-object type is refused `Type(TypeMismatch)`; each in the plain and the metered walk with the same cause, and the same type inside `Option` and `Collection` gives the same cause. | Test (TC-927) |
| FR-127-AC-4 | A union member position of `Map<Text, Integer>`, `Option<Map<Text, Composite>>` and `Map<Text, Map<Text, Boolean>>` is walked to its leaf without error, charging one unit per link; a work budget one unit below that charge stops with the established work limit. | Test (TC-927) |
| FR-127-AC-5 | A record that refers to itself only through `Map<Text, Self>` and through `Map<Text, Self>[0, 4]` is accepted; one that refers to itself only through `Map<Text, Self>[1, 4]` is refused as a non-escaping cycle, as `Sequence<Self>[1, 4]` is; `Option<Map<Text, Self>[1, 4]>` is accepted because the `Option` escapes; and a named edge reached through a map keeps its recorded name. | Test (TC-927) |
| FR-127-AC-6 | `BoundedQuantity` over a unit, bounded and unbounded and inverted, is accepted as a record field, an `Option` payload, a `Collection` element and a `Map` value; `contains_ieee` is false for each; it adds no cycle edge; it is never refused; and a `BoundedQuantity` equal to itself is admitted by `admits_equality_conversion`. The conversions in and out of a bounded type are the subject of AC-13. | Test (TC-927) |
| FR-127-AC-7 | `integer_source_admits` returns false for `Int[0, 10]` against a `Map<Text, Integer>` target and against a `BoundedQuantity` target, and the existing rows for the other targets are unchanged. | Test (TC-927) |
| FR-127-AC-8 | In supplied and structural membership, no value of any variant is admitted at `Map<Text, Boolean>` or at `Map<Text, Option<Boolean>>[1, 3]`: each result is invalid membership with cause `Kind` at the value's locus after one `ValueVisit` event, for a collection, a text, an option, a composite and a quantity value; there is no panic. | Test (TC-927) |
| FR-127-AC-9 | At `BoundedQuantity` over `U` with bounds `[0, 100]`, membership admits quantities in `U` of magnitude 0, 50 and 100, and refuses magnitude -1 and 101 with cause `Payload`, a magnitude-50 quantity in another unit with cause `Type`, and a text, boolean and option value with cause `Kind`; with `[None, 100]` and `[0, None]` the open side admits any magnitude; with inverted `[5, 3]` every value refuses; the event count at each is one `ValueVisit`. | Test (TC-927) |
| FR-127-AC-10 | In supplied membership, an `Option` value whose retained payload type is a `Map` type against an expected `Option` of an equal `Map` type is admitted; against an expected `Map` with a different bound (including `None` against `Some(0, max)`), a different value type, or a `Collection` of the same element, it is invalid type at the compared link. With a numeric `Int` value type inside the map the comparison takes its own `TypeLinkComparison` event per pair, uses the configured numeric bound, and returns `UnsupportedHelper::NumericDomain` when none is configured, as for the same `Int` inside a `Collection`. | Test (TC-927) |
| FR-127-AC-11 | An `Option` value whose retained payload type is a `BoundedQuantity` is admitted against an expected equal `BoundedQuantity` and refused invalid type against one differing only in unit, only in minimum, only in maximum, or only in bound presence, and against `Quantity(U)`, the bound comparison following Behavior 12 and AC-17. | Test (TC-927) |
| FR-127-AC-12 | Inspection verifies that these `ValueType` matches name `Map` and `BoundedQuantity` explicitly or are pinned by AC-8 and AC-9 where the generic arm already gives the behaviour: in `src/declaration.rs` the union-link descent, plain and metered `contains_ieee`, plain and metered type-refusal walks, cycle-edge walk, `integer_source_admits`, `admits_equality_conversion`, `check_equality_in`'s schedule selection, `TypeEnvironment::same_type` and `TypeEnvironment::runtime_type`; and in `src/declaration/membership.rs` `leaf_cause`, `same_type_walk` and `admit_occurrence`. | Inspection |
| FR-127-AC-13 | `admits_equality_conversion` admits `BoundedQuantity(U, [0, 10])`, `BoundedQuantity(U, [None, None])` and inverted `BoundedQuantity(U, [5, 3])` to `Quantity(U)`; admits `BoundedQuantity(U, bounds)` to `Quantity(V)` exactly when `Quantity(U)` to `Quantity(V)` is admitted and refuses it when it is not or when either unit is unresolved; admits a bounded type to an equal bounded type (identity) and refuses it to a bounded type over the same unit with different bounds; refuses `Quantity(U)` to `BoundedQuantity(U, bounds)`; and the conversion never consults the bounds, so it admits the same rows for bounds that no value satisfies. | Test (TC-927) |
| FR-127-AC-14 | `check_equality_in` refuses `OperatorIneligible` an equality whose operand is `Map<Text, Integer>` against an equal `Map<Text, Integer>`, `Option<Map<Text, Boolean>>`, `Sequence<Map<Text, Boolean>>`, and a record whose field is a map, and one whose conversion target contains a map; with no schedule selected and no unit resolved. | Test (TC-927) |
| FR-127-AC-15 | `check_equality_in` over `BoundedQuantity(U, [0, 10])` against `BoundedQuantity(U, [0, 10])`, against `BoundedQuantity(U, [5, 20])`, against `BoundedQuantity(U, [11, 5])` and against `Quantity(U)` selects `EqualitySchedule::Quantity`; against a quantity of another unit of the same dimension it refuses `DistinctUnits`, of another dimension `IncompatibleDimensions`, and over an unresolved unit `TypeMismatch`; evaluating it on `50 U` against `3 U` compares exact values and ignores the bounds; `Option<BoundedQuantity(U, [0, 10])>` against `Option<BoundedQuantity(U, [0, 10])>` is accepted by the plan schedule, while it against `Option<Quantity(U)>`, against `Option<BoundedQuantity(U, [0, 20])>` and against `Option<BoundedQuantity(V, [0, 10])>` refuses `TypeMismatch`, as do the same pairs in a `Sequence` element and a record field. | Test (TC-927) |
| FR-127-AC-16 | `runtime_type` of `Map<Text, Union>` resolves the union handle to its sealed key and keeps the map's bound, of `Option<Map<Text, Union>[1, 3]>` rebuilds both wrappers, and is `None` for an unsealed union under a map; `same_type` holds for two maps whose union handles name one sealed union and fails for two maps differing only in bound, in bound presence or in value type; `same_type` is true for `Option<BoundedQuantity(U, [0, 10])>` against itself and false against `Option<Quantity(U)>` and against `Option<BoundedQuantity(U, [0, 20])>`. | Test (TC-927) |
| FR-127-AC-17 | In supplied membership with an authenticated `B`, a retained `BoundedQuantity` pair with equal units, both bounds present and equal completes `true`; bounds differing at the minimum, at the maximum, or in presence, or units differing, complete a mismatch (invalid type) and, for units or presence, measure no magnitude even when a bound exceeds `B`; with a bound of exactly `B` bits on each side the comparison completes, and one bit over `B` on the expected minimum, expected maximum, actual minimum and actual maximum each return typed unsupported size naming that side and bound, the bits, `B` and the locus, in that stable order when several exceed; pre-cancellation beats an oversized bound and returns the original cause before any measure; without a configured `B` the pair returns `UnsupportedHelper::NumericDomain` after one event and before any measurement; a denied event at N-1 prevents any measurement and the cumulative spend is the independently enumerated events plus the one initiating event, with no evaluator debit. | Test (TC-927) |

## Dependencies

- Types: [QExact FR-386](ix://agent-ix/quire-exact/FR-386) and
  [QExact FR-387](ix://agent-ix/quire-exact/FR-387) own the type identity,
  construction and kernel `admits` of the two variants.
- Source of the package declarations: QSpec FR-154 (map `values`,
  `Quantity<u>[min, max]`), QSpec FR-153 (population binding contexts) and
  QSpec FR-143 (recursion edges, escape through a possibly empty collection).
- [FR-109](./FR-109-bound-supplied-value-traversal.md),
  [FR-115](./FR-115-stage-and-seal-union-declarations.md),
  [FR-117](./FR-117-bound-retained-numeric-type-links.md) own the supplied
  traversal, union topology and numeric type-link rules that Behavior 9 extends
  to the new links.
- [QExact FR-381](ix://agent-ix/quire-exact/FR-381) is the model for Behavior
  12. QExact exposes no helper for quantity bounds, so Behavior 12 composes the
  public `Rational` part-bit measure and exact equality, which are finite under
  `B`; an FR-381-style kernel helper for quantity bounds is a welcome
  follow-up that would replace steps (e) and (f) and is not required.
- QSpec FR-149 and FR-154 (the refinement read of a bounded quantity by
  conversion, comparison and equality, and the closed quantity conversion row)
  own Behaviors 6 and 10; QSpec states no map equality relation, so the map
  refusal in Behavior 10 is QSV's decision pending a QSpec ruling.
- The falsifying controls are
  [TC-927](../test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md).

## Status

IMPLEMENTED; acceptance tests ([TC-927](../test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md)) are implemented and pass.
