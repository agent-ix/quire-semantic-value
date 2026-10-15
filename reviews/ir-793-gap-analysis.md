---
id: SR-11066
title: "IR-793 gap analysis of bounded retained numeric type-link admission"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; PR #27 5fda5aa over 70a479c; FR-117, TC-917, src/declaration/membership.rs, src/supplied_membership.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-117
    type: reviews
---
# IR-793 gap analysis

## Summary

Ticket: IR-793. Planless, diff-scoped reconciliation of FR-117/TC-917, computed trace tags, tests and the changed membership path. Production behavior is backed by focused tests, but two specified acceptance cases have no QSV-level assertion. Plan completion was not assessed.

## Verdict

**CONDITIONAL** — two medium test gaps leave expected-first refusal when both sides exceed B and bounded-link pre-cancellation unverified at QSV's public entry point.

## Coverage

Plan completion: not assessed.

`quoin matrix --repo . --json` tags FR-117-AC-1 through AC-4 to the new tests and reports AC-5 as an inspection criterion. `quire matrix --scope . --strict --format json` reports one pre-existing untagged criterion, FR-109-AC-6; it is outside this diff. The author's first full `make ci` receipt passed. QExact 9a3411d independently tests its eight/four endpoint and cancellation contract. No new source stub or unowned behavior was found. Semantic intent alignment was examined manually for this changed path.

Examined FR-117-AC-1 through AC-5 and TC-917 steps 1 through 5. QSV tests cover individual over-B endpoints, exact-B completion, zero, equal/unequal, N/N-1 event budgets, two roots, late helper cancellation, and no-B event precedence. The exact helper has no reachable allocation or native-size conversion denial, so no synthetic resource fixture is required.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No QSV case makes both expected and actual endpoints oversized to assert expected-first refusal | src/declaration/membership.rs:1769 |
| FND-002 | medium | No bounded type-link case pre-cancels the original handle before entering the helper | src/declaration/membership.rs:1968 |

FR-117-AC-2 and TC-917 step 2 require simultaneous oversized expected and actual endpoints. The twelve new rows make only one side oversized at a time. A regression that chooses the wrong side only when both exceed B could pass those rows. Add a QSV-level case with both oversized and assert the first expected endpoint, measured bits, B, rooted locus, and charged spend.

FR-117-AC-3 and TC-917 step 3 require pre-cancellation for a paired numeric type link with configured B. The new late-cancel test enters the helper, while the new pre-cancel assertion invokes `admit_supplied` without B. Existing bounded pre-cancel tests exercise numeric VALUE admission. Add a pre-cancelled `admit_supplied_bounded_numeric` type-link case, ideally with an oversized endpoint, asserting the original cause, event/locus, zero successful spend, and cancellation precedence over the size stop.
