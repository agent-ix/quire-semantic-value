---
id: FR-060
title: "The crate mints no NodeKey, EffectiveId or PopulationId"
type: FR
relationships: []
---
# FR-060: The crate mints no NodeKey, EffectiveId or PopulationId

## Description

`quire-semantic-value` SHALL NOT mint a `quire_exact::NodeKey`, `quire_exact::EffectiveId` or `quire_exact::PopulationId`, nor decode a `NodeKey`. It resolves an identity by lookup among the structural values a caller admitted. This repository owns the no-minting rule for these three identities; the checking and model stages that supply them belong to `agent-ix/quire-spec-language`.

## Use case

A reviewer asks whether the shared leaf can create a node identity. It cannot: only the checking stage in the caller supplies an admitted `NodeKey`.

## Behavior

1. **No constructor call.** Production code calls none of `NodeKey::new`, `EffectiveId::new` or `PopulationId::new`, directly or as a function value. The retired `from_digest` and `decode_admitted` calls and digest-domain constants are absent from this crate. The caller's checking stage supplies `NodeKey` values, and its model stage supplies `EffectiveId` and `PopulationId` values. A QSV test-only local identity fixture SHALL carry an explicit lint exception. Production behavior SHALL carry the supplied value unchanged.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-060-AC-5 | A call to `quire_exact::NodeKey::new` anywhere in the crate fails `cargo clippy --all-targets -- -D warnings` through `disallowed-methods` in `clippy.toml`, except one test-only fixture with a documented reason. Source inspection finds no call to the retired `NodeKey::from_digest` or `NodeKey::decode_admitted`, no local digest decoder and no reminting from an admitted node's components. | Analysis |
| FR-060-AC-6 | A call to `quire_exact::EffectiveId::new` or `quire_exact::PopulationId::new` anywhere in the crate fails `cargo clippy --all-targets -- -D warnings` through `disallowed-methods` in `clippy.toml`, except one test-only fixture with a documented reason. Source inspection finds no call to the retired `from_digest` constructors and no replacement digest-domain constant or fixed identity oracle. | Analysis |

## Status

Planned for structural identity migration.

## Dependencies

- The structural identity API in `quire-exact` and admitted values from the QSL checking/model stages.
