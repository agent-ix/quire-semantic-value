---
id: TC-918
title: "Supplied Decimal type links preserve bounded identity and rooted stops"
type: TC
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-118
    type: verifies
---
# TC-918: Supplied Decimal type links preserve bounded identity and rooted stops

## Description

Verify [FR-118](../functional/FR-118-bound-retained-decimal-type-links.md)
through the public supplied-admission path after the exact-kernel bounded
Decimal type-equality helper is available.

## Test Procedure

1. Admit Set and Option values whose retained actual and expected Decimal
   links are equal, then vary lower bound, upper bound, minimum scale,
   maximum scale and rounding mode independently. Use two argument roots.
   Independently enumerate events through the target Decimal link, including
   prior argument spend. Define N as the cumulative spend immediately after
   that target event. With ceiling N-1, assert denial names that exact event
   and rooted locus before helper entry; with N, observe helper entry even
   if a later event is denied.
2. Configure B and put each of the expected/actual lower/upper coefficients
   at B and B+1 bits in valid Decimal domains. Check simultaneous oversized
   operands, zero at B=1 and B=0, and a metadata mismatch paired with an
   oversized endpoint. Observe the typed operand position, measured bits,
   configured B and rooted locus.
3. Pre-cancel the original handle before the target event and assert the
   prior cumulative spend is unchanged. Separately trigger cancellation on a
   late observed kernel limb pair after that event succeeds; assert prior
   spend plus one, the original cause and rooted locus, with no following
   pair or result. Observe the production helper's bound validation,
   fixed-width limb-pair count at B and 32-bit limb boundaries, cancellation
   response gap and temporary-storage use. Verify at most `2ceil(B / 32)`
   limb-pair steps and constant auxiliary storage for an allocation-free
   route; if the delivered kernel uses scratch instead, inspect its finite
   byte bound and execute its real typed checked-size/allocation denial and
   retry path. Check evaluator charges before and after refusal.
4. Repeat a paired Decimal link through QSV's direct no-B admission path,
   with a permitted event, denied event and cancelled event. Inspect the
   production call for expected-first operands, original B and Cancel, and
   the absence of unbounded Decimal equality fallback.

## Expected Results

Equal types continue; each completed unequal type returns invalid type only
after all four coefficient bounds pass. Every size, cancellation, work-budget
and unsupported stop remains distinct at the type-link locus, with spend
reflecting whether that event succeeded. No
helper enters after event denial, no no-B call invents a ceiling, and no
evaluation debit occurs on supplied-admission refusal.

## Status

PLANNED/UNRUN; an authored exact-kernel Decimal type-equality requirement,
its delivered helper and the QSV implementation are prerequisites.
