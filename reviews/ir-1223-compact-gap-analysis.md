---
id: SR-16689
title: "IR-1223 compact descriptor gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; IR-1223; src/declaration/admitted.rs; src/reader_admission.rs; tests/admitted_equality.rs"
review_set: subset
---

# IR-1223 compact descriptor gap-analysis

## Summary

Ticket IR-1223, PR quire-semantic-value#83. Reviewed frozen source 1993a4ba98b37c66882898d06c3c22ed606321fe against origin/main 7737daaa0c866693b56267a0df352b8a7f4e95e7. This is the user-directed compact-descriptor amendment to the merged QSV API, with the owning IR FR-353 inspected as contract context. Rust review is folded into code-review. No spec/plan edits warrant spec review. No applicable AssuranceProfile exists.

The descriptor removes quantity_type, declared key/root and compound terms. Materialization borrows the admitted UnitId key/compound terms; the declared root is the last retained edge target or the admitted declared key on an empty path. It neither recomputes identities nor selects/validates a path. Location, dimension declaration/base identities and all original conversion targets remain. The native unit retains the derived facts; opaque units() retains the full compact source descriptor and path. The private fields, hidden mint/seal convention and clone-only CheckedEquality adoption are unchanged. Wrong ID/form arms return a typed IdentityForm error at the original unit row, not a fabricated identity or panic. Its stable catalog is complete and round-tripped.

## Examined scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| EqualityUnitDescriptor (src/declaration/admitted.rs) | examined | `pub struct EqualityUnitDescriptor {` |
| EqualityUnitForm (src/declaration/admitted.rs) | examined | `pub enum EqualityUnitForm {` |
| unit (src/reader_admission.rs) | examined | `let root = path.last().map_or(key, |edge| &edge.target);` |
| unit-id-key (src/reader_admission.rs) | examined | `.declared_key()` |
| unit-id-terms (src/reader_admission.rs) | examined | `.compound_terms()` |
| IdentityForm (src/reader_admission.rs) | examined | `IdentityForm {` |
| materialization_codes_have_a_closed_round_trip_catalog (tests/admitted_equality.rs) | examined | `fn materialization_codes_have_a_closed_round_trip_catalog() {` |
| a_mismatched_unit_identity_arm_is_a_typed_materialization_failure (tests/admitted_equality.rs) | examined | `fn a_mismatched_unit_identity_arm_is_a_typed_materialization_failure() {` |
| selected_affine_path_materializes_canonical_without_reselecting_ids_or_conversion (tests/admitted_equality.rs) | examined | `fn selected_affine_path_materializes_canonical_without_reselecting_ids_or_conversion() {` |
| compound_descriptor_keeps_original_identity_dimensions_and_root_powers (tests/admitted_equality.rs) | examined | `fn compound_descriptor_keeps_original_identity_dimensions_and_root_powers() {` |
| a_long_retained_path_materializes_on_a_small_stack_without_a_depth_cap (tests/admitted_equality.rs) | examined | `fn a_long_retained_path_materializes_on_a_small_stack_without_a_depth_cap() {` |
| AdmittedEqualityDecision (src/declaration/admitted.rs) | examined | `pub struct AdmittedEqualityDecision {` |
| CheckedEquality::from_admitted_decision (src/declaration/admitted.rs) | examined | `decision.checked.clone()` |
| FR-353 (agent-ix/quire-contract-ir) | context_only | The value SHALL preserve source order and the producer's selected identities. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the reviewed compact-descriptor delta; no findings.

## Coverage

Plan completion: not assessed

Computed quoin matrix: 141 tagged criteria, 17 method-without-symbol criteria, five existing untagged criteria (FR-109-AC-6, FR-121-AC-6, FR-123-AC-6, FR-124-AC-6, FR-130-AC-3). These untouched baseline obligations are context, not introduced findings. Tests carry IR-1223 provenance without claiming the complete external IR reader contract. Manual delta mapping covers nonempty and empty declared paths, key derivation from the original UnitId, compound terms/dimensions/ID retention, real equality evaluation, all prior shared-budget refusal controls, long-path preservation and both wrong-arm failures. The negative private-field compile probe remains intact. No shims, depth/size caps, vendor copies, stubs, duplicate identity encoder or weakened gate were added. Optional repository-wide semantic audit was not requested.

Parent runtime receipt: full cargo test passed, including 152 library tests (one pre-existing ignored), all 13 admitted-equality integration tests, other integration targets and private-field doctest. Remaining full QSV gates are parent-owned. Reviewer independently inspected the diff/callers, Exact UnitId getters, native constructors, existing no-recheck adoption and private-field probe; ran git diff --check and computed the matrix. No Cargo rerun or Kani proof was performed.
