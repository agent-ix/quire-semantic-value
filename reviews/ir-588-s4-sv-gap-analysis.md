---
id: SR-13642
title: "Gap analysis of quire-semantic-value PR #62: SuppliedRun, render_identity and the state-model prefix move into SV (IR-588 S4)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@e331dd0af0723e3c0b036a02702bed9f3b913ac4; PR #62 diff origin/main...HEAD; QSV FR-126 (spec/functional/FR-126-own-the-evaluation-result-vocabulary.md); QSL origin/main e1fc9f0cf FR-403 S4 disposition rows for SV"
review_set: subset
date: '2026-10-23'
relationships:
- target: ix://agent-ix/quire-semantic-value/FR-126
  type: reviews
---
# Gap analysis of quire-semantic-value PR #62

## Summary

Ticket: IR-588 (S4). Plan completion: not assessed. `quire matrix` (quire
0.36.2, engine 0.50.2) on the PR head lists no binding for the three tests
this PR adds or moves. They carry QSL ids (`FR-277-AC-5`, `FR-403-AC-11`),
and this repo's spec does not define them.

- FR-403's SV rows: `render_identity` and `STATE_MODEL_CATALOG_PREFIX`
  resolve at `quire_semantic_value::catalog`. `SuppliedRun`,
  `SuppliedLimits`, `DEFAULT_SUPPLIED_WORK_UNITS` and
  `DEFAULT_NUMERIC_MAGNITUDE_BITS` resolve at
  `quire_semantic_value::supplied_stop`. This is the SV half of FR-403-AC-2.
  The other half, where the old paths fail to compile, is in QSL PR #1382.
- `SuppliedRun::admit`, `charge_conversion`, `check_cancelled` and
  `with_reservation_permit` have no test in this repo. Their behaviour tests
  stay in QSL (`qsl-eval/tests/it/supplied_phases.rs` and `qsl-replay`),
  which reach them through SV. FR-403 accepts that split.

## Verdict

Approve with findings. FND-001 is the one gap: the new public API has no
owning QSV requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-126 is this repo's owning requirement for what QSL places in SV. Its Outputs list the `catalog` vocabulary and the `supplied_stop` records only, and PR #61 (IR-583) added it. This PR adds public `catalog::{render_identity, STATE_MODEL_CATALOG_PREFIX}`, `supplied_stop::{SuppliedRun, SuppliedLimits, DEFAULT_SUPPLIED_WORK_UNITS, DEFAULT_NUMERIC_MAGNITUDE_BITS}` and the `test-support` feature. It does not update FR-126 and adds no AC, so the code has no owning QSV requirement and the tests tag QSL ids instead (SR-13641 FND-001, FND-003). Fix: add these items to FR-126 Outputs, under FR-403 as well as FR-401. Add one AC for `render_identity`: UTF-8 bytes render as themselves, and non-UTF-8 bytes render as `identity bytes 0x<hex> (not UTF-8)`, with distinct bytes rendering differently. Add one AC for the `SuppliedLimits` defaults and builders. Retag the tests to those ACs. | spec/functional/FR-126-own-the-evaluation-result-vocabulary.md:19-27; src/catalog.rs:18-41; src/supplied_stop.rs:392-575 |

## Dispositions

Round 1, reviewed at 22f00bc6fcb9aeb4e6aab3437a5aa1ec86f7eac8.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 22f00bc: FR-126 Outputs and Behavior name the moved items, FR-126-AC-6 to AC-9 and TC-925 were added, and `quire matrix` lists AC-6 to AC-9 as tagged |
