---
id: SR-16686
title: "IR-1223 QSV code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; IR-1223; src/declaration.rs; src/declaration/admitted.rs; src/enumeration.rs; src/lib.rs; src/quantity.rs; src/reader_admission.rs; src/unit.rs; tests/admitted_equality.rs"
review_set: subset
---

# IR-1223 QSV code-review

## Summary

Ticket IR-1223, PR quire-semantic-value#82. Reviewed frozen source 2130a98ab0dea1edc0932c7f6a74a7231297d62e against origin/main070a193017cd45d20010ab03fa902a6646655aa4. Rust review is folded into code-review. No spec/plan edits, so spec-review does not apply. No applicable AssuranceProfile exists.

The opaque decision retains application/occurrence, both operand types/conversions, selected schedule, all supplied held unit rows and original enum members. Materialization uses private native assembly without identity synthesis, member filtering or checker calls. The affine composition has one implementation shared with ordinary unit construction; its Exact meter charges precede each arithmetic operation and preserve original Incomplete/locus. Adoption clones the completed checked plan. Public hidden seal/mint callability is documented honestly; IR/QSL structural audits and read mapping belong to their assigned owners rather than this QSV delta.

## Examined scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| mint_admitted_equality_decision (src/reader_admission.rs) | examined | pub fn mint_admitted_equality_decision( |
| EqualityMaterializationCode (src/reader_admission.rs) | examined | pub enum EqualityMaterializationCode { |
| AdmittedEqualityDecision (src/declaration/admitted.rs) | examined | pub struct AdmittedEqualityDecision { |
| CheckedEquality::from_admitted_decision (src/declaration/admitted.rs) | examined |         decision.checked.clone() |
| admitted exports (src/declaration.rs) | examined | mod admitted; |
| reader_admission visibility (src/lib.rs) | examined | pub mod reader_admission; |
| UnitEdge::followed_by (src/unit.rs) | examined |         let scale = arithmetic.multiply(&edge.scale, &self.scale)?; |
| Dimension::from_admitted (src/unit.rs) | examined |     pub(crate) fn from_admitted(terms: Vec<(NodeKey, Integer)>) -> Self { |
| CompoundUnit::from_admitted (src/unit.rs) | examined |     pub(crate) fn from_admitted(terms: Vec<(NodeKey, Integer)>, dimension: Dimension) -> Self { |
| admitted enum construction (src/enumeration.rs) | examined |     pub(crate) fn record_admitted(&mut self, variant: VariantId, member: EnumValue) { |
| rational_event (src/quantity.rs) | examined | pub(crate) fn rational_event( |
| plan_adoption_keeps_provenance_operator_and_source_order (tests/admitted_equality.rs) | examined | fn plan_adoption_keeps_provenance_operator_and_source_order() { |
| an_admitted_composite_needs_no_rechecking_environment (tests/admitted_equality.rs) | examined | fn an_admitted_composite_needs_no_rechecking_environment() { |
| text_schedule_retains_the_original_exact_type_and_normalization_profile (tests/admitted_equality.rs) | examined | fn text_schedule_retains_the_original_exact_type_and_normalization_profile() { |
| selected_affine_path_materializes_canonical_without_reselecting_ids_or_conversion (tests/admitted_equality.rs) | examined | fn selected_affine_path_materializes_canonical_without_reselecting_ids_or_conversion() { |
| compound_descriptor_keeps_original_identity_dimensions_and_root_powers (tests/admitted_equality.rs) | examined | fn compound_descriptor_keeps_original_identity_dimensions_and_root_powers() { |
| materialization_shares_remaining_work_and_returns_no_partial_value_at_n_minus_one (tests/admitted_equality.rs) | examined | fn materialization_shares_remaining_work_and_returns_no_partial_value_at_n_minus_one() { |
| rational_growth_refuses_before_the_first_unpaid_fold_operation (tests/admitted_equality.rs) | examined | fn rational_growth_refuses_before_the_first_unpaid_fold_operation() { |
| each_unpaid_affine_event_keeps_its_original_typed_locus (tests/admitted_equality.rs) | examined | fn each_unpaid_affine_event_keeps_its_original_typed_locus() { |
| a_long_retained_path_materializes_on_a_small_stack_without_a_depth_cap (tests/admitted_equality.rs) | examined | fn a_long_retained_path_materializes_on_a_small_stack_without_a_depth_cap() { |
| enum_case_copy_respects_the_callers_byte_provision_without_normalizing (tests/admitted_equality.rs) | examined | fn enum_case_copy_respects_the_callers_byte_provision_without_normalizing() { |
| enum_materialization_preserves_utf8_cases_original_ids_positions_and_filter (tests/admitted_equality.rs) | examined | fn enum_materialization_preserves_utf8_cases_original_ids_positions_and_filter() { |
| materialization_codes_have_a_closed_round_trip_catalog (tests/admitted_equality.rs) | examined | fn materialization_codes_have_a_closed_round_trip_catalog() { |
| FR-353 (agent-ix/quire-contract-ir/spec/checked_package/functional/FR-353-read-admitted-equality-decisions.md) | examined | QSV SHALL own `AdmittedEqualityDecision` with private fields and no public constructor from raw decision fields. QSV SHALL expose a `#[doc(hidden)]` mint requiring a QSV-defined `ReaderSeal`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new public stable code enum has no stable spelling catalog, as_str, all/ALL enumeration, or from_code parser. Rust-style wire-and-persistence requires these for each surfaced code enum; existing SelectionRefusalCode demonstrates the local catalog pattern. Add these to the owning enum and verify round trips and unknown-code refusal. | src/reader_admission.rs:90 |

## Verdict

CONDITIONAL: one low API-contract finding.

## Coverage

Plan completion: not assessed

Computed `quoin matrix --repo . --json`: 141 tagged,17 method-without-symbol,5 existing untagged criteria (FR-109-AC-6,FR-121-AC-6,FR-123-AC-6,FR-124-AC-6,FR-130-AC-3). Those untouched repository obligations are baseline context, not introduced findings. Owning admitted-decision contract is external IR FR-353 at d9aadcea489f7e24e01635a8581be9d28e800d1c, read directly, and ticket API proposals033ae273/f65f343b. The new tests carry IR-1223 provenance rather than claiming entire IR read/join criteria. Manual component coverage includes all four schedules, provenance, original UTF-8/identities/order, compound dimensions, two-edge affine offsets, shared-budget exact/N-1, all six denied arithmetic loci, byte-bound copying and 2048-edge small-stack materialization. The external negative compile probe uses fully typed fields and checks private construction. No stub, duplicate arithmetic owner, vendoring, gate narrowing or newly unowned behavior found. Optional repository-wide semantic audit skipped; no plan supplied.

Parent gate evidence: formatter and all-target Clippy passed;152 library tests passed,1 existing ignored;all11 admitted-equality tests passed;private-field compile-fail doctest passed. no_std target is with parent. Reviewer independently ran git diff --check (passed), matrix computation and source/caller inspection; no redundant Cargo runs or Kani proof. Runtime evidence is supplied by the lead, not independently rerun.

## Dispositions

Round 1 inspected fix491e152ce707f5741bf599a4471bb420c35c47f8 against the frozen candidate. The code enum now owns all four literal spellings, the complete ALL catalog and exact from_code lookup. The public integration test fixes the literal spelling order, round-trips every entry and rejects empty, case-altered, unknown and trailing-space inputs. No new finding arose from the fix. Runtime rerun is with the parent.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | EqualityMaterializationCode now exposes documented ALL, as_str and from_code; the public regression checks all four exact spellings and round trips plus four unknown spellings. |

Current disposition verdict: PASS; the original finding and verdict remain above as review history.
