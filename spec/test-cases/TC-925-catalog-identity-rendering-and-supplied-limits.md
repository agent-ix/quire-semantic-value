---
id: TC-925
title: "Identity rendering, the state-model prefix and the supplied-value limits"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-126
    type: verifies
---
# TC-925: Identity rendering, the state-model prefix and the supplied-value limits

## Description

Verify FR-126-AC-6 to AC-8 through the public `catalog` and `supplied_stop`
API, and FR-126-AC-9 through the crate-internal step of `SuppliedRun` that
`admit` applies to each bounded-admission error (a unit test inside the crate,
because a public call cannot make the allocator deny or the size overflow on
demand). TC-926 verifies FR-126-AC-1 to AC-5.

## Test Procedure

1. Render a UTF-8 identity, a non-UTF-8 identity and two distinct malformed
   identities with `render_identity` (FR-126-AC-6).
2. Render the `catalog_code` of an `UndefinedCoded` implementation prefixed
   with `STATE_MODEL_CATALOG_PREFIX` (FR-126-AC-7).
3. Read `SuppliedLimits::default()` and apply each builder to it
   (FR-126-AC-8).
4. Map each storage-bearing bounded-admission error through `SuppliedRun`'s
   internal error mapping
   (FR-126-AC-9).

## Expected Results

- Step 1: `c9`, `identity bytes 0xfffe (not UTF-8)`, and two different renderings.
- Step 2: the rendering starts with `state-model/`.
- Step 3: 16777216, 16777216 and 4096; each builder changes only its field.
- Step 4: `Storage` for the allocator denial and `Capacity` for the
  unrepresentable size.

## Status

IMPLEMENTED (IR-588). The traced tests in `src/catalog.rs` and
`src/supplied_stop.rs` pass.
