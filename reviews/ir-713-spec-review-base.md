---
id: SR-4940
title: "IR-713 base review of QSV PR #7"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@a2ab97d4b369a05fe58d2b7afd7abca62da05bbc; spec/functional/FR-108-report-registry-allocation-failure.md; spec/test-cases/TC-907-registry-allocation-failure.md"
review_set: subset
---

## Summary

Ticket: IR-713. Checked ID, clarity, boundary, classification, and TC coverage for FR-108 and TC-907. The contract names an actual fallible reservation, its native request unit, a separate overflow failure, atomic admission, and preservation of prior refusal types.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-108 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | When storage for the type-environment registry, an index, or an admission traversal cannot be reserved, `quire-semantic-value` SHALL stop admission with a typed allocation failure that retains the request made at the failing reservation boundary. This failure is distinct from an invalid declaration, a configured environment limit, and canonical identity-preimage allocation. |
| FR-108-AC-1 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | At an actual fallible registry, index, or traversal reservation boundary after the storage redesign, an injected allocator denial returns the allocation member of `EnvironmentFailure` with the exact request amount and its byte or additional-element unit. No configured bound, setting, or declaration cause is present. The same declaration set admits when the denial is removed. |
| FR-108-AC-2 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | A reservation-size arithmetic or representation overflow returns a distinct capacity/size failure and is never reported as a measured allocator denial. A representable request preserves its original amount and unit through checked conversion where conversion is needed. |
| FR-108-AC-3 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | Denial during descriptor, index, key-join, or traversal construction exposes no partly admitted environment. Retrying the same declarations after removing the denial yields the complete deterministic registry and member links. |
| FR-108-AC-4 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | The existing exact `work_units` and `ancestor_steps` N-1/N controls retain their original limit kind, configured bound, and counter; invalid declarations, cancellation, canonical `IdentityRefusal::Allocation`, and released checked-invariant causes retain their established classifications. |
| FR-108-AC-5 | examined | spec/functional/FR-108-report-registry-allocation-failure.md | Every allocation-bearing growth path reachable from authored declarations during environment admission uses fallible construction and the new carrier; no ordinary `BTreeMap`/`BTreeSet` insertion, allocating clone, `Arc::new`, or unchecked vector growth remains on those paths. The replacement retains deterministic key order, lookup results, and work accounting. |
| TC-907 | examined | spec/test-cases/TC-907-registry-allocation-failure.md | Verify [FR-108](../functional/FR-108-report-registry-allocation-failure.md) at the production reservation boundary. The fixture must deny a selected fallible reservation without replacing the admission logic. It must observe the requested amount and unit at that boundary rather than synthesize an out-of-memory result from a malformed declaration or an exhausted budget. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

No defects found in the assigned spec diff at the reviewed commit.
