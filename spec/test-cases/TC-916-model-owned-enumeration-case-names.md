---
id: TC-916
title: "Model-owned enumerations accept any non-empty case name"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-116
    type: verifies
---
# TC-916: Model-owned enumerations accept any non-empty case name

## Description

Verify that `EnumDeclaration::model_owned` accepts any non-empty case name verbatim, refuses empty, repeated and unsorted cases, and leaves the identifier-only path unchanged.

Scope: FR-116-AC-1, FR-116-AC-2, FR-116-AC-3, FR-116-AC-4, FR-116-AC-5, FR-116-AC-6.

## Test Procedure

1. Build `model_owned` over `  padded `, `2x`, `Gold Plus`, `Zürich`, `日本` in byte order. Read the members, the ordered flag and the key. Build an `EnumValue` for each case and read its case and position. Ask for `gold plus`.
2. Build `model_owned` over an empty list, `[""]`, `["", "a"]`, `["Gold Plus", "Gold Plus"]`, the malformed and unsorted `["b", "a", "a"]`, and the unsorted `["b b", "a a"]`. Build one over `a`, `a `, `e` plus a combining acute, `é`, in byte order.
3. Build two declarations from one key and the cases `2x`, `Gold Plus`, `Silver`. Compare the declarations, the `Gold Plus` values, their variants, the `EnumMemberIndex` lookups, and `compare_enum` for `=` and `<`.
4. Call `EnumDeclaration::new` (unordered) over `Gold Plus`, `2x` and the empty name, and `is_member_list` over `Gold Plus`.

## Expected Results

- Step 1: the five names unchanged, unordered, the given key; each value has its name as case and its index as position; `gold plus` is `UndeclaredCase`.
- Step 2: the first five are `NonCanonicalPreimage`, the unsorted list is `UnsortedUnorderedMembers`, the byte-distinct list is accepted with four members.
- Step 3: declarations and values equal, same variant, index resolves each, `=` is true, `<` is `UnorderedEnumOrdering`.
- Step 4: `NonCanonicalPreimage` each; `is_member_list` is false.
