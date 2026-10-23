---
id: FR-126
title: "Own the catalog vocabulary, supplied-value stop records and supplied-value run an evaluation result carries"
type: FR
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-401
    type: implements
  - target: ix://agent-ix/quire-spec-language/FR-403
    type: implements
---
# FR-126: Own the catalog vocabulary, supplied-value stop records and supplied-value run an evaluation result carries

## Description

`quire-semantic-value` SHALL own the native-diagnostics vocabulary, the
supplied-value stop records that an evaluation result names, and the
supplied-value run (`SuppliedRun`) that produces them, so the shared
evaluation interface (`quire-eval`, QSL FR-401) carries them without depending
on a compiler crate. QSL FR-401's placement table assigns the vocabulary and
records here, and QSL FR-403's disposition table assigns the run and
`render_identity` here.

## Outputs

- `catalog`: `Category`, `CatalogCode`, `CatalogCoded`, `UndefinedCoded`,
  `UndefinedRecord`, `UndefinedReason` and `InternalFault`; the function
  `render_identity`; and the constant `STATE_MODEL_CATALOG_PREFIX`.
- `supplied_stop`: `SuppliedFailure`, `SuppliedStop`, `SuppliedContext`,
  `SuppliedCause`, `SuppliedPhase`, `SuppliedEvent`, `SuppliedLocus`,
  `SuppliedLimit`, `SuppliedLimitKind`, `AttemptedCount`,
  `SuppliedNumericOperand`, `SuppliedStorage`, `SuppliedStorageUnit`,
  `UnsupportedHelperStop`, and the limit-field names
  `ADMISSION_WORK_UNITS`, `CONVERSION_WORK_UNITS` and `NUMERIC_MAGNITUDE_BITS`;
  and the supplied-value run `SuppliedRun` with `SuppliedLimits` and the
  defaults `DEFAULT_SUPPLIED_WORK_UNITS` and `DEFAULT_NUMERIC_MAGNITUDE_BITS`.
  `SuppliedRun::with_reservation_permit` exists only under the `test-support`
  feature, which only a `[dev-dependencies]` entry enables.

## Behavior

- `Category` SHALL have the eight ADR-013 O-16 values.
- `Category::exit_code` SHALL map them to 0, 10, 10, 20, 21, 22, 22 and 30 in
  declared order.
- `CatalogCode` equality and hashing SHALL be lexical on the code and cause.
- A `CatalogCode` rendering prefix SHALL NOT change its equality or hash.
- `InternalFault::catalog_code` SHALL be
  `runtime_invariant`/`established-invariant-broken` and its category
  `InternalFailure`.
- `UndefinedCoded::catalog_code` SHALL render the optional prefix, then the
  reason's catalog spelling.
- `SuppliedLimitKind::field` and `SuppliedPhase::field` SHALL return the
  caller field that raises the bound: `supplied.admission_work_units`,
  `supplied.conversion_work_units` and `supplied.numeric_magnitude_bits` for
  the three supplied counters.
- `SuppliedFailure` SHALL convert from a `SuppliedStop` as `Stop`.
- `render_identity` SHALL render identity bytes that are valid UTF-8 as
  themselves, and any other bytes as
  `identity bytes 0x<lowercase hex> (not UTF-8)`, so two distinct malformed
  identities never render alike.
- `STATE_MODEL_CATALOG_PREFIX` SHALL be the `StateModel` family's catalog
  prefix, `state-model`.
- `SuppliedLimits::default` SHALL set the two work ceilings to
  `DEFAULT_SUPPLIED_WORK_UNITS` (16777216) and the numeric magnitude bound to
  `DEFAULT_NUMERIC_MAGNITUDE_BITS` (4096); each `with_*` builder SHALL set only
  its own field.
- `SuppliedRun` SHALL map a storage-bearing bounded-admission error to its own
  typed cause: an allocator denial is `Storage` and an unrepresentable size is
  `Capacity`.
- Building a refusal record at a source locus is not part of this crate.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-126-AC-1 | `Category::ALL` is exactly `[Success, Violation, Undefined, Refusal, Unsupported, Incomplete, Inconclusive, InternalFailure]`, in that order, and `exit_code` returns 0, 10, 10, 20, 21, 22, 22, 30 for them; `trace_exit_code` returns 0 for `Inconclusive`; `most_severe` of no codes is `None`. | Test (TC-926) |
| FR-126-AC-2 | Two `CatalogCode`s with equal code and cause are equal and hash equal whatever their prefix; `Display` renders `prefix/code/cause` with a prefix and `code/cause` without. | Test (TC-926) |
| FR-126-AC-3 | `InternalFault::new(stage, invariant)` reports the runtime-invariant code, category `InternalFailure`, its stage and invariant, and the named values it was given. | Test (TC-926) |
| FR-126-AC-4 | An `UndefinedCoded` implementation without a prefix renders the reason spelling; with a prefix it renders `prefix/reason`. | Test (TC-926) |
| FR-126-AC-5 | `SuppliedLimitKind::field` and `SuppliedPhase::field` return the field names above and `work_units`, `value_occurrences` and `scale_expansion` for the three kernel counters; a `SuppliedStop` converts to `SuppliedFailure::Stop`, and `SuppliedStop::limit` is `Some` exactly for a `Limit` cause. | Test (TC-926) |
| FR-126-AC-6 | `render_identity(b"c9")` is `c9`; `render_identity(&[0xFF, 0xFE])` is `identity bytes 0xfffe (not UTF-8)`; `render_identity(&[0xFF])` and `render_identity(&[0xFE])` differ. | Test (TC-925) |
| FR-126-AC-7 | An `UndefinedCoded` implementation whose prefix is `STATE_MODEL_CATALOG_PREFIX` renders a `catalog_code` that begins with `state-model/`. | Test (TC-925) |
| FR-126-AC-8 | `SuppliedLimits::default()` reports 16777216, 16777216 and 4096 for A, H and B; each of `with_admission_work_units`, `with_conversion_work_units` and `with_numeric_magnitude_bits` changes exactly its own field and leaves the other two unchanged. | Test (TC-925) |
| FR-126-AC-9 | A `SuppliedRun` maps an allocator-denied bounded-admission error to a `Storage` cause and an unrepresentable size to a `Capacity` cause, each with its own unit, no limit and zero consumed. | Test (TC-925) |

## Dependencies

- QSL FR-401 places these types; QSL FR-262 names them in the interface.
