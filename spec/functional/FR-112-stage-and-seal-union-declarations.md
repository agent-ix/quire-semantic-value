---
id: FR-112
title: "Resolve union topology before verified identity sealing"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-143
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-441
    type: depends_on
---
# FR-112: Resolve union topology before verified identity sealing

## Description

The shared type environment SHALL retain resolved union names and positional
types before the canonical producer settles the declaration key. Runtime
membership SHALL become available only after the complete declaration is
attached to its producer-verified final key and member bindings.

## Inputs

- One internal declaration handle, ordered member identifiers and position types.
- A settled canonical declaration key and complete FR-441 verified bindings.
- The caller's cumulative authored admission budget and original cancellation handle.

## Outputs

- The same descriptor, first resolved and subsequently sealed, with explicit
  inverse internal-handle/final-key joins.
- Or the existing typed declaration, work, allocation or capacity failure.

## Behavior

1. Resolved topology SHALL participate in the shared declaration registry's
   duplicate-name, member-type, named/escaping recursion and all-member IEEE
   checks. It SHALL NOT require final member bindings to expose names and
   positional types to canonical declaration lowering.
2. Sealing SHALL require exactly one binding per declared member, in declaration
   order, with exact identifier and final declaration identity. Duplicate keys,
   resealing and final-key collisions SHALL retain the existing typed refusal.
   The producer owns canonical final-key and FR-441 preimage verification;
   this environment SHALL NOT hash, remint or promote an internal handle.
3. Sealing SHALL publish all bindings and both key joins together. It SHALL use
   FR-108's fallible storage boundary and reserve complete destination growth
   before attachment. Failed validation or reservation SHALL leave the logical
   descriptor, member index and joins unchanged. A retry SHALL retain successful
   caller spend and use the same declarations and verified identities.
4. Union topology, sealing and subsequent Union-specific walks SHALL accept the
   same caller-owned budget. No Union-specific convenience entry point SHALL
   silently replace that budget or its cancellation handle. Existing non-Union
   declaration admission charge points and typed causes SHALL remain unchanged.
5. Unsealed topology SHALL grant no runtime member lookup or construction.
   A sealed carrier SHALL retain the verified final declaration/member/name
   binding, exact payload arity and positional membership. Record and Tuple
   identities SHALL retain their established behavior.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-112-AC-1 | Resolved identifiers and ordered position types are readable before binding, including named Option-escaping recursion and all-member IEEE inspection; unsealed runtime member construction and admission refuse. | Test |
| FR-112-AC-2 | Exact complete verified sealing retains the original handle and topology and exposes inverse final-key joins. Wrong count/order/name/declaration, duplicate member keys, resealing and key collisions preserve their typed cause and leave the environment unchanged. | Test |
| FR-112-AC-3 | Actual Union registry, traversal and each sealing destination reservation can deny with FR-108's exact native request and classification. No partial binding or join is exposed; identical input succeeds after removing denial. Byte-request and capacity controls retain distinct carriers. | Test |
| FR-112-AC-4 | One caller budget and borrowed cancellation handle span topology, sealing and Union member/type walks. Existing 5,000-member, 1,000-Option and 10,000-value fixtures and exact N/N-1 controls remain in the source. Released non-Union limits and both Record/Tuple prior-stop regressions remain unchanged. | Test |

## Dependencies

QSpec FR-143 and FR-441 retain ownership of value membership and canonical
identity. FR-108 retains storage units and failure classification; this
requirement introduces neither an allocator carrier nor a checked-invariant
cause. FR-109 and QSL-681 own the distinct supplied-membership phase contract;
authored work limits here do not establish that phase or evaluator accounting.
Kernel-owned runtime allocation and scalar helper qualification remain separate
implementation obligations. QSL-502/504 own the canonical stage handoff;
QSL-503 owns actual run admission, references, conversion and no-charge evidence.

## Status

Owned Union source proposal; all changed-head checks and qualification UNRUN.
No whole-criterion acceptance or independent review is claimed.
