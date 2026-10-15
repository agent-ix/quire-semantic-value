---
id: SR-11076
title: "Spec review of quire-semantic-value PR #12: rewritten FR-115 and TC-915"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@dad364a5cc77ad999c0870ea6c8c0b310fba832d; spec/functional/FR-115-stage-and-seal-union-declarations.md, spec/test-cases/TC-915-union-topology-sealing-and-storage.md, context FR-108, FR-109, FR-113"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-915
    type: reviews
---
# Spec review of quire-semantic-value PR #12 (ported head)

## Summary

Ticket: QSL-500. This review covers the rewritten FR-115 and TC-915. The
integrity checks (ids, structure and validation) are folded into this file.

## Verdict

**PASS**.

- **No id collision.** Main holds FR-116, FR-117, TC-916 and TC-917. FR-115
  and TC-915 exist only on this branch.
- **Validation.** `quire validate` on both files gives no errors, only the
  module's own duplicate-archetype warnings.
- **Behaviour 4 matches the code.** Topology charges the environment's
  `WorkBudget`, which polls cancellation. Sealing and lookup take no budget.
  Payload membership defers to the single FR-109 walk.
- **Status sections match the code.** They state that union value membership
  is pending and that it is refused as `UnsupportedUnion`. This agrees with
  FR-109/FR-113 and with membership.rs:562-568.
- **Behaviour 5 versus the code.** "Record and Tuple identities SHALL retain
  their established behavior" is consistent as text. The code's
  composite-carrier admission at a union type is a code defect (SR-11074
  FND-001), not a spec contradiction.

Recorded (low): both Status sections point to "the pull request" for the list
of waiting tests, which will go stale once the PR merges. The test-gap items
for AC-1 and AC-4 are recorded in SR-11075.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
