---
id: FR-118
title: "Bound retained Decimal type-link comparison in supplied membership"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-117
    type: references
  - target: ix://agent-ix/quire-specification/FR-140
    type: references
  - target: ix://agent-ix/quire-exact/FR-382
    type: depends_on
---
# FR-118: Bound retained Decimal type-link comparison in supplied membership

## Description

When supplied membership compares paired retained `Decimal` type links, QSV SHALL use
the caller's authenticated finite numeric magnitude ceiling `B` and
an exact-kernel bounded, cancellable `DecimalType` equality operation. The
comparison concerns declared type identity, not mathematical equality of
Decimal values or Decimal value membership.

## Inputs

- Borrowed expected and actual type chains, the argument-rooted membership
  path, the caller's cumulative supplied-membership budget and original
  `Cancel` handle.
- The finite `B` configured by the initiating caller for this invocation.
  The same `B` applies to every retained Decimal domain endpoint and numeric
  value reached in the invocation. QSV's direct API supplies no default.

## Outputs

- A completed type comparison followed by the existing membership walk; or
- A typed supplied-admission stop at the compared link's rooted locus.

## Behavior

1. For each compared pair of links, QSV SHALL request exactly one
   `TypeLinkComparison` event at the current zero-based link ordinal before
   invoking Decimal equality. Event denial SHALL prevent helper entry and
   retain the caller's prior successful spend. The budget SHALL poll the
   original `Cancel` before spending the event: pre-cancellation adds no
   spend. If cancellation is observed after that event succeeds, the
   initiating event remains spent once, including when the helper stops.
2. For paired Decimal links under configured `B`, QSV SHALL pass the borrowed
   expected `DecimalType` first, actual `DecimalType` second, `B`, and the
   original `Cancel` to the exact kernel. Before deciding equality, that
   operation SHALL check magnitude bits, with zero counting as one bit, in
   this order: expected lower coefficient bound, expected upper coefficient
   bound, actual lower coefficient bound, actual upper coefficient bound.
   The exact kernel SHALL check every operand, including operands after an earlier
   unequal field. The first over-`B` operand SHALL return its typed position,
   measured bits and `B` without reporting unequal types.
3. When all four bounds pass, the exact kernel SHALL compare signed lower and
   upper coefficient bounds, `min_scale`, `max_scale`, and rounding mode
   exactly. It SHALL NOT normalize, align, round, widen or infer a type.
   A completed `true` SHALL continue the walk. A completed `false` SHALL
   return the established invalid-type cause at this link. The exact kernel
   owns bounded borrowed-limb comparison and cancellation polling; QSV SHALL
   NOT duplicate that algorithm or fall through to unbounded `ValueType`
   equality. [Exact FR-382](ix://agent-ix/quire-exact/FR-382) establishes
   four fixed-width endpoint checks and, with `L = ceil(B / 32)`, at most
   `2L` fixed-width limb-pair steps, constant auxiliary storage and no
   input-sized clone, scratch allocation or native-size conversion. Its
   helper polls the original `Cancel` before bound validation, before each
   endpoint check, before each limb-pair step and before publishing equality.
   The arithmetic cancellation-response gap is at most one fixed-width
   limb-pair step. QSV SHALL verify the published exact-kernel source against
   these [FR-109](./FR-109-bound-supplied-value-traversal.md) qualification
   premises before claiming its own paired Decimal admission path supported.
4. QSV SHALL retain a kernel size stop as typed unsupported size with operand
   position, measured bits, `B` and this locus. It SHALL retain the original
   cancellation cause at the same locus, distinct from invalid type and
   supplied work-budget limit. The published allocation-free helper exposes
   no capacity or allocation stop for this operation.
   Helper steps and post-helper cancellation checks SHALL add no logical
   event or evaluator debit. A QSpec consumer SHALL project an over-`B` stop
   as `supplied.numeric_magnitude_bits` on the charged `type-comparison`
   event, distinct from work-budget exhaustion.
5. If no authentic `B` is configured, QSV SHALL return
   `UnsupportedHelper::DecimalDomain` at the rooted link after a permitted
   `TypeLinkComparison` event and before Decimal equality. Event denial or
   cancellation at that boundary SHALL retain [FR-109](./FR-109-bound-supplied-value-traversal.md)
   precedence. Nested Option and Collection links and later arguments SHALL
   reuse the same budget and cancellation handle. QSV SHALL preserve `no_std`
   and its unsafe prohibition.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-118-AC-1 | Public supplied admission compares equal and unequal Decimal links nested in Option and Collection, charging one type-link event per compared pair with the independently enumerated zero-based ordinal and argument-rooted locus. Define N as cumulative successful units through the target Decimal link event, including any earlier argument spend. With ceiling N-1, that exact event and locus are denied before helper entry; with N, that event succeeds and the helper may enter even if later events are denied. | Test |
| FR-118-AC-2 | With configured B, all four coefficient bounds at B complete. Each bound at B+1 yields typed unsupported size naming expected/actual and lower/upper, measured bits, B and rooted locus. Simultaneously oversized bounds refuse in expected-lower, expected-upper, actual-lower, actual-upper order. Zero counts as one bit, so B=1 admits it and B=0 refuses at expected lower. A scale or rounding-mode difference yields invalid type only after all four bounds pass. | Test |
| FR-118-AC-3 | Pre-cancellation retains the original cause and prior cumulative spend with zero new charge at the target type-link event. Cancellation during an admitted longest coefficient comparison retains the original cause and prior spend plus the one initiating event, with no next comparison or evaluator charge. Both stops retain the rooted link locus. | Test |
| FR-118-AC-4 | With no configured B, a permitted paired Decimal link returns `UnsupportedHelper::DecimalDomain` after its event without helper entry. Event denial and cancellation from the caller's original Cancel retain their established precedence. | Test |
| FR-118-AC-5 | Inspection of the published exact-kernel source confirms one authentic B, expected-first borrowed operands and original Cancel reach `DecimalType::try_eq_bounded`; four bound checks, at most 2ceil(B/32) fixed-width limb-pair steps, at most one endpoint check or limb-pair step between cancellation polls, and constant auxiliary storage are established without allocation or native-size conversion. No unbounded `ValueType` equality or QSV-owned duplicate comparator decides paired Decimal links. | Inspection |

## Dependencies

- [FR-109](./FR-109-bound-supplied-value-traversal.md) owns the supplied
  event, phase budget, cancellation and rooted locus.
- [FR-117](./FR-117-bound-retained-numeric-type-links.md) establishes the
  neighboring Int/Rational type-link boundary; its helper does not compare
  Decimal domains.
- [QSpec FR-140](ix://agent-ix/quire-specification/FR-140) defines the five
  declared Decimal type fields.
- [Exact FR-382](ix://agent-ix/quire-exact/FR-382), delivered by IR-799,
  owns the published bounded, cancellable `DecimalType::try_eq_bounded`
  operation. The typed frontmatter edge records this implementation
  dependency. QSV implementation and acceptance remain unrun.
- QSL-772 supplies the real public checked-call and replay consumer vector.

## Status

SPEC CANDIDATE; implementation and acceptance tests are unrun.
