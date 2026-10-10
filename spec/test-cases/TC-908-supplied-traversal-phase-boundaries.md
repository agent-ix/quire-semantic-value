---
id: TC-908
title: "Supplied membership retains its phase, cumulative work and original cancellation"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: verifies
---
# TC-908: Supplied membership retains its phase, cumulative work and original cancellation

## Description

Verify [FR-109](../functional/FR-109-bound-supplied-value-traversal.md)
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
   descendants. Add union cases only after the public owning union API and
   helper table are reviewed and available.
5. Deny an actual fallible traversal-worklist reservation without replacing
   membership logic. Compare the reported allocation amount/unit with the
   real reservation request. Separately trigger checked representation/size
   overflow. Retry without denial and compare membership with the fresh run.
6. Observe an external evaluator meter before and after membership and inspect
   the supplied result. Assert no semantic debit or evaluated FunctionCall
   failure. Exercise a shared helper initiated once from conversion and once
   from admission with their distinct caller-owned budgets; each call spends
   only its initiating budget, with no duplicate phase charge.
7. Inspect every claimed helper against its authoritative implementation and
   actual input bounds. Record comparisons/arithmetic, peak temporary storage,
   cancellation response boundary and reservation route. Fail qualification
   if any premise is missing. In particular, a bool-only Decimal helper with
   allocating radix/power operations and no cancellation/storage premise,
   or an unpublished union candidate, cannot qualify through a small logical
   event total.

## Expected Results

Exact N succeeds; N-1 and zero return the real denied supplied-admission event
with configured ceiling, successful spend and next request. Multi-argument and
nested calls preserve one cumulative budget and original cancellation. Caches
do not change logical accounting. Invalid/cancel/allocation/capacity outcomes
remain distinct, no partial result escapes, and evaluation accounting is
unchanged. Qualification records expose missing helper premises rather than
reporting supported behavior. Mutants for reset, fresh Cancel, fabricated
FunctionCall, duplicate phase debit, false storage cause and partial success
are killed by the corresponding assertions.

## Status

PLANNED/UNRUN. No implementation or test execution is claimed. The full deep
Tree defaults, occurrences, replay converted nodes and evaluation exact/one-less
parity controls belong to the aligned QSL consumer fixtures, not this QSV-only
case. Numerical default hypotheses and hosted source predictions are not an
oracle for N. Implementation awaits the aligned QSL phase carrier/contracts,
Decimal helper qualification under IR-718 SPEC and IR-719 implementation,
and the reviewed union API where applicable. The pending Decimal design adds
no fifth helper-entry event to this case and supplies no test-pass evidence.
