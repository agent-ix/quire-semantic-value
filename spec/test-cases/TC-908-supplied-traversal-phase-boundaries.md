---
id: TC-908
title: "Supplied membership retains its phase, cumulative work and original cancellation"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: verifies
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: verifies
---
# TC-908: Supplied membership retains its phase, cumulative work and original cancellation

## Description

Verify [FR-109](../functional/FR-109-bound-supplied-value-traversal.md) and [FR-122-AC-2/3](../functional/FR-122-supplied-membership-owner-obligations.md)
through the real bounded supplied-membership API after its supported-family
helpers are qualified. Source-enumerate the fixture's logical event sequence;
do not obtain the expected count by reading the counter being tested.

## Test Procedure

1. Use a supported fixture containing an outer membership visit, a nested
   retained-type comparison and a descriptor query. Independently enumerate
   its event order and N. Run with zero, N-1 and N, and inspect both the result
   and the actual event performed at the refusal boundary.
2. Check two arguments through one initially nonzero-spent budget. Add a
   nested helper invocation. Assert total cumulative successful spend and the
   second argument's denial locus. Repeat with cold and warm descriptor
   caches; compare counts and denial records. Mutating a helper to reset the
   budget or charge a cache hit differently must fail these assertions.
3. Cancel the original caller handle at a deterministic observed helper
   boundary. Assert typed cancellation before further events, with no work
   limit or invalid-value projection. A mutant using a fresh handle or
   reporting cancellation as exhaustion must fail.
4. Supply wrong-kind/type values and the invalid member/payload cases of
   each qualified family contract. Observe no partial admission. For released
   admitted Option/Composite/Collection inputs, assert the construction-custody
   boundary and actual logical event list; do not invent revalidation of all
   descendants. Include the implemented sealed-union cases; inspect their
   helper bounds and failure behavior against the public owning API.
5. Deny an actual fallible traversal-worklist reservation without replacing
   membership logic. Compare the reported allocation amount/unit with the
   real reservation request. Separately trigger checked representation/size
   overflow. Retry without denial and compare membership with the fresh run.
6. Exercise a membership-triggered shared helper through `src/supplied_membership.rs` with one cumulative supplied-admission budget. Assert each logical event is charged once to that budget, with the original cancellation handle and typed locus retained on a denied event. QSL FR-394 owns conversion and evaluator observations.
7. Inspect every claimed helper against its authoritative implementation and
   actual input bounds. Establish comparisons/arithmetic, peak temporary storage,
   cancellation response boundary and reservation route. For Decimal, check the
   actual `DecimalType::try_contains` source against [exact-kernel FR-372](ix://agent-ix/quire-exact/FR-372):
   the caller authenticates `B >= max(bits(c), bits(lo), bits(hi))`, with zero
   counted as one bit, and `S >=` the actual coefficient-alignment shift before
   the call. Confirm the one-request `4 × ceil(B / 32)`-byte scratch bound, the
   `2 × (min(S,B) + 2) × ceil(B / 32)` limb-step bound and original-handle polling.
   Fail caller qualification if either input bound or typed error projection is
   missing. An unpublished union candidate cannot qualify through a small
   logical event total.
8. Through the integrated QSV supplied-admission caller, use an independently
   enumerated Decimal value-visit fixture at exact N and N-1, followed by a
   second argument on the same budget. Assert inclusive membership and adjacent
   nonmembership against an independent oracle. Cancel the original handle at
   an observed boundary and assert its typed cause, Decimal locus, cumulative
   spend and no partial admission, extra helper-entry or per-limb event, or
   additional supplied-admission charge. Confirm a caller without source-backed `B` or `S` does not
   claim Decimal support or select a default.
9. Construct the typed exact-kernel capacity and allocation failure variants
   with distinct measured fields and pass each through QSV's production Decimal
   error-projection path at a rooted Decimal locus. Assert distinct
   supplied-admission causes, preserved measurements and locus, and no
   nonmembership, traversal-limit or evaluation projection. This tests QSV's
   mapping without replacing the kernel helper or claiming that the public
   `DecimalType::try_contains` call can inject an allocator denial or construct
   a synthetic capacity overflow. [Exact-kernel TC-920](ix://agent-ix/quire-exact/TC-920)
   owns the actual scratch-denial and checked-capacity mechanism controls.

## Expected Results

Exact N succeeds; N-1 and zero return the real denied supplied-admission event
with configured ceiling, successful spend and next request. Multi-argument and
nested calls preserve one cumulative budget and original cancellation. Caches
do not change logical accounting. Invalid/cancel/allocation/capacity outcomes
remain distinct, no partial result escapes, and supplied-admission accounting follows the independently enumerated events. A missing helper premise prevents qualification rather than
establishing supported behavior. Mutants for reset, fresh Cancel, duplicate supplied debit, false storage cause and partial success
are killed by the corresponding assertions. Integrated Decimal caller controls
retain the kernel's Boolean result and cancellation at the same rooted
supplied-admission locus without adding a logical event. Separate projection
controls retain typed kernel capacity/allocation measurements and causes;
actual scratch-denial and capacity mechanisms remain exact-kernel TC-920's
evidence.

## Status

The sealed-union QSV supplied-admission exact-N and N-1 controls are implemented
by `sealed_tree_10k_budgeted_admission_spends_exact_n_on_small_stack` and
`sealed_tree_10k_budgeted_admission_denies_last_event_on_small_stack` in
`tests/union_staging.rs`. They enumerate 60,002 logical events for a 10,000-link
Tree and check completion and the final denied descriptor event on a small stack.
Non-ignored QSV caller controls traced to FR-109-AC-1 through FR-109-AC-5
execute in `src/declaration/membership.rs`, `src/supplied_membership.rs`,
`src/declaration.rs`, `tests/union_staging.rs` and
`tests/bounded_numeric_reservation.rs`. The ignored
`supplied_path_arena_growth_probe_10k` is outside the default run. Integrated
Decimal controls traced to FR-109-AC-8 execute through
`admit_supplied_bounded_numeric`: they cover cumulative exact-N/N-1 and zero
budgets, size and nonmembership loci, original-handle cancellation, and typed
kernel resource-error projection. The actual kernel scratch-denial and checked-
capacity mechanisms remain exact-kernel TC-920's controls. Full deep Tree
defaults, value-occurrence limits, replay converted nodes and evaluation
exact/one-less parity remain QSL consumer
controls under QSL-778. Numerical default hypotheses and hosted source
predictions are not an oracle for N. The Decimal caller tests are traced to
FR-122-AC-2; bounded Integer/Rational and absent-B NumericDomain caller tests
are traced to FR-122-AC-3. The Decimal caller accepts an explicit
magnitude bound and derives its alignment-shift bound from the declared minimum
scale; it adds no fifth helper-entry event or numeric default. The union helper
premises still need inspection under FR-109-AC-7.
