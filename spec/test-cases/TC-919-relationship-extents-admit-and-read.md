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
FR-119-AC-15. This is a planned QSV counterpart of
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
   **bag-target**. For a missing end object check that type validation skips
   that end.
4. Add `p4`, then admit four composition targets under upper 3, no housing
   link under lower 1, and an empty complete composition extent. Separately
   admit a listed incomplete extent holding one link and omit housing
   entirely. Compare all three completeness states and their held reads.
5. Build real `PopulationManifest`s through `PopulationManifestBuilder::push`.
   After supplying the fixture's entire population set, consume the builder
   with `close`; separately consume a builder with `leave_open`. Pass each
   manifest and the admitted `TypeEnvironment` to `resolve`. Assert `Present`
   for an admitted reference even with an incomplete population entry. For a
   missing reference of a known type, assert `Dangling` with a `Closed`
   manifest containing at least one complete covering entry and no equal
   closure object. Independently assert `Undecidable` for an `Open` manifest,
   an incomplete covering entry, no covering entry, and an unknown object
   type. In a `Closed` manifest, add a complete entry covering the missing
   reference's exact type and a different population's incomplete entry
   covering its supertype; assert `Undecidable`. Use FR-082 conformance through
   `TypeEnvironment` to establish both covering relationships, and repeat
   with every covering entry complete to assert `Dangling`.
6. Push entries out of population-key order and inspect sorted manifest
   entries. Try a duplicate population key and an unknown covered type;
   assert their distinct typed `ManifestFailure<P>` causes. Deny a real
   representable manifest reservation and exercise capacity overflow; assert
   distinct typed `Allocation` with exact request amount and unit, and
   `Capacity` causes. Record the previously accepted entries before each
   failed push, then finish the builder and compare its published entries with
   that record; verify no refused entry appears in either an open or a closed
   manifest. Check that consuming `close` and `leave_open` yield the
   respective states. Use read instrumentation and structural clones to
   check zero allocations, zero charges, no external reads and no mutation
   across repeated `complete`, `targets`, `sources` and `resolve`.
7. Inject denial at actual table and extent reservation boundaries; assert
   `Allocation` carries the observed `StorageRequest` amount and unit.
   Exercise checked size overflow for `Capacity`. Compare state with its
   pre-attempt clone and retry after removing denial.

## Expected Results

Each isolated defect produces only its distinct typed refusal with exact
position payload, before later defects; refused input publishes nothing.
Counts outside declared multiplicity bounds admit. Reads are pure, total and
sorted by the whole reference key with repeats preserved. The three extent
states and three resolve results remain distinct. Missing references with no
covering population remain `Undecidable`; an incomplete covering supertype
prevents `Dangling`. Builder refusals are typed and publish no entry, and
denied storage never creates partial state.

## Status

PLANNED/UNRUN; QSV implementation and this acceptance test are pending.
