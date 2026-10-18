---
id: SR-12480
title: "dependency review of QSV PR 44"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-semantic-value; spec/functional/FR-121-shared-property-and-fairness-values.md; spec/test-cases/TC-921-shared-property-and-fairness-values.md"
review_set: subset
---

## Summary

Ticket IR-872, PR #44 at 60353fe6e6bbeed566a229644d305e53093e5bd8. The QSL contract amendment is a declared pre-merge dependency in prose, but it has no structured dependency edge.

## Examined scope

- **FR-121** (examined): quire-semantic-value SHALL provide the checked property-form and fairness value types consumed by QSL, IR and CG in a public fairness module.
- **FR-121-AC-1** (examined): Public construction and access through `quire_semantic_value::fairness` round-trip all four `PropertyForm` variants, both `FairnessKind` variants, both `FairnessGranularity` variants and all four kind/granularity pairs; an operation retains its supplied declaring-type `EffectiveId` and name.
- **FR-121-AC-2** (examined): Two `FairnessOperation` values with the same name and different declaring-type IDs are unequal, while equal IDs and names are equal; two `FairnessConstraint` values are equal exactly when kind, operation pair and granularity all match. Equal values have equal hashes, and unequal variants or fields remain distinct in a `HashSet`.
- **FR-121-AC-3** (examined): `Ord` and `PartialOrd` agree with `Eq`: property forms order `ReachableInvariant < BoundedMltl < Safety < Liveness`, kinds `Weak < Strong`, granularities `Whole < Each`; operations order by declaring-type digest bytes then name bytes, and constraints by kind, operation, granularity. A sort of fixtures in reverse insertion order produces the same order and retains both same-named operations of different declaring types.
- **FR-121-AC-4** (examined): Under a limit that admits the output, `quire_canonical::to_vec` of a weak-whole constraint on operation `tick` with an all-zero declaring-type digest equals the RFC 8785 bytes of `{"granularity":"whole","kind":"weak","operation":{"declaring_type":"0000000000000000000000000000000000000000000000000000000000000000","name":"tick"}}`; changing kind, granularity, declaring type or name changes those bytes, and a name containing a quote is escaped canonically.
- **FR-121-AC-5** (examined): `quire_canonical::to_vec` of each `PropertyForm` yields its specified JSON string spelling; encoding a `FairnessOperation` embeds its lower-case declaring-type digest and name, and an output limit one byte below the required bytes returns a `quire_canonical::Error::Limit` rather than a truncated successful value.
- **FR-121-AC-6** (examined): The implementing change adds no normal dependency, changes no existing public item or re-export, calls no `EffectiveId::from_digest` in production, and passes `make build-no-std`, `make lint` and `cargo build --locked` with the existing dependency set.
- **TC-921** (examined): Verify FR-121 through the public quire_semantic_value::fairness API; steps 1-5 test construction, equality, ordering and encoding, and step 6 inspects dependencies and build gates.
- **FR-123** (context_only): A checked temporal clause holds Vec<FairnessConstraint> with FairnessConstraint{kind: FairnessKind, operation: DeclarationKey, granularity: FairnessGranularity}; PropertyForm has the four named variants.
- **FR-129** (context_only): The checker resolves an unmarked fairness kind to Weak, admits Strong, and keeps the resolved kind in checked identity.
- **FR-122** (context_only): FR-122 concerns replay of state-clause counterexamples, not shared property or fairness values.
- **ADR-011** (context_only): X-11 locates quire-semantic-value as the no_std shared value leaf below the QSL and consumer layers.

## Verdict

**FINDINGS**. The cross-repository contract or dependency needs resolution before merge.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-121 says QSL FR-123 must confirm and amend its DeclarationKey output before this requirement merges, while FR-121 lists FR-123 only as references and IR-872 has no Linear relation. Record the actual blocking amendment/decision as a dependency so the PR cannot appear ready while the conflicting contract remains. | spec/functional/FR-121-shared-property-and-fairness-values.md:8-12 |

## Evidence

Reviewed the two-file PR diff against QSV main, QSL main 77ab19b (FR-122, FR-123, FR-129 and ADR-011 X-11), and the current quire-canonical and quire-exact API where relevant. `quire validate` reports 2/2 changed documents grammar-clean and `git diff --check` passes. The PR body names four QSL decisions without asserting approval; none is silently resolved in this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | rejected | A structured `FR-121 depends_on FR-123` edge would invert the value-provider-to-checker dependency; both companion PRs also use IR-872, so a Linear self-block is invalid. At QSV head `89708ce4c8dc1d5e23d25dc828a52b2217cce817`, FR-121 Description and Dependencies name open QSL PR #955, and the PR body names its open amendment and QSL ownership. QSL #955 head `ab18f7e3cdc8567a90810fd28860aa6ceedb2b1f` amends FR-123 to the same `EffectiveId` plus name pair. The merge lead has the QSL-first sequence; no missing FR or ticket dependency remains. |
