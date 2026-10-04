---
id: FR-106
title: "The object closure refuses a duplicate identity triple and a non-model object type"
type: FR
relationships: []
---
# FR-106: The object closure refuses a duplicate identity triple and a non-model object type

## Description

`quire-semantic-value` SHALL provide an `ObjectClosure` that admits objects over the kernel `ObjectReference`, checked against a `TypeEnvironment`. It SHALL refuse a second object with an admitted identity triple, refuse an object whose type is not a model object type of the environment, and refuse an ambiguous `find`. Admitting a snapshot and an invocation, which builds the closure, keeps the id `FR-106` in `agent-ix/quire-spec-language`.

## Use case

A caller admits a snapshot of objects. Two objects with one identity triple, or an object of an unknown type, stop the admission with a cause that names the fault, and a lookup by universe and key never picks one of two objects of different types.

## Behavior

1. **Admission.** The closure refuses a duplicate identity triple (universe, object type, object identity) with `DuplicateObject`, and an object whose type is not a model object type with `UnknownObjectType`.
2. **Find.** `find` by universe and object key returns the one admitted object that matches, and returns none when two admitted objects share that universe and key under different object types.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-106-AC-10 | The object closure refuses a second object whose identity triple (universe, object type, object identity) is already admitted with `DuplicateObject`, naming that triple, and refuses an object whose type is not a model object type of the type environment with `UnknownObjectType`. `find` by universe and object key returns the one admitted object that matches, and returns none when two admitted objects share that universe and key under different object types. | Test (TC-904) |

## Status

Implemented.

## Dependencies

- None. The caller's behaviour under the same id lives in `agent-ix/quire-spec-language`.
