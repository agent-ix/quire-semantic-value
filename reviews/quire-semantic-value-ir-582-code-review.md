---
id: SR-001
title: "Code review of quire-semantic-value PR #1: import quire-semantic-value from quire-spec-language"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@0ee40dfb58e0adb3513afb51debc39bb258e520d; PR #1 diff origin/main...HEAD (.github/workflows/ci.yml, CLAUDE.md, Cargo.toml, Cargo.lock, Makefile, clippy.toml, deny.toml, rust-toolchain.toml, src/*.rs, spec/, removed scripts/check_unsafe_comments.sh, scripts/unsafe_comment_baseline.txt, tests/integration.rs) plus the scaffold commit a5ba223 (.gitignore, .github/workflows/cla.yml, .agent/rules/writing_rust.md, AGENTS.md, README.md, org files); imported history 515bf00 and its 11 ancestors; rust-review lane folded in"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: reviews
---
# Code review of quire-semantic-value PR #1

## Summary

Ticket: IR-582. PR: quire-semantic-value#1 at 0ee40dfb. Rust-review lane folded in.

Checked:

- `diff -r` of `src/` against `agent-ix/quire-spec-language` origin/main
  (3dc4f522c) `quire-semantic-value/src`: every hunk is inside a `//`, `///` or
  `//!` comment. No code, attribute or test differs. The 15 unit tests are the
  same 15 tests QSL carries; the crate had no `tests/` directory in QSL.
- `Cargo.toml`: `unsafe_code = "forbid"`, `missing_docs = "warn"`,
  `clippy::all = "deny"`, `publish = false`, dependency set unchanged apart
  from the path-to-git switch for `quire-exact` and `quire-canonical`.
  `src/lib.rs` is `#![no_std]` with `extern crate alloc`, `std` only under
  `cfg(test)`.
- `make ci` runs `build-no-std` (`cargo build --locked --target
  thumbv7em-none-eabi`) and a no_std clippy pass. The log at
  ~/dev/worktrees/logs/ir582-quire-semantic-value-ci.log shows 15 tests passed,
  both no_std steps, `cargo deny check` (advisories, bans, licenses, sources
  ok) and docs, exit 0.
- Removing `scripts/check_unsafe_comments.sh`, its baseline and `make
  audit-unsafe` is correct: `unsafe_code = "forbid"` already refuses every
  `unsafe` block.
- History: every path ever committed is one of 38 root-level paths (scaffold
  files, src, spec, the removed scripts and tests); no blob over 200 KB; no
  `target/`, `.rlib` or other build artifact. Commit messages carry no
  quire-research reference, local path or branch name. Authors are Peter
  Krenesky and Agent IX.
- Org files present: LICENSE (AGPL-3.0), CLA.md, CONTENT_RIGHTS.md,
  CONTRIBUTING.md, cla.yml. `.gitignore` carries unanchored `target/`,
  `*-target/`, `target-*/`, `.worktrees/`.
- No quire-research reference, no `/home` or `~/dev` path, no Linear id in the
  tree. Imported history carries QSL's original ADR/PR comments; QSL is public,
  so that is not a leak.
- Non-test code calls no `NodeKey` constructor; the one `NodeKey::from_digest`
  is the `#[cfg(test)]` helper at src/enumeration.rs:266.

## Verdict

Approve with findings, no high. FND-004 (merge order against the
`task/ir582-import` branch dependency) must be settled before merge.
FND-001 and FND-003 are medium and belong in this PR. FND-005 is QSL's
follow-up. FND-002 and FND-006 are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The doc comments still name spec ids that resolve only in QSL or QSpec, unqualified, though the PR body says they were reworded to carry no QSL references: FR-089-AC-5/6, FR-091, FR-093, FR-094, FR-096, FR-104, FR-107, FR-109, FR-114, FR-115, FR-140..FR-143, FR-148, FR-149, FR-151..FR-153, FR-258, FR-259 B3/B4, FR-322, NFR-001/007/011, I04, and `` `check`'s `Scope` `` (a QSL module). In this repo they read as local ids that do not exist. `object_closure.rs:66` "FR-106 admission" means QSL's FR-106 snapshot admission while this repo's FR-106 is a different statement. Fix: prefix each with its owner (`QSL FR-142`, `QSpec FR-143`) or drop it. | src/checking.rs:4-44; src/quantity.rs:2-322; src/declaration.rs:2-83,2254; src/object_closure.rs:3-16,66,100; src/location.rs:40-67; src/call.rs:45-50; src/semantic_node.rs:106-110; src/definition.rs:63,79; src/enumeration.rs:2-172; src/loss.rs:14-18 |
| FND-002 | low | Comment reflow artifacts. A stray `///.` line where `(ADR-030 D-1)` was cut leaves a sentence fragment in the rustdoc of `Location`, and three doc lines exceed the 100-char width (128, 130, 122). | src/location.rs:82; src/declaration.rs:362; src/enumeration.rs:170; src/quantity.rs:35 |
| FND-003 | medium | `.agent/rules/writing_rust.md` (scaffold) tells agents to run `make audit-unsafe` and maintain `scripts/unsafe_comment_baseline.txt`, both deleted by this PR, says the crate root carries `#![warn(missing_docs)]` (now in `[lints.rust]`) and says integration tests live in `tests/`. An agent following it runs a target that does not exist. Fix: delete the Unsafe section (`unsafe_code = "forbid"` covers it) and correct the other two lines. | .agent/rules/writing_rust.md:11,22-26,36 |
| FND-004 | medium | `quire-exact` is a git dependency on branch `task/ir582-import`. If quire-exact#1 merges and that branch is deleted while this PR is on main with that dependency, a fresh resolve of main fails. Fix: merge quire-exact#1 first, switch this PR to `branch = "main"` (Cargo.toml + Cargo.lock), re-run `make ci`, then merge with a merge commit. | Cargo.toml:17; Cargo.lock |
| FND-005 | medium | The extraction leaves two copies of the crate. QSL origin/main still carries `quire-semantic-value/src` (13 modules), comment-only different from this repo. Until QSL switches to the git dependency and deletes its copy, the two drift. Fix: the QSL half of IR-582. The copy is gone only when QSL's `quire-semantic-value/` path 404s on main. | src/; agent-ix/quire-spec-language quire-semantic-value/src |
| FND-006 | low | Workflow, noted only (workflow edits need Peter's clearance). This PR edits `.github/workflows/ci.yml` (adds the thumbv7em target, removes the Unsafe audit step), and the clearance is not recorded in the PR. The workflow installs the target but runs neither the no_std build nor the no_std clippy pass, and its licenses job runs `cargo deny check licenses` while `make deny` runs the full check, so the `allow-git` source rule is not exercised there. Local `make ci` covers all of it. | .github/workflows/ci.yml:14-57; Makefile:36-66,86 |
