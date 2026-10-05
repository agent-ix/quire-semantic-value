---
id: SR-004
title: "Code review of quire-semantic-value PR #2 (IR-582: forbid EffectiveId and PopulationId minting by clippy)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@0603727bf63ac70f26875b02ab3339290e7ad196; clippy.toml, src/object_closure.rs, README.md"
review_set: subset
---

## Summary

Ticket: IR-582. Code review with the rust-review lane over
`git diff origin/main...0603727b`. It closes QSL #632 SR-1299 FND-001.

Checked:

- `clippy.toml` adds `quire_exact::EffectiveId::from_digest` and
  `quire_exact::PopulationId::from_digest` to `disallowed-methods`, beside the
  two `NodeKey` entries, each with a reason.
- The one shipped-tree call, `EffectiveId::from_digest` in
  `object_closure.rs`'s `object_type` test helper, sits inside
  `#[cfg(test)] mod tests` and carries
  `#[allow(clippy::disallowed_methods, reason = ...)]`. No other
  `EffectiveId::from_digest` or `PopulationId::from_digest` call is in `src/`
  (grep).
- Gate: `~/dev/worktrees/logs/ir582-quire-semantic-value-ci4.log` ends
  `exit=0`, written after the head commit. Clippy re-checked the crate
  ("Checking quire-semantic-value") and printed no unresolved-path warning for
  the new entries.
- The PR body reports a planted-call check: a call of each method added to
  `src/lib.rs` failed clippy with "use of a disallowed method". I did not
  re-run it, because the coordinator asked for no builds during this pass.
  That is why confidence on the lint firing is medium, not high.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. The change is small and correct, and the test-only allow carries its reason.
