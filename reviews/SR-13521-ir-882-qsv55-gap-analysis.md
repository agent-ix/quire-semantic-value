---
id: SR-13521
title: "IR-882 quire-semantic-value PR 55 gap analysis of FR-125 against TC-924"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@4a52c1105 vs origin/main e42f0d4; spec/functional/FR-125-definition-closure-refusal-vocabulary.md, spec/test-cases/TC-924-definition-closure-refusal-vocabulary.md, tests/definition_closure_refusal.rs, src/definition.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---
# Gap analysis of FR-125 (IR-882, PR #55)

## Summary

Ticket: IR-882. Plan completion: not assessed. `quoin matrix --json` (quoin 0.28.3)
on head 4a52c1105 binds exactly one test function to each FR-125 criterion:
AC-1 `spellings_match_fr_272_invalid_package_row`, AC-2
`invalid_package_pairs_each_cause_with_the_single_code`, AC-3
`package_cause_is_closed_and_has_no_digest_mismatch_spelling`, AC-4
`selection_codes_are_in_check_order_and_round_trip`. Each test asserts all of its
criterion's clauses and follows TC-924's step for it. The `src/definition.rs`
public items each belong to FR-125, so there is no unowned code and no stub.

## Examined scope

- FR-125-AC-1 (examined): spellings byte-equal to QSpec FR-272's `invalid_package` row, and `invalid_package`. Covered: four `assert_eq!` on `as_str`; literals checked against FR-272 c9a3ef6 line 74.
- FR-125-AC-2 (examined): `invalid_package(cause)` gives `InvalidPackage` and the cause for each of three; one variant. Covered: loop over the three causes plus the exhaustive `code_spelling` match.
- FR-125-AC-3 (examined): three variants, not `#[non_exhaustive]`, exhaustive match in `tests/` compiles, no digest-mismatch spelling. Covered: `cause_index` with no wildcard and the `matches!` over the three spellings.
- FR-125-AC-4 (examined): `ALL` order, `as_str`, `from_code` round-trip, `None` for `invalid_package`, `""` and other casing. Covered; the test also rejects a trailing space.
- TC-924 steps 1-4 (examined): each step maps to one test above.

## Verdict

Clean: every FR-125 criterion is fully tested, and each tag is honest (the test
behind it asserts that criterion and nothing tagged is untested). Mergeable. All
18 mutants of `src/definition.rs` were killed (see SR-13520).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

`quoin matrix --json`: each of FR-125-AC-1..4 has one `test_function` binder in
`tests/definition_closure_refusal.rs` (lines 34, 52, 63, 76). The status is "no run evidence";
the matrix was not fed run results.
