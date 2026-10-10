---
id: TC-906
title: "QSV reports the cause of each reached checked-invariant condition"
type: TC
relationships:
  - target: ix://agent-ix/quire-exact/FR-369
    type: verifies
---
# TC-906: QSV reports the cause of each reached checked-invariant condition

## Description

Verify the QSV source-site mapping in
[kernel FR-369](ix://agent-ix/quire-exact/FR-369), especially FR-369-AC-9,
through QSV's deferred-value, checked-equality and quantity-conversion paths.
The operational contracts remain
[QSpec FR-143](ix://agent-ix/quire-specification/FR-143) for deferred record
and tuple results,
[QSpec FR-149](ix://agent-ix/quire-specification/FR-149) for checked equality,
and [QSpec FR-142](ix://agent-ix/quire-specification/FR-142) for quantity
conversion. Scope: FR-369-AC-5 and FR-369-AC-9; FR-142-AC-7,
FR-143-AC-8, and FR-149-AC-7/AC-10 cover the corresponding operational
result and charge behavior.

## Test Procedure

1. Exercise `src/declaration.rs` deferred record and tuple result admission
   with an admitted value and one outside its checked type. Reach an earlier
   refused outcome and charge stop before admission. Check
   `DeferredResultNotAdmitted` only for the failed admission.
2. Exercise the equality operand conversion in `src/declaration.rs` with a
   source value outside its checked type, a nonintegral decimal sent to an
   integer target, an unresolved unit, a rejected exact quantity conversion,
   an unexpected nonexact placement, an unmatched conversion shape, and a
   converted value outside its target type. Assert the corresponding
   `EqualityOperandSourceNotAdmitted`,
   `EqualityOperandNonIntegralDecimal`, `EqualityUnitUnresolved`,
   `EqualityQuantityConversionRejected { cause }`,
   `EqualityQuantityNonExactPlacement`, `EqualityConversionShapeMismatch`,
   and `EqualityOperandTargetNotAdmitted` payloads from FR-369's table.
3. Exercise `CheckedEquality::evaluate` with a missing captured enum variant,
   an unresolved quantity unit, a value shape outside its selected text,
   enum or quantity schedule, and an `IllTyped { cause }` from a selected
   comparator. Assert `EqualityEnumVariantUnresolved`,
   `EqualityUnitUnresolved`, `EqualityScheduleMismatch`, and
   `ScheduledComparisonRefused { cause }` respectively. Compare the original
   `IllTypedCause` payloads in steps 2 and 3 with their resulting payloads;
   exercise a prior charge stop on each path.
4. Exercise successful integer quantity placement in `src/quantity.rs`.
   Inspect its noninteger retained-value branch for
   `ExpectedIntegerPlacement`; inspect every QSV production constructor to
   confirm the precise FR-369 table cause with no generic default.
5. For each reached QSV checked-invariant refusal, assert the public result
   carries the typed payload and `Refusal::code()` and `Refusal::cause()` both
   return `None`. Check an ordinary refusal retains its catalog mapping.

## Expected Results

Admitted deferred values and successful integer conversion return their
ordinary results. Each failed premise yields the exact cause assigned in
FR-369, and the two `IllTypedCause` payloads survive unchanged. Earlier
stops keep their original outcomes. Every checked-invariant refusal has no
catalog code or cause, while the ordinary refusal keeps both. The inspected
integer noninteger branch selects `ExpectedIntegerPlacement`, and each QSV
constructor has one specified cause without a generic default.

## Status

Planned with the IR-712 QSV implementation. Its tests bind to the authoritative
`FR-369-AC-9` criterion (and `FR-369-AC-5` where they check catalog absence),
with operational criteria from QSpec as applicable.
