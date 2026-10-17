---
id: SR-11976
title: "IR-830 ears-conformance review of QSV PR #40"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-semantic-value@a22519b3f16a10e9158082892c8eccf36aae6456; spec/functional/FR-120-model-transition-identity-type.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
---

## Summary

Ticket: IR-830. PR quire-semantic-value#40, reviewed head a22519b3f16a10e9158082892c8eccf36aae6456. This review checks the EARS conformance of FR-120's requirement statements (the Description and Behavior 1-7), and whether FR-120-AC-5 through AC-9 are singular and can fail.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-120 | examined | spec/functional/FR-120-model-transition-identity-type.md | Two values are equal exactly when their operation texts are equal, their receivers are equal (absent equals only absent) and their argument sequences are equal element by element. Argument order and count are significant, and two values that differ only in receiver are unequal. Equality agrees with equality of `canonical_bytes`. |
| FR-120-AC-5 | examined | spec/functional/FR-120-model-transition-identity-type.md | Values with the same operation, receiver and arguments are equal and have equal `canonical_bytes`; a different operation, a different argument, the same arguments in swapped order, a prefix of the arguments, a different receiver and a present receiver against an absent one are each unequal with unequal bytes, and two values differing only in receiver have identities differing only in `receiver`. |
| FR-120-AC-6 | examined | spec/functional/FR-120-model-transition-identity-type.md | Over operation texts whose nodes are `x` and `x/y` in one package, the order of the two values equals the order of their `canonical_bytes`, the opposite of the order of the two operation texts compared as raw strings; two values differing only in receiver order as their bytes do; over a set of at least 20 values the sorted order equals the order of sorted bytes, and `cmp` is antisymmetric and agrees with `==`. |
| FR-120-AC-7 | examined | spec/functional/FR-120-model-transition-identity-type.md | `Serialize` encoded with `quire-canonical` yields bytes equal to `canonical_bytes` for the AC-4 values. |
| FR-120-AC-8 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` and `canonical_bytes` under a `Limits` smaller than the input or output return a typed refusal naming the limit with site `Operation`, `Receiver` or `Argument(i)` as applicable, and a denied reservation returns `Allocation`; none panics or aborts. |
| FR-120-AC-9 | examined | spec/functional/FR-120-model-transition-identity-type.md | The public fields of `ModelTransition` and its inventory hold exactly `operation`, `receiver` and `arguments`: no probability, reward, weight or other member exists, and `ModelTransition` is the only new public transition type. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-8 bundles three checks (the `new` limit, the `canonical_bytes` limit, a denied reservation), and it contradicts the inventory. It requires `canonical_bytes` to refuse "with site `Operation`, `Receiver` or `Argument(i)`", but the inventory has `canonical_bytes` return `quire_canonical::Error`, which carries no site. It requires the refusal to name the limit, but `TransitionCause::Limit` carries no bound and `InvalidModelTransition` exposes none. Split AC-8 into one criterion per entry point and state what the `canonical_bytes` refusal carries. | spec/functional/FR-120-model-transition-identity-type.md:91,94-100,156 |
| FND-002 | low | AC-9 says "The public fields of `ModelTransition`", but the inventory makes the fields private (`/* private: operation, receiver, arguments */`). It also says "`ModelTransition` is the only new public transition type" while `InvalidModelTransition`, `TransitionSite` and `TransitionCause` are new public types. Reword it as "its private fields and public accessors" and "the only new public value type". | spec/functional/FR-120-model-transition-identity-type.md:79,94-100,157 |
| FND-003 | low | AC-6 does not say that the `x` and `x/y` values share receiver and arguments. `arguments` sorts first in RFC 8785, so with different arguments the arguments decide the order and the counterexample shows nothing. AC-6 also bundles three properties: the counterexample, receiver-only order, and the sorted-set and antisymmetry agreement. State "with equal receiver and arguments" and consider splitting it. | spec/functional/FR-120-model-transition-identity-type.md:154 |
| FND-004 | low | Behavior 4 (equality) has no SHALL; it is descriptive prose, unlike Behavior 1-3 and 5-7. Write it as "Two values SHALL be equal exactly when ...". | spec/functional/FR-120-model-transition-identity-type.md:129-131 |

## Verdict

Approve with findings for this method. The Description and Behavior 1-3 and 5-7 are ubiquitous SHALL statements with one responsible subject. AC-5 is a list of cases of one property and AC-7 is singular; both can fail (AC-7's substantive defect is in SR-11975 FND-001). AC-8 is compound and contradicts the inventory (medium). AC-9's wording and AC-6's missing precondition are low.

## Dispositions

Round 1 on reviewed head `b6004d37b13eff375c7ee7b4e6fc44b5c30ee9f8` against the review-pass head `a22519b3f16a10e9158082892c8eccf36aae6456`, scoped to `git diff a22519b..b6004d3` of FR-120 and TC-920. Each outcome was checked against the new spec text and, for the Encode claims, against quire-canonical at the locked revision 5dc4e12; the original finding rows above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-8 is now `new` only, with InputBytes{bound, required} and the per-text rule; `canonical_bytes` limits moved to AC-13 (IdentityRefusal, no site); allocation moved to AC-14. |
| FND-002 | fixed | AC-9 now reads 'private fields ... and its public accessors' and 'the only new public value type of the module besides the refusal types'. |
| FND-003 | fixed | AC-6 states 'With equal receiver and arguments' and the sort/antisymmetry property moved to AC-15. |
| FND-004 | fixed | Behavior 7 now reads 'Two values SHALL be equal exactly when ...'. |
