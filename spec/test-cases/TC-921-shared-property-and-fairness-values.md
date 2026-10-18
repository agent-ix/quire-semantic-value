---
id: TC-921
title: "Shared property and fairness values retain identity, order and encoding"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-121
    type: verifies
---
# TC-921: Shared property and fairness values retain identity, order and encoding

## Description

Verify [FR-121](../functional/FR-121-shared-property-and-fairness-values.md)
through the public `quire_semantic_value::fairness` API. The tests use two
distinct caller-supplied `EffectiveId` fixtures, including an all-zero digest;
they do not ask QSV to mint an identity. Steps 1 through 5 are executable
tests. Step 6 is the implementing PR's API/dependency inspection and build
gate. This case is planned and unrun until that code change.

## Test Procedure

1. Construct each property form and every kind/granularity pair. Build a
   `FairnessOperation` from a supplied type ID and `tick`, then a constraint;
   read each accessor and compare with the supplied values (AC-1).
2. Build a second `FairnessOperation` named `tick` under another type ID.
   Compare equal and unequal operation and constraint fixtures that vary one
   field at a time; hash equal pairs and insert all distinct fixtures in a
   `HashSet` (AC-2).
3. Sort the enum fixtures and mixed operation/constraint fixtures after
   reversing insertion order. Assert the exact variant and field order of
   AC-3, and compare every fixture pair's `cmp`, `partial_cmp` and equality
   results (AC-3).
4. Encode the weak-whole zero-ID `tick` constraint. Compare to the literal
   canonical bytes in AC-4. Change kind, granularity, type ID and name one at
   a time and compare bytes; encode a quoted name and compare RFC 8785
   escaping (AC-4).
5. Encode every property form and the operation alone, comparing their
   specified canonical spellings. Encode the constraint at its exact byte
   length limit and one byte below, checking success and the typed limit
   error respectively (AC-5).
6. Inspect the public module, root exports, `Cargo.toml` and `Cargo.lock`
   diff. Confirm no production identity minting, added normal dependency,
   compatibility re-export or `std` path. Run `make build-no-std`, `make lint`
   and `cargo build --locked` (AC-6).

## Expected Results

- Steps 1 and 2 retain all supplied values; same-named operations of distinct
  declaring types and constraints differing in any field are distinct.
- Step 3 gives the exact total order in AC-3, irrespective of insertion
  order, and `cmp == Equal` exactly when equality holds.
- Steps 4 and 5 produce AC-4's bytes and all specified string spellings;
  a too-small limit returns an error and no successful partial encoding.
- Step 6 finds only the new public `fairness` module and its items, unchanged
  existing exports and dependency set, no production ID minting, and green
  no-std, lint and locked build gates.

## Status

PLANNED / UNRUN (IR-872). No implementation test has been tagged yet.
