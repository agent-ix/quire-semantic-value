---
id: TC-923
title: "An exact conversion hands the next operation an identified quantity"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: verifies
---
# TC-923: An exact conversion hands the next operation an identified quantity

## Description

Verify [FR-124](../functional/FR-124-identified-quantity-from-an-exact-conversion.md)
through the public `quire_semantic_value::quantity` API: the pair that
`Conversion::identified_quantity` returns, its retention and resolution in a
`UnitScope`, its refusals, and that it adds no charge. Steps 1 to 5 are
executable tests; step 6 is the implementing PR's inspection and build gate.
The fixtures are RT TC-187 U25's units (a declared unit converted into a
compound target, then into joules).

## Test Procedure

1. Convert a declared quantity exactly into a compound target and into a
   declared target; call `identified_quantity` on each; compare the magnitude,
   the unit id and the unit (AC-1).
2. Form the compound pair in a `UnitScope`, resolve the quantity, convert the
   resolved quantity into joules, and compare the value and the meter with two
   back-to-back `convert_quantity` calls (AC-2).
3. Convert to a decimal target and to an integer target and call
   `identified_quantity` (AC-3).
4. Build a compound target whose id the identity byte limit refuses, convert,
   and call `identified_quantity`; resolve a quantity of that unit (AC-4).
5. Build a declared unit and a compound unit with equal digest bytes; compare
   their unit ids and resolve the compound quantity (AC-5).
6. Inspect the public method, `Cargo.toml` and `Cargo.lock` diff; run `make
   build-no-std`, `make lint` and `cargo build --locked` (AC-6).

## Expected Results

- Step 1: magnitude equals the converted exact value, the unit id equals the
  `IdentifiedUnit`'s id, and the unit equals the conversion's.
- Step 2: the returned id equals the quantity's unit id, `resolve` returns the
  conversion's unit, the second conversion is exact `1`, and the meters are
  equal.
- Step 3: `NotExact` for both, with no quantity.
- Step 4: `Identity` with the originating refusal, no quantity or unit, and
  nothing to resolve.
- Step 5: distinct ids, and the compound unit resolved.
- Step 6: only the new method and refusal type, and green gates.

## Status

PLANNED (IR-831).
