---
id: SR-20261102072317
title: "IR-722 QSV safe ordered lookup code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value PR86; private OrderedIndex lookup change only; IR-722 prerequisite"
review_set: subset
---

## Summary

Code/Rust scoped verdict PASS: no findings. Merge readiness remains CONDITIONAL on actual candidate downstream panic gate and QSV aggregate evidence.

Frozen QSV86 source2cd0ec2148767326a1d878bef43054d88fd15a4e over25535387dd6091d82cc885e6764b86d2b052e2ce; exactly four inserted/two removed lines in private OrderedIndex::get/get_mut. Read repository AGENTS/CLAUDE, code/Rust style and scoped gap method. Spec review/sub-analyses are inapplicable because no spec, plan, public contract or domain-object change is authored. No source edits, Cargo/heavy/backend jobs, merge or closure by reviewer.

Both lookups retain the same binary_search_by comparator and .ok() missing-key behavior. A successful search returns a valid position in the same immutably borrowed slice; get_mut's shared search borrow ends before its exclusive projection, with no mutation between. Checked get/get_mut yields the same field reference while avoiding explicit indexing's generated bounds-panic branch. There is no unsafe code, callback, fallback identity/value, changed declaration admission, allocation, iteration order or asymptotic lookup behavior. No runtime bad-index counterexample is asserted. Other index operations, including remove, are unchanged and are not claimed globally panic-free.

The actual downstream RT native gate failure archive identifies three panic_bounds_check relocations in a QSV admits_walk_core monomorph emitted into RT's object, attributed to old declaration.rs2101. That is an emitted-code gate failure rather than observed invalid admission. Supplied focused QSV evidence: all-target Clippy with warnings denied passes; index scaling1/0; membership43/0 with one existing ignored. Tests preserve existing tags, including FR108-AC1 index scaling and FR109 native membership ownership. The source transformation is appropriate, but the candidate RT exact-panic gate regression and complete QSV makeci aggregate remain owed before merge. No gate weakened and no shim/admission bypass introduced.

The scoped computed QSV matrix was generated and inspected; there is no criterion/tag/spec delta, no new uncovered behavior or stub introduced by these equivalent private lookups. This review does not discharge pre-existing FR109 descriptor-helper cost premises or broader producer qualification. Original RT193 coverage/external-discharge, final compatible integration cohort, aggregate and IEEE holds remain separate and unchanged. No IEEE backend replay credit.

Plan completion: not assessed

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: OrderedIndex::get
  path: src/declaration.rs
  role: examined
  excerpt: .and_then(|at| self.entries.get(at))
- id: OrderedIndex::get_mut
  path: src/declaration.rs
  role: examined
  excerpt: .and_then(|at| self.entries.get_mut(at))
- id: FR-108-AC-1
  path: src/declaration.rs
  role: examined
  excerpt: fn ascending_registry_insertion_uses_linear_key_comparisons() {
- id: FR-109
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: 'id: FR-109'
```

## Readiness disposition

Verified source-identical QSVd62a536 (only the two SRs added above reviewed2cd0ec2). Preserved QSV aggregate has thirteen suites totaling278PASS/0FAIL/1existingignored, plus doctests; fmtcheck, all-target Clippy, thumb no_std Clippy/build, warning-denied docs and deny pass on QSV own lock. Separately, unchanged RT exact_panic_gate with candidate QSVd62 and retained RT Exacta305 passes both actual no-panic relocation probe and injected-panic rejection control (2/0). This supplies both previously owed readiness gates. QSV86 scoped readiness is PASS; no new findings or source changes. RT193 coverage/external-discharge, compatible integration cohort, full aggregate and IEEE holds remain unchanged; no backend/proof credit.

Current code/scoped-gap verdict and prerequisite readiness: **PASS**. No finding IDs were opened, so there are no finding dispositions to fabricate.
