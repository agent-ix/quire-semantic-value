---
id: SR-9035
title: "Spec review of quire-semantic-value PR #12: FR-115 and TC-915"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@52786ce8ae19383a052e4d3a9a28804ecaf776b1; spec/functional/FR-115-stage-and-seal-union-declarations.md, spec/test-cases/TC-915-union-topology-sealing-and-storage.md, context FR-108, FR-109, FR-113, TC-907"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-915
    type: reviews
---
# Spec review of quire-semantic-value PR #12

## Summary

Ticket: QSL-500. This review covers FR-115 and TC-915, read against FR-108,
FR-109, FR-113 and TC-907 on main.

The PR has no edits to FR-106 or FR-113. The FR-106 behavior item 3 and
FR-106-AC-11 changes belong to QSV #19.

Both files pass `quire validate` with no warnings.

## Verdict

**CONDITIONAL**: one high, two medium and two low findings.

What holds:

- The staging model is coherent. Topology comes before identity, sealing is
  atomic, and the inverse joins are explicit. No hashing or reminting happens
  in QSV.
- FR-108's storage units and classification are reused without a new carrier.
- The Dependencies section states honestly that FR-115 neither establishes
  the supplied-membership phase nor provides QSL run evidence.
- The FR-115 and TC-915 ids are unique repo-wide.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Behavior 4 says "subsequent Union-specific walks SHALL accept the same caller-owned budget" that topology and sealing use, which is the authored-declaration `WorkBudget`. Those walks include supplied-value membership (`admits` of a union value). This contradicts FR-113 item 14 (`admits` takes no charge and reports no limit) and FR-109 Behavior 1 and 7 (supplied membership uses its own supplied-membership budget and a typed admission-phase result, never an authored-declaration failure). Scope Behavior 4 to topology, sealing and construction. Then either keep supplied membership uncharged under FR-113 item 14 until FR-109 lands, or amend item 14 and say explicitly that the authored budget stands in until FR-109's carrier exists. | spec/functional/FR-115-stage-and-seal-union-declarations.md:52-55 |
| FND-002 | medium | FR-115-AC-4 asserts that tests exist rather than a behavior: "Existing 5,000-member, 1,000-Option and 10,000-value fixtures and exact N/N-1 controls remain in the source". "Released non-Union limits and both Record/Tuple prior-stop regressions remain unchanged" names no observable outcome. TC-915 step 4 repeats this ("Retain all existing ... tests"). Deleting a test cannot be caught by a test. Restate it as behavior: a 5,000-member union admits at N and stops at N-1 with `WorkUnits`; a 10,000-deep supplied union is admitted on a small stack; Record and Tuple prior stops propagate unchanged. | spec/functional/FR-115-stage-and-seal-union-declarations.md:68 |
| FND-003 | medium | FR-115 does not state that a union declaration takes no part in composite generalization. quire-exact FR-379 Behavior 5 says a union declaration has no descendant. FR-115 is silent on whether a union may name supertypes or be named as one, and the code accepts both (SR-9033 FND-004). Add a Behavior item and an AC clause that refuses both edge directions with a typed cause. | spec/functional/FR-115-stage-and-seal-union-declarations.md:38-60 |
| FND-004 | low | The Status sections track process, not behavior. FR-115 says "Independent review is pending", which will go stale. TC-915 says "TC-907's protected Union obligation remains open under IR-713 until the complete QSL-500 delivery", and its Description says "Extend TC-907's protected Union portion". TC-907 has no "protected Union portion" and is not edited, and its own Status still reads "Planned" although its tests are tagged. Drop the review-state line, and either edit TC-907 or remove the cross-reference. | spec/functional/FR-115-stage-and-seal-union-declarations.md:84; spec/test-cases/TC-915-union-topology-sealing-and-storage.md:14-16, 46-48 |
| FND-005 | low | FR-109 is now stale against FR-115. Its helper table row "Future union payload/descriptor" and its Dependencies line "A reviewed public union declaration/member API remains another implementation prerequisite" both describe the API that FR-115 now provides. Point both at FR-115. | spec/functional/FR-109-bound-supplied-value-traversal.md:108, 145-146 |
