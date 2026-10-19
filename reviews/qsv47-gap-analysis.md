---
id: SR-13044
title: "IR-878 QSV PR 47 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; spec/functional/FR-109-bound-supplied-value-traversal.md, spec/functional/FR-122-supplied-membership-owner-obligations.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-878 QSV PR 47 gap-analysis

## Summary

Ticket: IR-878. Planless gap analysis of head aea435e16a77c862b7f52345afb253521ce92df1 against origin/main 02e5cf5, using `quire coverage --scope . --strict` in two throwaway worktrees and grep over `src/` and `tests/` for trace tags.

## Reviewed scope

- `FR-122-AC-1` — examined: unbacked. 8 tests in src/declaration.rs (TC-906) are tagged quire-exact `FR-369-AC-9`, whose text is identical.
- `FR-122-AC-2` — examined: unbacked. 4 tests in src/declaration/membership.rs (lines 3524, 3617, 3728, 3776) are tagged `FR-372-AC-5` and `FR-109-AC-8`.
- `FR-122-AC-3` — examined: unbacked. Existing membership.rs tests assert `UnsupportedHelperFamily::NumericDomain` without a bound.
- `FR-109-AC-6` — examined: unbacked on origin/main and on head. The narrowed text is now executable in QSV.
- `FR-109-AC-8` — examined: backed by the 4 membership.rs tests, unchanged.

## Verdict

**PASS WITH FINDINGS.** Strict coverage delta: 9 to 12 unbacked rows (93/103 to 93/106 backed). The +3 are exactly FR-122-AC-1..3, PLANNED as expected. 0 contradicted statuses before and after. No stubs or code changes. The status wording does not account for the narrowed FR-109-AC-6 or name the existing evidence for FR-122.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR rewrites FR-109-AC-6 into a QSV-executable criterion, but no test backs it on main or head. The FR-109 Status lists the state of AC-1..5, AC-7 and AC-8 and is silent on AC-6. So the narrowed QSV-owned obligation is neither asserted by a nameable test nor marked PLANNED. | spec/functional/FR-109-bound-supplied-value-traversal.md:119, spec/functional/FR-109-bound-supplied-value-traversal.md:157-169 |
| FND-002 | low | FR-122 Status says "existing QSV tests may cover subsets" but does not name them. 8 TC-906 tests tagged FR-369-AC-9 assert FR-122-AC-1's identical text, and 4 tests tagged FR-372-AC-5 back FR-122-AC-2. Those tags already match no QSV row and lose their quire-exact row once #56 merges. Naming the retag path would keep the PLANNED status honest and traceable. | spec/functional/FR-122-supplied-membership-owner-obligations.md:48, src/declaration.rs:5826-6254, src/declaration/membership.rs:3523-3776 |

## Coverage

origin/main: 93/103 rows backed, 9 unbacked, 0 contradicted. Head: 93/106 rows backed, 12 unbacked (+FR-122-AC-1, AC-2, AC-3), 0 contradicted. FR-109-AC-6 is unbacked in both.

Plan completion: not assessed
