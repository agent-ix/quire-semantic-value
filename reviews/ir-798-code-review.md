---
id: SR-11159
title: "Code and Rust review of IR-798 retained Decimal type links"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; Cargo.lock; src/declaration/membership.rs; FR-118-AC-1..5; TC-918; ticket IR-798"
review_set: subset
---

## Summary

Reviewed frozen head `172dd4436c493382ab779e9aa852be08acfce3de` against QSV main `dd9953a`, including the Rust lane. The bounded supplied path delegates paired Decimal type equality to the published exact kernel at `d18927e`; the unbounded path keeps its charged unsupported boundary. No code or Rust defect was found in this diff.

## Verdict

**PASS** — The changed code and focused Decimal tests agree with the examined requirements.

## Scope examined

- `FR-118-AC-1` (examined): Public supplied admission compares equal and unequal Decimal links nested in Option and Collection, charging one type-link event per compared pair with the independently enumerated zero-based ordinal and argument-rooted locus. N-1 denies the target event; N admits it.
- `FR-118-AC-2` (examined): Four Decimal coefficient bounds use the configured B, report the first oversized endpoint in expected-lower, expected-upper, actual-lower, actual-upper order, count zero as one bit, and compare scale and rounding metadata only after bound checks.
- `FR-118-AC-3` (examined): Original cancellation cause and rooted link locus survive pre-event and during-helper cancellation with the appropriate prior and event spend.
- `FR-118-AC-4` (examined): Without B, paired Decimal links stop as DecimalDomain after a permitted event, subject to event denial and cancellation precedence.
- `FR-118-AC-5` (examined): Published exact source receives expected then actual borrowed Decimal types, one B and original Cancel; its four endpoint checks and at most two B/32-ceiling limb comparisons poll cancellation and use constant auxiliary storage.
- `TC-918` (examined): Public supplied-admission test procedure for nested links, bounds, cancellation and no-B refusal.
- `src/declaration/membership.rs` (examined): Callback order, Decimal bounded dispatch, typed error projection, prior Int/Rational behavior, and five new traced tests.
- `Cargo.lock` (examined): One quire-exact package entry pins published `d18927e849e8b575113fbeffb9041f5f8812d97a`.
- Published `quire-exact` `d18927e` `src/numeric_type_equality.rs` (context_only): Borrowed Decimal comparison, endpoint order, limb bound, cancellation polls and result type.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

The callback is invoked after `TypeLinkComparison` charging and before the fallback qualification check. The bounded callback passes `expected.try_eq_bounded(actual, magnitude_bits, cancel)` and projects typed size and cancellation failures at the current link. The unbounded callback returns `None`, leaving the DecimalDomain refusal after the event. The exact helper validates all four endpoints before equality, compares borrowed limbs at most twice, and polls the original handle before each endpoint and limb step. No new copied artifact or duplicate comparator appears in the diff.

`cargo test --locked bounded_decimal --lib` passed 7 tests; `cargo fmt --check` and `git diff --check` passed. Full gates were intentionally left to the PR owner.
