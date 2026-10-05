---
id: TC-905
title: "Each checking limit builder sets only its own ceiling"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-107
    type: verifies
---
# TC-905: Each checking limit builder sets only its own ceiling

## Description

Verify that each `CheckingLimits` builder replaces one ceiling and keeps the others.

Scope: FR-107-AC-1.

## Test Procedure

1. Build the base `CheckingLimits::new(3).with_input_bytes(7).with_work_budget(9)`.
2. Apply `with_input_bytes(11)`, `with_work_budget(13)` and `with_nodes(5)` each alone to the base and read the three ceilings (nodes, input bytes, work budget).

## Expected Results

- Step 1: (3, 7, 9).
- Step 2: (3, 11, 9), (3, 7, 13) and (5, 7, 9), in that order.
