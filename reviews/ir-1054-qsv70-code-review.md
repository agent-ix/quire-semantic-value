---
id: SR-16674
title: "code-review of QSV PR #70"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration.rs, src/declaration/membership.rs, src/supplied_membership.rs, src/supplied_stop.rs, tests/map_and_bounded_quantity.rs; FR-127, TC-927"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-127
    type: references
---
# code-review of QSV PR #70

## Summary

Ticket: IR-1054. Independent review of exact head ff407e958ea509e497a3ddc9b1db8bcd3e787949 against merged spec acdb9d7c. Rust and public API inspected; no aggregate build run during review.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-127 | context_only | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | quire-semantic-value SHALL treat Map and BoundedQuantity explicitly in every ValueType walk it owns. |
| FR-127-AC-1 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `contains_ieee` and its metered form return true for `Map<Text, Float64>`, `Option<Map<Text, Float32>>`, `Map<Text, Option<Sequence<Float64>>>` and a record field of such a type, and false for `Map<Text, Integer>`, `Map<Text, BoundedQuantity>` and a map of a record without a float. The metered form's charge for a `Map` link equals its charge for the corresponding `Collection` link. |
| FR-127-AC-2 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Declaring a record field of type `Map<Text, Float64>`, `Option<Map<Text, Float32>>` and `Sequence<Map<Text, Float64>>` is accepted in plain and metered declaration admission, while `Set<Map<Text, Float64>>` is refused `OperatorIneligible` by the existing non-`Sequence` rule. |
| FR-127-AC-3 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A `Map` whose value type is `Population`, `Option<Population>` or `Sequence<Population>` is refused `OperatorIneligible`; one whose value type is an unregistered `Composite` is refused `UnknownDeclaration` with that key; one whose value type is a `Reference` to a non-object type is refused `Type(TypeMismatch)`; each in the plain and the metered walk with the same cause, and the same type inside `Option` and `Collection` gives the same cause. |
| FR-127-AC-4 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A union member position of `Map<Text, Integer>`, `Option<Map<Text, Composite>>` and `Map<Text, Map<Text, Boolean>>` is walked to its leaf without error, charging one unit per link; a work budget one unit below that charge stops with the established work limit. |
| FR-127-AC-5 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A record that refers to itself only through `Map<Text, Self>` and through `Map<Text, Self>[0, 4]` is accepted; one that refers to itself only through `Map<Text, Self>[1, 4]` is refused as a non-escaping cycle, as `Sequence<Self>[1, 4]` is; `Option<Map<Text, Self>[1, 4]>` is accepted because the `Option` escapes; and a named edge reached through a map keeps its recorded name. |
| FR-127-AC-6 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `BoundedQuantity` over a unit, bounded and unbounded and inverted, is accepted as a record field, an `Option` payload, a `Collection` element and a `Map` value; `contains_ieee` is false for each; it adds no cycle edge; it is never refused; and a `BoundedQuantity` equal to itself is admitted by `admits_equality_conversion`. The conversions in and out of a bounded type are the subject of AC-13. |
| FR-127-AC-7 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `integer_source_admits` returns false for `Int[0, 10]` against a `Map<Text, Integer>` target and against a `BoundedQuantity` target, and the existing rows for the other targets are unchanged. |
| FR-127-AC-8 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | In supplied and structural membership, no value of any variant is admitted at `Map<Text, Boolean>` or at `Map<Text, Option<Boolean>>[1, 3]`: each result is invalid membership with cause `Kind` at the value's locus after one `ValueVisit` event, for a collection, a text, an option, a composite and a quantity value; there is no panic. |
| FR-127-AC-9 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | At `BoundedQuantity` over `U` with bounds `[0, 100]`, membership admits quantities in `U` of magnitude 0, 50 and 100, and refuses magnitude -1 and 101 with cause `Payload`, a magnitude-50 quantity in another unit with cause `Type`, and a text, boolean and option value with cause `Kind`; with `[None, 100]` and `[0, None]` the open side admits any magnitude; with inverted `[5, 3]` every value refuses; the event count at each is one `ValueVisit`. |
| FR-127-AC-10 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | In supplied membership, an `Option` value whose retained payload type is a `Map` type against an expected `Option` of an equal `Map` type is admitted; against an expected `Map` with a different bound (including `None` against `Some(0, max)`), a different value type, or a `Collection` of the same element, it is invalid type at the compared link. With a numeric `Int` value type inside the map the comparison takes its own `TypeLinkComparison` event per pair, uses the configured numeric bound, and returns `UnsupportedHelper::NumericDomain` when none is configured, as for the same `Int` inside a `C |
| FR-127-AC-11 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | An `Option` value whose retained payload type is a `BoundedQuantity` is admitted against an expected equal `BoundedQuantity` and refused invalid type against one differing only in unit, only in minimum, only in maximum, or only in bound presence, and against `Quantity(U)`, the bound comparison following Behavior 12 and AC-17. |
| FR-127-AC-12 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Inspection verifies that these `ValueType` matches name `Map` and `BoundedQuantity` explicitly or are pinned by AC-8 and AC-9 where the generic arm already gives the behaviour: in `src/declaration.rs` the union-link descent, plain and metered `contains_ieee`, plain and metered type-refusal walks, cycle-edge walk, `integer_source_admits`, `admits_equality_conversion`, `check_equality_in`'s schedule selection, `TypeEnvironment::same_type` and `TypeEnvironment::runtime_type`; and in `src/declaration/membership.rs` `leaf_cause`, `same_type_walk` and `admit_occurrence`. |
| FR-127-AC-13 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `admits_equality_conversion` admits `BoundedQuantity(U, [0, 10])`, `BoundedQuantity(U, [None, None])` and inverted `BoundedQuantity(U, [5, 3])` to `Quantity(U)`; admits `BoundedQuantity(U, bounds)` to `Quantity(V)` exactly when `Quantity(U)` to `Quantity(V)` is admitted and refuses it when it is not or when either unit is unresolved; admits a bounded type to an equal bounded type (identity) and refuses it to a bounded type over the same unit with different bounds; refuses `Quantity(U)` to `BoundedQuantity(U, bounds)`; and the conversion never consults the bounds, so it admits the same rows for |
| FR-127-AC-14 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `check_equality_in` refuses `OperatorIneligible` an equality whose operand is `Map<Text, Integer>` against an equal `Map<Text, Integer>`, `Option<Map<Text, Boolean>>`, `Sequence<Map<Text, Boolean>>`, and a record whose field is a map, and one whose conversion target contains a map; with no schedule selected and no unit resolved. |
| FR-127-AC-15 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `check_equality_in` over `BoundedQuantity(U, [0, 10])` against `BoundedQuantity(U, [0, 10])`, against `BoundedQuantity(U, [5, 20])`, against `BoundedQuantity(U, [11, 5])` and against `Quantity(U)` selects `EqualitySchedule::Quantity`; against a quantity of another unit of the same dimension it refuses `DistinctUnits`, of another dimension `IncompatibleDimensions`, and over an unresolved unit `TypeMismatch`; evaluating it on `50 U` against `3 U` compares exact values and ignores the bounds; `Option<BoundedQuantity(U, [0, 10])>` against `Option<BoundedQuantity(U, [0, 10])>` is accepted by the pl |
| FR-127-AC-16 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `runtime_type` of `Map<Text, Union>` resolves the union handle to its sealed key and keeps the map's bound, of `Option<Map<Text, Union>[1, 3]>` rebuilds both wrappers, and is `None` for an unsealed union under a map; `same_type` holds for two maps whose union handles name one sealed union and fails for two maps differing only in bound, in bound presence or in value type; `same_type` is true for `Option<BoundedQuantity(U, [0, 10])>` against itself and false against `Option<Quantity(U)>` and against `Option<BoundedQuantity(U, [0, 20])>`. |
| FR-127-AC-17 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | In supplied membership with an authenticated `B`, a retained `BoundedQuantity` pair with equal units, both bounds present and equal completes `true`; bounds differing at the minimum, at the maximum, or in presence, or units differing, complete a mismatch (invalid type) and, for units or presence, measure no magnitude even when a bound exceeds `B`; with a bound of exactly `B` bits on each side the comparison completes, and one bit over `B` on the expected minimum, expected maximum, actual minimum and actual maximum each return typed unsupported size naming that side and bound, the bits, `B` and |
| TC-927 | examined | spec/test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Verify FR-127-AC-1 to AC-11 and AC-13 to AC-17 (and pin AC-12) for ValueType::Map and ValueType::BoundedQuantity through the public API. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-127 still says implementation and acceptance tests are unrun although this PR adds the implementation and TC-927 tests. | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md:268-270 |

## Verdict

**CONDITIONAL** — The implementation paths inspected conform, but FR-127 status is stale.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Verified against PR #70 head 90491bd. |
