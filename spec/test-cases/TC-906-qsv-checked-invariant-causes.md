---
id: TC-906
title: "QSV reports the cause of each reached checked-invariant condition"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: verifies
---
# TC-906: QSV reports the cause of each reached checked-invariant condition

## Description

Verify [FR-108](../functional/FR-108-qsv-checked-invariant-causes.md)
through the QSV deferred-value, checked-equality and quantity-conversion
paths. Scope: FR-108-AC-1 through FR-108-AC-5.

## Test Procedure

1. Evaluate a checked deferred record or tuple value that belongs to its
   declared type and one that does not. Inject an earlier refused outcome and
   a charge stop before admission.
2. Exercise each reachable equality operand, enum, unit and schedule failure
   named in FR-108's table, using checked operands or the public operand
   conversion function as appropriate. Compare full typed refusals, rather
   than their formatted messages.
3. Make a selected comparator and an exact equality quantity conversion
   return distinct `IllTypedCause` values. Also exercise a charge stop before
   each comparison or conversion completes.
4. Exercise successful integer quantity placement. Inspect the noninteger
   retained-value branch and all QSV production `CheckedInvariant`
   constructors against the FR-108 table.
5. For every reached checked-invariant refusal, read `code()` and `cause()`;
   compare an ordinary refusal on the same public path.

## Expected Results

Admitted deferred values and successful integer conversion return their
ordinary results. Each failed premise yields the exact cause assigned in
FR-108, and the two `IllTypedCause` payloads survive unchanged. Earlier
stops keep their original outcomes. Every checked-invariant refusal has no
catalog code or cause, while the ordinary refusal keeps both. The inspected
integer noninteger branch selects `ExpectedIntegerPlacement`, and each QSV
constructor has one specified cause without a generic default.

## Status

Planned with the IR-712 QSV implementation.
