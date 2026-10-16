---
id: TC-920
title: "ModelTransition builds, encodes, compares and orders by canonical identity bytes"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: verifies
---
# TC-920: ModelTransition builds, encodes, compares and orders by canonical identity bytes

## Description

Verify [FR-120](../functional/FR-120-model-transition-identity-type.md)
through QSV's public `model_transition` API. Scope: FR-120-AC-1 through
FR-120-AC-8 as tests; FR-120-AC-9 by inspection of the public items;
FR-120-AC-10 by `make build-no-std` and `make lint`; FR-120-AC-11 by the
`Cargo.toml` and `Cargo.lock` diff of the implementing change. Planned; the
tests are written with the code.

## Test Procedure

1. Build with the increment operation text of AC-4 and arguments `true`,
   `[1,2]`, `{"a":1}`; read both accessors. Build with no arguments and with 64.
2. Call `new` with operation texts `increment` (not JSON), `{"b":1,"a":2}`,
   ` {"a":1}`, `{"a":1} {"a":2}`, each with valid arguments.
3. Call `new` with a valid operation and arguments `true`, `{"b":1,"a":2}`,
   `false` and again with `[1, 2]` and `{"b":1,"a":2}`; then a bad operation
   with a bad argument.
4. Take `canonical_bytes` of the increment operation with `[true]`, with no
   arguments, and with `1`, `2`; compare to the exact bytes of AC-4.
5. Compare values over the same operation and arguments, a different operation,
   a different argument, swapped arguments and an argument prefix; compare their
   bytes.
6. Build operations whose node is `x` and `x/y` in one package; compare the
   values, their bytes and the raw operation strings. Sort 20 or more mixed
   values and the bytes; check antisymmetry and `cmp == Equal` against `==`.
7. Encode `Serialize` with `quire-canonical` and compare to `canonical_bytes`.
8. Call `new` and `canonical_bytes` under `Limits` smaller than the input and
   output, and through the crate's reservation seam deny one reservation.
9. Inspect the public items of the module, `make build-no-std`, `make lint`,
   and the `Cargo.toml` and `Cargo.lock` diff.

## Expected Results

- Step 1: verbatim, in order; empty and 64 arguments admitted.
- Step 2: each `Operation` site, `NotCanonicalJson`, no value.
- Step 3: `Argument(1)` first case, `Argument(1)` second case; the pair reports
  `Operation`.
- Step 4: the exact AC-4 bytes, `[]` for none, `[1,2]` in order.
- Step 5: only the identical pair is equal, with equal bytes.
- Step 6: value order equals byte order and is the reverse of the raw string
  order; the sorted sequences agree.
- Step 7: bytes equal.
- Step 8: a typed limit refusal with its site, `Allocation` on denial, no panic.
- Step 9: fields exactly `operation` and `arguments`; both builds succeed; no
  dependency, dev-dependency or lock package change.
