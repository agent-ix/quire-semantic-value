---
id: TC-907
title: "Type-environment storage denial retains its measured request"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: verifies
---
# TC-907: Type-environment storage denial retains its measured request

## Description

Verify [FR-108](../functional/FR-108-report-registry-allocation-failure.md)
at the production reservation boundary. The fixture must deny a selected
fallible reservation without replacing the admission logic. It must observe
the requested amount and unit at that boundary rather than synthesize an
out-of-memory result from a malformed declaration or an exhausted budget.

## Test Procedure

1. Admit a valid declaration set containing record, tuple, union, and object
   members under generous limits. Deny a selected registry, index, key-join,
   and traversal reservation in separate attempts through an isolated
   allocator seam. For each attempt, compare the public allocation failure's
   amount and unit with the attempted reservation. Remove the denial and
   admit the identical declarations.
2. Trigger a reservation-size arithmetic or representation overflow without
   allocator denial. Assert the capacity/size classification, and separately
   exercise a representable request whose byte conversion is checked.
3. Deny during member sealing after earlier descriptors have been staged.
   Assert no environment or partial descriptor/index/key join escapes. Retry
   without denial and compare the complete registry and member links with a
   fresh successful admission.
4. Run existing `work_units` and `ancestor_steps` N-1/N controls, an invalid
   declaration, cancellation, canonical identity-preimage allocation
   failure, and the released checked-invariant cause controls. Assert each
   original classification and its original bound/counter or cause. Inspect
   authored-declaration storage and worklist growth for infallible paths.

## Expected Results

An allocator denial reports the measured request and unit without a fabricated
limit or declaration cause. Size overflow has its own classification. Denied
admission publishes no partial environment, and a retry succeeds. Every
existing refusal preserves its original type and fields. Each growth path
that can reach the new failure uses the carrier.

## Status

Planned for the IR-713 implementation. QSL's consumer test and criterion are
owned in `agent-ix/quire-spec-language` under FR-082; this QSV test case does
not claim to verify QSL evaluation or charge behavior.
