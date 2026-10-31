---
id: SR-16682
title: "Code review of QSV PR #80 AbsenceMode placement"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/absence.rs, src/lib.rs; QSL FR-400 plain-value placement"
review_set: subset
---
# Code review of QSV PR #80

## Summary

Ticket: IR-1202. Reviewed frozen commit 9419489 and the Rust lane, including the public enum, its mapping, its test, the QSL source definition and FR-400 placement rule. QSL-1375 now tracks the later cleanup, but it has not merged and the old type remains live. No aggregate gate was run during review.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-400 | examined | QSL spec/functional/FR-400-separate-checked-terms-from-the-checker.md | A plain value type that a moved record carries and that `qsl-foundation` or `qsl-forms` defines moves down into SV in the same change and leaves no copy or re-export behind. |
| src/absence.rs:9 | examined | src/absence.rs | `pub enum AbsenceMode {` |
| src/absence.rs:20 | examined | src/absence.rs | `pub fn as_str(self) -> &'static str {` |
| src/absence.rs:33 | examined | src/absence.rs | `fn each_absence_mode_spells_its_authored_keyword() {` |
| src/lib.rs:15 | examined | src/lib.rs | `pub mod absence;` |

## Verdict

**FAIL** — the PR introduces a second live definition of the same cross-repository type. QSL-1375 names an expiry condition for the temporary duplicate but is not an owner approval of an exception. The value mapping and test assertions themselves are sound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `AbsenceMode` is copied into SV while QSL still defines the same enum and spelling function; FR-400 requires the move to leave no copy or re-export behind. QSL-1375 tracks later removal, but both definitions remain live and can drift until that work merges. | src/absence.rs:9-26; QSL qsl-foundation/src/absence.rs:11-28; QSL FR-400 Closure rule 2 |

## Checks

- Read QSV AGENTS.md and CLAUDE.md, Rust style and review rules, the PR diff, QSL's current `qsl-foundation::absence`, and FR-400.
- `cargo test --locked --lib absence::tests::each_absence_mode_spells_its_authored_keyword`: passed, one test.
- No CI workflow, QSL, or quire-checked file changed in this PR; no shim was added.
- No aggregate gate run; the code lead owns the final gate.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | accepted-no-change | Round 1: Peter approved this temporary duplicate for the staged AbsenceMode move, expiring when QSL-1375 merges. QSV remains the intended canonical home; QSL-1375 must repoint QSL consumers and delete its old definition. The original duplication finding remains recorded, but the explicit owner exception permits this PR to merge without changing it. |
