---
id: SR-9119
title: IR-714 PR 23 code-review at 897a4f53
type: SpecReview
analysis: code-review
scope: agent-ix/quire-semantic-value; src/declaration.rs, src/declaration/membership.rs,
  src/supplied_membership.rs, spec/functional/FR-109-bound-supplied-value-traversal.md
review_set: subset
---

## Summary

Ticket: IR-714. Reviewed PR #23 at exact head 897a4f53f2d745cf2707810d6678c8b166a10def against base 60107c5f900a324bc9ee8746a85dbef3b02a0a17.

## Verdict

**FAIL** — an unbounded supported numeric helper remains reachable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Integer and rational leaf helpers run on caller-supplied unbounded BigInts between cancellation polls; FR-109-AC-7 helper input and response bounds are not established. | src/declaration/membership.rs:358 |

## Evidence

- FND-001: The public admit_supplied API accepts any ValueType and Value. ValueType::admits calls IntegerInterval::contains and RationalDomain::contains, each comparing BigInts of unrestricted magnitude. One ValueVisit can therefore spend arbitrarily long in a helper before the next Cancel poll. No admitted B bound or fallible route for that work is checked here.

## Examined scope

```yaml
scope:
- id: FR-109-AC-1
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: A source-enumerated supported membership fixture requiring N logical events
    completes at N and fails at N-1 before the last event, retaining supplied-admission
    phase, configured N-1, successful spend N-1, next amount one and the actual denied
    event/locus. Zero denies the first event. No denied event is performed or spent.
- id: FR-109-AC-2
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Two arguments and a nested comparison share one already-spent budget. The
    second argument's denial identifies cumulative spend including the first and nesting;
    neither argument nor helper resets the budget. Warm and cold caches have identical
    logical counts and first-denial records.
- id: FR-109-AC-3
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Cancellation through the caller's original handle stops the next permitted
    response boundary with typed cancellation, no further traversal events and no
    replacement handle. Cancellation is not reported as work exhaustion or invalid
    input.
- id: FR-109-AC-4
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Invalid kind/type and member/payload failures permitted by the owning supported-family
    contracts return their typed membership cause and locus, never a partial admission.
    Released nested construction custody is preserved; a helper cannot manufacture
    extra payload visits.
- id: FR-109-AC-5
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: An actual fallible supplied-worklist reservation denial reports its measured
    request and native unit; checked-size overflow has its own classification. Neither
    becomes a traversal limit, declaration-storage failure or invalid value. Retry
    after removing the denial yields the same membership outcome.
- id: FR-109-AC-6
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Every reachable logical event and shared helper has one initiating phase/budget
    owner. Membership leaves the evaluator's meter and charge sequence unchanged,
    and conversion-triggered helper work is never also charged as membership. No failure
    is labelled evaluated FunctionCall.
- id: FR-109-AC-7
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Each claimed supported helper path has a source-backed input bound, finite
    comparison/arithmetic and temporary-storage bound, cancellation response boundary
    and fallible storage route where needed. Unqualified Decimal and future union
    paths remain implementation dependencies; logical-event totals alone do not establish
    those helper premises.
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 9089a008, supplied admission returns typed NumericDomain unsupported before Int/Rational value or retained-type comparison helpers; plain Integer remains supported. |
