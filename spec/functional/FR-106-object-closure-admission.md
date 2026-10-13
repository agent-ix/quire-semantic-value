---
id: FR-106
title: "The object closure refuses a duplicate identity triple, a non-model object type and a dangling reference"
type: FR
relationships: []
---
# FR-106: The object closure refuses a duplicate identity triple, a non-model object type and a dangling reference

## Description

`quire-semantic-value` SHALL provide an `ObjectClosure` that admits objects over the kernel `ObjectReference`, checked against a `TypeEnvironment`. It SHALL refuse a second object with an admitted identity triple, refuse an object whose type is not a model object type of the environment, refuse an ambiguous `find`, and refuse a present reference, at any depth through Option payloads, composite slots, collection elements and union payloads, that names no closure object and is not a tolerated target. Admitting a snapshot and an invocation, which builds the closure, keeps the id `FR-106` in `agent-ix/quire-spec-language`.

## Use case

A caller admits a snapshot of objects. Two objects with one identity triple, or an object of an unknown type, stop the admission with a cause that names the fault, and a lookup by universe and key never picks one of two objects of different types.

## Behavior

1. **Admission.** The closure refuses a duplicate identity triple (universe, object type, object identity) with `DuplicateObject`, and an object whose type is not a model object type with `UnknownObjectType`.
2. **Find.** `find` by universe and object key returns the one admitted object that matches, and returns none when two admitted objects share that universe and key under different object types.
3. **Dangling references.** Every present reference reached through `Option` payloads, composite slots, collection elements and union payloads, at any depth, SHALL name an object of the closure or a tolerated target; otherwise admission refuses with `DanglingReference`, naming that reference.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-106-AC-10 | The object closure refuses a second object whose identity triple (universe, object type, object identity) is already admitted with `DuplicateObject`, naming that triple, and refuses an object whose type is not a model object type of the type environment with `UnknownObjectType`. `find` by universe and object key returns the one admitted object that matches, and returns none when two admitted objects share that universe and key under different object types. | Test (TC-904) |
| FR-106-AC-11 | A present reference inside an `Option` payload, a composite slot, a collection element or a union payload, nested to any depth including a union inside a union, that names no closure object and is not in the tolerated set is refused with `DanglingReference` naming that reference. A reference in the tolerated set is accepted, and a reference to a closure object is accepted. | Test (TC-904) |

## Status

Implemented.

## Dependencies

- None. The caller's behaviour under the same id lives in `agent-ix/quire-spec-language`.
