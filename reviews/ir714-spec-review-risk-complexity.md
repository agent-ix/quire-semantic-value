---
id: SR-5326
title: risk-complexity review of IR-714 supplied membership draft
type: SpecReview
analysis: risk-complexity
scope: agent-ix/quire-semantic-value@1f9f9aaf48f9f95b8d8efab84bee78809f2f81e9; spec/functional/FR-109-bound-supplied-value-traversal.md;
  spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
review_set: subset
---

## Summary

Ticket: IR-714. FR-109 technical risk is high for bounded indivisible helpers and fallible storage; volatility is medium from pending owner contracts. Mitigations are source-enumerated oracles, storage fault injection, original-handle cancellation mutants and qualification before support. Top hazards are Decimal hidden allocation, carrier/consumer alignment and unpublished union support; each is an explicit prerequisite. Failure-domain cross-check found no additional changed-scope omission.

## Verdict

**PASS** — no defect found in the frozen normative draft. No implementation, helper-qualification, numerical default or runtime acceptance credit is granted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-109-statement-1
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: When a caller checks a supplied runtime value against its expected type,
- id: FR-109-statement-2
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '`quire-semantic-value` SHALL use a caller-owned cumulative supplied-membership'
- id: FR-109-statement-3
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: budget and return a typed admission-phase result, distinct from authored
- id: FR-109-statement-4
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: declaration admission and semantic evaluation.
- id: FR-109-statement-5
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- A borrowed admitted `TypeEnvironment`, expected type, and supplied value.'
- id: FR-109-statement-6
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- The caller''s already-spent supplied-membership budget, configured logical-work'
- id: FR-109-statement-7
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  ceiling, and original cancellation handle.'
- id: FR-109-statement-8
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- Source-backed input and cost bounds for each indivisible helper reached
    by'
- id: FR-109-statement-9
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  that membership path.'
- id: FR-109-statement-10
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- Completed membership, with the cumulative successful work retained.'
- id: FR-109-statement-11
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- Or a typed supplied-admission failure: invalid value/type, reached traversal'
- id: FR-109-statement-12
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  ceiling, cancellation, measured worklist allocation denial, or capacity/size'
- id: FR-109-statement-13
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  failure.'
- id: FR-109-statement-14
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 1. The supplied-membership interface SHALL borrow the expected type and
    value
- id: FR-109-statement-15
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   and accept the caller''s mutable budget and cancellation handle without
    an'
- id: FR-109-statement-16
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   evaluation `Meter`. Nested calls and subsequent arguments in the same'
- id: FR-109-statement-17
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   supplied-admission invocation SHALL reuse that budget and handle. The
    traversal SHALL NOT instantiate a replacement budget or cancellation handle.'
- id: FR-109-statement-18
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 2. When a logical event below is requested, the traversal SHALL poll the
    original
- id: FR-109-statement-19
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   cancellation handle and request one unit from its supplied-membership'
- id: FR-109-statement-20
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   budget. Successful work SHALL increase cumulative spend once. A denied'
- id: FR-109-statement-21
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   event SHALL retain successful spend without performing the event or'
- id: FR-109-statement-22
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   consuming its requested unit. Zero SHALL mean no available units.'
- id: FR-109-statement-23
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Value visit | One actual occurrence visited to check its runtime kind
    and supplied membership. Repeated occurrences are separate visits. | Supplied
    membership. |'
- id: FR-109-statement-24
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Type comparison | One compared pair of type-chain links, including the
    initial expected/actual pair. An entry that compares that pair is this event,
    not an additional entry charge. | The phase that initiates the comparison. |'
- id: FR-109-statement-25
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Child scheduling | One actual child/payload occurrence scheduled for
    a required membership visit. No event exists for an unvisited child whose construction
    custody is trusted. | Supplied membership. |'
- id: FR-109-statement-26
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Descriptor query | One logical lookup in the admitted environment or
    ranked member shape, before its helper runs. Its internal comparison cost is governed
    by the helper bounds below. | The phase that initiates the lookup. |'
- id: FR-109-statement-27
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 3. The traversal SHALL charge the same logical event when a cache supplies
    its
- id: FR-109-statement-28
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   result. A cache SHALL NOT change the event count or first denied event
    for'
- id: FR-109-statement-29
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   identical input, type environment and initial budget. Helper-internal
    limb'
- id: FR-109-statement-30
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   and search comparisons SHALL NOT become semantic evaluation charges.'
- id: FR-109-statement-31
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '4. Membership SHALL retain the released construction-custody boundary:'
- id: FR-109-statement-32
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   admitted Option, Composite and Collection values are checked against
    their'
- id: FR-109-statement-33
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   retained type/declaration without inventing a complete descendant'
- id: FR-109-statement-34
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   revalidation walk. Where the owning union contract requires payload'
- id: FR-109-statement-35
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   membership or reference visits, the traversal SHALL perform those visits'
- id: FR-109-statement-36
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   through the same cumulative budget. This requirement SHALL NOT create
    a'
- id: FR-109-statement-37
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   union declaration shape or member API absent from its owning contract.'
- id: FR-109-statement-38
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 5. When an indivisible helper is requested, the supplied-membership path
    SHALL
- id: FR-109-statement-39
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   establish its actual input-size bound, finite comparison/arithmetic
    and'
- id: FR-109-statement-40
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   temporary-storage bound, and cancellation-response boundary from the'
- id: FR-109-statement-41
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   authoritative helper implementation. The supplied-membership interface
    SHALL NOT invoke an input-unbounded helper'
- id: FR-109-statement-42
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   behind a nominal one-unit descriptor or scalar check. The implementation
    SHALL qualify those premises for the helper families in the table below'
- id: FR-109-statement-43
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   before claiming that path satisfies this requirement. The implementation
    SHALL treat missing qualification as an implementation'
- id: FR-109-statement-44
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   dependency without manufacturing an invalid-value or limit result.'
- id: FR-109-statement-45
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 6. A helper used by both conversion and supplied membership SHALL receive
    one
- id: FR-109-statement-46
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   explicit initiating phase owner. A membership-triggered type comparison
    or'
- id: FR-109-statement-47
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   query SHALL consume the supplied-membership budget once; a'
- id: FR-109-statement-48
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   conversion-triggered operation SHALL consume the conversion owner''s'
- id: FR-109-statement-49
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   budget once. Passing through the helper SHALL NOT debit both. Replay''s
    final'
- id: FR-109-statement-50
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   supplied-membership check SHALL use the same admission budget as its
    other'
- id: FR-109-statement-51
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   admission checks. QSL owns conversion-node accounting, phase ceilings
    and'
- id: FR-109-statement-52
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   consumer projection.'
- id: FR-109-statement-53
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 7. A traversal-limit failure SHALL preserve typed supplied-admission phase,
- id: FR-109-statement-54
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   configured ceiling, successful cumulative spend, next requested amount
    and'
- id: FR-109-statement-55
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   logical event/locus. Cancellation SHALL have its own typed classification.'
- id: FR-109-statement-56
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   Invalid input SHALL retain its membership cause and locus. The supplied-membership
    interface SHALL NOT project these results as an'
- id: FR-109-statement-57
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   evaluated `FunctionCall`, authored-declaration failure, or fabricated
    semantic charge.'
- id: FR-109-statement-58
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 8. When traversal-worklist storage growth is requested, the traversal SHALL
    use fallible
- id: FR-109-statement-59
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   reservation and checked size arithmetic. An actual allocator denial
    SHALL'
- id: FR-109-statement-60
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   retain the request and its native byte/additional-element unit; overflow
    SHALL have a distinct capacity/size classification. A stopped traversal SHALL
    publish no partial admission result.'
- id: FR-109-statement-61
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   When an attempt is retried with its denied next unit available or its
    storage'
- id: FR-109-statement-62
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   denial removed, the traversal SHALL use the same membership rules.'
- id: FR-109-statement-63
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 9. The supplied-membership API SHALL preserve the crate's `no_std` and
    unsafe
- id: FR-109-statement-64
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   prohibition. It SHALL NOT copy kernel/container algorithms to simulate'
- id: FR-109-statement-65
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   hidden helper counters, change semantic evaluation accounting, raise
    a'
- id: FR-109-statement-66
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '   consumer allowance, reset successful counters, or bypass admission.'
- id: FR-109-statement-67
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: These are cost premises, not extra logical-work charges or new input ceilings.
- id: FR-109-statement-68
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '`B` denotes the largest admitted magnitude bit length of the involved
    integers;'
- id: FR-109-statement-69
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '`E` denotes admitted enum variants; `T` denotes compared type links; `R`
    denotes'
- id: FR-109-statement-70
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: registry entries; `A` denotes retained ancestors. Each premise must identify
    the applicable
- id: FR-109-statement-71
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: caller/admission input bound and the helper operation it constrains.
- id: FR-109-statement-72
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Fixed identity/kind | Boolean/Integer kind, Float width, Quantity unit,
    Composite key and reference identity compare fixed-size retained data. | Poll
    before and after the helper; no input-sized temporary materialization. |'
- id: FR-109-statement-73
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Integer interval | At most two integer comparisons. The BigInt comparison
    checks signs/lengths then at most the larger operand limb count, bounded conservatively
    by B. | Establish the bound for value and both interval endpoints; poll before
    and after. No numeric semantic charge. |'
- id: FR-109-statement-74
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Rational domain | At most four integer comparisons over numerator/denominator
    and interval endpoints; this released membership path does not cross-multiply
    rationals. | Bound every operand''s magnitude, not merely the supplied numerator;
    poll before and after. |'
- id: FR-109-statement-75
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Enum member | Ranked lookup scans at most E retained variants and compares
    fixed-size variant identities. | Establish E from the admitted shape; a cache
    hit retains the logical query event. Poll around the bounded helper. |'
- id: FR-109-statement-76
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Nested retained type | At most T type-chain link comparisons; leaf comparisons
    include any input-sized retained enum/numeric data. | Charge type links through
    the bounded interface; bound leaf data separately. No outer one-event substitution
    for an entire unbounded type chain. |'
- id: FR-109-statement-77
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Reference conformance | The retained ancestry path uses registry lookups
    and a binary search over at most A ancestor positions; registry size is R. | Establish
    finite lookup-comparison bounds from the authoritative container implementation
    and actual admitted R/A. Poll around each bounded helper. A nominal query does
    not prove internal cost or cancellation latency. |'
- id: FR-109-statement-78-part-1
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Decimal domain | The released helper computes decimal digit counts;
    equal-digit comparisons can materialize an absolute coefficient multiplied by
    a power of ten.'
- id: FR-109-statement-78-part-2
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Bit/scale bounds, temporary allocation and cancellation latency require
    a source-backed qualified helper.'
- id: FR-109-statement-78-part-3
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: The released bool-only helper does not establish these premises.
- id: FR-109-statement-78-part-4
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: For coefficient 1 at scale 0 under interval [0,100] with min/max scale
    2, radix-digit conversion and the equal-digit power/multiply branch allocate even
    with tiny bounded inputs.
- id: FR-109-statement-78-part-5
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Allocation denial there has no typed result.
- id: FR-109-statement-78-part-6
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Finite input bounds alone cannot establish fallible storage; this path
    remains unqualified until its owner resolves it.
- id: FR-109-statement-78-part-7
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '|'
- id: FR-109-statement-79
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '| Future union payload/descriptor | Public union declaration/member and
    bounded helper ordering must be supplied by their owning contracts. | Unpublished
    candidate bodies and predicted Tree counts are not qualification evidence. |'
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
- id: FR-109-statement-87
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- [FR-108](./FR-108-report-registry-allocation-failure.md) owns authored'
- id: FR-109-statement-88
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  declaration/registry storage failure. This supplied-runtime carrier
    does not'
- id: FR-109-statement-89
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  revise that requirement or its implementation.'
- id: FR-109-statement-90
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- [QSL FR-321](ix://agent-ix/quire-spec-language/FR-321) owns public supplied'
- id: FR-109-statement-91
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  union admission; [QSL FR-322](ix://agent-ix/quire-spec-language/FR-322)
    owns'
- id: FR-109-statement-92
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  semantic evaluation. The event/API contract must align with QSL-681
    before'
- id: FR-109-statement-93
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  consumer implementation; evaluation''s existing charge schedule is unchanged.'
- id: FR-109-statement-94
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- [QSL FR-098](ix://agent-ix/quire-spec-language/FR-098) retains ownership
    of'
- id: FR-109-statement-95
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  per-argument converted-node and occurrence bounds, without evaluator
    debit.'
- id: FR-109-statement-96
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  This requirement does not reinterpret converted nodes as membership
    events.'
- id: FR-109-statement-97
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- QSL owns `supplied.admission_work_units`,'
- id: FR-109-statement-98
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  `supplied.conversion_work_units`, defaults and public typed phase projection.'
- id: FR-109-statement-99
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  The authoritative runtime request/result envelope is owned by'
- id: FR-109-statement-100
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  [QSpec FR-323](ix://agent-ix/quire-specification/FR-323). QSL owns consumer'
- id: FR-109-statement-101
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  projection and QSL-675 sequencing. Any needed phase-carrier amendment'
- id: FR-109-statement-102
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  remains pending; no wire/transport declaration is created here.'
- id: FR-109-statement-103
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- QSV''s existing `TypeEnvironment::admits` is bool-only; its authored-declaration'
- id: FR-109-statement-104
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  work budget is distinct from the supplied traversal contract. Decimal
    helper'
- id: FR-109-statement-105
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  qualification is allocated to IR-718 SPEC and IR-719 implementation
    in the'
- id: FR-109-statement-106
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  authoritative exact-kernel owner. Both remain prerequisites for claiming'
- id: FR-109-statement-107
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  this Decimal path; the pending design does not introduce a fifth helper-entry'
- id: FR-109-statement-108
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  event or select a default. A reviewed public union declaration/member
    API'
- id: FR-109-statement-109
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  remains another implementation prerequisite, not a claim of current
    support.'
- id: FR-109-statement-110
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '- QSL-681 owns the paired authoritative carrier amendment and consumer
    phase'
- id: FR-109-statement-111
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  contract. Implementation awaits their merge and confirmation of the
    resulting'
- id: FR-109-statement-112
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: '  public behavior; this requirement does not assume those amendments are
    delivered.'
- id: FR-109-statement-113
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: SPEC DRAFT; implementation and every acceptance test are PLANNED/UNRUN.
- id: FR-109-statement-114
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: No numerical default is selected by this QSV requirement. Research predictions
- id: FR-109-statement-115
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: are not qualified defaults, event-table evidence or runtime acceptance
    results.
- id: TC-908-statement-1
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: Verify [FR-109](../functional/FR-109-bound-supplied-value-traversal.md)
- id: TC-908-statement-2
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: through the real bounded supplied-membership API after its supported-family
- id: TC-908-statement-3
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: helpers are qualified. Source-enumerate the fixture's logical event sequence;
- id: TC-908-statement-4
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: do not obtain the expected count by reading the counter being tested.
- id: TC-908-statement-5
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 1. Use a supported fixture containing an outer membership visit, a nested
- id: TC-908-statement-6
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   retained-type comparison and a descriptor query. Independently enumerate'
- id: TC-908-statement-7
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   its event order and N. Run with zero, N-1 and N, and inspect both the
    result'
- id: TC-908-statement-8
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   and the actual event performed at the refusal boundary.'
- id: TC-908-statement-9
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 2. Check two arguments through one initially nonzero-spent budget. Add
    a
- id: TC-908-statement-10
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   nested helper invocation. Assert total cumulative successful spend
    and the'
- id: TC-908-statement-11
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   second argument''s denial locus. Repeat with cold and warm descriptor'
- id: TC-908-statement-12
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   caches; compare counts and denial records. Mutating a helper to reset
    the'
- id: TC-908-statement-13
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   budget or charge a cache hit differently must fail these assertions.'
- id: TC-908-statement-14
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 3. Cancel the original caller handle at a deterministic observed helper
- id: TC-908-statement-15
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   boundary. Assert typed cancellation before further events, with no
    work'
- id: TC-908-statement-16
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   limit or invalid-value projection. A mutant using a fresh handle or'
- id: TC-908-statement-17
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   reporting cancellation as exhaustion must fail.'
- id: TC-908-statement-18
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 4. Supply wrong-kind/type values and the invalid member/payload cases of
- id: TC-908-statement-19
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   each qualified family contract. Observe no partial admission. For released'
- id: TC-908-statement-20
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   admitted Option/Composite/Collection inputs, assert the construction-custody'
- id: TC-908-statement-21
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   boundary and actual logical event list; do not invent revalidation
    of all'
- id: TC-908-statement-22
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   descendants. Add union cases only after the public owning union API
    and'
- id: TC-908-statement-23
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   helper table are reviewed and available.'
- id: TC-908-statement-24
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 5. Deny an actual fallible traversal-worklist reservation without replacing
- id: TC-908-statement-25
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   membership logic. Compare the reported allocation amount/unit with
    the'
- id: TC-908-statement-26
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   real reservation request. Separately trigger checked representation/size'
- id: TC-908-statement-27
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   overflow. Retry without denial and compare membership with the fresh
    run.'
- id: TC-908-statement-28
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 6. Observe an external evaluator meter before and after membership and
    inspect
- id: TC-908-statement-29
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   the supplied result. Assert no semantic debit or evaluated FunctionCall'
- id: TC-908-statement-30
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   failure. Exercise a shared helper initiated once from conversion and
    once'
- id: TC-908-statement-31
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   from admission with their distinct caller-owned budgets; each call
    spends'
- id: TC-908-statement-32
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   only its initiating budget, with no duplicate phase charge.'
- id: TC-908-statement-33
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: 7. Inspect every claimed helper against its authoritative implementation
    and
- id: TC-908-statement-34
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   actual input bounds. Establish comparisons/arithmetic, peak temporary
    storage,'
- id: TC-908-statement-35
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   cancellation response boundary and reservation route. Fail qualification'
- id: TC-908-statement-36
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   if any premise is missing. In particular, a bool-only Decimal helper
    with'
- id: TC-908-statement-37
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   allocating radix/power operations and no cancellation/storage premise,'
- id: TC-908-statement-38
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   or an unpublished union candidate, cannot qualify through a small logical'
- id: TC-908-statement-39
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: '   event total.'
- id: TC-908-statement-40
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: Exact N succeeds; N-1 and zero return the real denied supplied-admission
    event
- id: TC-908-statement-41
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: with configured ceiling, successful spend and next request. Multi-argument
    and
- id: TC-908-statement-42
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: nested calls preserve one cumulative budget and original cancellation.
    Caches
- id: TC-908-statement-43
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: do not change logical accounting. Invalid/cancel/allocation/capacity outcomes
- id: TC-908-statement-44
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: remain distinct, no partial result escapes, and evaluation accounting is
- id: TC-908-statement-45
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: unchanged. A missing helper premise prevents qualification rather than
- id: TC-908-statement-46
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: establishing supported behavior. Mutants for reset, fresh Cancel, fabricated
- id: TC-908-statement-47
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: FunctionCall, duplicate phase debit, false storage cause and partial success
- id: TC-908-statement-48
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: are killed by the corresponding assertions.
- id: TC-908-statement-49
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: PLANNED/UNRUN. No implementation or test execution is claimed. The full
    deep
- id: TC-908-statement-50
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: Tree defaults, occurrences, replay converted nodes and evaluation exact/one-less
- id: TC-908-statement-51
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: parity controls belong to the aligned QSL consumer fixtures, not this QSV-only
- id: TC-908-statement-52
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: case. Numerical default hypotheses and hosted source predictions are not
    an
- id: TC-908-statement-53
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: oracle for N. Implementation awaits the aligned authoritative phase carrier
    and QSL consumer
- id: TC-908-statement-54
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: contracts,
- id: TC-908-statement-55
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: Decimal helper qualification under IR-718 SPEC and IR-719 implementation,
- id: TC-908-statement-56
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: and the reviewed union API where applicable. The pending Decimal design
    adds
- id: TC-908-statement-57
  path: spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md
  role: examined
  excerpt: no fifth helper-entry event to this case and supplies no test-pass evidence.
```
