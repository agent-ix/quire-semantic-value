---
id: SR-16678
title: "Code review of QSV PR #73 structural identity lint guard"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; PR #73 head 480cb277; clippy.toml; IR-1142"
review_set: subset
date: '2026-10-30'
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-060
    type: reviews
---

## Summary

The changed file is `clippy.toml` only. The new `NodeKey::new`, `EffectiveId::new`, and `PopulationId::new` paths match the constructors in exact PR #101. QSV's current `Cargo.lock` still resolves exact at f4d9c97f, where the retired digest constructors remain callable. `git diff --check` passed. Build and Clippy were deferred because this draft depends on exact #101 and the shared build lane is reserved.

## Scope

| Unit | Path | Role |
| --- | --- | --- |
| Clippy constructor bans | clippy.toml | examined |
| FR-060-AC-5 node constructor ban | spec/functional/FR-060-the-crate-mints-no-identity.md | examined |
| FR-060-AC-6 model constructor bans | spec/functional/FR-060-the-crate-mints-no-identity.md | examined |
| exact structural constructors | exact PR #101 | context_only |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | This change removes four active digest-constructor bans while the locked exact revision still exposes those methods, weakening FR-060-AC-5/6 until the dependency and callers migrate. | clippy.toml:7-11 |

## Verdict

Not mergeable at this head. Keep the former bans effective until the locked exact revision removes those methods and the QSV consumer migration replaces all calls; then verify the new constructor bans with the final all-targets Clippy gate. This is a review of the static guard slice, not an acceptance verdict for all of IR-1142.
