---
id: SR-9049
title: "IR-720 PR 21 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-110
    type: reviews
---
# IR-720 PR 21 code-review

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

**FAIL.** Pending reads can exceed the retained work/cancellation budget.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A pending parent-field request scans every declared field and every parent slot but spends one work unit. A model with many fields can make repeated source-backed projections consume quadratic work under a small work limit; the same meter no longer bounds checked-fact work. Charge the traversed entries or use the admitted index. | src/declaration.rs:1013-1027 |
| FND-002 | medium | Pending composite and object iteration, object lookup, and conforms are public reads that neither charge the pending budget nor poll its retained cancellation handle. After cancellation, a caller can keep traversing declarations and querying ancestry while spent is unchanged. Route these restricted reads through the pending meter or define an explicit bounded unmetered contract. | src/declaration.rs:935-956 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The parent selection, field-key walk, slot walk, and lineage membership now charge the retained budget. |
| FND-002 | fixed | All pending declaration and ancestry read APIs now charge the retained budget and poll the cloned cancel handle. |
