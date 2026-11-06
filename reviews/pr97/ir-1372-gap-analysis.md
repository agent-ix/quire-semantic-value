---
id: SR-20261102072342
title: "IR-1372 gap-analysis for QSV PR 97"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/quantity.rs, tests/identified_quantity.rs, spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: references
---

# IR-1372 gap-analysis for QSV PR 97

## Summary

Ticket: IR-1372. Frozen head c25d49c. The computed matrix tags FR-124-AC-7 to all three new tests. The public path and checks cover completion, work, result, occurrence and injected-denial boundaries; no changed code lacks an owning requirement.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-124-AC-7 | examined | Exact `2 m` into `Int[-2,2]` completes as integer `2` with no loss under QSpec `CQ-E13` limits `L(2,0,0,1,2,1)`, admitting identity-read, target-domain, then result-retain with final work 2, occurrence 1 and result 1. Work limit 1 denies target-domain; result limit 0 and injected result-retain denial stop atomically without adding its counters, with the injected record naming z |
| src/quantity.rs | examined | `charge_retain(meter, 0)` on integer conversion; other calls pass 1. |
| tests/identified_quantity.rs | examined | Three new traced tests for CQ-E13 completion, atomic denial, and noninteger work retention. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS; no findings.
