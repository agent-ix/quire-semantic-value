---
id: TC-906
title: "QSV reports the cause of each reached checked-invariant condition"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: verifies
---
# TC-906: QSV reports the cause of each reached checked-invariant condition

## Description

Verify the QSV source-site mapping in
[QSV FR-122](ix://agent-ix/quire-semantic-value/FR-122), especially the reachable
behavior in FR-122-AC-1 and the defensive Inspection in FR-122-AC-4,
through QSV's deferred-value, checked-equality and quantity-conversion paths.
The operational contracts remain
[QSpec FR-143](ix://agent-ix/quire-specification/FR-143) for deferred record
and tuple results,
[QSpec FR-149](ix://agent-ix/quire-specification/FR-149) for checked equality,
and [QSpec FR-142](ix://agent-ix/quire-specification/FR-142) for quantity
conversion. Scope: FR-122-AC-1, FR-122-AC-4 and kernel FR-369-AC-5; FR-142-AC-7,
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
   an unmatched conversion shape, and a
   converted value outside its target type. Assert the corresponding
   `EqualityOperandSourceNotAdmitted`,
   `EqualityOperandNonIntegralDecimal`, `EqualityUnitUnresolved`,
   `EqualityQuantityConversionRejected { cause }`,
   `EqualityConversionShapeMismatch`,
   and `EqualityOperandTargetNotAdmitted` payloads from FR-369's table.
3. Exercise `CheckedEquality::evaluate` (given the checking `TypeEnvironment`) with a missing captured enum variant,
   an unresolved quantity unit, a value shape outside its selected text,
   enum or quantity schedule, and an `IllTyped { cause }` from a selected
   comparator. Assert `EqualityEnumVariantUnresolved`,
   `EqualityUnitUnresolved`, `EqualityScheduleMismatch`, and
   `ScheduledComparisonRefused { cause }` respectively. Compare the original
   `IllTypedCause` payloads in steps 2 and 3 with their resulting payloads;
   exercise a prior charge stop on each path.
4. Exercise successful scale-zero integer quantity placement and call the
   private converter with an admitted scale-one integer placement in
   `src/quantity.rs`. Assert `ExpectedIntegerPlacement` for the latter.
   Inspect every QSV production constructor to confirm the precise FR-369
   table cause with no generic default. For FR-122-AC-4, inspect the
   non-exact placement arm in `operand_value`, its hard-coded `Exact` target,
   and the `Exact` target mapping and result arm in `src/quantity.rs`.
5. For each reached QSV checked-invariant refusal, assert the public result
   carries the typed payload and `Refusal::code()` and `Refusal::cause()` both
   return `None`. Check an ordinary refusal retains its catalog mapping.

## Expected Results

Admitted deferred values and successful integer conversion return their
ordinary results. Each failed premise yields the exact cause assigned in
FR-369, and the two `IllTypedCause` payloads survive unchanged. Earlier
stops keep their original outcomes. Every checked-invariant refusal has no
catalog code or cause, while the ordinary refusal keeps both. The admitted
scale-one integer placement returns `ExpectedIntegerPlacement`, and each QSV
constructor has one specified cause without a generic default. The defensive
non-exact equality placement maps to `EqualityQuantityNonExactPlacement`,
while the current exact target returns only exact placement.

## Status

The eight declaration tests and the integer converter test are tagged
`FR-122-AC-1` (and `FR-369-AC-5` where catalog absence is checked). They
fully exercise AC-1's reachable causes, payloads, prior outcomes and charge
stops, including failed tuple admission and the private converter's noninteger
placement. FR-122-AC-4 covers the defensive `EqualityQuantityNonExactPlacement`
row by Inspection of `src/declaration.rs` at `operand_value` and
`src/quantity.rs` at `convert_quantity` and `convert`: equality requests a
hard-coded `QuantityTarget::Exact`, which produces only `ConvertedValue::Exact`.
AC-4 has no trace tag; Inspection mints no source symbol and strict coverage
lists it as an unbacked Inspection row. If that target becomes configurable,
AC-4 becomes a Test criterion.
