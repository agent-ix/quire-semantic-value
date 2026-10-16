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
references supplied by a model-layer caller. Scope: FR-119-AC-1 through
FR-119-AC-18. This is a planned QSV counterpart of
[QSL TC-1033](ix://agent-ix/quire-spec-language/TC-1033).

## Test Procedure

1. Construct the TC-1033 `orders` fixture with `Sys`, `Pump`, `Tank`, objects
   `s1`, `s2`, `p1`, `p2`, `p3`, `t1`, and caller-defined ordered keys for
   `Sys-composition-Pump`, `Pump-housing-Tank` and non-navigation `pipe`.
   Use the fixture's **bag**, **bag-target** and **min** variants. Admit the
   table supplied by the model owner from authenticated declarations; try a
   repeated key and unknown type on each end. Assert the exact typed causes
   and payloads of FR-119-AC-1, including explicit end-type membership checks.
2. Supply composition `s1`-`p2`, `s1`-`p1` and housing `p2`-`t1`, `p1`-`t1`
   as complete extents. Reverse the extent and instance input orders in a
   second store. Compare every forward and inverse read and complete state.
   Add full references named `p1` in another universe and another type;
   query each as receiver and check that neither aliases `Pump p1`.
3. Independently submit an extent for `pipe`, then an unknown key, a duplicate
   composition extent, `t1`-`p1`, `s1`-`t1`, and repeated `s1`-`p1`.
   Inspect exact cause, relationship, zero-based extent and instance indices,
   and mismatch side and types. Put faults in competing instances and
   extents to check first-failure order. Repeat the triple with **bag** and
   **bag-target**. Supply a missing end object whose full reference type mismatches
   its authenticated declared end and assert `EndTypeMismatch` with the actual type; repeat
   with an unknown actual type that compares equal to itself under `conforms`.
   Supply a missing end object whose reference type conforms and assert that
   the instance admits and the end remains available to `resolve`.
4. Add `p4`, then admit four composition targets under upper 3, no housing
   link under lower 1, and an empty complete composition extent. Separately
   admit a listed incomplete extent holding one link and omit housing
   entirely. Compare all three completeness states and their held reads.
5. Implement real `PopulationEnumerator`s over the fixture's authenticated
   population set and build outcomes with `ClosedPopulationManifest::from_enumerator`,
   passing a finite `row_limit: usize` on every call. One enumerator
   returns each row then `Ok(None)`; assert that only this normal exhaustion
   produces `ManifestOutcome::Closed` and that the closed manifest equals its
   clone. Pass the outcome and admitted `TypeEnvironment` to `resolve`. Assert `Present`
   for an admitted reference even with an incomplete population entry. For a
   missing reference of a known type, assert `Dangling` with a `Closed`
   outcome containing at least one complete covering entry and no equal
   closure object. Independently assert `Undecidable` for a `Partial` outcome,
   an incomplete covering entry, no covering entry, and an unknown object
   type. In a `Closed` manifest, add a complete entry covering the missing
   reference's exact type and a different population's incomplete entry
   covering its supertype; assert `Undecidable`. Use FR-082 conformance through
   `TypeEnvironment` to establish both covering relationships, and repeat
   with every covering entry complete to assert `Dangling`.
6. Yield entries out of population-key order and inspect sorted retained rows.
   Set `row_limit` below the source row count. Assert that `from_enumerator`
   pulls the next `Some(row)` before checking the limit, counts that pulled row
   in both `source_rows` fields, does not retain or validate it, and returns
   `ManifestOutcome::Partial { rows, source_rows, stop: RowLimit { limit } }`.
   Resolve a missing object covered by a retained complete row as `Undecidable`,
   and an admitted object as `Present`. With `row_limit` equal to the row count,
   assert that the next pull is `Ok(None)` and yields `Closed`. With `row_limit`
   zero, test an already exhausted enumerator (`Ok(None)` on the first pull)
   producing empty `Closed`, and a nonempty enumerator producing `Partial` with
   zero retained rows, `source_rows == 1`, and `RowLimit { limit: 0 }`.
   Separately return `Err` after retained rows. Assert `Partial` with those
   rows, equal `source_rows` counts, and `EnumeratorError` carrying the original
   error; resolve a missing object covered by a retained complete row as
   `Undecidable`.
   Try a duplicate population key and an unknown covered type; assert their
   distinct typed `ManifestFailure<P>` causes and that neither row is silently
   dropped or merged. Deny a real representable manifest reservation and
   exercise capacity overflow; assert distinct typed `Allocation` with exact
   request amount and unit, and `Capacity` causes. Assert there is no public
   constructor taking a closed flag, no caller-controlled `close`, no state
   flag, no `resolve` on `PartialPopulationRows`, and no conversion of partial
   rows into a closed manifest. Use read instrumentation and
   structural clones to check zero allocations, zero charges, no external
   reads and no mutation across repeated `complete`, `targets`, `sources`
   and `resolve`.
7. Inject denial at actual table and extent reservation boundaries; assert
   `Allocation` carries the observed `StorageRequest` amount and unit.
   Exercise checked size overflow for `Capacity`. Compare state with its
   pre-attempt clone and retry after removing denial.

## Expected Results

Each isolated defect produces only its distinct typed refusal with exact
position payload, before later defects; refused input publishes nothing.
Counts outside declared multiplicity bounds admit. A missing end is checked
against its reference type even when no object is held; a conforming missing
end admits. Reads are pure, total and
sorted by the whole reference key with repeats preserved. The three extent
states and three resolve results remain distinct. Missing references with no
covering population remain `Undecidable`; an incomplete covering supertype
prevents `Dangling`. Only normal enumerator exhaustion closes the manifest; an
exact row limit still allows the following exhausted pull to close it. Errors
and exceeded row limits retain rows in `Partial` without enabling `Dangling`;
`source_rows` counts every pulled `Some(row)`, including the row exceeding the limit.
Manifest refusals are typed and do not silently accept duplicate or unknown
rows; denied storage never publishes partially admitted state.

## Status

PLANNED/UNRUN; QSV implementation and this acceptance test are pending.
