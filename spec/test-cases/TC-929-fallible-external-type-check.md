---
id: TC-929
title: "External type checking preserves semantics under fallible storage"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-132
    type: verifies
---
# TC-929: External type checking preserves semantics under fallible storage

## Description

Verify [FR-132](../functional/FR-132-check-external-types-with-fallible-storage.md)
through the public `TypeEnvironment::try_check_type` API, with explicit semantic
oracles and actual allocator denial in a separate test binary.

## Test Procedure

1. Build one admitted environment containing object, record, tuple and sealed
   union declarations, including nested IEEE content and a valid recursive
   composite. Check admitted leaves and declaration keys, a sealed union handle,
   references, nested options, sequences and maps. Check unknown composite and
   reference keys, bare/nested populations, and IEEE-bearing set, bag and ordered
   set elements. Assert explicit success or the established `IllTypedCause`,
   including IEEE content reached through record, tuple and union members.
2. In a dedicated test binary with a failing allocator, construct fixtures before
   enabling denial. Deny real allocations reached during the public call: the
   initial pending worklist, nested IEEE traversal worklist growth and visited
   composite storage. Precompute all expected values before denial; restore
   allocation before allocating diagnostics. Assert the typed `Allocation`
   request's amount and unit at each failing boundary and normal process exit.
   Confirm that each intended allocator denial was actually reached.
3. Exercise checked reservation-size overflow at the shared production reservation
   boundary and assert `Capacity`, with the original request and unit, rather
   than `Allocation` or a semantic refusal.
4. After each denial, restore allocation and repeat the same public check against
   the same environment and type. Compare results with the explicit semantic
   oracle from step 1 and confirm declaration lookups and type equality are
   unchanged.
5. Inspect all allocation-bearing calls reachable from the public check, the
   exported typed failure, and in-repository caller migrations. Record downstream
   callers in the implementation handoff. Bind executable tests with both
   `TC-929` and their corresponding `FR-132-AC-N` trace tags.

## Expected Results

- Step 1 preserves admitted results and the exact `TypeMismatch` or
  `OperatorIneligible` causes; enum scope remains unchecked here, sealed handles
  resolve, and recursive IEEE discovery terminates with its established result.
- Step 2 returns `TypeCheckError::Allocation` with each measured native request;
  the actual failed allocator call does not abort the process or become an
  unrelated semantic, limit or identity outcome.
- Step 3 returns only `TypeCheckError::Capacity` with the reservation request.
- Step 4 shows no retained partial state and the same subsequent result.
- Step 5 finds no infallible allocation on the public checking path and no public
  `check_type` compatibility wrapper; callers consume the typed fallible result.
