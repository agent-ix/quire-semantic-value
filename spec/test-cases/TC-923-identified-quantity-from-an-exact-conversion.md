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
`UnitScope`, its refusals, and that it adds no charge. Steps 1 to 6 are
executable tests; step 7 is the implementing PR's inspection and build gate.
The fixtures are QSpec TC-187 U25's units (a declared unit converted into a
compound target, then into joules).

## Test Procedure

1. Convert a declared quantity exactly into a compound target and into a
   declared target; call `identified_quantity` on each; compare the magnitude,
   the unit id and the unit (AC-1).
2. Form the compound pair in a `UnitScope`, resolve the quantity, convert the
   resolved quantity into joules, and compare the value and the charges of both conversions with QSpec TC-187
   U25 and RT FR-007-AC-23 (AC-2).
3. Convert to a decimal target and to an integer target and call
   `identified_quantity` (AC-3).
4. Build a compound target whose dimensions cancel and whose ordered structural
   term encoding exceeds the 16 MiB handoff limit (200,000 roots over distinct
   dimension nodes that share one base map, the first 100,000 by structural
   key order at exponent +1 and the rest at -1, from a dimensionless source),
   convert, and call `identified_quantity`. Check
   `IdentityRefusal::InputBytes { bound: 16_777_216, required }` with
   `required > bound`; neither the encoded byte count nor a digest of the
   encoding is a fixed identity oracle.
   A target made large by a big exponent on one root has a different
   dimension from its source and is refused `IllTyped` before any `Conversion`
   exists, so it cannot reach the handoff. Check the caller's `UnitScope` holds
   no new unit (AC-4).
5. Build a declared root unit with admitted node `r` and a compound unit whose
   sole term is `(r, 1)`; compare `Declared(r)` with `Compound([(r, 1)])`,
   assert declared-before-compound order, and resolve the compound quantity
   (AC-5).
6. Convert into the step-4 target with a decimal representation, which
   completes, and call `identified_quantity`: `NotExact`, not `Identity`,
   because exactness is checked first (AC-3, AC-4).
7. Inspect the public method, `Cargo.toml` and `Cargo.lock` diff; run `make
   build-no-std`, `make lint` and `cargo build --locked` (AC-6).

## Expected Results

- Step 1: magnitude equals the converted exact value, the unit id equals the
  `IdentifiedUnit`'s id, and the unit equals the conversion's.
- Step 2: the returned id equals the quantity's unit id, `resolve` returns the
  conversion's unit, the second conversion is exact `1`, and each conversion makes the U25 charges.
- Step 3: `NotExact` for both, with no quantity.
- Step 4: `Identity(InputBytes { bound: 16_777_216, required })` with
  `required > bound`, no quantity or unit, and the scope's formed table
  unchanged.
- Step 5: distinct ids in declared-before-compound order, and the compound
  unit resolved.
- Step 6: `NotExact`.
- Step 7: only the new method and refusal type, and green gates.

## Status

The existing handoff is implemented (IR-831); the structural identity controls
in steps 4 and 5 are Planned under IR-1142.
