---
id: TC-911
title: "Native String scalar and composite admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-112
    type: verifies
---
# TC-911: Native String scalar and composite admission

## Description

Verify [FR-112](../functional/FR-112-admit-native-string-leaves.md) at QSV's
type-environment boundary for scalar equality and a declared record member.

## Test Procedure

1. Construct an empty `TypeEnvironment`, a String value with composed `é`, a
   String value with decomposed `e` plus combining acute, an equal copy of the
   decomposed value, and a Text value admitted under an NFC Text type. Check
   `check_type`, `contains_ieee`, all String/Text admission pairs, equality
   conversion for String operands, and equality outcomes for the three String
   values.
2. Declare a record with one required String field. Construct the record with
   a matching `Value::String`, inspect its field slot, and ask the type
   environment to admit the record value under the declared composite type.

## Expected Results

1. String is accepted as a type and has no IEEE value. Only String-to-String
   admission succeeds; Text and crossed kinds refuse. Equality conversion is
   available, composed and decomposed scalar sequences compare unequal, and
   equal copies compare equal.
2. The record declaration and constructed value are admitted. Its one field
   slot contains the original String payload, and the composite value is
   admitted under the declared record type.
