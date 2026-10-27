---
id: SR-16667
title: "Base spec review: QSV PR #69"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value; spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md; spec/test-cases/TC-927-map-and-bounded-quantity-in-declaration-admission-and-membership.md"
review_set: subset
---
# Base spec review: QSV PR #69

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
| FND-001 | high | The criterion refuses BoundedQuantity(U,bounds) to Quantity(U) equality conversion and wider bounded quantity targets. QSpec FR-154 at 46c7a465 lines 254-260 expressly makes it a Quantity(U) subtype read by conversion and equality, while FR-149 includes the quantity U to V equality-conversion row. An identity-only rule also leaves check_equality_in without unit resolution or a Quantity schedule for the new type. A bounded source [0,10] to plain Quantity(U) would be rejected despite the owning spec requiring admission. | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md:146 |
| FND-002 | high | Identity is an equality-conversion admission row, not an equality relation. QSpec FR-149 has no Map equality relation, and QExact FR-386 has no Map value; QSV check_equality_in currently falls through to a generic same-type Plan schedule. For equal Map types the spec must require static operator-ineligible refusal, while equal BoundedQuantity operands select the Quantity(U) schedule after the FR-154 refinement read. The criterion currently permits a Map equality and leaves bounded quantity schedule undefined. | spec/functional/FR-127-map-and-bounded-quantity-in-declaration-admission-and-membership.md:146 |

## Verdict

Findings require correction in the same PR before merge.

## Dispositions

At exact PR head c3c28511b5144af90a0219b0e8082445c88bb0bc, round 1 independently rechecked each original finding against QSpec 46c7a465, QExact f4d9c97f and the revised FR-127/TC-927 text.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Revised text verified at the same PR head. |
| FND-002 | still-open | Behavior 10 / AC-15 now treats Option<BoundedQuantity(U,bounds)> and Option<Quantity(U)> as the same comparison type. QSpec FR-149 at 46c7a465 lines 81-85 and 195-223 requires an explicit admitted conversion for different operand types and expressly has no option conversion; QExact FR-387 says BoundedQuantity differs from Quantity. FR-154 permits reading a bounded quantity value as Quantity for equality, not silently converting the enclosing Option type. Keep the top-level Quantity schedule but refuse unequal enclosing Option types absent an upstream ruling. |

## Dispositions (round 2)

At exact PR head b20d128c021cbaff7d6e2c35e316e5a9342e8bf4, the sole remaining finding was rechecked against QSpec FR-149 and QExact FR-387.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | Behavior 10 and AC-15 now keep the top-level BoundedQuantity Quantity schedule and refuse unequal enclosing Option, Collection, record and other composite types with TypeMismatch; TC-927 step 15 checks equal and differing wrappers. |
