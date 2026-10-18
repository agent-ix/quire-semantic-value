---
id: SR-12410
title: "code-review review of QSV PR 44"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; spec/functional/FR-121-shared-property-and-fairness-values.md; spec/test-cases/TC-921-shared-property-and-fairness-values.md"
review_set: subset
---

## Summary

Ticket IR-872, PR #44 at 60353fe6e6bbeed566a229644d305e53093e5bd8. The two-file spec diff follows the QSV no_std module boundary, but it would create two authoritative fairness type contracts while QSL FR-123 remains unchanged.

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
| FND-001 | high | FR-121 declares FairnessConstraint with EffectiveId plus name while current QSL FR-123 declares the same shared type with operation: DeclarationKey; merging both leaves consumers with incompatible identity contracts. The PR discloses the decision and holds merge, but the QSL contract must be amended or the QSV definition withdrawn before merge. | spec/functional/FR-121-shared-property-and-fairness-values.md:27-34 |

## Evidence

Reviewed the two-file PR diff against QSV main, QSL main 77ab19b (FR-122, FR-123, FR-129 and ADR-011 X-11), and the current quire-canonical and quire-exact API where relevant. `quire validate` reports 2/2 changed documents grammar-clean and `git diff --check` passes. The PR body names four QSL decisions without asserting approval; none is silently resolved in this PR.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | QSV head `89708ce4c8dc1d5e23d25dc828a52b2217cce817` retains the `EffectiveId` plus name pair and records the companion QSL amendment; QSL PR #955 head `ab18f7e3cdc8567a90810fd28860aa6ceedb2b1f` changes FR-123 to QSV-owned `FairnessOperation` with that pair, removes the `DeclarationKey` value definition, and passed independent spec-content review plus merged-tree spec validation. QSL #955 remains open and must land before QSV #44 merges. |
