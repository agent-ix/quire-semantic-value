---
id: SR-11078
title: "IR-714 PR 30 union membership gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/declaration/membership.rs, tests/union_registry.rs, tests/union_staging.rs, FR-109, FR-115, TC-908, TC-915; PR 30 head 3c9974d27bbc36bd22aec106f1b47abefb917694"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/FR-115
    type: reviews
---

# IR-714 PR 30 union membership gap analysis

## Summary

Ticket: IR-714. Compared the changed union membership path and enabled tests with the computed Test Matrix and the FR-109/FR-115 criteria. The matrix contains trace tags for this slice, but two assertions are missing behind those tags.

## Verdict

**CONDITIONAL** — two medium oracle gaps; the implemented behavior passed the focused tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Tagged union failure tests do not assert typed member and payload causes with rooted loci | tests/union_registry.rs:208; tests/union_staging.rs:194 |
| FND-002 | medium | The after-sealing Record/Tuple control omits the required Option-wrapped carrier refusals | tests/union_staging.rs:138 |

## Coverage

Plan completion: not assessed

- The strict computed matrix reports FR-109-AC-6 as the sole untagged criterion. It is a pre-existing QSL-772 cross-phase integration dependency outside this union diff.
- `supplied_trusted_carriers_still_require_registry_admission` checks several wrong bindings and short or wrong-kind payloads only with boolean `admits` results. The newly tagged `sealed_union_payload_uses_one_rooted_supplied_budget` checks limit results for a valid carrier. A regression mapping a bad member to `Payload`, or short arity to `Member`, would leave these assertions green. Assert `admit_supplied` cause and descriptor locus for representative wrong member, wrong binding and wrong arity cases.
- The before-sealing test `record_and_tuple_carriers_are_not_admitted_at_a_union_type` covers direct and Option-wrapped carriers at a handle. The after-sealing test covers handle and final-key direct carriers but no Option wrapper. An Option-specific regression after the final-key join would leave the AC-5 controls green.
- The 27 union integration tests passed. The source path has no stub, `todo!`, test-only bypass, or copied kernel implementation in the changed lines.

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
```

## Bindings checked

```yaml
bindings:
  - {test_id: sealed_union_payload_uses_one_rooted_supplied_budget, ac_id: FR-109-AC-1, trace: correct}
  - {test_id: sealed_union_payload_uses_one_rooted_supplied_budget, ac_id: FR-109-AC-4, trace: correct}
  - {test_id: union_payload_worklist_denial_is_typed_and_retry_succeeds, ac_id: FR-109-AC-5, trace: correct}
  - {test_id: sealed_union_final_key_never_admits_a_record_or_tuple_carrier, ac_id: FR-115-AC-5, trace: correct}
  - {test_id: record_and_tuple_carriers_are_not_admitted_at_a_union_type, ac_id: FR-115-AC-5, trace: correct}
```
