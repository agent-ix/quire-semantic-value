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

Scope: FR-106-AC-10.

## Test Procedure

Over a type environment with model object types `A` and `B`, each with one required `Integer` attribute `x`:

| # | Input | Expected |
| --- | --- | --- |
| 1 | two objects with one identity triple (universe `u`, type `A`, key `a`) | `ObjectClosureRefusal` at that triple with cause `DuplicateObject` |
| 2 | one object whose type identity names no model object type of the environment | `ObjectClosureRefusal` at that object with cause `UnknownObjectType` |
| 3 | one `A` object with key `k` in universe `u`; `find(u, "k")` | that object's reference |
| 4 | an `A` object and a `B` object, both with key `k` in universe `u`; `find(u, "k")` | none |

## Expected Results

Rows 1 and 2 refuse with exactly the stated cause, row 3 finds the one object, and row 4 finds none.

## Status

Implemented. The tests are unit tests in `src/object_closure.rs`, tagged `#[trace("TC-904", "FR-106-AC-10")]`.
