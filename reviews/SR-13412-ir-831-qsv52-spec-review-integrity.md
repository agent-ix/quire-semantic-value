---
id: SR-13412
title: "IR-831 quire-semantic-value PR 52 integrity review of FR-124 and TC-923"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@f0cc3120be0a6f15eac62c9e2ca45e8b3630aca0 vs origin/main 9f1453a; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: reviews
---

## Summary

Ticket: IR-831. Integrity review: are FR-124's behaviors covered by its ACs, do the ACs follow
TC-923's steps, and can each AC be tested? Each AC was worked by hand against the source, and where it
mattered with a scratch probe (see SR-13409):

- AC-1, AC-3 and AC-5 can be tested as written. AC-5's equal-byte fixture can be built with
  `UnitGraph::from_checked_nodes`, which accepts arbitrary keys, plus a test-only
  `NodeKey::from_digest`; the probe confirmed it.
- AC-2's value check holds per QSpec U25.
- TC-923 steps 1-6 map one-to-one onto AC-1..AC-6. Its Expected Results match the ACs.
- IDs are sequential after FR-123 and TC-922. Section layout and the `verifies` edge follow the FR-123
  and TC-922 pattern.

## Examined scope

- FR-124#behavior-1 (examined): When the conversion's value is `ConvertedValue::Exact(v)`, `identified_quantity` SHALL return the quantity with magnitude `v` and the unit id of the conversion's target unit, paired with the `IdentifiedUnit` of that unit, whose `unit()` equals the conversion's `unit()`. It SHALL compute the unit's id once.
- FR-124#behavior-2 (examined): When the conversion's value is a decimal or an integer representation, `identified_quantity` SHALL return `HandoffRefusal::NotExact` and form no quantity: such a value is a number in the target representation, not a quantity of the unit.
- FR-124#behavior-3 (examined): When the target is a compound unit whose id the identity byte limit refuses, `identified_quantity` SHALL return `HandoffRefusal::Identity` with the originating `IdentityRefusal` and return no quantity and no unit.
- FR-124#behavior-4 (examined): `identified_quantity` SHALL charge no meter and read no clock or environment: the conversion has charged its work, and the result is a function of the conversion.
- FR-124#behavior-5 (examined): A caller that records the unit with `UnitScope::form` SHALL get back the id the quantity carries, and `UnitScope::resolve` of the quantity SHALL then return the same `QuantityUnit` as the conversion's `unit()`.
- FR-124#behavior-6 (examined): The declared and compound arms of `UnitId` SHALL stay distinct even when their digest bytes are equal, so a compound quantity never resolves to a declared unit.
- FR-124#behavior-7 (examined): The method SHALL use only the crate's existing dependencies, call no identity-minting method of `NodeKey`, and add no re-export (FR-060).
- FR-124-AC-1 (examined): An exact conversion of a declared quantity into a compound target unit returns a quantity whose magnitude is the converted exact value, whose unit id equals the `IdentifiedUnit`'s id, and whose `IdentifiedUnit::unit()` equals the conversion's `unit()`; an exact conversion into a declared unit does the same with the declared unit's id.
- FR-124-AC-2 (examined): After `UnitScope::form` of the returned `IdentifiedUnit`, the returned id equals the quantity's unit id and `UnitScope::resolve` of the quantity returns the conversion's `QuantityUnit`; converting the resolved quantity to the units of RT TC-187 U25's second step yields the exact value `1`, and the meter after the two conversions with the handoff between them equals the meter after the two `convert_quantity` calls charged back to back.
- FR-124-AC-3 (examined): A conversion to a decimal target and one to an integer target each return `HandoffRefusal::NotExact` and no quantity.
- FR-124-AC-4 (examined): A compound target whose id the identity byte limit refuses returns `HandoffRefusal::Identity` carrying the same `IdentityRefusal` as `QuantityUnit::id`, with no quantity and no unit, and a following `UnitScope::resolve` of any quantity of that unit finds nothing.
- FR-124-AC-5 (examined): A declared unit and a compound unit whose digest bytes are equal produce distinct unit ids from `identified_quantity`, and resolving the compound quantity returns the compound unit.
- FR-124-AC-6 (examined): The change adds no normal dependency, changes no existing public item, calls no `NodeKey::from_digest` in production, and passes `make build-no-std`, `make lint` and `cargo build --locked`.
- TC-923 (examined): Verify [FR-124](../functional/FR-124-identified-quantity-from-an-exact-conversion.md) through the public `quire_semantic_value::quantity` API: the pair that `Conversion::identified_quantity` returns, its retention and resolution in a `UnitScope`, its refusals, and that it adds no charge. Steps 1 to 5 are executable tests; step 6 is the implementing PR's inspection and build gate. The fixtures are RT TC-187 U25's units (a declared unit converted into a compound target, then into joules).
- TC-923#step-1 (examined): Convert a declared quantity exactly into a compound target and into a declared target; call `identified_quantity` on each; compare the magnitude, the unit id and the unit (AC-1).
- TC-923#step-2 (examined): Form the compound pair in a `UnitScope`, resolve the quantity, convert the resolved quantity into joules, and compare the value and the meter with two back-to-back `convert_quantity` calls (AC-2).
- TC-923#step-3 (examined): Convert to a decimal target and to an integer target and call `identified_quantity` (AC-3).
- TC-923#step-4 (examined): Build a compound target whose id the identity byte limit refuses, convert, and call `identified_quantity`; resolve a quantity of that unit (AC-4).
- TC-923#step-5 (examined): Build a declared unit and a compound unit with equal digest bytes; compare their unit ids and resolve the compound quantity (AC-5).
- TC-923#step-6 (examined): Inspect the public method, `Cargo.toml` and `Cargo.lock` diff; run `make build-no-std`, `make lint` and `cargo build --locked` (AC-6).

## Verdict

**PASS with findings (1 medium, 4 low).** The structure is sound. The medium finding is a TC
procedure that, followed as written, cannot reach the refusal it tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Step 4 says 'Build a compound target whose id the identity byte limit refuses, convert', but gives no fixture, and the obvious one cannot reach the handoff. `convert_quantity` checks base-dimension equality first. A compound target made large by a big exponent on one root (for example m^N) has a different dimension from its source, so it is refused as `IllTyped(IncompatibleDimensions)` before any `Conversion` exists (probe confirmed). `CompoundUnit::id` uses the fixed `IDENTITY_LIMITS` of 16 MiB, which no caller can lower. The refusal is reachable only through a target whose dimensions cancel. The probe used 200,000 roots of distinct dimension nodes that share one base map, with exponents +1 and -1, from a dimensionless source; that preimage needed 16,777,240 bytes. Fix: state the fixture in step 4 (a dimension-cancelling compound over the 16 MiB limit, or a target of the source's dimension times a cancelling remainder). | spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md:31-32 |
| FND-002 | low | The last clause of AC-4, 'a following `UnitScope::resolve` of any quantity of that unit finds nothing', is vacuous. A unit with no id has no quantity, since a `Quantity` carries a `UnitId`, so the clause has nothing to resolve and cannot fail. The method also takes no `UnitScope`, so it cannot retain anything. The real property is that no unit is retained. State it as the caller's `UnitScope` being unchanged (`into_formed()` holds no new unit), or drop the clause. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:107 |
| FND-003 | low | Behavior 1 says the method 'SHALL compute the unit's id once', but no AC or TC step checks it, and the public API cannot show it. Either drop the clause from Behavior 1 (it repeats what `IdentifiedUnit` already documents) or move it into AC-6's inspection. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:76-80 |
| FND-004 | low | AC-2's meter oracle is relative, and its baseline cannot be built without the handoff. 'Two `convert_quantity` calls charged back to back' needs a source `Quantity` for the second call. Only the handoff, or a hand-built `Quantity::new`, which is the method under test, can supply one. The specified method takes no `Meter`, so the comparison can only fail if the baseline is wrong. Use the absolute oracle the vectors already state: QSpec TC-187 U25 and RT FR-007-AC-23, where each step makes one `unit.identity-read`, `unit.target-domain` and `unit.result-retain` with no `unit.edge`. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:105 |
| FND-005 | low | AC-6's Verification cell reads 'Inspection' with no TC, but TC-923 step 6 claims AC-6. AC-6 also requires running `make build-no-std`, `make lint` and `cargo build --locked`, which is Analysis. The neighbouring FR-123-AC-6 and FR-121-AC-6 use 'Inspection and Analysis (TC-922)' and '(TC-921)'. Use 'Inspection and Analysis (TC-923)'. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:109 |

## Evidence

Probe: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/zz_probe_ir831.rs; quoin matrix: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/matrix.json (FR-124-AC-1..6 untagged / no run evidence, as expected for PLANNED).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1 head aa62a43e1 |
| FND-002 | fixed | round 1 head aa62a43e1 |
| FND-003 | fixed | round 1 head aa62a43e1 |
| FND-004 | fixed | round 1 head aa62a43e1 |
| FND-005 | fixed | round 1 head aa62a43e1 |
