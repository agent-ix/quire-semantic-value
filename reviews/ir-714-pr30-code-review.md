---
id: SR-11077
title: "IR-714 PR 30 union membership code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration/membership.rs, tests/union_registry.rs, tests/union_staging.rs, FR-109, FR-115, TC-908, TC-915; PR 30 head 3c9974d27bbc36bd22aec106f1b47abefb917694"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---

# IR-714 PR 30 union membership code review

## Summary

Ticket: IR-714. Reviewed the Rust diff and tests at head 3c9974d27bbc36bd22aec106f1b47abefb917694 against c67c19be2596a4c5591ec5a56e52e936e4d72e9a. The union arm checks the sealed final key and complete member binding before scheduling positional payloads, and preserves typed event, cancellation, storage and rooted locus boundaries. The spec's delivery status now contradicts the implementation and enabled tests.

## Verdict

**CONDITIONAL** — one medium documentation drift finding; no confirmed Rust behavior defect in this diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Union delivery status still says the membership arm and dependent tests are absent | spec/functional/FR-115-stage-and-seal-union-declarations.md:88; spec/test-cases/TC-915-union-topology-sealing-and-storage.md:45 |

## Evidence

- `src/declaration/membership.rs:569-615` emits `DescriptorQuery`, resolves the union handle and verified final key, checks the exact member and payload arity, then schedules payloads through the shared frontier. A non-Union composite expected type returns `UnsupportedUnion` before admitting a Union carrier; a Record or Tuple carrier is sent through `composite_conforms`, which refuses union types.
- The new `same_type_walk` composite check resolves the final-key/handle pair with the environment's existing `same_type` method. The registry's ordered indexes use binary search; the descriptor comparison is bounded by admitted registry size and the two retained identifier lengths, with no temporary allocation. The callback polls after the helper.
- `cargo test --test union_registry --test union_staging` passed 27 tests. `git diff --check` passed. CI/workflow files did not change. No full Cargo gate or Kani was run in this diff-scoped review.
- The FR-115 Status still says the walk refuses every Union value as `UnsupportedUnion`, and TC-915 Status says the dependent controls remain ignored. Both statements become false at this PR head. Update these status sections in this PR so the reviewed artifacts describe the shipped behavior; retain the separate QSL and Decimal obligations as pending.

## Examined scope

```yaml
scope:
  - {id: FR-109-AC-1, path: spec/functional/FR-109-bound-supplied-value-traversal.md, role: examined, excerpt: "A source-enumerated supported membership fixture requiring N logical events completes at N and fails at N-1 before the last event, retaining supplied-admission phase, configured N-1, successful spend N-1, next amount one and the actual denied event/locus. Zero denies the first event. No denied event is performed or spent."}
  - {id: FR-109-AC-4, path: spec/functional/FR-109-bound-supplied-value-traversal.md, role: examined, excerpt: "Invalid kind/type and member/payload failures permitted by the owning supported-family contracts return their typed membership cause and locus, never a partial admission. Released nested construction custody is preserved; a helper cannot manufacture extra payload visits."}
  - {id: FR-109-AC-5, path: spec/functional/FR-109-bound-supplied-value-traversal.md, role: examined, excerpt: "An actual fallible supplied-worklist reservation denial reports its measured request and native unit; checked-size overflow has its own classification. Neither becomes a traversal limit, declaration-storage failure or invalid value. Retry after removing the denial yields the same membership outcome."}
  - {id: FR-109-AC-7, path: spec/functional/FR-109-bound-supplied-value-traversal.md, role: examined, excerpt: "Each claimed supported helper path has a source-backed input bound, finite comparison/arithmetic and temporary-storage bound, cancellation response boundary and fallible storage route where needed."}
  - {id: FR-115-AC-1, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "Resolved identifiers and ordered position types are readable before binding, including named Option-escaping recursion and all-member IEEE inspection; unsealed runtime member construction and admission refuse."}
  - {id: FR-115-AC-2, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "Exact complete verified sealing retains the original handle and topology and exposes inverse final-key joins. Wrong count/order/name/declaration, duplicate member keys, resealing and key collisions preserve their typed cause and leave the environment unchanged."}
  - {id: FR-115-AC-3, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "Actual Union registry, traversal and each sealing destination reservation can deny with FR-108's exact native request and classification. No partial binding or join is exposed; identical input succeeds after removing denial. Byte-request and capacity controls retain distinct carriers."}
  - {id: FR-115-AC-4, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "Union topology admission charges the environment's admission work limit with exact N/N-1 behaviour over the 5,000-member and 1,000-Option fixtures, and observes its cancellation handle. Released non-Union limits and both Record/Tuple prior-stop regressions remain unchanged."}
  - {id: FR-115-AC-5, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "A Record or Tuple carrier keyed with a union's internal handle or final key is not admitted at that union's type, directly or inside an Option, before and after sealing."}
  - {id: FR-115-Status, path: spec/functional/FR-115-stage-and-seal-union-declarations.md, role: examined, excerpt: "Union VALUE membership through the typed-event admission walk (FR-109) is not implemented here: that walk still refuses a union value as `UnsupportedUnion` until its union arms land"}
  - {id: TC-915-Status, path: spec/test-cases/TC-915-union-topology-sealing-and-storage.md, role: examined, excerpt: "Implemented except the controls that need Union value membership. Those are tracked in the pull request under \"waits for the union arms in admits_walk_core\""}
```

## Review limits

This PR does not complete IR-714's Numeric/Decimal or QSL cross-phase requirements. The strict matrix's one untagged criterion is FR-109-AC-6, already owned by QSL-772 integration, and is not caused by this diff.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At head 1dd12c28453f4c9b5a6f96418928380357ecf10c, FR-115 and TC-915 Status now describe enabled typed-event Union membership and keep separate consumer obligations distinct. |
