---
id: SR-5400
title: "Code review of admission storage and its tagged controls"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@a6922fe7e396e22a8989963a50d911cb748ba4a6; src/declaration.rs, spec/functional/FR-108-report-registry-allocation-failure.md, spec/test-cases/TC-907-registry-allocation-failure.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: references
---
# Code review of admission storage and its tagged controls

## Summary

Ticket: IR-713. The storage redesign routes observed reservations through fallible helpers. The AC-4 work-budget boundary still lacks public admission evidence.

## Verdict

**CONDITIONAL** — The medium finding needs an acceptance control before this draft PR can be qualified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-4 lacks an admission-level work_units N-1/N control; the tagged tests check cancellation and duplicate-key refusal, while the only budget check calls WorkBudget::charge directly. A changed admission charge point or wrong public limit counter can pass. | src/declaration.rs:224 |

## Coverage

FR-108-AC-1 through FR-108-AC-5 and TC-907 were examined against the frozen Rust diff and static trace matrix. QSL exact-consumer compilation and the Union fixture remain pending. No full build was repeated for this review.
