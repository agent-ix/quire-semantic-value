---
id: SR-5341
title: base review of FR-110 and TC-909
type: SpecReview
analysis: base
scope: agent-ix/quire-semantic-value@56fc5ae8676e8f5bab7c60e12f092d5deae78cb3; spec/functional/FR-110-model-conformance-precedence.md;
  spec/test-cases/TC-909-model-conformance-precedence.md
review_set: subset
---

## Summary

Independent review of IR-720 frozen PR #9 at 56fc5ae8676e8f5bab7c60e12f092d5deae78cb3. The examined units are listed below.

## Examined scope

- FR-110-AC-1 (examined): Given a structurally valid model-derived redefinition whose complete QSL conformance stage refuses variance, multiplicity, or refinement after checked facts and all-model barriers, QSL returns its collected conformance refusal and abandons QSV's pending admission before QSV reports `RedefinitionWidens`; no completed QSV environment or attribute can be observed. QSV needs no QSL failure payload to abandon admission.
- FR-110-AC-2 (examined): Given an unknown member type, a composite or supertype cycle, a missing supertype, an invalid `redefines` target, an undominated redefinition conflict, a duplicate name on unrelated flattened fields, exhausted `ancestor_steps` or `work_units`, or cancellation at an existing charge checkpoint, staged QSV admission stops with its established typed structural or resource failure before QSL's later conformance verdict. No ambiguous member or completed environment is exposed, and the exact bound/counter and cancellation handle are retained.
- FR-110-AC-3 (examined): Given QSL acceptance for the identical model declaration set, effective attribute lookup and reference admission become available only after QSV completion and before dispatch finalization or public checked-graph sealing, with the same slot and lineage result and work accounting as strict admission of a valid set. If completion refuses or runs out of resources, no partial environment escapes.
- FR-110-AC-4 (examined): Direct calls to `new`, `bounded`, and `bounded_with_cancel` still refuse an invalid redefinition target, undominated conflict, widening, and duplicate effective name with their existing QSV causes, and admit a valid redefinition with its existing effective attributes. No path from a pending result can expose a completed environment without QSV's final safety checks.
- FR-110-AC-5 (examined): The QSL-owned and direct entry points share QSV's structural admission and effective-attribute algorithms; no second model redefinition checker or alternate completion path is introduced.
- TC-909 (examined): Verify [FR-110](../functional/FR-110-model-conformance-precedence.md)
through QSV's strict entry points and its QSL-owned model-assembly boundary.

## Verdict

**FAIL** — The findings below prevent spec approval.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A conforming Reference<Subtype> redefinition can fail QSV completion as RedefinitionWidens: QSpec FR-151 and FR-149 admit subtype covariance, but QSV narrows() in src/declaration.rs:1462-1485 accepts references only when the types are identical. TypeEnvironment::admits at 936-942 already supports reference subtypes. Specify how final QSV widening safety checks honor a completed QSL verdict for this valid case, and test it without bypassing runtime safety. | spec/functional/FR-110-model-conformance-precedence.md:95 |

## Evidence

Review compares the PR diff with QSV src/declaration.rs, QSL FR-082 and current check-stage field typing, and QSpec FR-149/FR-151. TC-909 is planned; this review does not claim implementation or runtime tests exist.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | d72a6fbc38c41f9bf67e1c33d79d109a038c21f5 |
| FND-002 | still-open | QSpec FR-151 gives narrowed object types no FR-146 proof form; the new writer case must require `unproved-refinement`/`no-proof-form`, not an established fact. |
| FND-002 | fixed | 7fda883ef0a6075527045858617de36101751b28 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | A narrowed reference field with an exposed writer is described as satisfiable by an established postcondition fact. QSpec FR-151 explicitly has no FR-146 proof form for a narrowed object type and requires `unproved-refinement`/`no-proof-form`. Separate the subtype no-writer success case from this mandatory refusal; reserve checked-fact success for expressible numeric or presence narrowing. | spec/functional/FR-110-model-conformance-precedence.md:123 |
