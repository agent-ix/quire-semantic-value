---
id: FR-128
title: "Consume admitted structural identities in semantic values"
type: FR
relationships:
  - target: "ix://agent-ix/quire-semantic-value/FR-060"
    type: "depends_on"
---
# FR-128: Consume admitted structural identities in semantic values

## Description

When the caller supplies admitted structural `NodeKey`, `EffectiveId`, `UniverseId` and `UnitId`
values, `quire-semantic-value` SHALL carry and compare those values by their typed components.
It SHALL NOT convert them to byte-array keys or derive an identity digest. The crate's separate
no-minting boundary is [FR-060](./FR-060-the-crate-mints-no-identity.md).

## Inputs

- Caller-admitted declaration, model, universe and unit identities.
- Checked dimension and unit nodes, including canonical compound-unit terms.
- Semantic values and declarations that refer to those identities.

## Outputs

- Typed values, graph lookups and refusals retaining the supplied identities.
- Declared or compound `UnitId` values with their distinct structural arms.

## Behavior

1. **Structural lookup.** The crate SHALL index checked dimensions, units, declarations,
   references and object types by the admitted typed identity itself. A lookup using a foreign
   package, path, ordinal, owner or case SHALL not alias an admitted entry merely because a
   displayed name or another component matches.
2. **Unit identity.** The crate SHALL keep declared units keyed by their admitted `NodeKey` and
   compound units keyed by the canonical ordered root-unit terms and exponents. The crate SHALL
   keep a declared arm distinct from a compound arm even when their displayed components coincide.
3. **No byte bridge.** The crate SHALL not index a unit or dimension by `[u8; 32]`, call
   `NodeKey::as_bytes`, use `NODE_KEY_DOMAIN` or `COMPOUND_UNIT_DOMAIN`, or rebuild a typed
   identity from constituent bytes. When a referenced node is malformed or absent, the crate SHALL
   retain the existing typed refusal and charge order.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-128-AC-1 | Two admitted `NodeKey` values that differ only by package, path or ordinal remain distinct in dimension, unit, composite and enum lookups; an absent target keeps the existing missing-node refusal rather than resolving to a similarly named node. | Test |
| FR-128-AC-2 | Two admitted `EffectiveId` values that differ only by owner chain or original declaration key remain distinct in model-object lookup, conformance and reference checks; the unknown or foreign identity retains its typed refusal. | Test |
| FR-128-AC-3 | Unit lookup and quantity handoff preserve the declared `UnitId::Declared(r)` arm and the compound `UnitId::Compound([(r, 1)])` arm as unequal even though both carry the same admitted root node `r`; the declared arm orders first, compound terms remain in admitted canonical order, the 16 MiB bounded handoff refusal of FR-124-AC-4 remains, and the result and meter behavior of two-step conversion stay unchanged. | Test |
| FR-128-AC-4 | Source inspection finds no `NodeKey::as_bytes`, `[u8; 32]` identity index, digest-domain constant, fixed digest oracle or structural-to-digest bridge in production unit, quantity, semantic-node, declaration or enumeration code. | Inspection |
| FR-128-AC-5 | For a malformed or missing dimension or unit reference, the first typed structural refusal and the charge count match the corresponding pre-migration behavior; no candidate is selected by partial structural identity. | Test |

## Dependencies

- `quire-exact` supplies the structural identity types and distinct `UnitId` arms.
- The caller supplies identities already admitted by its checking or model stage.
