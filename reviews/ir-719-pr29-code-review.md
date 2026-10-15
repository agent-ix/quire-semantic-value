---
id: SR-11072
title: "IR-719 code and Rust review of supplied Decimal admission"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; PR #29 6d892044 over 0e12429; src/declaration/membership.rs, src/supplied_membership.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---
# IR-719 code and Rust review

## Summary

Ticket: IR-719. Reviewed the two-file diff against QSV FR-109/TC-908, the pending QSV FR-109 amendment in spec PR #28, and exact-kernel FR-372 at the Cargo.lock revision 9a3411d. The direct bounded Decimal path checks the normalized coefficient and both declared endpoints, derives a finite alignment-shift ceiling from the admitted nonnegative minimum scale, then calls the authoritative fallible kernel with the original Cancel.

## Verdict

**PASS** — no production code or Rust-idiom defect found in the scoped diff. The separate gap analysis records missing acceptance tests. Full Cargo gates were not run during this review.

## Coverage

Examined FR-109-AC-1 through AC-5 and AC-7, proposed AC-8, exact FR-372-AC-5, and TC-908 steps 7 and 8. Checked the event charge, rooted error mapping, cancellation, size order, no-default path, and the unchanged unsupported branches for retained Decimal type links and Union. The QSV code uses the kernel helper rather than copying its arithmetic. No CI workflow, lockfile, or lint configuration changed. The declaration scale is u32 and nonnegative; `k = max(s, min_scale) - s` is bounded by the admitted `min_scale`. Production review is conditional on spec PR #28 merging before acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Round 1 re-review

Reviewed the rebased fix head `c507024d7faf51bea788048936497f8a6ba95cb2` against merged QSV spec `1140d400`. The extracted `decimal_value_result` is the sole production mapping for direct Decimal helper results and is tested with constructed resource variants. No new code or Rust-idiom defect was found. The separate gap artifact SR-11073 records the newly untagged FR-109-AC-8 criterion.
