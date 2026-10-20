---
id: SR-13520
title: "IR-882 quire-semantic-value PR 55 code review of the TC-924 tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@4a52c1105 vs origin/main e42f0d4; tests/definition_closure_refusal.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-125
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-924
    type: reviews
---
# Code review of PR #55 (IR-882)

## Summary

Ticket: IR-882. The diff adds one file, `tests/definition_closure_refusal.rs`,
with four tests tagged `#[trace("TC-924", "FR-125-AC-n")]` for n = 1..4. It does
not touch `src/`. The Rust lane (rust-review) was applied in this file.

- The four tests exercise the public `quire_semantic_value::definition` API from
  an integration-test crate. `cause_index` and `code_spelling` are exhaustive
  matches with no wildcard arm, so a new `PackageCause` or `PackageRefusalCode`
  variant, or `#[non_exhaustive]` on either enum, fails to compile here (E0004 at
  `tests/definition_closure_refusal.rs:18`).
- The spellings were checked against QSpec FR-272's actual `invalid_package` row
  (quire-specification origin/main c9a3ef6,
  `spec/objects/foundation/FR-272-typed-diagnostic-cause.md:74`). That row lists
  `missing-member`, `conflicting-definition` and `incompatible-definition`, and the
  test literals match them byte for byte.
- The trace tag form matches the repo's other tests (`#[trace("TC-920", "FR-120-AC-1")]`).
  `quoin matrix` binds exactly one test to each of FR-125-AC-1..4.

## Examined scope

- FR-125-AC-1 (examined): `PackageCause::as_str` returns `missing-member`, `conflicting-definition` and `incompatible-definition`, each byte-equal to the spelling in QSpec FR-272's `invalid_package` row, and `PackageRefusalCode::InvalidPackage.as_str()` is `invalid_package`.
- FR-125-AC-2 (examined): `PackageRefusal::invalid_package(cause)` returns `code` `InvalidPackage` and the given `cause` for each of the three causes, and `PackageRefusalCode` has the one variant `InvalidPackage`.
- FR-125-AC-3 (examined): `PackageCause` has exactly three variants and is not `#[non_exhaustive]`: an exhaustive `match` over it with no wildcard arm, written in an integration test under `tests/`, compiles, and none of `byte-digest-mismatch`, `digest-domain-mismatch` or `content-mismatch` is the spelling of any variant.
- FR-125-AC-4 (examined): `SelectionRefusalCode::ALL` holds the eight codes in the order above, `as_str` returns their spellings, `from_code` round-trips each, and `from_code` returns `None` for `invalid_package`, an empty string and a differently cased spelling.
- QSpec FR-272 `invalid_package` row (context_only): the cause cell lists `malformed-json`, `missing-member`, ..., `conflicting-definition`, `incompatible-definition`, ...
- src/definition.rs (context_only): the implementation under test; unchanged by this PR.

## Verdict

Clean: no defect. Mergeable.

Mutation run (each mutant applied to `src/definition.rs`, then
`cargo test --test definition_closure_refusal`, then the file restored). All 18
mutants were killed:

| Mutant | Result |
| --- | --- |
| `missing-member` renamed to `missing_member` | AC-1 test fails |
| `conflicting-definition` renamed | AC-1 test fails |
| `incompatible-definition` renamed to `content-mismatch` | AC-1 and AC-3 tests fail |
| two cause spellings swapped | AC-1 test fails |
| fourth `PackageCause` variant | E0004 compile error in the test crate |
| `#[non_exhaustive]` on `PackageCause` | E0004 compile error |
| `#[non_exhaustive]` on `PackageRefusalCode` | E0004 compile error |
| second `PackageRefusalCode` variant | E0004 compile error |
| `invalid_package` spelled `invalid-package` | AC-1 and AC-2 tests fail |
| `invalid_package()` ignores its cause | AC-2 test fails |
| `SelectionRefusalCode::ALL` reordered | AC-4 test fails |
| a selection spelling changed | AC-4 test fails |
| `from_code` accepts `invalid_package` | AC-4 test fails |
| `from_code` case-insensitive | AC-4 test fails |
| `from_code` trims whitespace | AC-4 test fails |
| `from_code` accepts the empty string | AC-4 test fails |
| `from_code` accepts a prefix | AC-4 test fails |
| `invalid_package()` returns a second code | E0004 compile error |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- `make ci` on head 4a52c1105 (fmt-check, clippy `-D warnings` for all targets
  and the no_std target, test, no_std build, cargo deny, docs) exited 0. The new
  test binary ran 4 passed, 0 failed.
- `git diff --name-only origin/main...4a52c1105` lists only `tests/definition_closure_refusal.rs`.
