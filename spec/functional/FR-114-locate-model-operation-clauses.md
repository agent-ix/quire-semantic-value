---
id: FR-114
title: "Locate a model operation's requires and ensures clauses by their own origin"
type: FR
relationships: []
---
# FR-114: Locate a model operation's requires and ensures clauses by their own origin

## Description

`quire-semantic-value` SHALL give `Origin` a variant for one `requires` or
`ensures` clause of a model operation. It names the operation, the clause kind
and the clause's position in the operation's list of that kind. A checking
refusal, an evaluation outcome or a loss record located in such a clause
carries that origin and no other.

## Use case

A caller checks the Boolean clauses a domain package authors on its own
operations. A refusal in the second `requires` clause of operation `place`
names that operation, that kind and that position. It does not borrow the
origin of a unit state clause, whose index counts a different list.

## Behavior

1. `Origin::ModelOperationClause { operation, kind, position }` holds the
   operation's identifying text, an `OperationClauseKind` (`Requires` or
   `Ensures`) and the zero-based position in that kind's list.
2. Two locations in the same clause, with equal paths, are equal. Locations
   that differ in operation, kind or position are unequal, and order by
   operation, then kind (`Requires` before `Ensures`), then position.
3. The variant is distinct from `Origin::StateClause`: equal names and indices
   in the two variants are unequal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-114-AC-1 | Locations built on `ModelOperationClause` origins compare equal exactly when operation, kind, position and path are equal; they order by operation, then kind with `Requires` before `Ensures`, then position, then path; and a `ModelOperationClause` origin never equals a `StateClause` origin whose name or index matches its operation or position. | Test (TC-914) |

## Status

Implemented.

## Dependencies

- None. The caller's behaviour under this origin lives in `agent-ix/quire-spec-language`.
