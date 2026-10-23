---
id: SR-13641
title: "Code review of quire-semantic-value PR #62: SuppliedRun, render_identity and the state-model prefix move into SV (IR-588 S4)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@e331dd0af0723e3c0b036a02702bed9f3b913ac4; PR #62 diff origin/main...HEAD (Cargo.toml, src/catalog.rs, src/supplied_stop.rs); against QSL origin/main e1fc9f0cf FR-403 (S4 disposition table, Argument admission), FR-401, NFR-015; rust-review lane folded in"
review_set: subset
date: '2026-10-23'
---
# Code review of quire-semantic-value PR #62

## Summary

Ticket: IR-588 (S4). Rust-review lane folded in. The spec was read at
agent-ix/quire-spec-language origin/main e1fc9f0cf: FR-403 "Which
`qsl-semantics` code the evaluator needs (S4)" and its Argument admission
paragraph, FR-401 and NFR-015.

Checked and clean:

- Matches FR-403's table. `render_identity` is in SV's `catalog`. The
  `"state-model"` string is in `catalog` as `STATE_MODEL_CATALOG_PREFIX`.
  `SuppliedRun`, `SuppliedLimits`, the two defaults and the private
  event/storage helpers are in `supplied_stop`, beside the stop records.
  Nothing in this PR re-exports a moved item. `divide` and `modulo` go to
  `quire-eval`, so they are correctly absent here.
- Faithful move. `src/supplied_stop.rs` matches
  `qsl-semantics/src/value/supplied.rs` at QSL main. The only changes are
  import paths and the module doc. `render_identity` gives the same output
  as the original's `key::hex` rendering: lowercase, two digits per byte.
  The new code names only SV, K (`quire_exact`) and SV's own catalog
  vocabulary, as FR-403 requires.
- `no_std` + `alloc`. Every `alloc` item is imported explicitly. The
  thumbv7em `clippy --lib -D warnings` pass is clean, both without and with
  `--features test-support`. Host `clippy --all-targets -D warnings` is
  clean with and without the feature. `cargo fmt --check` is clean.
- Feature gating. `test-support` is off by default and adds no dependency.
  The QSV manifest does not enable it for itself. The paired QSL PR #1382
  enables it only in `qsl-eval`'s `[dev-dependencies]`, so it does not reach
  a non-test build.
- No vendoring. This is a move. The paired QSL PR #1382 deletes
  `qsl-semantics/src/value/supplied.rs` and `render_identity`, and adds
  `compile_fail` doctests for the old paths.
- Focused tests pass: `catalog::tests` (2) and `supplied_stop::tests` (1).

## Verdict

Approve with findings. There is no high finding, and the code is correct.
FND-001 is a medium trace defect, and a one-line fix in this PR. FND-002 to
FND-004 are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The moved test keeps the QSL tag `#[trace("TC-906", "FR-277-AC-5")]`. In this repo `TC-906` is a different, local test case (`spec/test-cases/TC-906-qsv-checked-invariant-causes.md`, which verifies QSV FR-122). `FR-277-AC-5` does not exist in this repo's spec, and QSL's matrix does not scan this repo, so neither matrix counts the binding. The tag now names a QSV test case it does not belong to. Fix: retag it to the QSV owning AC (see SR-13642 FND-001), or drop the tag. | src/supplied_stop.rs:1078 |
| FND-002 | low | `state_model_prefix_is_the_catalog_spelling` asserts that `STATE_MODEL_CATALOG_PREFIX == "state-model"`, which only repeats the constant's own literal. It cannot fail unless someone edits the literal, and then they edit both copies. FR-403-AC-11's real check is that `FamilyKind::StateModel.catalog_code_prefix()` returns this constant, and that check belongs in QSL. Fix: delete the test, or keep one assertion as the spelling contract under a QSV AC. | src/catalog.rs:437-441 |
| FND-003 | low | Both `catalog` tests are tagged `#[trace("TC-1121", "FR-403-AC-11")]`, and the first one's doc also cites AC-2. These are QSL ids that no matrix resolves from this repo. AC-11 is about the StateModel undefined result of a `lookup` raised by each evaluator, and AC-2 is about old paths failing to compile; neither test checks that behaviour. Fix: tag them to a QSV AC for `render_identity` (SR-13642 FND-001). | src/catalog.rs:422-425 |
| FND-004 | low | `render_identity` allocates one temporary `String` per byte through `format!("{byte:02x}")`. Fix: `use core::fmt::Write` and `write!(rendered, "{byte:02x}")`, which pushes into the one buffer. | src/catalog.rs:34-36 |
