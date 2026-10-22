---
id: SR-12345
title: "Gap analysis of quire-semantic-value PR #61: catalog vocabulary and supplied-value stop records (IR-583 S0)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@1adf9ce983c155a91983bd99538c9fbf1071266e; PR #61 diff origin/main...HEAD; FR-126-AC-1..AC-5 (spec/functional/FR-126-own-the-evaluation-result-vocabulary.md); QSL origin/main 48108c9c1 FR-401 S0 placement rows for SV; quick spec-review of the new FR-126 folded in"
review_set: subset
date: '2026-10-22'
relationships:
- target: ix://agent-ix/quire-semantic-value/FR-126
  type: reviews
---
# Gap analysis of quire-semantic-value PR #61

## Summary

Ticket: IR-583 (S0). Plan completion: not assessed. `quoin matrix` lists
FR-126-AC-1 to AC-5 as `tagged`, each by one `#[trace]` test in
`tests/evaluation_vocabulary.rs`. `quire validate` on FR-126 passes, with one
EARS warning (FND-002). A quick spec-review of the new FR-126 is folded in
here.

- FR-126-AC-1 to AC-4: each test asserts the stated values. AC-1 pins the
  exit-code sequence and `trace_exit_code(Inconclusive) == 0`, and also
  checks `most_severe([]) == None`.
- FR-126-AC-5: the `SuppliedLimitKind` kernel-counter fields, the
  `Stop` wrap and `limit()`'s `Some`/`None` split are asserted. The phase
  field names are asserted only by equality with each other (FND-001).
- FR-401's SV rows: every catalog type and every listed stop record resolves
  at `quire_semantic_value::{catalog, supplied_stop}`, which is the SV half
  of FR-401-AC-3. The QSL half, where the old paths fail to compile, is the
  QSL repoint. It is not in this PR (SR-12347 FND-004).
- All production code in the PR has an owning requirement (FR-126 under
  FR-401), apart from the two constants SR-12344 FND-001 names.

## Verdict

Approve with findings. FND-001 is a real gap in AC-5, but it is a one-line
test addition. FND-002 is a spec nit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-126-AC-5 test checks `SuppliedLimitKind::AdmissionWorkUnits.field() == SuppliedPhase::Admission.field()` and the same for Conversion. It checks the literal only for the admission limit, through `SuppliedLimit::field`. If `CONVERSION_WORK_UNITS` held the admission string, or both phase methods returned it, the test would still pass. `NUMERIC_MAGNITUDE_BITS` is never asserted. Fix: assert the three literals `supplied.admission_work_units`, `supplied.conversion_work_units` and `supplied.numeric_magnitude_bits` against `SuppliedPhase::field` and `SuppliedLimitKind::field`. | tests/evaluation_vocabulary.rs:92-99 |
| FND-002 | low | quire reports `[ears:non-singular]` on FR-126's Behavior, line 32: one statement carries more than one SHALL. Fix: split the bullet into one SHALL per statement. | spec/functional/FR-126-own-the-evaluation-result-vocabulary.md:32 |
