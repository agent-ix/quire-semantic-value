---
id: TC-926
title: "The catalog vocabulary and the supplied-value stop records through the public API"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-126
    type: verifies
---
# TC-926: The catalog vocabulary and the supplied-value stop records through the public API

## Description

Verify FR-126-AC-1 to AC-5 through the public `catalog` and `supplied_stop`
API, as an integration test of the crate (`tests/evaluation_vocabulary.rs`):
the order and exit codes of `Category`, the identity and rendering of
`CatalogCode`, `InternalFault`, the rendering of an `UndefinedCoded` cause,
and the field names and conversions of the supplied-value stop records.

## Test Procedure

1. Read `Category::ALL`; map each value through `exit_code`; read
   `Category::Inconclusive.trace_exit_code()`; fold no codes with
   `Category::most_severe` (FR-126-AC-1).
2. Build `CatalogCode::new("ill_typed", "x")` and the same code with prefix
   `value`; compare them for equality and hash; compare the first with
   `CatalogCode::new("ill_typed", "y")`; render both with `Display`
   (FR-126-AC-2).
3. Build `InternalFault::new("S6a", "broken")` with the named value
   `("node", "n1")`; read its catalog code, category, stage, invariant and
   named values (FR-126-AC-3).
4. Render the `catalog_code` of an `UndefinedCoded` implementation whose
   reason is `AbsentKey`, with no prefix and with prefix `value`
   (FR-126-AC-4).
5. Read `SuppliedLimitKind::field` for every kind and `SuppliedPhase::field`
   for `Admission` and `Conversion`; build a `SuppliedStop` with a `Limit`
   cause at `AdmissionWorkUnits` and one with a `Cancelled` cause, read
   `SuppliedStop::limit` of each, and convert the second to a
   `SuppliedFailure` (FR-126-AC-5).

## Expected Results

- Step 1: `Category::ALL` is `[Success, Violation, Undefined, Refusal,
  Unsupported, Incomplete, Inconclusive, InternalFailure]` in that order;
  the exit codes are 0, 10, 10, 20, 21, 22, 22 and 30; `trace_exit_code` of
  `Inconclusive` is 0; `most_severe` of no codes is `None`.
- Step 2: the plain and prefixed codes are equal and hash equal; the `y`
  code is unequal; the renderings are `ill_typed/x` and `value/ill_typed/x`.
- Step 3: catalog code `runtime_invariant`/`established-invariant-broken`,
  category `InternalFailure`, stage `S6a`, invariant `broken`, named values
  `[("node", "n1")]`.
- Step 4: `absent-key` without a prefix and `value/absent-key` with it.
- Step 5: `work_units`, `value_occurrences` and `scale_expansion` for the
  three kernel counters; `supplied.admission_work_units`,
  `supplied.conversion_work_units` and `supplied.numeric_magnitude_bits` for
  the three supplied counters, and the same two work-unit names from
  `SuppliedPhase::field`; `limit` is `Some` for the `Limit` stop, naming
  `supplied.admission_work_units`, and `None` for the `Cancelled` stop, which
  converts to `SuppliedFailure::Stop`.
