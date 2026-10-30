---
id: SR-16681
title: "IR-1142 QSV PR 73 FR-123 structural encoding specification review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value; PR #73 head 41eb9b2; spec/functional/FR-123-step-simulation-record-and-obligation.md; spec/test-cases/TC-922-step-simulation-record-and-obligation.md; src/step_simulation.rs test delta"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-123
    type: reviews
---

# IR-1142 QSV PR 73 FR-123 structural encoding specification review

## Summary

Independent review of the spec and literal-oracle correction at PR #73 head 41eb9b2. `NodeKey::Display` in the locked structural exact API writes `package:{:? path}:ordinal`; a one-segment `PathSegment::Text` uses Rust string Debug quotes. For the shared fixture, `common::node(0)` therefore displays `qsv-test:[Text("00000000000000000000")]:0`, and `quire-canonical` JSON string encoding escapes those two quotes. The new independent raw byte literals in `src/step_simulation.rs` match this spelling for 0, 1, 171 and 205. The test retains initial/step objects, two ordered hypotheses, obligation and maximum-row literals, field variations, and the exact-length/one-byte-short limit control. No digest KAT remains in the changed test. Targeted Quire validation and `git diff --check` passed; the aggregate build is owned by the code lead.

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| FR-123 Behavior 3 structural JSON encoding | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| FR-123-AC-4 exact bytes and all field variations | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| FR-123-AC-5 exact and one-byte-short limit | spec/functional/FR-123-step-simulation-record-and-obligation.md | examined |
| TC-922 steps 4 and 5 | spec/test-cases/TC-922-step-simulation-record-and-obligation.md | examined |
| quire-exact NodeKey and PathSegment Display implementation | quire-exact ea8669d src/node.rs | context_only |
| FR-123-AC-1/2/3/6 unchanged controls | spec/functional/FR-123-step-simulation-record-and-obligation.md | context_only |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS** for the FR-123/TC-922 delta and its code/test oracle at 41eb9b2. The correction states structural display bytes and JSON escaping without narrowing the AC-4 field controls or AC-5 limit boundary. The main PR's full gate remains with the code lead.
