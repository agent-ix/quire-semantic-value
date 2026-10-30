---
id: TC-928
title: "Compound unit exponent encoding preserves values and typed refusals"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-129
    type: verifies
---
# TC-928: Compound unit exponent encoding preserves values and typed refusals

## Description

Verify [FR-129](../functional/FR-129-bound-compound-unit-exponent-reservations.md)
at QSV's `compound_unit_id` boundary using two distinct admitted root keys,
an inline positive exponent, and a large negative exponent whose decimal
spelling exceeds the inline integer range. Exercise exact byte bounds and
controlled reservation refusals without fixing a digest as an oracle.

## Test Procedure

1. Supply the two roots in structural order with exponents `1` and a negative
   integer outside `i128`, and encode under a byte limit that fits. Compare
   the returned `UnitId::Compound` with the supplied ordered terms and the
   RFC 8785 measurement with independently constructed complete canonical
   term bytes, including both exponent strings (AC-1).
2. Measure that complete canonical byte length `n`; repeat with limits `n`
   and `n - 1`. Also choose an earlier limit crossed while writing an exponent
   and compare the refusal's `required` with the canonical encoder's reached
   output count, including the structural prefix. The earlier refusal need
   not report `n` (AC-2).
3. In a QSV unit test, install a safe caller-scoped test hook at the exact API
   result boundary, then call public `compound_unit_id`. Substitute
   `IntegerCopyError::Allocation { requested: 4096 }` at exponent copy and
   `IntegerDecimalError::Allocation { requested: 8192 }` at decimal emission,
   one at a time. Restore the caller-scoped hook before another test runs;
   concurrent tests must not observe it. Assert each public result and no
   partial id. Confirm the production path uses the same mappings. Use exact's
   FR-390-AC-1/4 controlled-denial test as the independent oracle that real
   exact API failures report actual additional reservation bytes (AC-3).
4. Inspect the implementation path for the exact fallible calls, error
   mapping, structural id output, and absence of infallible exponent copies,
   formatting, digest conversion, or a second integer implementation (AC-4).

## Expected Results

- Step 1 returns the two original roots and exponents in order and the full
  signed decimal spellings in the canonical measurement; the id is structural.
- Step 2 succeeds at `n`, and at `n - 1` returns only
  `InputBytes { bound: n - 1, required: n }` with no id. The earlier refusal
  reports the canonical encoder's reached count including the prefix, not
  only the exponent length or an assumed complete preimage length.
- Step 3 returns `Allocation { requested: 4096 }` and
  `Allocation { requested: 8192 }` respectively, with no id, limit outcome,
  or malformed outcome. The exact test confirms the upstream requested-byte
  provenance; QSV's production mapping passes the value through unchanged.
- Step 4 confirms the specified calls and mappings and no prohibited path.
