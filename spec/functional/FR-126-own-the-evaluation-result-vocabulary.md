---
id: FR-126
title: "Own the catalog vocabulary and supplied-value stop records an evaluation result carries"
type: FR
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-401
    type: implements
---
# FR-126: Own the catalog vocabulary and supplied-value stop records an evaluation result carries

## Description

`quire-semantic-value` SHALL own the native-diagnostics vocabulary and the
supplied-value stop records that an evaluation result names, so the shared
evaluation interface (`quire-eval`, QSL FR-401) carries them without depending
on a compiler crate. QSL FR-401's placement table assigns them here.

## Outputs

- `catalog`: `Category`, `CatalogCode`, `CatalogCoded`, `UndefinedCoded`,
  `UndefinedRecord`, `UndefinedReason`, `InternalFault`, and the codes
  `ALLOCATION_FAILED` and `RESERVATION_SIZE_OVERFLOW`.
- `supplied_stop`: `SuppliedFailure`, `SuppliedStop`, `SuppliedContext`,
  `SuppliedCause`, `SuppliedPhase`, `SuppliedEvent`, `SuppliedLocus`,
  `SuppliedLimit`, `SuppliedLimitKind`, `AttemptedCount`,
  `SuppliedNumericOperand`, `SuppliedStorage`, `SuppliedStorageUnit`,
  `UnsupportedHelperStop`, and the limit-field names
  `ADMISSION_WORK_UNITS`, `CONVERSION_WORK_UNITS` and `NUMERIC_MAGNITUDE_BITS`.

## Behavior

- `Category` SHALL have the eight ADR-013 O-16 values; `exit_code` SHALL map
  them to 0, 10, 10, 20, 21, 22, 22 and 30 in declared order.
- `CatalogCode` equality and hashing SHALL be lexical on the code and cause;
  a rendering prefix SHALL NOT change either.
- `InternalFault::catalog_code` SHALL be
  `runtime_invariant`/`established-invariant-broken` and its category
  `InternalFailure`.
- `UndefinedCoded::catalog_code` SHALL render the optional prefix, then the
  reason's catalog spelling.
- `SuppliedLimitKind::field` and `SuppliedPhase::field` SHALL return the
  caller field that raises the bound.
- `SuppliedFailure` SHALL convert from a `SuppliedStop` as `Stop`.
- Building a refusal record at a source locus is not part of this crate.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-126-AC-1 | `Category::ALL` lists the eight values in order and `exit_code` returns 0, 10, 10, 20, 21, 22, 22, 30 for them; `trace_exit_code` returns 0 for `Inconclusive`; `most_severe` of no codes is `None`. | Test |
| FR-126-AC-2 | Two `CatalogCode`s with equal code and cause are equal and hash equal whatever their prefix; `Display` renders `prefix/code/cause` with a prefix and `code/cause` without. | Test |
| FR-126-AC-3 | `InternalFault::new(stage, invariant)` reports the runtime-invariant code, category `InternalFailure`, its stage and invariant, and the named values it was given. | Test |
| FR-126-AC-4 | An `UndefinedCoded` implementation without a prefix renders the reason spelling; with a prefix it renders `prefix/reason`. | Test |
| FR-126-AC-5 | `SuppliedLimitKind::field` and `SuppliedPhase::field` return the field names above (`work_units`, `value_occurrences`, `scale_expansion` for the three kernel counters); a `SuppliedStop` converts to `SuppliedFailure::Stop`, and `SuppliedStop::limit` is `Some` exactly for a `Limit` cause. | Test |

## Dependencies

- QSL FR-401 places these types; QSL FR-262 names them in the interface.
