---
id: SR-9024
title: "Gap analysis of quire-semantic-value PR #19: union payload closure"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@e2627f96f9f76250b56a13d75ed9b61c916bcf39; FR-106, FR-106-AC-10, TC-904, FR-113 item 8, src/object_closure.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: reviews
---
# Gap analysis of quire-semantic-value PR #19

## Summary

Ticket: QSL-737. This is a planless audit scoped to the PR diff: the union arm in
`ObjectClosure::check_closed` and its test. The matrix was computed with
`quire matrix --scope <repo> --format json` (quire 0.36.2, engine 0.50.2).

- FR-106-AC-10 is `tagged`, with four binders: the three existing TC-904 tests
  and the new `union_payload_references_are_closed_over`.
- The untagged FR-109 and FR-110 criteria are outside this diff and were already
  untagged.
- There are no stubs and no `todo!`.
- FR-113-AC-6's test is no longer ignored (0 ignored in the test log), now that
  the lock includes quire-exact FR-376 to FR-378.

## Verdict

**CONDITIONAL**: one medium finding.

The union arm implements the behaviour it describes. The test exercises the
payload walk (see SR-9023). No requirement in this repository owns the behaviour
the arm adds to.

FR-113 item 8 owns union payload *admission*. It does not own closure.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No QSV requirement owns object-closure reference checking. `check_closed` refuses a reference with `DanglingReference` unless it names a closure object or a `tolerated_dangling` target, and walks Option payloads, composite slots, collection elements and now union payloads. FR-106's Behavior and AC-10 state only duplicate-triple, unknown-type and ambiguous-`find`. The module doc points upstream to QSpec FR-143-AC-3/AC-9 and QSL FR-106. This PR adds a fifth carrier to an unowned rule, and its test is traced to AC-10 (SR-9023 FND-002). Add an FR-106 behavior item and acceptance criterion: every present reference reached through Option payloads, composite slots, collection elements and union payloads must name a closure object or a tolerated target, otherwise `DanglingReference`. Add matching TC-904 rows, including the nested-union and tolerated cases. | spec/functional/FR-106-object-closure-admission.md:17-26; src/object_closure.rs:141-178 |

## Coverage

- FR-106-AC-10: bound by `a_duplicate_identity_triple_refuses`, `a_non_model_object_type_refuses` and `find_returns_none_for_an_ambiguous_key`. All three bindings are correct. It is also bound by `union_payload_references_are_closed_over`, which is the wrong binding (SR-9023 FND-002).
- Reverse gap: dangling-reference closure has no owning criterion (FND-001).
- FR-113 item 8: the union payload admission rule is owned there, but not implemented at this sha (SR-9023 FND-001). It is in QSV #12's scope.
- Semantic review: not run as a separate pass. This reviewer subagent could not take an opt-in. The test oracle was judged by hand in SR-9023.

Plan completion: not assessed
