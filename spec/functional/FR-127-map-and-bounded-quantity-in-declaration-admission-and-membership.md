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
2. **Map is not refused for IEEE content.** The declaration type-refusal walk
   SHALL NOT refuse a `Map` whose value type contains `Float32` or `Float64`.
   The existing refusal of IEEE content under `Set`, `Bag` and other
   non-`Sequence` collection kinds exists because those kinds deduplicate,
   order or key their elements by element equality; a map's keys are `Text`,
   its values are neither deduplicated, ordered nor keyed, and a map therefore
   behaves like a `Sequence` here. `Map<Text, Float64>` is a valid declared
   type. A `Set` or `Bag` whose element is a map over floats remains refused,
   because its element `contains_ieee` (Behavior 1) and the existing rule
   applies to the collection, not to the map.
3. **Map in the declaration type-refusal walk.** The type-refusal walk, plain
   and metered, SHALL push a `Map` link's value type and apply every refusal to
   it that it applies inside `Option` and `Collection`: a `Population` inside a
   map's value type is refused `OperatorIneligible`; an unknown `Composite` is
   refused `UnknownDeclaration`; a `Reference` whose target is not a model
   object type is refused `Type(TypeMismatch)`. The plain and metered walks
   SHALL agree on cause and on which cause is first.
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
6. **No equality-conversion source or target.** `integer_source_admits`
   SHALL return `false` when the target is `Map` or `BoundedQuantity`, so an
   `Int` source is never convertible to either. A `Map` or `BoundedQuantity`
   source or target is admitted by `admits_equality_conversion` only by the
   existing identity row (the source equals the target); no new row is added.
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
   `BoundedQuantity` links is a leaf pair, equal exactly when QExact FR-387
   type identity (unit and both bounds) is equal. A `Map` link against a
   non-`Map` link, or a `BoundedQuantity` link against any other link, is a
   mismatch.
10. **Wildcards are not an implementation.** Every `ValueType` match this
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
| FR-127-AC-6 | `BoundedQuantity` over a unit, bounded and unbounded and inverted, is accepted as a record field, an `Option` payload, a `Collection` element and a `Map` value; `contains_ieee` is false for each; it adds no cycle edge; it is never refused; and an `Int` or `Quantity(U)` source with a `BoundedQuantity` target, and a `BoundedQuantity` source with a `Quantity(U)` target, is not admitted by `admits_equality_conversion` while a `BoundedQuantity` equal to itself is. | Test (TC-927) |
| FR-127-AC-7 | `integer_source_admits` returns false for `Int[0, 10]` against a `Map<Text, Integer>` target and against a `BoundedQuantity` target, and the existing rows for the other targets are unchanged. | Test (TC-927) |
| FR-127-AC-8 | In supplied and structural membership, no value of any variant is admitted at `Map<Text, Boolean>` or at `Map<Text, Option<Boolean>>[1, 3]`: each result is invalid membership with cause `Kind` at the value's locus after one `ValueVisit` event, for a collection, a text, an option, a composite and a quantity value; there is no panic. | Test (TC-927) |
| FR-127-AC-9 | At `BoundedQuantity` over `U` with bounds `[0, 100]`, membership admits quantities in `U` of magnitude 0, 50 and 100, and refuses magnitude -1 and 101 with cause `Payload`, a magnitude-50 quantity in another unit with cause `Type`, and a text, boolean and option value with cause `Kind`; with `[None, 100]` and `[0, None]` the open side admits any magnitude; with inverted `[5, 3]` every value refuses; the event count at each is one `ValueVisit`. | Test (TC-927) |
| FR-127-AC-10 | In supplied membership, an `Option` value whose retained payload type is a `Map` type against an expected `Option` of an equal `Map` type is admitted; against an expected `Map` with a different bound (including `None` against `Some(0, max)`), a different value type, or a `Collection` of the same element, it is invalid type at the compared link. With a numeric `Int` value type inside the map the comparison takes its own `TypeLinkComparison` event per pair, uses the configured numeric bound, and returns `UnsupportedHelper::NumericDomain` when none is configured, as for the same `Int` inside a `Collection`. | Test (TC-927) |
| FR-127-AC-11 | An `Option` value whose retained payload type is a `BoundedQuantity` is admitted against an expected equal `BoundedQuantity` and refused invalid type against one differing only in unit, only in minimum, only in maximum, or only in bound presence, and against `Quantity(U)`. | Test (TC-927) |
| FR-127-AC-12 | Inspection verifies that the seven `ValueType` matches in `src/declaration.rs` named in the Description (union-link descent, plain and metered `contains_ieee`, plain and metered type-refusal walks, cycle-edge walk, `integer_source_admits`) and the three tuple matches in `src/declaration/membership.rs` (`leaf_cause`, `same_type_walk` and `admit_occurrence`) name `Map` and `BoundedQuantity` explicitly or are pinned by AC-8 and AC-9 where the generic arm already gives the behaviour. | Inspection |

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
- The falsifying controls are
  [TC-927](../test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md).

## Status

SPEC CANDIDATE; implementation and acceptance tests are unrun.
