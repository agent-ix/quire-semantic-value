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

1. Build `CheckingLimits::default()` with `with_input_bytes(7)` and `with_work_budget(9)` and read the three ceilings.
2. Build `CheckingLimits::new(3)` with `with_input_bytes(7)`, `with_work_budget(9)` and `with_nodes(5)` and read the three ceilings.

## Expected Results

- Step 1: 7 input bytes, 9 work budget, the default node ceiling.
- Step 2: 5 nodes, 7 input bytes, 9 work budget.
