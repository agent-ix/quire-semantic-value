---
id: SR-9050
title: "IR-720 PR 21 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; src/declaration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-110
    type: reviews
---
# IR-720 PR 21 gap-analysis

## Summary

Ticket: IR-720. Independent review of frozen head aa6e8d00a1debfd0765525e927816af61caa6126, diff against origin/main. Rust lane is folded into code-review. FR-110-AC-1 through AC-7 were examined; the spec itself is unchanged. No applicable AssuranceProfile was found.

## Reviewed scope

- `FR-110-AC-1` — examined: Given a structurally valid model-derived redefinition whose complete QSL conformance stage refuses variance, multiplicity, or refinement after checked facts and all-model barriers, QSL returns its collected conformance refusal and abandons QSV's pending admission before QSV reports `RedefinitionWidens`; no completed QSV environment or attribute can be observed. QSV needs no QSL failure payload to abandon admission.
- `FR-110-AC-2` — examined: Given an unknown member type, a composite or supertype cycle, a missing supertype, an invalid `redefines` target, an undominated redefinition conflict, a duplicate name on unrelated flattened fields, exhausted `ancestor_steps` or `work_units`, or cancellation at an existing charge checkpoint, staged QSV admission stops with its established typed structural or resource failure before QSL's later conformance verdict. No ambiguous member or completed environment is exposed, and the exact bound/counter and cancellation handle are retained.
- `FR-110-AC-3` — examined: Given QSL acceptance for the identical model declaration set, effective attribute lookup and reference admission become available only after QSV completion and before dispatch finalization or public checked-graph sealing, with the same slot and lineage result and work accounting as strict admission of a valid set. If completion refuses or runs out of resources, no partial environment escapes.
- `FR-110-AC-4` — examined: Direct calls to `new`, `bounded`, and `bounded_with_cancel` still refuse an invalid redefinition target, undominated conflict, widening, and duplicate effective name with their existing QSV causes, and admit a valid redefinition with its existing effective attributes. No path from a pending result can expose a completed environment without QSV's final safety checks.
- `FR-110-AC-5` — examined: During pending admission, a source-backed `self.f` read for a redefining child returns the immediate parent's actual `FieldRef`, declared type and presence, while retaining the original child `self.f` source occurrence and QSL observation identity on the same checked-fact graph under the original budget and cancellation handle. It never substitutes the child's narrowed declaration, invents a fact or parent source occurrence, or permits evaluation. An unknown or ambiguous member refuses before a fact can use it.
- `FR-110-AC-6` — examined: The QSL-owned and direct entry points share QSV's structural admission and effective-attribute algorithms; no second model redefinition checker, unchecked public `TypeEnvironment`, or alternate completion path is introduced.
- `FR-110-AC-7` — examined: Given a structurally valid model whose child field `Reference<S>` redefines a parent field `Reference<T>`, where `S` is a proper subtype of `T`, multiplicity and presence are unchanged, and no exposed operation writes that field, the complete QSL conformance verdict and QSV finalization succeed. The completed QSV environment admits an `S` reference through the `T` field without changing its most-specific identity. Reversing `S` and `T` or using unrelated object types refuses. With an exposed writer narrowing that object-typed field, QSL instead refuses `undefined_expression`/`unproved-refinement` with obligation `no-proof-form`, regardless of authored facts; QSV finalization does not run. Numeric interval and presence narrowing retain their separate actual checked-fact route.

## Verdict

**FAIL.** Tagged tests do not assert several required acceptance cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The sole tagged AC-2 test omits unknown member type, composite and supertype cycles, missing supertype, and N-1/N ancestor/work limits; it checks only a zero limit and broad error kinds. Regressions in those required early typed refusals would leave this binder green. | src/declaration.rs:5651-5788 |
| FND-002 | high | The sole tagged AC-4 test calls only TypeEnvironment::new with a widening fixture. It never exercises bounded or bounded_with_cancel, invalid target, conflict, duplicate name, or valid effective attributes, so a direct-entry regression on those required cases would pass. | src/declaration.rs:5376-5407 |
| FND-003 | medium | The pending projection tests use identical parent/child types and required presence; they do not test a narrowed child type or presence, nor a rejected ambiguous parent member. A projection that substitutes the child declaration or accepts ambiguity could pass these tests. | src/declaration.rs:5413-5647 |
| FND-004 | medium | The tagged reference test proves a subtype through a standalone T reference and reverse refusal, but does not assert admission through the completed redefined parent field or unrelated-type refusal. It also does not cover the QSL writer/no-proof-form branch, which remains a downstream integration gate. | src/declaration.rs:5794-5842 |

## Coverage

All FR-110 ACs have matrix binders. The findings identify hollow or partial bindings. Reverse scope: the new pending API and subtype safety rule are owned by FR-110. No stubs or ignored new tests were found. Semantic review of the changed tests was performed manually for this PR.

Plan completion: not assessed

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Added staged unknown-member, missing-supertype and cycle controls, plus exact work and ancestor boundary checks. |
| FND-002 | fixed | A new table drives all three direct entry points through target, conflict, duplicate, widening, and valid-effective cases. |
| FND-003 | fixed | A narrowed child test checks the actual parent type and presence; structural flattening rejects undominated ambiguity before PendingTypeEnvironment exists. |
| FND-004 | deferred | QSV checks now use the completed parent field and refuse an unrelated reference. QSL-56 owns the exposed-writer no-proof-form verdict and remains the downstream integration gate. |
