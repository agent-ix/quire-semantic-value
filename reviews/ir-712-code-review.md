---
id: SR-1340
title: "Code and Rust review of QSV PR 6 typed checked-invariant causes"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@5c11379791cd7b8f3c330337b21168d3de597634; Cargo.lock, src/declaration.rs, src/quantity.rs; FR-369-AC-9, TC-906"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-exact/FR-369
    type: reviews
---
# Code and Rust review of QSV PR 6

## Summary

Ticket: IR-712. Reviewed the complete PR diff and every QSV production checked-invariant constructor against the merged kernel FR-369 cause table. The Rust lane is included in this code review. The lockfile pins the merged kernel implementation at dc7891740d04a3ff4c76c1315e94216f0bdc15ec. Examined FR-369-AC-9 and TC-906 in full, including source/target operand admission, decimal integrality, unresolved identities, comparator and conversion payloads, schedule shape, deferred admission, and scale-zero placement.

## Verdict

**PASS** for production code. Every QSV constructor selects its assigned closed cause, both IllTypedCause payloads are retained without formatting, and non-result outcomes still propagate. No new production panic, unsafe block, copied type or compatibility layer appears in the diff. The separate gap analysis records missing acceptance evidence. The coder's exact-head `make ci` report was exit 0; this review did not rerun the full gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-369-AC-9: examined — QSV's deferred field or tuple result uses `DeferredResultNotAdmitted`; missing enum variant and unresolved unit use their distinct rows; schedule mismatch and comparator refusal retain their existing rows; each equality operand failure uses its exact table row; and scale-zero integer placement uses `ExpectedIntegerPlacement`. The comparator and quantity conversion retain their original `IllTypedCause` payload, while prior non-result and charge behavior is unchanged.
- TC-906: examined — QSV reports the cause of each reached checked-invariant condition.
- Source: src/declaration.rs and src/quantity.rs, all production CheckedInvariant sites; Cargo.lock dependency identity. CI workflow diff is empty.
