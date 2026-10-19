---
id: SR-13116
title: "Integrity review of quire-semantic-value FR-123 and TC-922"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@e992478ef9f8e8400e0aa2c1664edcd5f6df7828; spec/functional/FR-123-step-simulation-record-and-obligation.md, spec/test-cases/TC-922-step-simulation-record-and-obligation.md; read against agent-ix/quire-spec-language FR-146, FR-136, ADR-020 RS-2..RS-4 and RM-5, ADR-018 LA-5, ADR-013 O-09"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-123
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-922
    type: references
---
# Integrity review of quire-semantic-value FR-123 and TC-922

## Summary

Ticket: QSL-899. FR-123 places FR-146's output types in QSV. Its API inventory matches
FR-146 Outputs field for field, its variant comments match ADR-020 RS-2 (Initial), RS-3
(`stutter`) and RS-4 (abstract operation), and the placement is ADR-018 LA-5's, which already
lists `StepObligation` as an SV type. TC-922 has one `verifies` edge to FR-123, procedure
steps that map one to one to AC-1..AC-6, and expected results each AC can fail. AC-6 is
Inspection and Analysis with no trace tag, as FR-121-AC-6 is. `quire validate` passes both
files (EARS warning in SR-13117).

The one open question is what a `StepRowRef` names. FR-146 uses the type but defines it
nowhere in QSL's spec; FR-123 defines it as the zero-based position of the row in the
refinement's step rows in declaration order. RM-5 (with ADR-027 PR-1) gives exactly one row
per concrete step class, so a position is unique for a fixed `CheckedRefinement.steps`
sequence. Two things are missing for it to be a sound obligation name:

1. Neither FR-135 (`pub steps: Vec<StepRow>`) nor FR-136 states the order of `steps`. FR-123
   says "in declaration order" and the `StepRowRef` rustdoc ties that to "QSL FR-135
   `steps`", an order QSL does not specify.
2. FR-146 binds the obligation identity to (refinement node, obligation, hypotheses)
   (ADR-013 O-09). If the refinement node's identity is independent of row order (rows as a
   set keyed by class), reordering two rows leaves the refinement key unchanged while
   `Step{row: 0}` now names a different row: two different obligations share one identity,
   and stored evidence for one would be read as evidence for the other. If row order is in
   the refinement node's preimage, a reorder changes every record's identity, which is
   sound but churns evidence on a non-semantic edit, against O-09's "an edit that leaves it
   unchanged leaves its identity unchanged".

A row key avoids both: RM-5's uniqueness means the row's `StepClass` (FR-136: a
`DeclarationKey` or `NodeKey` for every class but `MemoryStep(kind)`) names the row
independently of order. That type lives in QSL, so QSV would need either a class key it can
hold or QSL's guarantee that `steps` has a canonical order (for example sorted by class key)
that the refinement node preimage covers. The choice is QSL's (FR-146/FR-136); the position
is sound once QSL states that order.

## Verdict

**CONDITIONAL**: FR-123 and TC-922 are well formed and traced. The `StepRowRef` meaning rests
on an order QSL has not specified; QSL should settle it (FR-136 order of `steps`, or a row key
in FR-146) before QSL-548 starts writing records keyed by it.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | `StepRowRef` is defined as a row's position "in declaration order" (rustdoc: "QSL FR-135 `steps`"), but QSL defines no order for `CheckedRefinement.steps` and FR-146 never defines `StepRowRef`. Depending on whether the refinement node preimage covers row order, a row reorder either aliases two obligations under one O-09 identity or churns every record's identity. QSL needs to fix the order (canonical, covered by the refinement key) or key rows by `StepClass` | spec/functional/FR-123-step-simulation-record-and-obligation.md:31-35, src/step_simulation.rs:15-19, quire-spec-language spec/functional/FR-146:60-75, FR-136:48, FR-135:64 | missing-requirement |
