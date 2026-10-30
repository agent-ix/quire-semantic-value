---
id: TC-922
title: "Step simulation records retain identity, order and encoding"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-123
    type: verifies
---
# TC-922: Step simulation records retain identity, order and encoding

## Description

Verify [FR-123](../functional/FR-123-step-simulation-record-and-obligation.md)
through the public `quire_semantic_value::step_simulation` API. The tests use
caller-supplied `NodeKey` fixtures; QSV mints none. Steps 1 through 5 are
executable tests; step 6 is the implementing PR's inspection and build gate.

## Test Procedure

1. Construct records with each obligation form, repeated and empty hypotheses;
   read every field (AC-1).
2. Vary one field at a time; compare, hash and insert into a `HashSet` (AC-2).
3. Sort obligations and records after reversing insertion order; compare `cmp`,
   `partial_cmp` and equality over all pairs (AC-3).
4. Encode `Initial` and `Step` records, an obligation and the maximum row
   position; compare with independently written literal JSON bytes using
   the structural `NodeKey` display spelling and escaped text-path quotes of
   Behavior 3 (quire-exact PR #101). Vary refinement, row, hypothesis order
   and hypothesis count and compare the resulting bytes (AC-4).
5. Encode at the exact length and one byte below (AC-5).
6. Inspect the public module, `Cargo.toml` and `Cargo.lock` diff; run
   `make build-no-std`, `make lint` and `cargo build --locked` (AC-6).

## Expected Results

- Steps 1 to 3 retain values, distinguish every field and give the specified
  total order.
- Step 4 gives the exact structural-key bytes, including JSON escaping; step 5
  gives a typed limit error below the length.
- Step 6 finds only the new module and green gates.

## Status

IMPLEMENTED (QSL-899). The five traced tests pass.
