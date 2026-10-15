---
id: FR-117
title: "Bound retained numeric type-link comparison in supplied membership"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: references
  - target: ix://agent-ix/quire-exact/FR-381
    type: references
---
# FR-117: Bound retained numeric type-link comparison in supplied membership

## Description

When supplied membership compares retained `Int` or `Rational` type links,
`quire-semantic-value` SHALL use the caller's authenticated finite magnitude
ceiling `B` and the bounded cancellable equality operations owned by
[QExact FR-381](ix://agent-ix/quire-exact/FR-381). This comparison is distinct
from numeric value membership under QExact FR-380.

## Inputs

- Borrowed actual and expected type chains, the caller's cumulative
  supplied-membership work budget, rooted path arena and original `Cancel`.
- An authentic finite `B` in magnitude bits covering all four endpoints of
  any paired Integer intervals or all eight endpoints of any paired Rational
  domains reached by this invocation. Its source is the admitted or
  caller-owned magnitude carrier, not the logical-work ceiling, a value-only
  measurement, or an inferred default.

## Outputs

- Completed type-link comparison and subsequent membership under
  [FR-109](./FR-109-bound-supplied-value-traversal.md); or
- A typed supplied-admission failure at the compared link's rooted locus.

## Behavior

1. At each compared pair of retained type links, including the initial pair,
   QSV SHALL request exactly one `TypeLinkComparison` event before its action.
   It SHALL preserve the current left/actual and right/expected operand order,
   the existing zero-based link ordinal and the occurrence's argument-rooted
   path. A denied event SHALL prevent the equality helper from running and
   retain the caller's cumulative successful spend.
2. For paired `Int` links, QSV SHALL pass the borrowed left and right
   `IntegerInterval` values, the authentic `B` and original `Cancel` to
   QExact's bounded interval equality operation. For paired `Rational` links,
   it SHALL pass the borrowed left and right `RationalDomain` values to its
   bounded domain equality operation. Their endpoint order, 4/8 endpoint
   checks, finite comparison/storage envelope and cancellation points belong
   to [QExact FR-381](ix://agent-ix/quire-exact/FR-381). QSV SHALL NOT use
   unbounded `ValueType` equality or duplicate the numeric comparator.
3. A completed `true` SHALL continue the membership walk; a completed
   `false` SHALL return the established invalid-type cause at this type-link
   locus. A kernel size, cancellation, capacity or actual allocation stop SHALL
   remain distinct from invalid type and traversal `Limit`, retaining the
   kernel's typed position/request/cause and this same rooted locus. The
   QSV SHALL charge the initiating event once, including on a helper stop; limb
   steps and post-helper cancellation boundaries SHALL add no logical event or
   evaluator debit.
4. If the initiating supplied invocation has no authentic `B` covering both
   retained types, QSV SHALL retain `UnsupportedHelper::NumericDomain` at the
   type-link locus after the permitted `TypeLinkComparison` event and before
   numeric equality. No default, value-only bound or nominal one-unit budget
   qualifies the path. A work denial or cancellation at that event keeps the
   [FR-109](./FR-109-bound-supplied-value-traversal.md) precedence.
5. The caller SHALL reuse its original budget and cancellation handle through
   nested Option and Collection type chains and later arguments. QSV SHALL
   preserve `no_std` and its unsafe prohibition. Decimal and nonnumeric type
   comparisons retain their separately owned contracts.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-117-AC-1 | In Option and Collection chains with paired Int or Rational links, the independently enumerated event sequence charges exactly one `TypeLinkComparison` for each compared pair, with the original left/actual and right/expected order and the correct argument-rooted, zero-based link locus. At N-1 the denied event prevents helper entry; at N it can enter. | Test |
| FR-117-AC-2 | With authentic B, equal retained domains continue and a completed unequal domain returns invalid type at the compared link. Every one of the four Int and eight Rational endpoint positions at B completes; one-over-B on either side is projected as typed unsupported size with position, measured bits, B and rooted locus, never as unequal or traversal Limit. | Test |
| FR-117-AC-3 | Pre-cancellation and cancellation during an admitted longest numeric comparison return the original cause at the link locus before another step/result; cumulative successful spend includes the one initiating event only. A reachable kernel capacity or actual allocation denial retains its typed request and locus. Evaluator charges remain unchanged. | Test |
| FR-117-AC-4 | Without an authentic B, a permitted paired numeric type link returns `UnsupportedHelper::NumericDomain` at its rooted locus after one type-link event, without equality work. Event denial and cancellation from the caller's original `Cancel` retain their established precedence; no numeric default is introduced. | Test |
| FR-117-AC-5 | Inspection verifies that the QSV call invokes the separate QExact FR-381 equality operation, applies one authentic B to all 4/8 endpoints, and neither falls through to unbounded `ValueType` equality nor copies borrowed-limb algorithms. | Inspection |

## Dependencies

- [FR-109](./FR-109-bound-supplied-value-traversal.md) owns the supplied
  logical event, phase budget, cancellation, locus and general failure rules.
- [QExact FR-381](ix://agent-ix/quire-exact/FR-381) owns the separate retained
  numeric type equality kernel. [QExact FR-380](ix://agent-ix/quire-exact/FR-380)
  qualifies numeric *value* membership, whose three/six operands do not cover
  these four/eight retained type endpoints.
- QSL-779 owns the structured consumer blocker and authentic `B` carrier.
  A proposed QSpec carrier amendment remains under review; this requirement
  assumes neither its default nor its event-order resolution. Until the
  authentic carrier and QExact helper are available, NumericDomain stays
  unsupported for this type-link path.

## Status

SPEC CANDIDATE; implementation and acceptance tests are unrun.
