---
id: SR-12811
title: "Gap analysis of QSV fairness values"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/fairness.rs, src/lib.rs, spec/functional/FR-121-shared-property-and-fairness-values.md, spec/test-cases/TC-921-shared-property-and-fairness-values.md"
review_set: subset
---
# Gap analysis of QSV fairness values

## Summary

Ticket: IR-872. PR #45 at 1c88d24f19d2f4ad988a355f76b472188b5d7298, base 8e53f6772ed29ce266437ab2acdf9ea3830056bc. Examined FR-121-AC-1 through AC-6 and TC-921 against the changed code and spec status.

## Verdict

**PASS** — All five Test-kind criteria bind to one executable traced test each. AC-6 is inspection and build evidence. Repository coverage is 93/103 backed with 9 existing unbacked rows and 0 status contradictions; no new reverse gap or stub appears in this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Five `#[trace]` tests in `src/fairness.rs` bind FR-121-AC-1 through AC-5 and TC-921. The tests check enum variants and accessors, equality and hashing, exact ordering, literal canonical bytes and escaping, and byte limits. Inspection confirms AC-6: no new normal dependency, no root re-export, no production `EffectiveId::from_digest`, and no `std` outside tests. `make ci` exited 0: format, host and thumbv7em-none-eabi Clippy, tests, no-std build, cargo deny, and docs. Strict coverage reports 93/103 backed, 9 unbacked, 0 contradicted. Plan completion: not assessed.
