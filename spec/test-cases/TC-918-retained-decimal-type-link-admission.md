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
   Independently enumerate the type-link event count and rooted ordinal;
   run with N and N-1 cumulative units.
2. Configure B and put each of the expected/actual lower/upper coefficients
   at B and B+1 bits in valid Decimal domains. Check simultaneous oversized
   operands, zero at B=1 and B=0, and a metadata mismatch paired with an
   oversized endpoint. Observe the typed operand position, measured bits,
   configured B and rooted locus.
3. Pre-cancel the original handle and trigger cancellation on a late observed
   kernel limb comparison. Observe the cause, spent event count, locus and
   absence of further work. Exercise an actual kernel resource failure only
   if the helper has a reachable resource-denial route; inspect its typed
   request without substituting a synthetic QSV failure. Check evaluator
   charges before and after refusal.
4. Repeat a paired Decimal link through QSV's direct no-B admission path,
   with a permitted event, denied event and cancelled event. Inspect the
   production call for expected-first operands, original B and Cancel, and
   the absence of unbounded Decimal equality fallback.

## Expected Results

Equal types continue; each completed unequal type returns invalid type only
after all four coefficient bounds pass. Every size, cancellation, work-budget
and unsupported stop remains distinct at the charged type-link locus. No
helper enters after event denial, no no-B call invents a ceiling, and no
evaluation debit occurs on supplied-admission refusal.

## Status

PLANNED/UNRUN; the exact-kernel helper and QSV implementation are prerequisites.
