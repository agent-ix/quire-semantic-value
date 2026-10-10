---
id: TC-910
title: "Native UUID and Timestamp declaration admission and object closure"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-111
    type: verifies
---
# TC-910: Native UUID and Timestamp declaration admission and object closure

## Description

Verify [FR-111](../functional/FR-111-admit-native-uuid-and-timestamp-leaves.md)
at QSV's type-environment and object-closure boundaries.

Scope: FR-111-AC-1 and FR-111-AC-2.

## Test Procedure

1. Construct an empty `TypeEnvironment`, one canonical native UUID value and
   one signed native Timestamp value using the exact kernel's constructors.
   Check each native type with `check_type` and `contains_ieee`. Check both
   matching `admits` pairs, both crossed pairs, and an Integer value against
   each native type. Check equality conversion from a bounded Integer type
   to each native type.
2. Declare one model object type with required `Uuid` and `Timestamp`
   attributes. Admit one object whose fields hold the matching native values,
   with no other object supplied to the closure. Read each admitted attribute.
3. In four separate admissions, replace the UUID attribute with a Timestamp
   or Integer value, then replace the Timestamp attribute with a UUID or
   Integer value. Inspect the refusal's object, field, and typed cause.

## Expected Results

- Step 1: Both native declarations are accepted and contain no IEEE value.
  Only the matching native admission pairs succeed; crossed and Integer
  values refuse. Bounded Integer converts to neither native target.
- Step 2: The closure succeeds with no dangling reference, and both
  attribute lookups retain their supplied native payload and kind.
- Step 3: Each wrong-kind admission refuses its owning object with
  `Attribute(TypeMismatch)` naming the replaced field; no closure returns.

## Status

Implemented by the focused tests in `src/declaration.rs` and
`src/object_closure.rs`, both tagged `TC-910` and their respective
`FR-111` acceptance criterion.
