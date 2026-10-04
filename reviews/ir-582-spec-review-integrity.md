---
id: SR-005
title: "Spec review of quire-semantic-value PR #2 (IR-582: FR-060-AC-6)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@0603727bf63ac70f26875b02ab3339290e7ad196; spec/functional/FR-060-the-crate-mints-no-identity.md"
review_set: subset
---

## Summary

Ticket: IR-582. FR-060 is renamed to cover all three identities, its Behavior
names all four constructors, and the new FR-060-AC-6 is verified by clippy
analysis (`make lint`), the same way as AC-5. AC-6 does not collide with any
existing id.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The Description's SHALL still reads "SHALL NOT mint or decode a `quire_exact::NodeKey`" only, while the title, Behavior and FR-060-AC-6 add `EffectiveId` and `PopulationId`. AC-6 has no SHALL behind it; extend the statement | spec/functional/FR-060-the-crate-mints-no-identity.md:11 |

## Verdict

One low finding. Mergeable once it is fixed or accepted.
