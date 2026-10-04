---
id: SR-002
title: "Gap analysis of quire-semantic-value PR #1: spec subset, traces and the NodeKey boundary"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@0ee40dfb58e0adb3513afb51debc39bb258e520d; IR-582 extraction goal for quire-semantic-value; FR-106-AC-10, TC-904 and every test in src/; QSL origin/main 3dc4f522c FR-060-AC-5, TC-157 and tools/arch-lint/api_surface.rs tc_arch_lint_api_surface_026 (context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: reviews
---
# Gap analysis of quire-semantic-value PR #1

## Summary

Ticket: IR-582. PR: quire-semantic-value#1 at 0ee40dfb.

Checked:

- All crate tests moved. QSL's crate had only inline unit tests; this repo has
  the same 15 (`make ci` log: 15 passed). Tests in other QSL crates that use
  this crate (qsl-semantics, qsl-package) are consumer tests and stay in QSL.
- FR-106-AC-10 is backed by three tests tagged `#[trace("TC-904",
  "FR-106-AC-10")]`; each asserts the exact refusal cause or `find` result,
  so each fails if the behaviour breaks.
- FR-060-AC-5 in QSL ("a `NodeKey` constructor call in the shared leaf
  `quire-semantic-value`'s `src/` is a T12-B call site outside the allowed
  callers") was enforced by arch-lint's T12-B token scan over QSL's tree, with
  negative control `tc_arch_lint_api_surface_026`. Once QSL drops its copy, no
  scan reaches this crate. Measured today: non-test code in src/ calls neither
  `NodeKey::from_digest` nor `NodeKey::decode_admitted`; the one call is the
  `#[cfg(test)]` helper at src/enumeration.rs:266 (T12-B exempts test code too).

## Verdict

Approve with findings, no high. FND-001 should land in this PR: it is a few
lines of `clippy.toml` and keeps a real identity boundary enforced. FND-002 is
a statement of where the rest of the crate's requirements live.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Coverage lost: nothing in this repo enforces that the crate mints no `NodeKey` (QSL FR-060-AC-5). Recommend a type-resolved clippy lint, not a text scan or a new test: in `clippy.toml`, `disallowed-methods = [{ path = "quire_exact::NodeKey::from_digest", reason = "only the checking stage mints a NodeKey" }, { path = "quire_exact::NodeKey::decode_admitted", reason = "..." }]`, plus `#[allow(clippy::disallowed_methods)]` with a reason on the test helper `key()` in `enumeration.rs`. `make lint` already runs clippy with `-D warnings` and `clippy::all = "deny"`, so a call fails the gate. Probed in a scratch clone at 0ee40dfb with clippy 1.98.1: the re-exported paths resolve, a planted `NodeKey::decode_admitted(b)` call and a planted `.map(NodeKey::from_digest)` function value both fail `cargo clippy --all-targets -- -D warnings` (`use of a disallowed method`), and the existing test helper at enumeration.rs:266 is flagged too, so it needs the allow. Record it as an Inspection AC here only if the team wants it in the spec; the lint itself is the enforcement. QSL side (QSL lane): after QSL deletes its copy, FR-060-AC-5 and `tc_arch_lint_api_surface_026` scan a path that no longer exists, so delete them there rather than keep a vacuous rule. | clippy.toml; src/enumeration.rs:266; Makefile:36-39 |
| FND-002 | medium | The spec subset covers one module of thirteen. 12 of the 15 tests trace nothing, and `quantity`, `unit`, `declaration`, `enumeration`, `location`, `checking`, `call`, `semantic_node`, `definition`, `containment`, `loss` and `stop` have no owning requirement in this repo; their requirements are QSL's (FR-089, FR-140..FR-143, FR-149, FR-151..FR-153, NFR-011 and others). Nothing in the repo says so. FR-106 is honest about its own scope, but a gap analysis of this repo alone reads the rest as unspecified code. Fix: one paragraph in README.md (or a spec index) naming QSL as the owner of those requirements, consistent with qualifying the ids in the comments (SR-001 FND-001); or move the ACs here under a follow-up ticket. | spec/; src/*.rs; README.md |

## Dispositions

Round 1, reviewed at dd89582de14ca3a2737a216ce0de9e9e04e3a678.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | dd89582 |
| FND-002 | fixed | dd89582 |
