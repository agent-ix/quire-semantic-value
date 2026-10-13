---
id: TC-914
title: "A model operation clause has its own origin"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-114
    type: verifies
---
# TC-914: A model operation clause has its own origin

## Description

Verify that locations in a model operation's `requires` and `ensures` clauses
are told apart by operation, kind, position and path, and never equal a
location in a state clause.

Scope: FR-114-AC-1.

## Test Procedure

1. Build `ModelOperationClause` origins for operation `place` as `Requires`
   at positions 0 and 1 and as `Ensures` at position 0, and for operation
   `cancel` as `Requires` at position 0.
2. Compare the root locations of each pair, and a location at path `[0]` with
   the root of the same origin.
3. Order the locations of step 1 together.
4. Compare a `ModelOperationClause` origin for operation `Q` at position 2
   with `Origin::StateClause { clause: "Q", index: 2 }`.

## Expected Results

1. All four origins build.
2. Each distinct pair is unequal; the same origin built twice is equal; the
   `[0]` location differs from the root.
3. The order is `cancel` requires 0, `place` requires 0, `place` requires 1,
   `place` ensures 0 (operation first, then kind, then position).
4. The two origins are unequal.
