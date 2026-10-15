---
id: SR-9125
title: "IR-793 spec-review/integrity review of retained numeric type-link admission"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value; spec/functional/FR-117-bound-retained-numeric-type-links.md; spec/test-cases/TC-917-retained-numeric-type-link-admission.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-117
    type: references
---

## Summary

Reviewed frozen PR #25 at 5acae91 for IR-793. Examined FR-117 and TC-917 against QSV FR-109/TC-908 and the related unmerged QExact FR-381/TC-929 proposal.

## Verdict

**FAIL** — 1 defect(s) in this analysis. No implementation or runtime acceptance is claimed before the QSL-779 carrier and QExact helper land.

## Examined scope

- `FR-117` (examined): When supplied membership compares retained `Int` or `Rational` type links, `quire-semantic-value` SHALL use the caller's authenticated finite magnitude ceiling `B` and the bounded cancellable equality operations owned by [QExact FR-381](ix://agent-ix/quire-exact/FR-381). This comparison is distinct from numeric value membership under QExact FR-380.
- `TC-917` (examined): Verify [FR-117](../functional/FR-117-bound-retained-numeric-type-links.md) through QSV's real supplied-membership path after the authentic caller `B` carrier and QExact bounded retained-type equality helper are available.
- `FR-117-AC-1` (examined): In Option and Collection chains with paired Int or Rational links, the independently enumerated event sequence charges exactly one `TypeLinkComparison` for each compared pair, with the original left/actual and right/expected order and the correct argument-rooted, zero-based link locus. At N-1 the denied event prevents helper entry; at N it can enter.
- `FR-117-AC-2` (examined): With authentic B, equal retained domains continue and a completed unequal domain returns invalid type at the compared link. Every one of the four Int and eight Rational endpoint positions at B completes; one-over-B on either side is projected as typed unsupported size with position, measured bits, B and rooted locus, never as unequal or traversal Limit.
- `FR-117-AC-3` (examined): Pre-cancellation and cancellation during an admitted longest numeric comparison return the original cause at the link locus before another step/result; cumulative successful spend includes the one initiating event only. A reachable kernel capacity or actual allocation denial retains its typed request and locus. Evaluator charges remain unchanged.
- `FR-117-AC-4` (examined): Without an authentic B, a permitted paired numeric type link returns `UnsupportedHelper::NumericDomain` at its rooted locus after one type-link event, without equality work. Event denial and cancellation from the caller's original `Cancel` retain their established precedence; no numeric default is introduced.
- `FR-117-AC-5` (examined): Inspection verifies that the QSV call invokes the separate QExact FR-381 equality operation, applies one authentic B to all 4/8 endpoints, and neither falls through to unbounded `ValueType` equality nor copies borrowed-limb algorithms.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The Inputs and Behavior 4 define an authentic B as covering both retained types. A valid caller ceiling B with one endpoint at B+1 would then be treated as missing qualification and return NumericDomain, while AC-2 requires typed unsupported size. Define authenticity independently of endpoint compliance and always pass a present authentic B to the helper for its ordered endpoint checks. | spec/functional/FR-117-bound-retained-numeric-type-links.md:26-29,62-66 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR #25 head 0551175; revised text below resolves the original defect. |
