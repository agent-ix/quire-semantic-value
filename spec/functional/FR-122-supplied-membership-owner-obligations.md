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
| FR-122-AC-1 | QSV's deferred field or tuple result uses `DeferredResultNotAdmitted`; missing enum variant and unresolved unit use their distinct rows; schedule mismatch and comparator refusal retain their existing rows; each equality operand failure uses its exact table row; and scale-zero integer placement uses `ExpectedIntegerPlacement`. The comparator and quantity conversion retain their original `IllTypedCause` payload, while prior non-result and charge behavior is unchanged. | Test |
| FR-122-AC-2 | An integrated supplied Decimal fixture with independently enumerated N owning logical events succeeds at N and denies its actual next event at N-1 before entering that event's helper. Reachable membership, nonmembership and original-handle cancellation retain the owning supplied-admission locus and already-spent budget across arguments, with no additional helper event, per-limb debit or partial descriptor. A separate QSV test passes constructed typed kernel allocation and capacity failures through its production error projection and retains their distinct causes, measured fields and rooted locus; quire-exact FR-372-AC-3 verifies the actual kernel failure mechanisms. | Test |
| FR-122-AC-3 | The QSV supplied caller documents and tests the authenticated finite B for value plus every endpoint, invokes the qualified helper after charging one initiating logical event, and preserves original cancellation and locus. The direct QSV caller does not default B; absent B returns `SuppliedAdmissionError::UnsupportedHelper { family: UnsupportedHelperFamily::NumericDomain, .. }`. The exact helper requires B as a parameter and selects no default under [quire-exact FR-380 AC-7](ix://agent-ix/quire-exact/FR-380). | Test |

## Dependencies

- [quire-exact FR-369](ix://agent-ix/quire-exact/FR-369), [FR-372](ix://agent-ix/quire-exact/FR-372) and [FR-380](ix://agent-ix/quire-exact/FR-380) define the originating obligations.
- [QSL FR-394](ix://agent-ix/quire-spec-language/FR-394) owns evaluator and caller-bound effects.

## Status

FR-122-AC-2 and FR-122-AC-3 are backed by the bounded Decimal and missing-bound supplied-caller tests in `src/declaration/membership.rs`. FR-122-AC-1 remains planned: the eight TC-906 tests in `src/declaration.rs` retain their former owner tags pending test evidence for the noninteger `ExpectedIntegerPlacement` branch and the remaining equality-operand rows.
