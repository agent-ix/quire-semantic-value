---
id: SR-16310
title: "spec-review of STD-444 point 7, TC-926 for FR-126-AC-1 to AC-5 (quire-semantic-value PR 63)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@39fb412f; spec/functional/FR-126-own-the-evaluation-result-vocabulary.md; spec/test-cases/TC-925-catalog-identity-rendering-and-supplied-limits.md; spec/test-cases/TC-926-catalog-vocabulary-and-supplied-stop-records.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-126
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-926
    type: references
---

# Spec review of STD-444 point 7, TC-926 (quire-semantic-value PR 63)

## Summary

Ticket: STD-444 (point 7, companion to quire-spec-language #1389). This is one fresh spec-review pass over QSV #63 at head 39fb412f, which has main as an ancestor.

Examined, clean:

- FR-126-AC-1 to AC-5 now read `Test (TC-926)`. AC-1 states `Category::ALL` as `[Success, Violation, Undefined, Refusal, Unsupported, Incomplete, Inconclusive, InternalFailure]`, which is the order of `src/catalog.rs` `Category::ALL`.
- I checked each TC-926 vector against the source:
  - Exit codes are 0, 10, 10, 20, 21, 22, 22, 30, and `trace_exit_code(Inconclusive)` is 0.
  - `CatalogCode` equality and hash ignore the prefix (`with_prefix`), and `Display` renders `value/ill_typed/x`.
  - `InternalFault::catalog_code` is `runtime_invariant`/`established-invariant-broken`.
  - `AbsentKey` renders `absent-key`.
  - `SuppliedLimitKind::field` covers all six kinds, and the three supplied names are `supplied.admission_work_units`, `supplied.conversion_work_units` and `supplied.numeric_magnitude_bits`.
  - `SuppliedPhase::field` covers both phases, `SuppliedStop::limit` is `Some` only for `SuppliedCause::Limit`, and `From<SuppliedStop> for SuppliedFailure` exists.
- TC-925 now excludes AC-1 to AC-5 by pointing at TC-926, so the two TCs partition AC-1 to AC-9 with no overlap.
- TC-926 is the next free TC id on main, and no other QSV PR is open.
- Linear IR-989 is the ticket for retagging the five tests and adding the `Category::ALL` order assertion.

Gate: `quire validate --scope . 'spec/**/*.md'` exits 0 with no error or fail lines, the same as on main.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Verdict

Mergeable. TC-926 is a correct, testable verification of FR-126-AC-1 to AC-5, and it resolves STD-444 point 7 together with IR-989.
