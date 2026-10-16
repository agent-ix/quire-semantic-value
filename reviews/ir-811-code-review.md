---
id: SR-11720
title: "IR-811 InputRefusal::DuplicateClause code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; PR #39 diff origin/main...c229bc5: src/call.rs"
review_set: subset
---

# IR-811 InputRefusal::DuplicateClause code review (with Rust review)

## Summary

Ticket: IR-811. PR: quire-semantic-value#39 at c229bc5c8cf11b781ff145c9ef64bde771fdb3dc (one file, src/call.rs, 14 lines added and 3 removed). This is one pass of code review with the rust-review lane folded in, scoped to the PR diff. The change adds `InputRefusal::DuplicateClause(String)` and maps it to code `invalid_runtime_input` and cause `duplicate-member`, with one new row in the existing table test.

Focused gates, run with `CARGO_TARGET_DIR` set to the review worktree's own `target/`:

- `cargo fmt --check`: clean.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo test --lib call::`: 1 test, passes.
- `quire coverage --scope . --strict`: 5 unbacked rows and 0 contradicted, the same as origin/main. `make ci` was not run.

## Verdict

**PASS**, with one low finding. The change is mergeable.

- **Catalogue pair.** The pair `invalid_runtime_input`/`duplicate-member` exists in the QSL catalogue on main (b542238). FR-106-AC-43 uses it for a duplicate state-machine context entry. FR-056 lists `duplicate-member` among the existing `invalid_runtime_input` causes. FR-107's "Clause sets" statement and FR-107-AC-16 name `InputRefusal::DuplicateClause`, and the statement requires the refusal to name the clause, which the `String` payload does.
- **Exhaustive matches.** `code()` and `cause()` both match every variant with no wildcard arm. The new variant joins the `invalid_runtime_input` or-pattern in `code()` and has its own arm in `cause()`.
- **Test oracle.** The test compares exact strings for both code and cause. Mapping the variant to any other code (`missing_declaration`, `dangling_reference`) or any other cause (`wrong-role-mapping`, `missing-name`) fails the test.
- **Rust surface.** Nothing in the change can panic, there is no unsafe code and no std import, and the crate stays no_std through `alloc::string::String`. The derive list is unchanged. The `thiserror` message follows the existing variants.
- **Other matches.** Nothing else in this repository, its tests or its spec matches on `InputRefusal`.
- **Downstream effect (informational, not a defect of this PR).** QSL depends on this crate's `main` by git. QSL has two exhaustive matches with no wildcard: `input_refusal_code` (qsl-eval/src/value/expression/mod.rs:78) and the clause disposition match (qsl-replay/src/spine/clause.rs:752). Both will fail with E0004 when QSL bumps its lock to include this commit, until they gain a `DuplicateClause` arm. That failure is the intended effect of having no wildcard. The arms belong to the QSL consumer work (QSL-790, QSL PR #897).
- **Spec row.** The README states that the `call` module's requirements are owned by agent-ix/quire-spec-language, so this repository's trace rules require no acceptance criterion row here. The earlier `UnknownClause` and `ObservationsMismatch` variants were added the same way.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The doc comment attributes the code/cause pair to "QSL FR-107", but FR-107 names only the variant `InputRefusal::DuplicateClause` and does not state its code or cause. The pair is inferred from the catalogue's other uses (FR-106-AC-43, FR-056). A reader following the citation to FR-107 will not find it there. | src/call.rs:55-56 |

## Examined scope

```yaml
scope:
- id: src/call.rs
  path: src/call.rs
  role: examined
  excerpt: "/// QSL FR-107: `evaluate_clause_set`'s slice names one clause twice:\n/// `invalid_runtime_input` / `duplicate-member`.\n#[error(\"the clause set names {0} more than once\")]\nDuplicateClause(String),"
- id: FR-107-AC-16
  path: "agent-ix/quire-spec-language:spec/functional/FR-107-evaluate-state-clauses-at-s6a.md"
  role: context_only
  excerpt: "A set naming `UpdateAllowed` twice is refused `InputRefusal::DuplicateClause`; a set naming the function `sameIdentity` is refused `InputRefusal::UnknownClause`; a set pairing `UpdateAllowed`'s name with observations admitted for `UpdateRaises` is refused `InputRefusal::ObservationsMismatch`; none returns a result."
- id: FR-106-AC-43
  path: "agent-ix/quire-spec-language:spec/functional/FR-106-admit-snapshots-and-invocations.md"
  role: context_only
  excerpt: "Two entries naming `OrderLifecycle` in one snapshot refuse `invalid_runtime_input`/`duplicate-member` at the second"
```
