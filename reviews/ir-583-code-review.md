---
id: SR-12344
title: "Code review of quire-semantic-value PR #61: catalog vocabulary and supplied-value stop records (IR-583 S0)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@1adf9ce983c155a91983bd99538c9fbf1071266e; PR #61 diff origin/main...HEAD (spec/functional/FR-126-own-the-evaluation-result-vocabulary.md, src/catalog.rs, src/lib.rs, src/supplied_stop.rs, tests/evaluation_vocabulary.rs); against QSL origin/main 48108c9c1 FR-401, FR-262, NFR-015, AD-001; rust-review lane folded in; reviewed together with quire-eval PR #2 (SR-12346, SR-12347)"
review_set: subset
date: '2026-10-22'
---
# Code review of quire-semantic-value PR #61

## Summary

Ticket: IR-583 (S0). Rust-review lane folded in. One reviewer covered this PR
and its pair, agent-ix/quire-eval#2. The specs were read at
agent-ix/quire-spec-language origin/main 48108c9c1: FR-401 (placement table,
"These stay where they are", the moved-method rules, S0 scope), FR-262,
NFR-015 Scope, and AD-001 S0.

Checked and clean:

- Faithful move. I diffed `src/catalog.rs` against `qsl-foundation/src/diagnostic.rs`
  (`Category`, `CatalogCode`, `InternalFault`, `CatalogCoded`,
  `UndefinedCoded`, `UndefinedReason`, `UndefinedRecord`). I also diffed
  `src/supplied_stop.rs` against `qsl-semantics/src/value/supplied.rs` (the
  stop records and the three field-name constants). The only differences
  are `std::` to `core::`/`alloc::` paths, doc wording, and the removals
  FR-401 orders: `CatalogCoded::refusal_record`, and the private
  `SuppliedEvent::of_membership`, `of_part`, `SuppliedStorage::of_request`
  and `of_arena` helpers. `SuppliedRun`, `SuppliedLimits` and the defaults
  correctly stay in QSL. Every value, equality, hash and category is
  unchanged.
- The crate stays `no_std` + `alloc`. The new modules import `alloc` items
  explicitly. `cargo build --locked --target thumbv7em-none-eabi` and the
  thumbv7em clippy pass both succeed.
- No new dependency. Everything the new code names (`quire_exact`,
  `call::InputRefusal`, `supplied_membership`) was already a dependency or
  module of SV.
- `make ci` on 1adf9ce passed with EXIT=0: fmt-check, clippy on host and
  thumbv7em, tests, no-std build, deny and docs.
- The change is additive. No existing SV item changed, so SV's current
  dependents are unaffected.

## Verdict

Approve with findings. There is no high finding. FND-001 is a scope error
against FR-401. It is cheap to fix by deleting two constants, so fix it
before merge. FND-002 is low. Merge order and the duplicate-definition
window are in SR-12347 FND-004.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR adds `ALLOCATION_FAILED` and `RESERVATION_SIZE_OVERFLOW` to SV's `catalog`, and FR-126 lists them as Outputs. FR-401 says the reverse: "These stay where they are and are not part of the leaf: `qsl-foundation`'s ... the `CatalogCode` constants such as `ALLOCATION_FAILED`". Nothing in SV or quire-eval names either constant. They become a second definition beside `qsl-foundation`'s, with no move planned that would delete the original. Fix: delete both constants and remove them from FR-126 Outputs. Amending FR-401 is the alternative, and is right only if QSL wants them moved. | src/catalog.rs:201-213; spec/functional/FR-126-own-the-evaluation-result-vocabulary.md:20-21 |
| FND-002 | low | `impl SuppliedStorage {}` is an empty impl block, left behind after the private `of_request`/`of_arena` helpers stayed in QSL. Fix: delete it. | src/supplied_stop.rs:287 |
