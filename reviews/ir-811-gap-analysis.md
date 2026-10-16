---
id: SR-11721
title: "IR-811 InputRefusal::DuplicateClause gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; PR #39 diff origin/main...c229bc5: src/call.rs"
review_set: subset
---

# IR-811 InputRefusal::DuplicateClause gap analysis

## Summary

Ticket: IR-811. PR: quire-semantic-value#39 at c229bc5c8cf11b781ff145c9ef64bde771fdb3dc. This analysis is planless, so plan completion was not assessed. It used the computed matrix (`quire coverage --scope . --strict`; quire 0.36.2, engine 0.50.2; quoin 0.28.3) and checked test-oracle strength, scoped to the diff.

- **Matrix.** The result is 5 unbacked rows and 0 contradicted, identical on origin/main and at c229bc5. The PR adds no criterion and removes no tag.
- **Ownership.** The README assigns the `call` module's requirements to agent-ix/quire-spec-language. The owning requirement for the new variant is QSL FR-107 ("Clause sets": "A clause named twice in the slice SHALL be refused `CallFailure::Input(InputRefusal::DuplicateClause)` naming it") and its FR-107-AC-16. The code/cause pair matches the catalogue's existing use in QSL FR-106-AC-43. The new code has an owner, and no AC row is required in this repository.
- **Coverage.** The behaviour this crate owns, the variant's code and cause, is covered by an exact-value table test. The behaviour FR-107-AC-16 tests, refusing a duplicate in `evaluate_clause_set`, lives in QSL and is outside this repository.

## Verdict

**PASS**, clean. There are no stubs, no coverage inflation and no untraced claims. The new variant fills the gap that blocked QSL FR-107-AC-16: before this PR, `InputRefusal::DuplicateClause` did not exist on this crate's main.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: src/call.rs
  path: src/call.rs
  role: examined
  excerpt: "Self::DuplicateClause(_) => \"duplicate-member\","
- id: FR-107-AC-16
  path: "agent-ix/quire-spec-language:spec/functional/FR-107-evaluate-state-clauses-at-s6a.md"
  role: examined
  excerpt: "A set naming `UpdateAllowed` twice is refused `InputRefusal::DuplicateClause`; a set naming the function `sameIdentity` is refused `InputRefusal::UnknownClause`; a set pairing `UpdateAllowed`'s name with observations admitted for `UpdateRaises` is refused `InputRefusal::ObservationsMismatch`; none returns a result."
- id: README.md
  path: README.md
  role: context_only
  excerpt: "The requirements of the other modules (... `call` ...) are owned by `agent-ix/quire-spec-language`"
```
