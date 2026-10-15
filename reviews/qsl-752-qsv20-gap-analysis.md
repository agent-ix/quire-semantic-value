---
id: SR-9047
title: "Gap analysis of quire-semantic-value PR #20: model-owned enumeration case names"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@346076d6d1009b7019de8f8e97e08428d56d4ea3; FR-116, TC-916, src/enumeration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-116
    type: reviews
---
# Gap analysis of quire-semantic-value PR #20

## Summary

Ticket: QSL-752. This is a planless audit scoped to the PR diff. The matrix was
computed with `quire matrix --scope .` on a read-only checkout at the reviewed
sha.

- FR-116-AC-1 to AC-6 are all `tagged`. AC-1, AC-2 and AC-3 bind to
  `a_model_owned_declaration_accepts_any_nonempty_case_name_verbatim`, AC-4 to
  `a_model_owned_declaration_refuses_empty_duplicate_and_unsorted_cases`, AC-5 to
  `two_model_owned_declarations_of_one_shape_agree_on_equality_and_variants` and
  AC-6 to `the_identifier_path_still_refuses_non_identifier_cases`. Each test
  asserts what its AC states, so all six bindings are correct.
- Reverse direction: the one new production item, `model_owned`, is owned by
  FR-116. The private helper `sorted_if_unordered` is shared by FR-116 and the
  existing `new`.
- There are no stubs, no `todo!` and no ignored tests.
- Oracle strength: each of the following mutations makes a test fail. Using
  `is_member_list` inside `model_owned` fails AC-1. Dropping the empty-name
  check fails `[""]`. Dropping the duplicate check fails
  `["Gold Plus","Gold Plus"]`, which `is_sorted` alone accepts. Dropping the
  sort check fails `["b b","a a"]`. Trimming or case folding fails the
  verbatim `members()` and `gold plus` assertions.
- Semantic review: done by hand in this pass, because this reviewer subagent
  could not take an opt-in. Intent, test and code agree.

## Verdict

**PASS**. There are no coverage gaps, no wrong bindings and no unowned code.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- FR-116-AC-1..AC-6: tagged, bindings correct (see Summary).

Plan completion: not assessed
