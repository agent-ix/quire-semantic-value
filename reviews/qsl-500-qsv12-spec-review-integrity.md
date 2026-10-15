---
id: SR-9036
title: "Integrity review of quire-semantic-value PR #12: FR-115 and TC-915"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@52786ce8ae19383a052e4d3a9a28804ecaf776b1; spec/functional/FR-115-stage-and-seal-union-declarations.md, spec/test-cases/TC-915-union-topology-sealing-and-storage.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---
# Integrity review of quire-semantic-value PR #12

## Summary

Ticket: QSL-500. This review checks FR-115 and TC-915 for atomicity,
defined terms and declared relationships.

## Verdict

**CONDITIONAL**: one medium and two low findings.

Each criterion is testable in principle, and every criterion is bound to at
least one test (SR-9034). Structure and frontmatter validate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | All four criteria are compound. AC-2 alone bundles a success path with eight refusal variants: count, order, name, declaration, duplicate keys, resealing, collision, and an unchanged environment. AC-3 bundles registry, traversal and three sealing destinations, plus retry and two carrier controls. When a criterion fails, the matrix cannot tell which clause failed, and a single tagged test can mark the whole criterion covered. Split each criterion along its clauses, at least into success, refusal and storage parts. | spec/functional/FR-115-stage-and-seal-union-declarations.md:63-68 |
| FND-002 | low | Several terms are undefined: "Union-specific walks", "Released non-Union limits", "Record/Tuple prior-stop regressions", "native reservation amounts", and "source unqualified" in the PR title. A reader cannot tell which APIs Behavior 4 binds. Name the APIs, or define each term once. | spec/functional/FR-115-stage-and-seal-union-declarations.md:52-55, 68 |
| FND-003 | low | The `relationships` list names FR-108, QSpec FR-143 and FR-441. It omits FR-109 and FR-113, which FR-115 constrains (Behavior 4, Dependencies) and whose `admits` rules it changes in practice. It also omits quire-exact FR-379, the carrier it seals into. Add these as `depends_on` or `references` edges. | spec/functional/FR-115-stage-and-seal-union-declarations.md:1-11 |
