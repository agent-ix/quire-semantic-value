---
id: FR-115
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
# FR-115: Resolve union topology before verified identity sealing

## Description

The shared type environment SHALL retain resolved union names and positional
types before the canonical producer settles the declaration key. Runtime
membership SHALL become available only after the complete declaration is
attached to its producer-verified final key and member bindings.

## Inputs

- One internal declaration handle, ordered member identifiers and position types.
- A settled canonical declaration key and complete FR-441 verified bindings.
- The environment's admission work limit and the caller's cancellation handle.

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
   identical input to succeed after the denial is removed.
4. Union topology admission SHALL charge the environment's one admission work
   limit and poll its cancellation handle; running out SHALL be the named work
   limit. Sealing and Union member lookup SHALL NOT take a separate budget.
   Union payload membership SHALL be decided by the single typed-event
   admission walk (FR-109), never by a second walk in this module. Existing
   non-Union declaration admission charge points and typed causes SHALL remain
   unchanged.
5. Unsealed topology SHALL grant no runtime member lookup or construction.
   A sealed carrier SHALL retain the verified final declaration/member/name
   binding, exact payload arity and positional membership. Record and Tuple
   identities SHALL retain their established behavior, and a Record or Tuple
   carrier SHALL never be admitted at a union type, whichever key it carries.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-115-AC-1 | Resolved identifiers and ordered position types are readable before binding, including named Option-escaping recursion and all-member IEEE inspection; unsealed runtime member construction and admission refuse. | Test |
| FR-115-AC-2 | Exact complete verified sealing retains the original handle and topology and exposes inverse final-key joins. Wrong count/order/name/declaration, resealing and final-key collisions preserve their typed cause and leave the environment unchanged. With the structural `VariantId` from exact #101, a member's key is derived from the final declaration key and its identifier, so a distinct-name duplicate member key is unrepresentable; duplicate member names remain refused during topology admission. | Test |
| FR-115-AC-3 | Actual Union registry, traversal and each sealing destination reservation can deny with FR-108's exact native request and classification. No partial binding or join is exposed; identical input succeeds after removing denial. Byte-request and capacity controls retain distinct carriers. | Test |
| FR-115-AC-4 | Union topology admission charges the environment's admission work limit with exact N/N-1 behaviour over the 5,000-member and 1,000-Option fixtures, and observes its cancellation handle. Released non-Union limits and both Record/Tuple prior-stop regressions remain unchanged. | Test |
| FR-115-AC-5 | A Record or Tuple carrier keyed with a union's internal handle or final key is not admitted at that union's type, directly or inside an Option, before and after sealing. | Test |

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

Declaration, sealing, member lookup, construction and graph/closure staging
are implemented in this crate and covered by tests/union_registry.rs and
tests/union_staging.rs. Union VALUE membership uses FR-109's typed-event
admission walk: a sealed final key, verified member binding, payload arity and
positional payload visits are checked under one supplied budget. The Union
membership tests in those files run in the default test lane. Supplied-phase
failure transport and caller migration remain owned elsewhere (FR-109,
QSL-681, QSL-502 to QSL-504).
