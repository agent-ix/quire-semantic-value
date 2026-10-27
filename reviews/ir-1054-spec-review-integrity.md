---
id: SR-16669
title: "Spec integrity review: QSV PR #69"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value; spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md; spec/test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md"
review_set: subset
---
# Spec integrity review: QSV PR #69

## Summary

Ticket: IR-1054. Independent review of PR #69 exact head fa91a443013ff0caa9e0fd620bb7fbf861e9fc03 against QSV main 2cab5d0 and QSpec 46c7a465 (FR-149/FR-154 unchanged at current e3d48d76), plus QExact f4d9c97f. Spec and TC only; no Rust or Cargo changes.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-127-AC-1 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `contains_ieee` and its metered form return true for `Map<Text, Float64>`, `Option<Map<Text, Float32>>`, `Map<Text, Option<Sequence<Float64>>>` and a record field of such a type, and false for `Map<Text, Integer>`, `Map<Text, BoundedQuantity>` and a map of a record without a float. The metered form's charge for a `Map` link equals its charge for the corresponding `Collection` link. |
| FR-127-AC-2 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Declaring a record field of type `Map<Text, Float64>`, `Option<Map<Text, Float32>>` and `Sequence<Map<Text, Float64>>` is accepted in plain and metered declaration admission, while `Set<Map<Text, Float64>>` is refused `OperatorIneligible` by the existing non-`Sequence` rule. |
| FR-127-AC-3 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A `Map` whose value type is `Population`, `Option<Population>` or `Sequence<Population>` is refused `OperatorIneligible`; one whose value type is an unregistered `Composite` is refused `UnknownDeclaration` with that key; one whose value type is a `Reference` to a non-object type is refused `Type(TypeMismatch)`; each in the plain and the metered walk with the same cause, and the same type inside `Option` and `Collection` gives the same cause. |
| FR-127-AC-4 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A union member position of `Map<Text, Integer>`, `Option<Map<Text, Composite>>` and `Map<Text, Map<Text, Boolean>>` is walked to its leaf without error, charging one unit per link; a work budget one unit below that charge stops with the established work limit. |
| FR-127-AC-5 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | A record that refers to itself only through `Map<Text, Self>` and through `Map<Text, Self>[0, 4]` is accepted; one that refers to itself only through `Map<Text, Self>[1, 4]` is refused as a non-escaping cycle, as `Sequence<Self>[1, 4]` is; `Option<Map<Text, Self>[1, 4]>` is accepted because the `Option` escapes; and a named edge reached through a map keeps its recorded name. |
| FR-127-AC-6 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `BoundedQuantity` over a unit, bounded and unbounded and inverted, is accepted as a record field, an `Option` payload, a `Collection` element and a `Map` value; `contains_ieee` is false for each; it adds no cycle edge; it is never refused; and an `Int` or `Quantity(U)` source with a `BoundedQuantity` target, and a `BoundedQuantity` source with a `Quantity(U)` target, is not admitted by `admits_equality_conversion` while a `BoundedQuantity` equal to itself is. |
| FR-127-AC-7 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | `integer_source_admits` returns false for `Int[0, 10]` against a `Map<Text, Integer>` target and against a `BoundedQuantity` target, and the existing rows for the other targets are unchanged. |
| FR-127-AC-8 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | In supplied and structural membership, no value of any variant is admitted at `Map<Text, Boolean>` or at `Map<Text, Option<Boolean>>[1, 3]`: each result is invalid membership with cause `Kind` at the value's locus after one `ValueVisit` event, for a collection, a text, an option, a composite and a quantity value; there is no panic. |
| FR-127-AC-9 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | At `BoundedQuantity` over `U` with bounds `[0, 100]`, membership admits quantities in `U` of magnitude 0, 50 and 100, and refuses magnitude -1 and 101 with cause `Payload`, a magnitude-50 quantity in another unit with cause `Type`, and a text, boolean and option value with cause `Kind`; with `[None, 100]` and `[0, None]` the open side admits any magnitude; with inverted `[5, 3]` every value refuses; the event count at each is one `ValueVisit`. |
| FR-127-AC-10 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | In supplied membership, an `Option` value whose retained payload type is a `Map` type against an expected `Option` of an equal `Map` type is admitted; against an expected `Map` with a different bound (including `None` against `Some(0, max)`), a different value type, or a `Collection` of the same element, it is invalid type at the compared link. With a numeric `Int` value type inside the map the comparison takes its own `TypeLinkComparison` event per pair, uses the configured numeric bound, and returns `UnsupportedHelper::NumericDomain` when none is configured, as for the same `Int` inside a `C |
| FR-127-AC-11 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | An `Option` value whose retained payload type is a `BoundedQuantity` is admitted against an expected equal `BoundedQuantity` and refused invalid type against one differing only in unit, only in minimum, only in maximum, or only in bound presence, and against `Quantity(U)`. |
| FR-127-AC-12 | examined | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Inspection verifies that the seven `ValueType` matches in `src/declaration.rs` named in the Description (union-link descent, plain and metered `contains_ieee`, plain and metered type-refusal walks, cycle-edge walk, `integer_source_admits`) and the three tuple matches in `src/declaration/membership.rs` (`leaf_cause`, `same_type_walk` and `admit_occurrence`) name `Map` and `BoundedQuantity` explicitly or are pinned by AC-8 and AC-9 where the generic arm already gives the behaviour. |
| TC-927 | examined | spec/test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md | Verify FR-127-AC-1 to AC-11 (and pin AC-12) for `ValueType::Map` and `ValueType::BoundedQuantity` through the public `TypeEnvironment`, `admits_equality_conversion` and supplied-membership API, as integration tests of the crate (`tests/map_and_bounded_quantity.rs`). Every control is a negative or adverse case that fails if a walk's wildcard or leaf arm handles a new variant as it handled the old o |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The inspection inventory omits TypeEnvironment::runtime_type and same_type in src/declaration.rs. Both are ValueType walks with wildcard fallthrough; runtime_type treats Map<Text, CompositeUnion> as an opaque leaf, so nested union resolution is skipped, and same_type compares nested Map links by unbounded ValueType equality instead of its sealed union-key join. The Description promises every owned walk, but AC-12 cannot detect these misses. | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md:152 |

## Verdict

Findings require correction in the same PR before merge.

## Dispositions

At exact PR head c3c28511b5144af90a0219b0e8082445c88bb0bc, round 1 independently rechecked each original finding against QSpec 46c7a465, QExact f4d9c97f and the revised FR-127/TC-927 text.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Revised text verified at the same PR head. |
