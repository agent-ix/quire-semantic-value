---
id: SR-006
title: "Code review of quire-semantic-value PR #3: CheckingLimits::with_nodes"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@c6d46da4df23525a8a0ffca582f56e12380e68a2; PR #3 diff origin/main...HEAD (src/checking.rs, spec/functional/FR-107-set-each-checking-limit-on-its-own.md, spec/test-cases/TC-905-each-checking-limit-builder-sets-only-its-own-ceiling.md); qsl-cst/src/lexer.rs:56-78 (pattern, context); rust-review lane folded in"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-107
    type: reviews
---
# Code review of quire-semantic-value PR #3

## Summary

Ticket: QSL-489. PR: quire-semantic-value#3. Rust-review lane folded in.

Checked:

- `CheckingLimits` has three private fields (`nodes`, `input_bytes`,
  `work_budget`). After this PR each has a consuming builder that writes its
  own field and returns `self`: `with_nodes` (new), `with_input_bytes` and
  `with_work_budget` (already on main). A caller can change any one field and
  keep the other two.
- Naming: the qsl-cst lexer pattern is `with_<field>` (`with_source_bytes`,
  `with_tokens`, `with_nodes`, `with_work_units` over fields of the same
  names). `with_work_budget` follows that rule here: the field is
  `work_budget`, the accessor is `work_budget()`, the default constant is
  `DEFAULT_CHECKING_WORK_BUDGET` and the refusal kind is
  `CheckingLimitKind::WorkBudget`. The plan's `with_work_units` copied the
  lexer's field name, which is a different quantity (parser work units per
  token). Renaming to `with_work_units` would make it the one builder that
  does not match its field. Recommendation: keep `with_work_budget`. Not a
  finding.
- No panics, no integer conversion, no unsafe. make ci exit 0 per
  ~/dev/worktrees/logs/qsv-489-make-ci.log (clippy -D warnings, no_std
  clippy, tests, doc).

## Verdict

Approve with one low finding. The builder is correct and complete for all
three fields.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `with_nodes` carries `#[must_use]` (as every builder in the lexer pattern does), but the two existing builders `with_input_bytes` and `with_work_budget` do not. A dropped `limits.with_work_budget(9);` silently keeps the old ceiling with no warning. Fix: add `#[must_use]` to both. | src/checking.rs:93-125 |
