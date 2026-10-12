---
id: FR-112
title: "Admit native String values and preserve them in composite values"
type: FR
relationships:
  - target: ix://agent-ix/quire-exact/FR-370
    type: references
---
# FR-112: Admit native String values and preserve them in composite values

## Description

`quire-semantic-value` SHALL treat the kernel's native String type as a
terminal semantic-value type distinct from Text. QSV SHALL admit matching
String values directly and through declared composite members while preserving
the kernel String value and its equality semantics. Text normalization and Text
admission rules do not apply to native String values.

## Behavior

1. QSV SHALL accept `ValueType::String` as a terminal declared type, with no
   IEEE value and no composite-recursion edge. Type admission SHALL accept
   `Value::String` only under `ValueType::String` and SHALL refuse a Text value
   under String and a String value under Text.
2. When QSV compares two String operands for equality, QSV SHALL return the
   kernel-defined exact scalar-sequence result.
3. QSV's equality check SHALL NOT normalize a String value as Text before
   equality.
4. QSV SHALL admit a record declaration with a required String member, construct
   that record from a matching String value, preserve that value in the record
   slot, and admit the resulting record under its declared composite type.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-112-AC-1 | In an empty type environment, `check_type` accepts String and `contains_ieee` is false. `admits` accepts `Value::String` only for `ValueType::String`, rejecting Text and crossed String/Text pairs. Equality conversion accepts String operands, and equality distinguishes canonically composed from decomposed scalar sequences while accepting equal copies. | Test |
| FR-112-AC-2 | Given a record declaration with one required String member, the type environment admits the declaration, record construction accepts a matching String value, the resulting slot retains the same String payload, and `admits` accepts the record under its declared composite type. | Test |

## Dependencies

- [Quire Exact FR-370](ix://agent-ix/quire-exact/FR-370) owns the kernel's native
  semantic-value types and values. QSV preserves the native String value and
  delegates its scalar equality semantics to the kernel.
