---
id: TC-919
title: "Relationship extents admit in condition order and read by full reference key"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-119
    type: verifies
---
# TC-919: Relationship extents admit in condition order and read by full reference key

## Description

Verify [FR-119](../functional/FR-119-admit-and-read-relationship-extents.md)
through QSV's public table, extent and `ObjectClosure` APIs, using full
references supplied by a model-layer caller. The reservation-denial and
capacity cases of AC-11 and AC-15 use in-crate unit tests beside the code and
the existing crate-private reservation seam, as in FR-108/TC-907. Scope:
FR-119-AC-1 through FR-119-AC-20. This is a planned QSV counterpart of
[QSL TC-1033](ix://agent-ix/quire-spec-language/TC-1033).

## Test Procedure

1. Construct the TC-1033 `orders` fixture with `Sys`, `Pump`, `Tank`, objects
   `s1`, `s2`, `p1`, `p2`, `p3`, `t1`, and caller-defined ordered keys for
   `Sys-composition-Pump`, `Pump-housing-Tank` and non-navigation `pipe`.
   Use the fixture's **bag**, **bag-target** and **min** variants. Admit the
   table supplied by the model owner from authenticated declarations; try a
   repeated key and unknown type on each end. Assert the exact typed causes
   and payloads of FR-119-AC-1, including explicit end-type membership checks.
   Give a duplicate key an unknown end type and require `DuplicateRelationshipKey`;
   give one declaration unknown source and target types and require the source cause.
2. Supply composition `s1`-`p2`, `s1`-`p1` and housing `p2`-`t1`, `p1`-`t1`
   as complete extents. Reverse the extent and instance input orders in a
   second store. Compare every forward and inverse read and complete state.
   Add full references named `p1` in another universe and another type;
   query each as receiver and check that neither aliases `Pump p1`.
3. Map the fixture's non-navigation `pipe` to a key absent from QSV's table;
   submit it and then an unknown key in separate calls, followed by a duplicate
   composition extent, `t1`-`p1`, `s1`-`t1`, and repeated `s1`-`p1`.
   Inspect exact cause, relationship, zero-based position within each call and
   instance indices, and mismatch side and types. In one call submit valid
   extent 0 followed by absent-key extent 1; assert neither appears after refusal,
   and an earlier successfully built store stays unchanged. In another call make
   extent 1 repeat extent 0 and contain otherwise invalid instances; require
   `DuplicateExtent` and no extent from that call.
   Make a second one-shot constructor call containing an extent for a
   relationship already present in the first successful store. Assert that the
   new independent store admits it, while the first store remains unchanged;
   inspect the API for absence of incremental add.
   Put a duplicate triple at instance 1 before a type mismatch at instance 2
   and require `DuplicateTriple`.
   Put a type mismatch in extent 0 before a duplicate extent at index 1 and
   require `EndTypeMismatch`. Repeat the triple with **bag** and
   **bag-target**. Supply a missing end object whose full reference type mismatches
   its authenticated declared end and assert `EndTypeMismatch` with the actual type; repeat
   with an actual type unknown to the declared environment; inspect the explicit
   membership check separately from the behavioral assertion.
   Supply a missing end object whose reference type conforms and assert that
   the instance admits and the end remains available to `resolve`.
4. Inspect the public table input and admission path: each end carries only
   type and `unique`, no lower/upper bound or count comparison. Add many targets
   for one source and admit 0, 1 and that large count in separate fixtures,
   including an empty complete extent. QSL fixture bounds are not passed to QSV. Separately
   admit a listed incomplete extent holding one link and omit housing
   entirely. Compare all three completeness states and their held reads. Assert
   incomplete with instances remains incomplete and that a sibling listed
   complete leaves the unlisted relationship incomplete.
5. Implement real `PopulationEnumerator`s over the fixture's authenticated
   population set: population `Q` has one entry with caller-ordered
   `covered_types = [Sys, Pump, Tank]`, rather than three repeated `Q` rows.
   Construct entries fallibly and check typed empty and duplicate-covered-type
   refusals. Build outcomes with `ClosedPopulationManifest::from_enumerator`,
   passing a finite `row_limit: usize` on every call. One enumerator
   returns each row then `Ok(None)`; assert that only this normal exhaustion
   produces the `Closed` outcome and that the closed manifest equals its
   clone. Pass the outcome and admitted `TypeEnvironment` to `resolve`. Assert `Present`
   for an admitted reference even with an incomplete population entry. For a
   missing reference of a known type, assert `Dangling` with a `Closed`
   outcome containing at least one complete covering entry and no equal
   closure object. Check positive `Dangling` cases for exact-type coverage,
   supertype-only coverage, and coverage through one member of the multi-type
   Q entry, using a fixture with an admitted subtype for the supertype case.
   Covering means the reference type conforms, reflexively or as a child, to
   at least one declared member type under FR-082. In a separate `Closed`
   manifest with only a child type declared by a complete population entry,
   assert `Undecidable` for a missing parent-typed reference and `Dangling`
   for a missing child-typed reference, using the manifest's environment
   reference as the supplied `types`.
   Independently assert `Undecidable` for a `Partial` outcome,
   an incomplete covering entry, no covering entry, and an unknown actual object
   type. For the unknown-type case, use a `Closed` manifest with complete
   entries covering an admitted control type and the same environment value as
   supplied `types`; assert `Dangling` for a missing reference of
   the control type, then `Undecidable` for a missing reference whose actual
   type is not a member. Separately, name a covered type unknown to the
   manifest's environment in a retained row and assert the typed
   `ManifestFailure::UnknownCoveredType` at intake. Inspect `resolve` for an
   explicit actual-type membership check; absence of coverage could also
   produce the same result. Also check a known type whose complete coverage
   differs across two environments and assert `Undecidable` from their mismatch.
   With a held object, pass `types` as a different environment value from the
   one used to build a `Partial` manifest; assert `Present` despite both the
   environment mismatch and partial state.
   In a `Closed` manifest, add a complete entry covering the missing
   reference's exact type and a different population's incomplete entry
   covering its supertype; assert `Undecidable`. Repeat with the incomplete
   supertype as one member of a multi-type entry. Use
   [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) conformance through
   `TypeEnvironment` to establish both covering relationships, and repeat
   with every covering entry complete to assert `Dangling`.
6. Yield entries out of population-key order and inspect sorted retained rows.
   Set `row_limit` below the source row count. Assert that `from_enumerator`
   pulls the next `Some(row)` before checking the limit, counts that pulled row
   in both `source_rows` accessors as N + 1 for N retained rows, does not retain
   or validate it, and returns `Partial` with `RowLimit { limit: N }`.
   Repeat with an excess row that duplicates `P`, then one with an unknown
   covered type; each must still yield `Partial`/`RowLimit` before validation.
   Resolve a missing object covered by a retained complete row as `Undecidable`,
   and an admitted object as `Present`. With `row_limit` equal to the row count,
   assert that the next pull is `Ok(None)` and yields `Closed`. With `row_limit`
   zero, test an already exhausted enumerator (`Ok(None)` on the first pull)
   producing empty `Closed`, and a nonempty enumerator producing `Partial` with
   zero retained rows, `source_rows == 1`, and `RowLimit { limit: 0 }`.
   Separately return `Err` after retained rows. Assert `Partial` with those
   rows, both `source_rows` accessors equal N for N successful `Some` pulls,
   and `EnumeratorError` carrying the original error; the failing pull is not
   counted. Resolve a missing object covered by a retained complete row as
   `Undecidable`.
   Try a duplicate population key that also has an unknown covered type;
   require `DuplicatePopulation` before type validation or reservation. Then
   try an unknown covered type alone; assert their
   distinct typed `ManifestFailure<P>` causes and that neither row is silently
   dropped or merged. Then place two different unknown covered types in one
   row; assert `UnknownCoveredType` names the first in caller order, including
   when that type sorts after the second by `EffectiveId`. Deny a real
   representable manifest reservation and
   exercise capacity overflow; assert distinct typed `Allocation` with exact
   request amount and unit, and `Capacity` causes. Assert there is no public
   constructor taking a closed flag, no caller-controlled `close`, no mutable
   manifest fields, no `resolve` on `PartialPopulationRows`, and no conversion
   of partial rows into a closed manifest. Check read-only accessors for state,
   rows in `P` order, `source_rows` and stop. Use read instrumentation and
   structural clones to check equal repeated results, zero allocations and
   charges, and no mutation across `complete`, `targets`, `sources` and
   `resolve`. Inspect read signatures and bodies for no object-field, population
   member-list, path, clock or global access, and allocation-free reads. Inspect
   that only the transient manifest types borrow the environment. Inspect
   `ObjectClosure` for no environment field or lifetime, its unchanged `new`
   signature, and its added `resolve` and hand-written structural `PartialEq`/`Eq`.
   Admit a closure with a Float attribute containing NaN and assert it equals
   itself and its clone. Admit two otherwise identical closures whose Float
   attribute holds positive zero and negative zero respectively; assert they
   compare unequal. Inspect that field values compare by exact structural
   identity, with Float width and IEEE-754 bits compared directly, and that
   closure equality is never used for model numeric `=`. Compare
   two closed manifests with equal rows and state but distinct, value-equal
   environment values: they must compare unequal by reference identity.
   Check that `ManifestOutcome` equality requires `E: PartialEq`; the extent
   store compares its state by value and holds no environment reference.
   Pass a closed manifest built against one environment with `types` referring
   to another value; assert `Undecidable` even when its rows would cover a
   missing object. Inspect `resolve` for a shared `'t` and `core::ptr::eq`
   applied only after a missing-object check.
   Inspect that QSV does not authenticate the closure and manifest's
   checked-unit/source pairing; a common `TypeEnvironment` alone cannot
   establish that the caller paired the same source.
7. Inject denial at actual table and extent reservation boundaries; assert
   `Allocation` carries the observed `StorageRequest` amount and unit.
   Exercise checked size overflow for `Capacity`. Compare state with its
   pre-attempt clone and retry after removing denial.

## Expected Results

Each isolated defect produces only its distinct typed refusal with exact
position payload, before later defects; refused input publishes nothing.
The QSV input has no multiplicity bounds; 0, 1 and many instances admit. A missing end is checked
against its reference type even when no object is held; a conforming missing
end admits. Reads are pure, total and
sorted by the whole reference key with repeats preserved. The three extent
states and three resolve results remain distinct. Missing references with no
covering population remain `Undecidable`; a child-only declaration does not
cover a parent; an incomplete covering supertype
prevents `Dangling`. Only normal enumerator exhaustion closes the manifest; an
exact row limit still allows the following exhausted pull to close it. Errors
and exceeded row limits retain rows in `Partial` without enabling `Dangling`;
held objects remain `Present` regardless of manifest state or environment;
closure equality is reflexive for Float values, including NaN, while signed
zeros have distinct structural identity;
`source_rows` counts every pulled `Some(row)`, including the row exceeding the limit,
and excludes a failing `Err` pull.
Manifest refusals are typed and do not silently accept duplicate or unknown
rows; denied storage never publishes partially admitted state.

## Status

IMPLEMENTED/TESTS PASSED: FR-119-AC-1 through FR-119-AC-9 and FR-119-AC-11 through FR-119-AC-20.

FR-119-AC-10 remains PLANNED/UNRUN for its runtime zero-allocation assertion. Its repeated-read, charge, purity and Float-identity checks passed; the repository forbids unsafe code, so the attempted allocator instrumentation was removed. The read bodies were inspected for allocations.
