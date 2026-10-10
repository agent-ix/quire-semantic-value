---
id: FR-108
title: "Name checked-invariant failures in shared semantic values"
type: FR
relationships:
  - target: ix://agent-ix/quire-exact/FR-369
    type: references
---
# FR-108: Name checked-invariant failures in shared semantic values

## Description

When `quire-semantic-value` detects a failed premise after checking a deferred
value, equality, or quantity conversion, it SHALL return
`quire_exact::Refusal::CheckedInvariant` with the precise
`CheckedInvariantCause` assigned below. The closed cause type and its variant
names are owned by [kernel FR-369](ix://agent-ix/quire-exact/FR-369); this
requirement owns only QSV's choice of cause at its own source sites.

## Inputs

- A checked deferred record or tuple result and its declared value type.
- A checked equality, its completed operands and resolved units.
- A quantity conversion with a requested integer placement.

## Outputs

- The existing `Stop::Refused(Refusal::CheckedInvariant { cause })` result, or
  `Outcome::Refused` when the public operation returns an `Outcome`.

## Behavior

When a deferred field or tuple result does not belong to its checked value
type, QSV SHALL use `DeferredResultNotAdmitted`. When checked equality or
quantity conversion encounters a condition in the table, QSV SHALL use the
corresponding kernel cause. A reached `IllTyped { cause }` from a selected
comparison or exact quantity conversion SHALL retain that original
`IllTypedCause` in the typed payload. QSV SHALL preserve a preceding stop,
including a charge failure, without replacing it with a checked-invariant
cause.

| QSV condition | Kernel `CheckedInvariantCause` |
| --- | --- |
| Deferred record or tuple result fails its checked type admission | `DeferredResultNotAdmitted` |
| Equality operand fails admission to its checked source type | `EqualityOperandSourceNotAdmitted` |
| Decimal converted to an integer has a nonintegral normalized rational | `EqualityOperandNonIntegralDecimal` |
| Checked equality operand's quantity unit, or selected quantity comparison's unit, cannot be resolved | `EqualityUnitUnresolved` |
| Exact equality quantity conversion returns `IllTyped { cause }` | `EqualityQuantityConversionRejected { cause }` |
| Exact equality quantity conversion returns decimal or integer placement | `EqualityQuantityNonExactPlacement` |
| Checked source, target and value reach no equality conversion arm | `EqualityConversionShapeMismatch` |
| Converted equality value fails admission to its checked target type | `EqualityOperandTargetNotAdmitted` |
| Checked enum operand's variant cannot be resolved in the captured declaration | `EqualityEnumVariantUnresolved` |
| Checked text, enum or quantity schedule receives another value shape | `EqualityScheduleMismatch` |
| Selected text, enum or quantity comparator returns `IllTyped { cause }` | `ScheduledComparisonRefused { cause }` |
| Integer quantity placement retains a noninteger value | `ExpectedIntegerPlacement` |

For every table condition, the public result SHALL remain an internal checked
invariant: the caller can match the typed payload without parsing a message,
and the kernel's `Refusal::code()` and `Refusal::cause()` both return `None`.
QSV SHALL NOT turn a reached checked-invariant condition into an ordinary
catalog refusal. QSV SHALL use the kernel's closed cause type. QSV SHALL NOT
define another cause enum or a default cause for multiple conditions.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-108-AC-1 | A deferred result outside its checked value type yields `CheckedInvariant { cause: DeferredResultNotAdmitted }`; an admitted value is returned, and an earlier refusal or charge stop retains its original outcome. | Test |
| FR-108-AC-2 | For each equality operand, enum lookup, unit lookup and schedule condition in the table, the reached failure yields exactly its assigned typed cause; distinct reached conditions can be distinguished by matching the public `Refusal` payload. | Test |
| FR-108-AC-3 | A selected comparator or checked equality quantity conversion returning `IllTyped { cause }` yields the corresponding `ScheduledComparisonRefused { cause }` or `EqualityQuantityConversionRejected { cause }` with the same `IllTypedCause`; an earlier charge stop is retained. | Test |
| FR-108-AC-4 | The integer placement noninteger arm yields `ExpectedIntegerPlacement`, while integer placement success returns the integer conversion; inspection of every QSV production constructor confirms an exact table cause and no unit `CheckedInvariant` or catch-all default. | Inspection |
| FR-108-AC-5 | For every reached QSV checked-invariant failure, the public result carries `Refusal::CheckedInvariant { cause }`, and `Refusal::code()` and `Refusal::cause()` return `None`; ordinary refusals keep their original code and cause. | Test |

## Status

Planned. IR-712 implements the QSV source and test changes after kernel
FR-369's typed carrier is available.

## Dependencies

- [Kernel FR-369](ix://agent-ix/quire-exact/FR-369) owns the closed cause
  catalog, typed payload and code/cause behavior.
- QSpec FR-142, FR-143 and FR-149 own the operational quantity, deferred
  value and equality contracts respectively; this QSV requirement assigns
  causes to the shared leaf's internal-fault paths.
