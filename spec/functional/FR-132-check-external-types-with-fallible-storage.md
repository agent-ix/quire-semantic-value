---
id: FR-132
title: "Check external types with fallible storage"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: references
---
# FR-132: Check external types with fallible storage

## Description

When a caller checks an external parameter or result `ValueType` against an
admitted `TypeEnvironment`, QSV SHALL expose `try_check_type` with fallible
storage throughout its traversal, including nested IEEE-content discovery.
The public result SHALL distinguish semantic refusal from allocation denial
and reservation capacity overflow without retaining partial checking state.

## Inputs

- An immutable admitted `TypeEnvironment` and a borrowed `ValueType`.

## Outputs

- `Result<(), TypeCheckError>`, where `TypeCheckError::Refused(IllTyped)`
  retains the established semantic cause, `Allocation(StorageRequest)` names
  an allocator denial, and `Capacity(StorageRequest)` names reservation-size
  arithmetic or representation overflow.

## Behavior

1. QSV SHALL replace the public `check_type` entry point with
   `try_check_type` and switch its in-repository callers to the fallible
   result. QSV SHALL expose no infallible compatibility wrapper for this
   external-type check.
2. When the public check requires storage on its reachable path, QSV SHALL
   use `try_reserve` or `try_reserve_exact` with checked reservation sizing
   before allocating. This includes the root worklist, later worklist growth, and visited
   composite storage used by nested IEEE discovery. Error construction SHALL
   require no further allocation.
3. If an allocator denies a representable reservation, then QSV SHALL return
   `TypeCheckError::Allocation` retaining the actual additional-element or
   byte request and its native unit. If reservation sizing overflows, then
   QSV SHALL return `TypeCheckError::Capacity` with that request instead.
   Neither failure SHALL become an `IllTyped`, declaration refusal, limit,
   cancellation, or canonical identity refusal.
4. When reservations succeed, QSV SHALL preserve the external-type check's
   existing traversal and semantic results: admitted composite keys and
   sealed union handles resolve; unknown composites and non-object reference
   targets yield `TypeMismatch`; populations and non-sequence collections
   with IEEE-bearing elements yield `OperatorIneligible`; option payloads,
   collection elements and map values retain their existing traversal.
   Enum scope remains the caller's responsibility. Nested IEEE discovery
   SHALL preserve its existing record, tuple, union-member and cycle behavior.
5. When the check returns success, semantic refusal, or storage failure, QSV SHALL
   leave the environment and supplied type unchanged and retain no traversal state.
   A retry after removing a denial SHALL produce the same result as a fresh
   successful-reservation attempt.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-132-AC-1 | Public `try_check_type` returns success for admitted leaves, references, composites, sealed union handles, nested options, sequences and maps; unknown composites and non-object references return `Refused(IllTyped { cause: TypeMismatch })`, and populations and IEEE-bearing set/bag/ordered-set elements return `Refused(IllTyped { cause: OperatorIneligible })`, preserving the previous external-type traversal's outcomes. | Test |
| FR-132-AC-2 | A dedicated test binary denies a real allocator request while public `try_check_type` runs, after fixture construction. The call returns `Allocation` with the failing reservation's measured request and unit, and the process completes without aborting. Real denials cover the root worklist and nested IEEE discovery's worklist/visited storage; policy-only injection is not a substitute. | Test |
| FR-132-AC-3 | Checked reservation-size overflow returns `Capacity` with its request and native unit, distinct from `Allocation` and `Refused`; controlled tests exercise the same reservation boundary used by the public check. | Test |
| FR-132-AC-4 | After each controlled storage denial, retrying the same type against the same environment after restoring allocation yields its original success or semantic refusal; declaration lookups and the supplied type are unchanged and no partial checking result is exposed. | Test |
| FR-132-AC-5 | Inspection finds fallible checked reservations for every allocation reachable from public `try_check_type`, including nested IEEE traversal and error construction; in-repository callers consume its typed result and no public `check_type` compatibility shim remains. | Inspection |

## Dependencies

- [FR-108](./FR-108-report-registry-allocation-failure.md) owns the existing
  `StorageRequest` amount/unit vocabulary and environment-admission reservation
  machinery. This external-type check has its own narrow public failure carrier;
  it does not fabricate an invalid declaration or environment budget.
- [FR-115](./FR-115-stage-and-seal-union-declarations.md) owns sealed union
  handle admission. This check preserves the existing handle lookup.
- Downstream callers own projection into their checker/runtime refusal types;
  this requirement does not change equality checking or other public helpers.
