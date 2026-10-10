---
id: FR-109
title: "Bound supplied-value membership with its own traversal budget"
type: FR
relationships: []
---
# FR-109: Bound supplied-value membership with its own traversal budget

## Description

When a caller checks a supplied runtime value against its expected type,
`quire-semantic-value` SHALL use a caller-owned cumulative supplied-membership
budget and return a typed admission-phase result, distinct from authored
declaration admission and semantic evaluation.

## Inputs

- A borrowed admitted `TypeEnvironment`, expected type, and supplied value.
- The caller's already-spent supplied-membership budget, configured logical-work
  ceiling, and original cancellation handle.
- Source-backed input and cost bounds for each indivisible helper reached by
  that membership path.

## Outputs

- Completed membership, with the cumulative successful work retained.
- Or a typed supplied-admission failure: invalid value/type, reached traversal
  ceiling, cancellation, measured worklist allocation denial, or capacity/size
  failure.

## Behavior

1. The supplied-membership interface SHALL borrow the expected type and value
   and accept the caller's mutable budget and cancellation handle without an
   evaluation `Meter`. Nested calls and subsequent arguments in the same
   supplied-admission invocation SHALL reuse that budget and handle. The traversal SHALL NOT instantiate a replacement budget or cancellation handle.
2. When a logical event below is requested, the traversal SHALL poll the original
   cancellation handle and request one unit from its supplied-membership
   budget. Successful work SHALL increase cumulative spend once. A denied
   event SHALL retain successful spend without performing the event or
   consuming its requested unit. Zero SHALL mean no available units.

| Event | Unit and boundary | Ownership |
| --- | --- | --- |
| Value visit | One actual occurrence visited to check its runtime kind and supplied membership. Repeated occurrences are separate visits. | Supplied membership. |
| Type comparison | One compared pair of type-chain links, including the initial expected/actual pair. An entry that compares that pair is this event, not an additional entry charge. | The phase that initiates the comparison. |
| Child scheduling | One actual child/payload occurrence scheduled for a required membership visit. No event exists for an unvisited child whose construction custody is trusted. | Supplied membership. |
| Descriptor query | One logical lookup in the admitted environment or ranked member shape, before its helper runs. Its internal comparison cost is governed by the helper bounds below. | The phase that initiates the lookup. |

3. The traversal SHALL charge the same logical event when a cache supplies its
   result. A cache SHALL NOT change the event count or first denied event for
   identical input, type environment and initial budget. Helper-internal limb
   and search comparisons SHALL NOT become semantic evaluation charges.
4. Membership SHALL retain the released construction-custody boundary:
   admitted Option, Composite and Collection values are checked against their
   retained type/declaration without inventing a complete descendant
   revalidation walk. Where the owning union contract requires payload
   membership or reference visits, the traversal SHALL perform those visits
   through the same cumulative budget. This requirement SHALL NOT create a
   union declaration shape or member API absent from its owning contract.
5. When an indivisible helper is requested, the supplied-membership path SHALL
   establish its actual input-size bound, finite comparison/arithmetic and
   temporary-storage bound, and cancellation-response boundary from the
   authoritative helper implementation. The supplied-membership interface SHALL NOT invoke an input-unbounded helper
   behind a nominal one-unit descriptor or scalar check. The implementation SHALL qualify those premises for the helper families in the table below
   before claiming that path satisfies this requirement. The implementation SHALL record missing qualification as an explicit
   implementation dependency without manufacturing an invalid-value or limit result.
6. A helper used by both conversion and supplied membership SHALL receive one
   explicit initiating phase owner. A membership-triggered type comparison or
   query SHALL consume the supplied-membership budget once; a
   conversion-triggered operation SHALL consume the conversion owner's
   budget once. Passing through the helper SHALL NOT debit both. Replay's final
   supplied-membership check SHALL use the same admission budget as its other
   admission checks. QSL owns conversion-node accounting, phase ceilings and
   consumer projection.
7. A traversal-limit failure SHALL preserve typed supplied-admission phase,
   configured ceiling, successful cumulative spend, next requested amount and
   logical event/locus. Cancellation SHALL have its own typed classification.
   Invalid input SHALL retain its membership cause and locus. The supplied-membership interface SHALL NOT project these results as an
   evaluated `FunctionCall`, authored-declaration failure, or fabricated semantic charge.
8. When traversal-worklist storage growth is requested, the traversal SHALL use fallible
   reservation and checked size arithmetic. An actual allocator denial SHALL
   retain the request and its native byte/additional-element unit; overflow SHALL have a distinct capacity/size classification. A stopped traversal SHALL publish no partial admission result.
   When an attempt is retried with its denied next unit available or its storage
   denial removed, the traversal SHALL use the same membership rules.
9. The supplied-membership API SHALL preserve the crate's `no_std` and unsafe
   prohibition. It SHALL NOT copy kernel/container algorithms to simulate
   hidden helper counters, change semantic evaluation accounting, raise a
   consumer allowance, reset successful counters, or bypass admission.

### Helper qualification table

These are cost premises, not extra logical-work charges or new input ceilings.
`B` denotes the largest admitted magnitude bit length of the involved integers;
`E` denotes admitted enum variants; `T` denotes compared type links; `R` denotes
registry entries; `A` denotes retained ancestors. Each premise must name its
actual caller/admission bound and authoritative helper version.

| Helper family | Released source behavior and finite-cost premise | Cancellation/storage qualification |
| --- | --- | --- |
| Fixed identity/kind | Boolean/Integer kind, Float width, Quantity unit, Composite key and reference identity compare fixed-size retained data. | Poll before and after the helper; no input-sized temporary materialization. |
| Integer interval | At most two integer comparisons. The locked BigInt comparison checks signs/lengths then at most the larger operand limb count, bounded conservatively by B. | Establish the bound for value and both interval endpoints; poll before and after. No numeric semantic charge. |
| Rational domain | At most four integer comparisons over numerator/denominator and interval endpoints; this released membership path does not cross-multiply rationals. | Bound every operand's magnitude, not merely the supplied numerator; poll before and after. |
| Enum member | Ranked lookup scans at most E retained variants and compares fixed-size variant identities. | Establish E from the admitted shape; a cache hit retains the logical query event. Poll around the bounded helper. |
| Nested retained type | At most T type-chain link comparisons; leaf comparisons include any input-sized retained enum/numeric data. | Charge type links through the bounded interface; bound leaf data separately. No outer one-event substitution for an entire unbounded type chain. |
| Reference conformance | The retained ancestry path uses registry lookups and a binary search over at most A ancestor positions; registry size is R. | Establish finite lookup-comparison bounds from the authoritative container implementation and actual admitted R/A. Poll around each bounded helper. A nominal query does not prove internal cost or cancellation latency. |
| Decimal domain | The released helper computes decimal digit counts; equal-digit comparisons can materialize an absolute coefficient multiplied by a power of ten. | Bit/scale bounds, temporary allocation and cancellation latency require a source-backed qualified helper. The released bool-only helper does not establish these premises. For coefficient 1 at scale 0 under interval [0,100] with min/max scale 2, radix-digit conversion and the equal-digit power/multiply branch allocate even with tiny bounded inputs. Allocation denial there has no typed result. Finite input bounds alone cannot establish fallible storage; this path remains unqualified until its owner resolves it. |
| Future union payload/descriptor | Public union declaration/member and bounded helper ordering must be supplied by their owning contracts. | Unpublished candidate bodies and predicted Tree counts are not qualification evidence. |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-109-AC-1 | A source-enumerated supported membership fixture requiring N logical events completes at N and fails at N-1 before the last event, retaining supplied-admission phase, configured N-1, successful spend N-1, next amount one and the actual denied event/locus. Zero denies the first event. No denied event is performed or spent. | Test |
| FR-109-AC-2 | Two arguments and a nested comparison share one already-spent budget. The second argument's denial identifies cumulative spend including the first and nesting; neither argument nor helper resets the budget. Warm and cold caches have identical logical counts and first-denial records. | Test |
| FR-109-AC-3 | Cancellation through the caller's original handle stops the next permitted response boundary with typed cancellation, no further traversal events and no replacement handle. Cancellation is not reported as work exhaustion or invalid input. | Test |
| FR-109-AC-4 | Invalid kind/type and member/payload failures permitted by the owning supported-family contracts return their typed membership cause and locus, never a partial admission. Released nested construction custody is preserved; a helper cannot manufacture extra payload visits. | Test |
| FR-109-AC-5 | An actual fallible supplied-worklist reservation denial reports its measured request and native unit; checked-size overflow has its own classification. Neither becomes a traversal limit, declaration-storage failure or invalid value. Retry after removing the denial yields the same membership outcome. | Test |
| FR-109-AC-6 | Every reachable logical event and shared helper has one initiating phase/budget owner. Membership leaves the evaluator's meter and charge sequence unchanged, and conversion-triggered helper work is never also charged as membership. No failure is labelled evaluated FunctionCall. | Test |
| FR-109-AC-7 | Each claimed supported helper path has a source-backed input bound, finite comparison/arithmetic and temporary-storage bound, cancellation response boundary and fallible storage route where needed. Unqualified Decimal and future union paths remain recorded dependencies; logical-event totals alone do not establish those helper premises. | Inspection |

## Dependencies

- [FR-108](./FR-108-report-registry-allocation-failure.md) owns authored
  declaration/registry storage failure. This supplied-runtime carrier does not
  revise that requirement or its implementation.
- [QSL FR-321](ix://agent-ix/quire-spec-language/FR-321) owns public supplied
  union admission; [QSL FR-322](ix://agent-ix/quire-spec-language/FR-322) owns
  semantic evaluation. The event/API contract must align with QSL-681 before
  consumer implementation; evaluation's existing charge schedule is unchanged.
- [QSL FR-098](ix://agent-ix/quire-spec-language/FR-098) retains ownership of
  per-argument converted-node and occurrence bounds, without evaluator debit.
  This requirement does not reinterpret converted nodes as membership events.
- QSL owns `supplied.admission_work_units`,
  `supplied.conversion_work_units`, defaults and public typed phase projection.
  The authoritative runtime request/result envelope is owned by
  [QSpec FR-323](ix://agent-ix/quire-specification/FR-323). QSL owns consumer
  projection and QSL-675 sequencing. Any needed phase-carrier amendment
  remains pending; no wire/transport declaration is created here.
- The released reference inspected is QSV
  `bd3e1500050bbd35fce8094794930d15327d05fd`, whose lock pins exact
  `dc7891740d04a3ff4c76c1315e94216f0bdc15ec` and num-bigint 0.4.8.
  QSV `TypeEnvironment::admits` is bool-only; its declaration WorkBudget is
  private and is not the new supplied traversal contract. Decimal helper
  qualification is allocated to IR-718 SPEC and IR-719 implementation in the
  authoritative exact-kernel owner. Both remain prerequisites for claiming
  this Decimal path; the pending design does not introduce a fifth helper-entry
  event or select a default. A reviewed public union declaration/member API
  remains another implementation prerequisite, not a claim of current support.

## Status

SPEC DRAFT; implementation and every acceptance test are PLANNED/UNRUN.
No numerical default is selected by this QSV requirement. The research
hypotheses 262,144 admission / 524,288 conversion and hosted Tree counts are
not qualified defaults, event-table evidence or runtime acceptance results.
