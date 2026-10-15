---
id: SR-11160
title: "Gap analysis of IR-798 retained Decimal type-link evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/declaration/membership.rs; Cargo.lock; FR-118-AC-1..5; TC-918; ticket IR-798"
review_set: subset
---

## Summary

The changed production path and its tests cover the FR-118 Decimal type-link boundary. The computed matrix tags the four Test criteria; the Inspection criterion was checked against the published exact source. No gap was found in this diff.

## Verdict

**PASS** — The scoped requirement, trace, source and test chain is complete.

## Scope examined

- `FR-118-AC-1` (examined): Nested Option and Collection equal and unequal Decimal links, zero-based type-link ordinal, rooted locus, cumulative N/N-1 event budget and later-event denial.
- `FR-118-AC-2` (examined): Four ordered endpoints at B and B+1, simultaneous oversize, zero at B=0/1, and metadata mismatch after bounds.
- `FR-118-AC-3` (examined): Original cause and rooted locus for pre-event and mid-comparison cancellation, preserving cumulative spend and avoiding extra charges.
- `FR-118-AC-4` (examined): Direct no-B admission refuses at a charged DecimalDomain link with denial and cancellation precedence.
- `FR-118-AC-5` (examined): Published helper source inspection for authentic B, expected-first operands, original Cancel, four ordered checks, bounded limb steps and constant storage.
- `TC-918` (examined): Declared public supplied-admission procedure and expected typed stops.
- `src/declaration/membership.rs` (examined): Production callback and traced tests for all four Test criteria.
- `Cargo.lock` (examined): Single published exact kernel pin at `d18927e`.
- Published exact `d18927e` source (context_only): The underlying helper's bound, cancellation and comparison implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found (placeholder) | - |

## Coverage

Plan completion: not assessed.

`quire matrix --scope . --format json` reports `FR-118-AC-1` through `FR-118-AC-4` as tagged to the new tests. `FR-118-AC-5` is classified `Inspection` and therefore `method-without-symbol`; direct inspection of the pinned exact source established the criterion. The five new tests assert the charged event and rooted part, cumulative N/N-1 spend, ordered typed size positions, B=0/1, metadata mismatch, pre-event and in-helper cancellation, and direct no-B refusal. Existing Int/Rational tests remain in the file; the production diff adds only the Decimal arm and changes the fallback order. QSL evaluator-charge evidence is a consumer concern outside this QSV diff.

`cargo test --locked bounded_decimal --lib` passed 7 tests. No full gate was run in this scoped review.
