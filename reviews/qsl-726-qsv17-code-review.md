---
id: SR-8695
title: "Code review of quire-semantic-value PR #17: model operation clause origin"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@b90ac1fc50fef599a4ad59c4c1554bd717da6df1; src/location.rs (Origin::ModelOperationClause, OperationClauseKind, a_model_operation_clause_origin_is_its_own)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-114
    type: reviews
---
# Code review of quire-semantic-value PR #17

## Summary

Ticket: QSL-726. Code review with the Rust lane (rust-review) folded in, over
the one changed source file `src/location.rs` against base `8e21d50`.
`Cargo.toml` and `Cargo.lock` are untouched. Gates were not run: the brief is
reviewer-only with no builds.

## Verdict

PASS. No defects in the code.

- **Exhaustive matches.** The crate has no `match` on `Origin`. The only uses
  are construction sites in the tests. Nothing inside the crate needs a new
  arm.
- **Trait contracts.** `Origin` derives `Clone, Debug, Eq, Hash, Ord,
  PartialEq, PartialOrd`. `OperationClauseKind` derives the same set plus
  `Copy`, so the derived `Origin` impls still compile and stay consistent:
  `Hash` agrees with `Eq`, and `Ord` is total. The derived order places
  `ModelOperationClause` after every existing variant. Inside the variant, the
  order is `operation` (string order), then `kind` (`Requires` before
  `Ensures`, by declaration order, as the doc comment says), then `position`.
  `Location`'s hand-written `Eq`, `Hash` and `Ord` delegate to `Origin` first,
  so they need no change.
- **Serialization and Display.** `Origin` has no `Serialize` and no
  `Display`. Nothing about wire format or rendering changes in this crate.
- **Public API.** The only change is the new variant and the new public enum
  `location::OperationClauseKind`. `Origin` is not `#[non_exhaustive]`, so an
  exhaustive `match` in a downstream crate breaks. This is accepted, not a
  finding. Earlier variants such as `ProtocolAttempt` were added the same way.
  Marking the enum `#[non_exhaustive]` would force wildcard arms downstream and
  lose the compiler's check that each consumer handles every origin.
- **Downstream impact (QSL).** QSL depends on QSV with `branch = "main"`, and
  its `Cargo.lock` holds the rev. QSL main has exhaustive matches at
  `src/command/output.rs:435`, `qsl-replay/src/result.rs:202`,
  `qsl-replay/src/spine/native_result.rs:1725` and
  `qsl-semantics/src/check/region.rs:101-104`. These stop compiling once QSL's
  lock moves to a QSV rev that has this variant. The held QSL commit `8e705ee`
  (impl/keep-model-operation-contracts, QSL PR #802) adds arms in exactly
  those files. The `matches!` uses at `qsl-semantics/src/check/lowering.rs:4208`
  and `tests/it/formatter_identity.rs:498` are not exhaustive and still
  compile. **Landing order:** merge this PR, then land QSL #802 with the held
  commit and the QSV lock bump together. Any other QSL PR that bumps the QSV
  lock in between goes red. The held commit has no `Cargo.lock` change. It
  builds today against a local, git-excluded path patch, so the lock bump to
  the merged QSV rev is still to be made in #802.
- **Rust idioms.** Every new item has a doc comment. `usize` for a position
  is used in memory only and never reaches a wire. The test uses the repo's
  `#[trace("TC-NNN", "FR-NNN-AC-N")]` convention.
- **Test oracle strength.** `a_model_operation_clause_origin_is_its_own` checks
  the full equality matrix of four origins, rebuild equality, path
  sensitivity, the sorted order `[cancel req 0, place req 0, place req 1,
  place ens 0]`, and inequality with `StateClause`. Swapping the order of the
  `OperationClauseKind` variants turns the sort assertion red. Dropping a field
  from the variant makes two of the four origins collide, which turns the
  matrix red. The oracle discriminates. Its divergence from the TC-914 step 4
  vector is recorded in the gap analysis, SR-8696.
- **Public-repo rules.** The diff has no private-repo text, local paths,
  research content or external protected material (CONTENT_RIGHTS.md). It
  cites only QSL FR-103, in the public `agent-ix/quire-spec-language`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
