---
id: FR-060
title: "The crate mints no NodeKey, EffectiveId or PopulationId"
type: FR
relationships: []
---
# FR-060: The crate mints no NodeKey, EffectiveId or PopulationId

## Description

`quire-semantic-value` SHALL NOT mint a `quire_exact::NodeKey`, `quire_exact::EffectiveId` or `quire_exact::PopulationId`, nor decode a `NodeKey`. It resolves a node id read from a preimage by lookup among the keys a caller admitted. The successor of FR-060-AC-5 and of the T12-C and T12-D rules in `agent-ix/quire-spec-language`, whose API-surface check scanned this crate while it lived there. This repository's `FR-060` is that criterion and its two identity siblings.

## Use case

A reviewer asks whether the shared leaf can create a node identity. It cannot: only the checking stage in the caller mints a `NodeKey`.

## Behavior

1. **No constructor call.** No function of this crate calls `NodeKey::from_digest` or `NodeKey::decode_admitted`, and none calls `EffectiveId::from_digest` or `PopulationId::from_digest`, directly or as a function value. Only the caller's checking stage mints a `NodeKey`, and only its model stage mints an `EffectiveId` or a `PopulationId`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-060-AC-5 | A call to `quire_exact::NodeKey::from_digest` or `quire_exact::NodeKey::decode_admitted` anywhere in the crate fails `cargo clippy --all-targets -- -D warnings` through `disallowed-methods` in `clippy.toml`. The one test fixture that needs a `NodeKey` carries `#[allow(clippy::disallowed_methods)]` with a reason. | Analysis (clippy `disallowed-methods`, `make lint`) |
| FR-060-AC-6 | A call to `quire_exact::EffectiveId::from_digest` or `quire_exact::PopulationId::from_digest` anywhere in the crate fails `cargo clippy --all-targets -- -D warnings` through `disallowed-methods` in `clippy.toml`. The one test fixture that needs an `EffectiveId` carries `#[allow(clippy::disallowed_methods)]` with a reason. | Analysis (clippy `disallowed-methods`, `make lint`) |

## Status

Implemented.

## Dependencies

- None.
