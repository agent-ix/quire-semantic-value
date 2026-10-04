---
id: FR-060
title: "The crate mints no NodeKey"
type: FR
relationships: []
---
# FR-060: The crate mints no NodeKey

## Description

`quire-semantic-value` SHALL NOT mint or decode a `quire_exact::NodeKey`. It resolves a node id read from a preimage by lookup among the keys a caller admitted. The successor of FR-060-AC-5 in `agent-ix/quire-spec-language`, whose API-surface check scanned this crate while it lived there. This repository's `FR-060` is that one criterion only.

## Use case

A reviewer asks whether the shared leaf can create a node identity. It cannot: only the checking stage in the caller mints a `NodeKey`.

## Behavior

1. **No constructor call.** No function of this crate calls `NodeKey::from_digest` or `NodeKey::decode_admitted`, directly or as a function value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-060-AC-5 | A call to `quire_exact::NodeKey::from_digest` or `quire_exact::NodeKey::decode_admitted` anywhere in the crate fails `cargo clippy --all-targets -- -D warnings` through `disallowed-methods` in `clippy.toml`. The one test fixture that needs a `NodeKey` carries `#[allow(clippy::disallowed_methods)]` with a reason. | Analysis (clippy `disallowed-methods`, `make lint`) |

## Status

Implemented.

## Dependencies

- None.
