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
3. In QSV unit tests, call the private construction core with copy and render
   closures. Substitute `IntegerCopyError::Allocation { requested: 4096 }`
   at exponent copy and `IntegerDecimalError::Allocation { requested: 8192 }`
   at decimal emission, one at a time. Assert each core result and no partial
   id. Inspect that public `compound_unit_id` delegates to this core with
   the real exact APIs, so the tested projections serve the public production
   path. Use exact's FR-390-AC-1/4 controlled-denial test as the independent
   oracle that real exact API failures report actual additional reservation
   bytes (AC-3).
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
- Step 3's private core returns `Allocation { requested: 4096 }` and
  `Allocation { requested: 8192 }` respectively, with no id, limit outcome,
  or malformed outcome. The public function delegates with the real exact
  APIs; the exact test confirms upstream requested-byte provenance, and
  QSV's shared production mapping passes each value through unchanged.
- Step 4 confirms the specified calls and mappings and no prohibited path.
