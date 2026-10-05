---
id: SR-007
title: "Gap analysis of quire-semantic-value PR #3: FR-107 and its test oracle"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@c6d46da4df23525a8a0ffca582f56e12380e68a2; FR-107, FR-107-AC-1, TC-905, src/checking.rs builders_set_only_their_own_ceiling"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-107
    type: reviews
---
# Gap analysis of quire-semantic-value PR #3

## Summary

Ticket: QSL-489. Task: `with_nodes`, `with_input_bytes`, `with_work_units`
builders. Two existed; `with_nodes` is added; the third is named
`with_work_budget` after its field (see SR-006). Code traces to FR-107; the
test carries `#[trace("TC-905", "FR-107-AC-1")]`, matching TC-905.

## Verdict

Changes requested on the test oracle (one medium). The code is right; the
test does not prove AC-1 for two of the three builders.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-107-AC-1 says "starting from any limits value, each builder ... leaves the other two equal to their previous values", but only `with_nodes` is applied last over non-default values of both other fields. `with_input_bytes` is never applied last, and `with_work_budget` never runs over a non-default `nodes` that survives to the assert. Traced: a `with_input_bytes` written as `Self { input_bytes, ..Self::default() }` passes both steps (step 1 starts from defaults; in step 2 `with_work_budget` and `with_nodes` overwrite whatever it reset). Fix: start from a value with all three non-default and apply each builder alone, asserting all three fields, e.g. `let base = CheckingLimits::new(3).with_input_bytes(7).with_work_budget(9);` then `base.with_input_bytes(11)` gives (3, 11, 9), `base.with_work_budget(13)` gives (3, 7, 13), `base.with_nodes(5)` gives (5, 7, 9). Update TC-905's procedure to match. | src/checking.rs:145-163; spec/test-cases/TC-905-each-checking-limit-builder-sets-only-its-own-ceiling.md:17-25 |
