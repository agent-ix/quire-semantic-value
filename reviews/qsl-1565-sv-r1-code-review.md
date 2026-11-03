---
id: SR-20261102072319
title: "Code review of quire-semantic-value PR #87: named ObjectClosure slots (rust-review lane folded in)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@b3616d478; PR #87 diff origin/main...HEAD: src/object_closure.rs; ticket QSL-1565"
review_set: subset
date: '2026-11-03'
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: references
---

# Code review of quire-semantic-value PR #87

## Summary

Ticket: QSL-1565. PR quire-semantic-value#87, head b3616d478383a221c6a3550f0fcf8ee0fc89f877, base main.

The PR records each admitted object type's effective attribute names, in slot order, in `names: BTreeMap<EffectiveId, Box<[String]>>`. `attribute(types, reference, field)` now resolves `field` under `types` to an attribute name and reads the slot of that name. The PR also adds `object()`, `iter()` and `ObjectAttributes::{get, iter}`, and `PartialEq` now compares `names`.

Correctness (examined, clean):

- **Layout per type is uniform.** `ObjectClosure::new` is the only constructor besides `Default`, which is empty. It takes a single `types`, and every object of one type gets `fill_slots(types, declared, ..)` with `declared = types.attributes(type)`. `fill_slots` emits slots in `declared` order. So one closure has one layout per `EffectiveId`, and keying `names` by type is sound. `names` is filled only after the duplicate and `fill_slots` checks, and any refusal discards the whole closure.
- **Duplicate names cannot occur.** The effective set refuses two exposed attributes with one name (`DeclarationCause::DuplicateMember`). A redefined field is hidden, so `get(name)` is unambiguous.
- **Inherited and redefined attributes.** These resolve through the reader's `stands_for` to the exposed attribute's own name. That is the name the admitting environment records when the two environments agree on the redefinition.
- **Reordered, missing and extra attributes.** Reordered layouts read by name. An attribute the reader declares but the admitting environment does not reads `None`. Extra admitting attributes are ignored. An absent optional slot is returned as `Absent`.

Rust lane:

- The crate stays `no_std` + `alloc` (`alloc::string::String`).
- `make lint`, which runs clippy on the host and on thumbv7em, exits 0.
- There is no `unsafe`, no indexing and no panic path. The unreachable missing-type case in `attributes_of` degrades to an empty name list.
- `ObjectAttributes` is a `Copy` borrowed view, and the `impl Iterator + Clone` returns are fine.
- The API is minimal and each item is used by the stated QSL follow-up, which walks the object environment at entry.

Gates and mutations:

- `cargo test --lib object_closure`: 8 passed.
- Mutation: restoring the old positional `position(..)` lookup fails `attribute_reads_by_name_across_reordered_environments` (left `Some(Integer(2))`, right `Some(Integer(1))`).
- Mutation: dropping the last name from `ObjectAttributes::iter` fails `objects_iterate_as_named_slots` and `native_attributes_admit_and_wrong_kinds_refuse`.
- Mutation: skipping the first recorded name in `new` fails 3 tests.

QSL consumers: I ran `git grep` over `agent-ix/quire-spec-language` origin/main at b992dc74d. `ObjectClosure` is built in qsl-eval tests and production. `attribute` is called in `qsl-eval/src/value/expression/evaluate.rs:3612` and `value_invariants.rs:217`. `ObjectEnvironment` derives no `PartialEq`, and no consumer compares closures, so the `PartialEq` change affects no QSL code. The derived `Debug` now prints `names`, and I found no snapshot of it. The finding below is the one consumer effect.

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| ObjectClosure::new names recording | src/object_closure.rs | examined |
| ObjectClosure::attribute | src/object_closure.rs | examined |
| ObjectClosure::object / iter / attributes_of | src/object_closure.rs | examined |
| ObjectAttributes::get / iter | src/object_closure.rs | examined |
| impl PartialEq for ObjectClosure | src/object_closure.rs | examined |
| tests rows 9 and 10 | src/object_closure.rs | examined |
| fill_slots, TypeEnvironment::attributes, DuplicateMember | src/declaration.rs | context_only |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `attribute` now reads by name, which changes behaviour for QSL. The PR says "Additive; no signature changes". But the QSL origin/main test `a_caller_attribute_layout_that_cannot_hold_the_package_attributes_is_refused` (qsl-eval/tests/it/equality_matrix.rs) asserts that the swapped `{b: Boolean, a: Integer}` caller layout is refused for both reads. After this PR it completes, so the test fails as soon as QSL's lock picks up SV main. `evaluate.rs:3600`'s "slots are stored by position" layout check also goes stale. The QSL adaptation has to land together with this PR. | src/object_closure.rs:193-209 |

## Verdict

The code is correct, and the tests kill the positional-lookup and dropped-name mutants. Merge this PR together with the QSL follow-up that updates the TC-136 layout test and the positional comment and check in `evaluate.rs` (FND-001). Merged alone, it leaves QSL red on its next `cargo update` of quire-semantic-value.

## Dispositions

- FND-001: deferred: it belongs to the QSL follow-up PR for QSL-1565. That PR bumps QSL's Cargo.lock to this commit, rewrites `a_caller_attribute_layout_that_cannot_hold_the_package_attributes_is_refused` (a by-name read of the swapped layout), and removes the positional layout check and comment in `evaluate.rs`. QSL stays green until then because its lock pins the earlier QSV commit.
