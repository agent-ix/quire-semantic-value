---
id: SR-9120
title: IR-714 PR 23 gap-analysis at 897a4f53
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-semantic-value; src/declaration.rs, src/declaration/membership.rs,
  src/supplied_membership.rs, spec/functional/FR-109-bound-supplied-value-traversal.md
review_set: subset
---

## Summary

Ticket: IR-714. Reviewed PR #23 at exact head 897a4f53f2d745cf2707810d6678c8b166a10def against base 60107c5f900a324bc9ee8746a85dbef3b02a0a17.

## Verdict

**CONDITIONAL** — three acceptance-evidence gaps remain. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-109-AC-4 tests do not exercise supported Enum member or Integer/Rational payload failures through admit_supplied. | src/declaration/membership.rs:739 |
| FND-002 | medium | FR-109-AC-5 capacity test only maps a fabricated StorageFailure::Capacity; no checked-size overflow is triggered at a supplied worklist boundary. | src/declaration/membership.rs:788 |
| FND-003 | medium | FR-109-AC-6 has no tagged test for evaluator meter preservation, conversion ownership, or FunctionCall classification. | src/declaration.rs:2142 |

## Evidence

- FND-001: The tagged cases assert Kind and Type, and a supported kind mismatch, but none constructs an Enum rank miss or numeric out-of-domain input and checks InvalidMembershipCause::Member/Payload and locus through the new public API.
- FND-002: The test calls storage(StorageFailure::Capacity(request), at) directly. It cannot fail if checked_add or Layout::array is removed from the actual pending or path arena reservation code.
- FND-003: The computed Quire matrix reports FR-109-AC-6 untagged; current public API tests only inspect supplied budget/error state, leaving the cross-phase assertions unverified in this PR.

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

## Coverage

Quire matrix: FR-109-AC-6 untagged; FR-109-AC-7 is Inspection and therefore method-without-symbol. All other FR-109 criteria have tags, which were checked for assertion strength. Plan completion: not assessed.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | Public Enum Member failure is now tested. Int/Rational Payload remains outside the qualified supported subset: supplied admission returns NumericDomain unsupported before the helper. Revisit after numeric helper qualification. |
| FND-002 | fixed | The new Frontier::reserve test triggers actual checked-size overflow with usize::MAX and asserts Capacity request, locus, and no reservation callback or child. |
| FND-003 | deferred | QSL-772 owns the checked-call/replay consumer and cross-phase evaluator/conversion evidence; this QSV-only fix cannot establish evaluator meter preservation or FunctionCall classification. |
