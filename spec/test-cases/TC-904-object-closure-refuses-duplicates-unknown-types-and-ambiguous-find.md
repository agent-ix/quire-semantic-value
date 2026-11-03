---
id: TC-904
title: "The object closure refuses a duplicate identity triple and a non-model object type, and find refuses an ambiguous key"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: verifies
---
# TC-904: The object closure refuses a duplicate identity triple and a non-model object type, and find refuses an ambiguous key

## Description

Verify the admission rules of `ObjectClosure`.

Scope: FR-106-AC-10, FR-106-AC-11, FR-106-AC-12.

## Test Procedure

Over a type environment with model object types `A` and `B`, each with one required `Integer` attribute `x`:

| # | Input | Expected |
| --- | --- | --- |
| 1 | two objects with one identity triple (universe `u`, type `A`, key `a`) | `ObjectClosureRefusal` at that triple with cause `DuplicateObject` |
| 2 | one object whose type identity names no model object type of the environment | `ObjectClosureRefusal` at that object with cause `UnknownObjectType` |
| 3 | one `A` object with key `k` in universe `u`; `find(u, "k")` | that object's reference |
| 4 | an `A` object and a `B` object, both with key `k` in universe `u`; `find(u, "k")` | none |
| 5 | an attribute holding a union whose payload holds a reference to an admitted object | closes with no refusal |
| 6 | an attribute holding a union whose payload holds a union whose payload references an object that is not admitted | `ObjectClosureRefusal` at the owner with cause `DanglingReference` naming that reference |
| 7 | the row 6 reference listed in the tolerated set | closes with no refusal |
| 8 | an attribute holding an `Option` payload, a collection element or a composite slot that references an object that is not admitted, with and without that reference tolerated | `DanglingReference` naming it; closes when tolerated |
| 9 | an `A` object admitted with `a = 1`, `b = 2` under an environment declaring `A`'s `Integer` attributes `{b, a}`; `attribute` for `a` and `b` under one declaring `{a, b}`, for `c` under one declaring `{a, c}`, and for an object not admitted | `1` for `a`, `2` for `b`; none for `c` and for the unadmitted object |
| 10 | two `A` objects admitted under `{b, a, note?}`, the lower reference without `note`; `iter`, `object(...).get` | `iter` lists the lower reference first, each as `(b, ·), (a, ·), (note, ·)` with the omitted `note` `Absent`; `get("note")` returns the slot, `get("missing")` and `object` of an unadmitted reference none |

## Expected Results

Rows 1 and 2 refuse with exactly the stated cause, row 3 finds the one object, row 4 finds none, rows 5 and 7 close, and rows 6 and 8 refuse with the stated cause unless tolerated, and rows 9 and 10 read each slot by its attribute name.

## Status

Implemented. The tests are unit tests in `src/object_closure.rs`, tagged `#[trace("TC-904", ...)]`; rows 5 to 8 bind FR-106-AC-11 and rows 9 and 10 bind FR-106-AC-12.
