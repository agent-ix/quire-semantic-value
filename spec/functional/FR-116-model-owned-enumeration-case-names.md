---
id: FR-116
title: "Model-owned enumerations accept any non-empty case name"
type: FR
relationships: []
---
# FR-116: Model-owned enumerations accept any non-empty case name

## Description

`EnumDeclaration::model_owned` SHALL build the runtime enumeration of a model-owned variant type, whose case names are any non-empty strings (QSpec FR-141-AC-8). `EnumDeclaration::new` keeps the identifier-only rule for source-declared enumerations. A variant type with a case such as `Gold Plus`, `2x` or a non-ASCII name has a runtime enumeration.

## Use case

A checker holds a model variant type whose cases are `2x` and `Gold Plus` in ascending UTF-8 byte order. It builds the declaration with `model_owned`, builds each `EnumValue` through `member`, and compares the values with the same schedule as any other unordered enumeration.

## Behavior

1. `model_owned` takes the declaration key and the case names, already in ascending UTF-8 byte order, and returns an unordered declaration holding the names verbatim.
2. An empty list, an empty case name or a repeated case name is refused with `NonCanonicalPreimage`; cases not in ascending byte order are refused with `UnsortedUnorderedMembers`. A list that is both malformed (empty, empty name or repeated name) and unsorted is refused with `NonCanonicalPreimage`, which is checked first.
3. Names are never normalized: names that differ in any byte (including leading or trailing spaces and composed versus decomposed letters) are distinct cases.
4. `EnumDeclaration::new` and `is_member_list` keep requiring identifier cases.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-116-AC-1 | `model_owned` accepts the cases `  padded `, `2x`, `Gold Plus`, `Zürich` and `日本` in byte order and returns them unchanged, unordered, under the given key. | Test (TC-916) |
| FR-116-AC-2 | `member` on such a declaration builds an `EnumValue` whose case equals the name and whose position is the name's index; a name outside the declaration is `UndeclaredCase`. | Test (TC-916) |
| FR-116-AC-3 | `model_owned` is not case-insensitive and does not trim: `gold plus` is not a case of a declaration holding `Gold Plus`. | Test (TC-916) |
| FR-116-AC-4 | `model_owned` refuses an empty list, an empty name and a repeated name with `NonCanonicalPreimage`, and otherwise unsorted cases with `UnsortedUnorderedMembers`; a list both malformed and unsorted (`b`, `a`, `a`) gives `NonCanonicalPreimage`; byte-distinct names (`a` and `a `, composed and decomposed `é`) are all accepted. | Test (TC-916) |
| FR-116-AC-5 | Two declarations built from the same key and cases are equal; their values for one case are equal and share a `variant`; `EnumMemberIndex` resolves each value by variant; `compare_enum` equality is true for equal values and an ordering operator is `UnorderedEnumOrdering`. | Test (TC-916) |
| FR-116-AC-6 | `EnumDeclaration::new` and `is_member_list` still refuse `Gold Plus`, `2x` and the empty name. | Test (TC-916) |

## Status

Implemented.

## Dependencies

- None.
