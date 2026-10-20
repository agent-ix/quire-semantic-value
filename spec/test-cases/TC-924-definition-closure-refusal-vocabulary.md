---
id: TC-924
title: "The definition-closure refusal vocabulary is closed and matches QSpec"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: verifies
---
# TC-924: The definition-closure refusal vocabulary is closed and matches QSpec

## Description

Verify [FR-125](../functional/FR-125-definition-closure-refusal-vocabulary.md)
through the public `quire_semantic_value::definition` API: the spellings, the
pairing with `invalid_package`, the closed cause set and the selection codes.

## Test Procedure

1. Compare each `PackageCause::as_str` and `PackageRefusalCode::as_str` with the
   spellings in QSpec FR-272's `invalid_package` row (FR-125-AC-1).
2. Build `PackageRefusal::invalid_package` for each cause and read `code` and
   `cause`; check that `PackageRefusalCode` has the single variant `InvalidPackage` (FR-125-AC-2).
3. Compile, in an integration test under `tests/` (a crate other than the
   defining one), an exhaustive `match` over `PackageCause` with no wildcard arm, and
   compare the three spellings against `byte-digest-mismatch`,
   `digest-domain-mismatch` and `content-mismatch` (FR-125-AC-3).
4. Compare `SelectionRefusalCode::ALL` with the eight codes in order, round-trip
   `from_code`, and call it with `invalid_package`, `""` and an upper-cased
   spelling (FR-125-AC-4).

## Expected Results

- Step 1: `missing-member`, `conflicting-definition`, `incompatible-definition`
  and `invalid_package`, byte-equal to FR-272.
- Step 2: `InvalidPackage` with the given cause for each of the three; `PackageRefusalCode` has no other variant.
- Step 3: the match compiles; no variant spells a digest-mismatch cause.
- Step 4: the eight codes in check order; every code round-trips; the three
  other strings give `None`.

## Status

IMPLEMENTED (IR-882). The four traced tests in `tests/definition_closure_refusal.rs` pass,
including the exhaustive no-wildcard `PackageCause` match, and `make ci` passes.
