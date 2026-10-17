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
FR-120-AC-8, AC-12, AC-13, AC-15 and AC-16 as tests; FR-120-AC-9 and AC-14 by
inspection of the public items and source; FR-120-AC-10 by `make build-no-std` and `make lint`; FR-120-AC-11 by the
`Cargo.toml` and `Cargo.lock` diff of the implementing change. Implemented: steps 1-12 are
`tests/model_transition.rs`; step 13 (AC-9, AC-10, AC-11, AC-14) is inspection and
analysis, recorded on the implementing pull request, and stays planned.

## Test Procedure

1. Build with the increment operation text of AC-4, receiver `"c1"` and
   arguments `true`, `[1,2]`, `{"a":1}`; read all three accessors. Build with
   an absent receiver, with no arguments and with 64.
2. Call `new` with operation texts `increment` (not JSON), `{"b":1,"a":2}`,
   ` {"a":1}`, `{"a":1} {"a":2}`, each with a valid receiver and arguments; then
   a valid operation with receiver `c1` and with receiver `{"b":1,"a":2}`.
3. Call `new` with a valid operation and arguments `true`, `{"b":1,"a":2}`,
   `false` and again with `[1, 2]` and `{"b":1,"a":2}`; then a bad operation
   with a bad receiver and a bad argument, and a bad receiver with a bad argument.
4. Take `canonical_bytes` of the increment operation with receiver `"c1"` and
   `[true]`, with an absent receiver, with no arguments, and with `1`, `2`;
   compare to the exact bytes of AC-4.
5. Compare values over the same operation, receiver and arguments, a different
   operation, a different argument, swapped arguments, an argument prefix, a
   different receiver, and an absent receiver against receiver `"null"`; compare
   their bytes.
6. Build operations whose node is `x` and `x/y` in one package with equal
   receiver and arguments; compare the values, their bytes and the raw operation
   strings. Build arguments `[1]` and `[1,2]` over equal operation and receiver
   and compare.
7. Encode each AC-4 value with `quire_canonical::to_vec` and compare to
   `canonical_bytes`; repeat under a limit one byte short.
8. Call `new` with a limit smaller than the operation, then the receiver, then
   argument 1, each other text within it; then with every text within the limit
   and their sum over it.
9. Call `canonical_bytes` under a limit one byte short and under the exact
   length.
10. Call `new` with receiver `null`. Compare `cmp` with `==` over every pair of
    the fixtures of steps 5 and 6.
11. Measure `cmp` and `partial_cmp` with `allocation-counter` over 20 or more
    mixed values; sort values and bytes; check antisymmetry and transitivity.
12. Build a value whose receiver and one argument are arrays nested 100,000 deep
    under a large limit; encode both ways, compare, clone, format and drop it.
13. Inspect the public items of the module and the source of `new` and
    `canonical_bytes` for panic paths and allocation mapping, run
    `make build-no-std` and `make lint`, and inspect the `Cargo.toml` and
    `Cargo.lock` diff.

## Expected Results

- Step 1: verbatim, in order, absent receiver reads `None`; empty and 64
  arguments admitted.
- Step 2: each `Operation` site for the first four, `Receiver` for the last two,
  `NotCanonicalJson`, no value.
- Step 3: `Argument(1)` for the first case, `Argument(0)` for the second (`[1, 2]`
  has insignificant whitespace); the first pair reports `Operation`, the second
  `Receiver`.
- Step 4: the exact AC-4 bytes, `"receiver":null` when absent, `[]` for none,
  `[1,2]` in order.
- Step 5: only the identical pair is equal, with equal bytes; receiver-only
  differences are unequal and differ only in `receiver`.
- Step 6: value order equals byte order and is the reverse of the raw string
  order; `[1,2]` orders before `[1]`.
- Step 7: bytes equal; the short limit gives the same refusal both ways.
- Step 8: `InputBytes` at the site of the oversize text with `bound` the limit and
  `required` over it; the texts within the limit are admitted.
- Step 9: `IdentityRefusal::InputBytes` when short, bytes at the exact length.
- Step 10: `NullReceiver` at site `Receiver`; `cmp` is `Equal` exactly when `==`.
- Step 11: zero allocations; sorted values and sorted bytes agree.
- Step 12: no stack overflow; equal encodings.
- Step 13: fields exactly `operation`, `receiver` and `arguments`; no panic path;
  allocation refusals mapped; both builds succeed; no dependency,
  dev-dependency or lock package change.
