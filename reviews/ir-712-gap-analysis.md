---
id: SR-1341
title: "Gap analysis of QSV PR 6 typed checked-invariant evidence"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@5c11379791cd7b8f3c330337b21168d3de597634; src/declaration.rs, src/quantity.rs, spec/test-cases/TC-906-qsv-checked-invariant-causes.md; FR-369-AC-9"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-exact/FR-369
    type: reviews
---
# Gap analysis of QSV PR 6

## Summary

Ticket: IR-712. Planless review of the changed production sites, their tagged tests, QSV TC-906, and kernel FR-369-AC-9. The new tests exercise most reachable typed refusal causes but omit several independent behavior checks required by TC-906. Plan completion: not assessed.

## Verdict

**CONDITIONAL** — cause mapping is correct, but acceptance evidence has the gaps below. The QSV local matrix cannot bind the external kernel FR-369 criterion; this audit checked the `#[trace("TC-906", "FR-369-AC-9")]` tags and test bodies directly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-906 step 1 requires deferred record and tuple admission paths with an admitted value and an earlier refusal, but the added test calls the private `admitted()` helper with synthetic outcomes and asserts only failed admission, prior undefined, and prior incomplete outcomes. It does not reach a deferred record/tuple evaluation or assert admitted success. A broken caller could therefore escape the test. Add a tagged test through the deferred evaluation path covering one admitted composite and one failed member plus an earlier refused outcome. | src/declaration.rs:2754; spec/test-cases/TC-906-qsv-checked-invariant-causes.md:28 |
| FND-002 | medium | TC-906 steps 3-4 require an earlier charge stop on the scheduled comparator and quantity conversion paths, and successful integer quantity placement. The added tests assert only the two IllTypedCause refusals and never call integer placement. A change that converts a charge stop into CheckedInvariant, or breaks normal integer conversion, would pass. Add tagged tests for the prior stop on both paths and an ordinary successful integer placement. | src/declaration.rs:2949; src/quantity.rs:800; spec/test-cases/TC-906-qsv-checked-invariant-causes.md:47 |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.28.3); QSV local FR-060-AC-5 and FR-060-AC-6 remain method-without-symbol before this PR, while FR-106-AC-10 and FR-107-AC-1 are tagged. FR-369 is external and absent from the local matrix.
- Plan completion: not assessed
- Examined FR-369-AC-9: QSV's deferred field or tuple result uses `DeferredResultNotAdmitted`; missing enum variant and unresolved unit use their distinct rows; schedule mismatch and comparator refusal retain their existing rows; each equality operand failure uses its exact table row; and scale-zero integer placement uses `ExpectedIntegerPlacement`. The comparator and quantity conversion retain their original `IllTypedCause` payload, while prior non-result and charge behavior is unchanged.
- Examined TC-906: QSV reports the cause of each reached checked-invariant condition, including the five procedure steps and expected results.
- Existing tests cover source/target admission, nonintegral decimal, unresolved unit and enum, shape mismatch, schedule mismatch, and distinct IllTypedCause payloads. The nonexact placement branch is structurally unreachable through `QuantityTarget::Exact`; TC-906 explicitly calls for inspection there, which was done.
- Untraced behaviors / stubs: no new production behavior without an owning kernel or QSpec requirement; no new stub.
- Semantic review: checked FR-369-AC-9 and TC-906 intent against changed code and tests as requested by the review brief.

## Dispositions

Round 1, reviewed at c71053aa084a61d122e9cd08e6e5d688768f9b31. The focused `cargo test checked_invariant_tests` run passed 8 tests. Production mapping was unchanged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | c71053aa084a61d122e9cd08e6e5d688768f9b31; `deferred_record_and_tuple_evaluation_preserve_admission_and_prior_refusal` calls both public evaluation paths, asserts admitted composites, failed record admission, and the original tuple refusal. |
| FND-002 | fixed | c71053aa084a61d122e9cd08e6e5d688768f9b31; `scheduled_comparison_keeps_an_earlier_charge_stop` and `quantity_conversion_keeps_charge_stop_and_places_integer_successfully` assert charge-stop propagation and successful integer placement. |
