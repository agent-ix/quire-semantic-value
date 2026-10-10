---
id: SR-4946
title: "Coder Rust prehandoff review of IR-713 admission storage"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@ec6e801, @e2b9b5f, and @0ec2209; src/declaration.rs; FR-108-AC-1 through FR-108-AC-5, TC-907"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: reviews
---
# Coder Rust prehandoff review of IR-713

## Summary

Reviewed the authored-declaration admission diff at ec6e801 using the repository conventions and dev-tools Rust review checklist. The reservation carrier and checked storage paths have no remaining infallible growth in the admission call graph; two narrow acceptance-test gaps were found and fixed afterward.

## Verdict

**CONDITIONAL** at the reviewed revision because the two test controls below were absent. Both are fixed at fd8781d. Full gate, QSL consumer compilation, and a union fixture remain for coordinated qualification; this coder prehandoff does not replace independent PR review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | AC2 had no direct test of a denied native byte request, so a bytes-to-elements reporting regression could pass the new controls. | src/declaration.rs:144 |
| FND-002 | low | AC4 had no public admission test for cancellation retaining its work-limit classification and counter. | src/declaration.rs:215 |

## Dispositions

| ID | Outcome |
| --- | --- |
| FND-001 | fixed fd8781d: `byte_reservation_reports_its_native_request` asserts the denied request and public failure both carry five bytes. |
| FND-002 | fixed fd8781d: `cancellation_keeps_the_work_limit_classification` asserts kind, configured bound, actual counter, and tripped cause. |

## Coverage

The admission audit followed `TypeEnvironment::bounded_with_reservations` through duplicate detection, member-type walks, recursion and generalization traversals, ancestry, field indexing, sealing, flattening, refusal construction, and key join. Tree insertion and clone sites in `contains_ieee` and `type_refusal` are public query paths; admission uses their separate fallible walkers. Construction and evaluation allocations elsewhere in `src/declaration.rs` are outside FR-108 admission. No unsafe code, production panic, compatibility layer, or copied source was added. The CI workflow diff is empty.

Focused evidence after fixes at fd8781d: `cargo test --lib declaration --locked` passed 20 tests; `cargo clippy --lib --tests --locked -- -D warnings`, `cargo fmt --check`, and `git diff --check` passed.

## Final coder prehandoff pass

The full `origin/main...e2b9b5f` Rust diff was inspected again after rebasing onto the IR-720 specification-only merge. The CI workflow diff is empty. All authored-declaration admission growth remains behind the fallible reservation carrier; the ordinary tree insertions and allocating evaluation helpers are outside admission. No new unsafe, panic on a request path, unbounded storage growth, or test-only production branch was found. The new admission byte-denial test exercises the production `bounded_with_reservations` call rather than invoking a cloning helper alone. Its non-denied counterpart preserves the duplicate-key refusal. An order and inheritance control checks public key iteration, slot order, and member resolution after reversed declaration inputs. The phase-denial control checks the restored member link after each retry.

The order control initially caught a lifetime defect: `EffectiveAttribute::field()` tied its returned reference to a temporary view, which prevented retaining a field name after consuming that view. The final `e2b9b5f` returns the environment-backed field reference for the view's lifetime; this compiles and passes the focused test. `cargo fmt --check`, `cargo clippy --lib --tests --locked -- -D warnings`, `cargo test --lib declaration --locked` (21 passed), and `git diff --check` passed on that head. The Union fixture cannot run until the separate QSL-500 union shape reaches QSV; QSL's exact consumer has not been compiled against this head. No full gate or PR review is claimed.

## IR-732 native-kind rebase prehandoff

The eight earlier source and review commits remain patch-identical after rebasing onto QSV main `010dca0`; `git range-diff` shows only the new `0ec2209` adaptation. Its four explicit `Uuid` and `Timestamp` match arms make the two fallible admission walkers agree with the four released declaration match sites. Both kinds are leaves: neither represents an IEEE value nor an invalid member type. The arms introduce no allocation, panic, wildcard fallback, test-only production branch, or changed charge point. The branch changes no CI workflow, Cargo manifest, or lockfile relative to main. The still-unavailable Union shape remains an open TC-907 fixture obligation; this review makes no Union claim.

At `0ec2209`, `cargo fmt --all -- --check`, `cargo check --locked --offline --all-targets`, `cargo clippy --locked --offline --lib --tests -- -D warnings`, `cargo test --locked --offline --lib declaration` (23 passed), and `git diff --check` passed. The reused worktree target and exact-head log are `/home/peter/dev/worktrees/ir-713-qsv/target` and `/tmp/ir732-qsv-pr10-focused.log`. This coder pass does not replace the independent PR review or the final full gate.
