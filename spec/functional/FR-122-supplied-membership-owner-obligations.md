---
id: FR-122
title: "Preserve supplied membership and typed checked-invariant behavior at QSV owner paths"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: references
  - target: ix://agent-ix/quire-exact/FR-369
    type: depends_on
  - target: ix://agent-ix/quire-exact/FR-372
    type: depends_on
  - target: ix://agent-ix/quire-exact/FR-380
    type: depends_on
---
# FR-122: Preserve supplied membership and typed checked-invariant behavior at QSV owner paths

## Description

QSV SHALL preserve the shared typed checked-invariant causes at its producers and the supplied-membership budget, cancellation and projection at its own `src/supplied_membership.rs` path. This allocation carries quire-exact FR-369-AC-9, the QSV supplied-caller portion of FR-372-AC-5 and the QSV portion of FR-380-AC-6 without assigning evaluator behavior to QSV.

## Inputs

- Admitted declarations, supplied values, bounds, original cancellation handle and supplied-membership budget.

## Outputs

- Existing typed QSV checked-invariant or supplied-admission result and rooted locus.

## Behavior

The QSV producer paths SHALL use the precise shared causes. The supplied caller SHALL charge only its initiating logical event, invoke qualified exact helpers with authenticated bounds, and preserve typed causes and loci. QSL owns evaluator meter observations and omitted-bound default resolution.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-122-AC-1 | QSV's deferred field or tuple result uses `DeferredResultNotAdmitted`; missing enum variant and unresolved unit use their distinct rows; schedule mismatch and comparator refusal retain their existing rows; each reachable equality operand failure uses its exact table row; and scale-zero integer placement uses `ExpectedIntegerPlacement`. The comparator and quantity conversion retain their original `IllTypedCause` payload, while prior non-result and charge behavior is unchanged. | Test |
| FR-122-AC-2 | An integrated supplied Decimal fixture with independently enumerated N owning logical events succeeds at N and denies its actual next event at N-1 before entering that event's helper. Reachable membership, nonmembership and original-handle cancellation retain the owning supplied-admission locus and already-spent budget across arguments, with no additional helper event, per-limb debit or partial descriptor. A separate QSV test passes constructed typed kernel allocation and capacity failures through its production error projection and retains their distinct causes, measured fields and rooted locus; quire-exact FR-372-AC-3 verifies the actual kernel failure mechanisms. | Test |
| FR-122-AC-3 | The QSV supplied caller documents and tests the authenticated finite B for value plus every endpoint, invokes the qualified helper after charging one initiating logical event, and preserves original cancellation and locus. The direct QSV caller does not default B; absent B returns `SuppliedAdmissionError::UnsupportedHelper { family: UnsupportedHelperFamily::NumericDomain, .. }`. The exact helper requires B as a parameter and selects no default under [quire-exact FR-380 AC-7](ix://agent-ix/quire-exact/FR-380). | Test |
| FR-122-AC-4 | Inspection of the equality operand mapping in `src/declaration.rs` and the quantity conversion mapping in `src/quantity.rs` confirms that a decimal or integer placement returned for an exact equality quantity conversion maps to `EqualityQuantityNonExactPlacement`, while `operand_value` requests the hard-coded `QuantityTarget::Exact` and the converter returns only `ConvertedValue::Exact` for that target. If the equality quantity target becomes configurable, this criterion becomes a Test criterion for the newly reachable non-exact placement. | Inspection |

## Dependencies

- [quire-exact FR-369](ix://agent-ix/quire-exact/FR-369), [FR-372](ix://agent-ix/quire-exact/FR-372) and [FR-380](ix://agent-ix/quire-exact/FR-380) define the originating obligations.
- [QSL FR-394](ix://agent-ix/quire-spec-language/FR-394) owns evaluator and caller-bound effects.

## Status

FR-122-AC-1 is fully behaviorally backed and tagged by eight TC-906 tests in `src/declaration.rs` and one in `src/quantity.rs`. They assert the reachable deferred, equality, payload, prior-outcome and charge paths, plus `ExpectedIntegerPlacement` through the private converter with an admitted scale-one placement. FR-122-AC-4 covers the unreachable defensive `EqualityQuantityNonExactPlacement` row by Inspection: inspect `src/declaration.rs` at `operand_value` and its `ConvertedValue::Decimal(_) | ConvertedValue::Integer { .. }` arm, and `src/quantity.rs` at `convert_quantity`'s `QuantityTarget::Exact` mapping and `convert`'s `Target::Exact` arm. The hard-coded Exact target makes that defensive row unreachable today; if it becomes configurable, AC-4 becomes a Test criterion. AC-4 has no trace tag because Inspection mints no source symbol, so strict coverage lists it as an unbacked Inspection row. FR-122-AC-2 is backed by the bounded Decimal tests; FR-122-AC-3 is backed by the bounded Integer/Rational and missing-bound NumericDomain supplied-caller tests in `src/declaration/membership.rs`.
