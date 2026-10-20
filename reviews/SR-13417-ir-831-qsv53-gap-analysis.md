---
id: SR-13417
title: "IR-831 quire-semantic-value PR 53 gap analysis: FR-124 and TC-923 against the implementation"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@175f21fc53067f94374731116bfb9fdbd372d192 vs origin/main 9270383; src/quantity.rs, tests/identified_quantity.rs, spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: references
---

## Summary

Ticket: IR-831. Planless gap analysis (Plan completion: not assessed) of PR #53 against merged FR-124 and
TC-923 (QSV #52, merge 9270383). `quoin matrix --json` (quoin 0.28.3): FR-124-AC-1..5 each have one binder and are
`tagged`. AC-6 is `untagged`, which is expected for its Inspection and Analysis method; the evidence status
everywhere is 'no run evidence' because no run evidence is ingested.

Behavior to code to test:
- B1: let-else Exact plus `Quantity::new(v, unit.id())`; tested by AC-1, for a compound target and a declared target.
- B2: let-else returns `NotExact`; tested by AC-3, for decimal and integer targets.
- B3: `?` lifts any IdentityRefusal; tested by AC-4 with InputBytes, compared to `QuantityUnit::id`.
- B4: no `Meter` parameter, no clock or env; verified by inspection.
- B5: form, then resolve; tested by AC-2.
- B6: arm comes from `QuantityUnit::id`; tested by AC-5, with domains asserted.
- B7: no dependency, mint or re-export; verified by inspection (AC-6).

AC-6 inspection: the diff adds only `identified_quantity` and `HandoffRefusal`. `Cargo.toml` and `Cargo.lock` are
unchanged. Production code has no `from_digest`. `make build-no-std`, `make lint` and `cargo build --locked` are
green as part of `make ci`, EXIT 0.

TC-923 step 4 term order was checked against quire-canonical 5dc4e12 `src/writer.rs:478-492`. The `required` value
is the encoder's running byte count at the write that crosses the bound; it is not the preimage length. Results
for the 200,000-root fixture:
- +1 then -1 in ascending id: 16,777,240.
- alternating, +1 first: 16,777,250.
- alternating, -1 first: 16,777,251.
- -1 then +1: 16,777,251.
- full preimage in all four orders: 30,500,052 bytes.
The coder's claim is confirmed.

## Examined scope

- FR-124 (examined): `quire-semantic-value` SHALL turn a completed exact quantity conversion into the pair a later quantity operation needs: a kernel `Quantity` and the `IdentifiedUnit` that carries the quantity's unit id. `convert_quantity` returns a `Conversion` that holds the converted value and a bare `QuantityUnit`. A later operation reads a `Quantity` through `UnitScope::resolve`, which needs the quantity's `UnitId` and the unit retained under it. Without the handoff a consumer would compute a compound unit's id and keep its own table, which the runtime's single-core rule forbids: RT FR-275-AC-23 gives compo
- FR-124#behavior-1 (examined): When the conversion's value is `ConvertedValue::Exact(v)`, `identified_quantity` SHALL return the quantity with magnitude `v` and the unit id of the conversion's target unit, paired with the `IdentifiedUnit` of that unit, whose `unit()` equals the conversion's `unit()`.
- FR-124#behavior-2 (examined): When the conversion's value is a decimal or an integer representation, `identified_quantity` SHALL return `HandoffRefusal::NotExact` and form no quantity: such a value is a number in the target representation, not a quantity of the unit.
- FR-124#behavior-3 (examined): When the target unit's id is refused with any `IdentityRefusal` (the identity byte limit, a failed allocation, or a preimage with no RFC 8785 encoding), `identified_quantity` SHALL return `HandoffRefusal::Identity` carrying that refusal and return no quantity and no unit.
- FR-124#behavior-4 (examined): `identified_quantity` SHALL charge no meter and read no clock or environment: the conversion has charged its work, and the result is a function of the conversion.
- FR-124#behavior-5 (examined): When a caller records the returned `IdentifiedUnit` with `UnitScope::form`, `UnitScope::form` SHALL return the quantity's unit id and `UnitScope::resolve` of the quantity SHALL return the same `QuantityUnit` as the conversion's `unit()`.
- FR-124#behavior-6 (examined): Because `UnitId` compares its domain before its digest (quire-exact FR-088), `identified_quantity` SHALL give a compound target the compound-arm id and a declared target the declared-arm id, so a compound quantity never resolves to a declared unit.
- FR-124#behavior-7 (examined): The method SHALL use only the crate's existing dependencies, call no identity-minting method of `NodeKey`, and add no re-export (FR-060).
- FR-124-AC-1 (examined): An exact conversion of a declared quantity into a compound target unit returns a quantity whose magnitude is the converted exact value, whose unit id equals the `IdentifiedUnit`'s id, and whose `IdentifiedUnit::unit()` equals the conversion's `unit()`; an exact conversion into a declared unit does the same with the declared unit's id.
- FR-124-AC-2 (examined): After `UnitScope::form` of the returned `IdentifiedUnit`, the returned id equals the quantity's unit id and `UnitScope::resolve` of the quantity returns the conversion's `QuantityUnit`; converting the resolved quantity to joules as in QSpec TC-187 U25 yields the exact value `1`, and each of the two conversions makes one `unit.identity-read`, one `unit.target-domain` and one `unit.result-retain` and no `unit.edge` charge (RT FR-007-AC-23).
- FR-124-AC-3 (examined): A conversion to a decimal target and one to an integer target each return `HandoffRefusal::NotExact` and no quantity.
- FR-124-AC-4 (examined): A compound target whose dimensions cancel and whose id the 16 MiB identity limit refuses returns `HandoffRefusal::Identity` carrying the same `IdentityRefusal` as `QuantityUnit::id`, with no quantity and no unit, and the caller's `UnitScope` holds no new unit afterwards.
- FR-124-AC-5 (examined): A declared target and a compound target whose digest bytes equal the declared unit's key bytes produce distinct unit ids from `identified_quantity` (the declared arm and the compound arm), and resolving the compound quantity returns the compound unit.
- FR-124-AC-6 (examined): The change adds no normal dependency, changes no existing public item, calls no `NodeKey::from_digest` in production, and passes `make build-no-std`, `make lint` and `cargo build --locked`.
- TC-923#step-1 (examined): Convert a declared quantity exactly into a compound target and into a declared target; call `identified_quantity` on each; compare the magnitude, the unit id and the unit (AC-1).
- TC-923#step-2 (examined): Form the compound pair in a `UnitScope`, resolve the quantity, convert the resolved quantity into joules, and compare the value and the charges of both conversions with QSpec TC-187 U25 and RT FR-007-AC-23 (AC-2).
- TC-923#step-3 (examined): Convert to a decimal target and to an integer target and call `identified_quantity` (AC-3).
- TC-923#step-4 (examined): Build a compound target whose dimensions cancel and whose preimage exceeds the 16 MiB identity limit (200,000 roots over distinct dimension nodes that share one base map, with exponents +1 and -1, from a dimensionless source; the preimage needs 16,777,240 bytes), convert, and call `identified_quantity`. A target made large by a big exponent on one root has a different dimension from its source and is refused `IllTyped` before any `Conversion` exists, so it cannot reach the handoff. Check the caller's `UnitScope` holds no new unit (AC-4).
- TC-923#step-5 (examined): Build a declared unit and a compound unit with equal digest bytes; compare their unit ids and resolve the compound quantity (AC-5).
- TC-923#step-6 (examined): Inspect the public method, `Cargo.toml` and `Cargo.lock` diff; run `make build-no-std`, `make lint` and `cargo build --locked` (AC-6).
- FR-124#status (examined): PLANNED (IR-831; RT AD-004 step 4). Updating RT AD-004's proposed `exact_quantity` signature to this one is a follow-up under IR-831. The public items, the tests and the RT cutover follow this specification.
- TC-923#status (examined): PLANNED (IR-831).

## Verdict

**PASS with three medium findings; mergeable after one fix round.** FR-124's behaviors and AC-1..AC-6 are
implemented and traced. None of the findings is a code defect.
- FND-003, the Status flip, belongs in this PR, by repo convention.
- FND-001 (the precedence of Behaviors 2 and 3) is a spec follow-up for the lead to route. It can be fixed in this
  PR as a one-line FR edit plus a pinning test.
- FND-002 (TC-923 step 4's byte claim and term order) is a spec follow-up for the lead to route. It can be fixed in
  this PR as a one-line TC edit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Behaviors 2 and 3 both apply to a reachable input and require different results. A Decimal-target conversion into the TC-923 step-4 cancelling compound target completes, and its target's id is refused (InputBytes{16777216,16777240}). Behavior 2 says NotExact; Behavior 3 says Identity. The implementation returns NotExact (probe 3), and no test pins that: mutant M7, which checks identity first, passes all five tests. Spec follow-up: say which refusal wins, for example 'when the value is not exact, NotExact, whatever the target's id'. Then add a test for the combined case. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:90-97 |
| FND-002 | medium | Step 4 says 'the preimage needs 16,777,240 bytes'. The 200,000-root preimage is in fact 30,500,052 bytes. 16,777,240 is quire-canonical's running count at the write that crosses the 16 MiB bound, so it changes with which roots carry -1: +1 then -1 by ascending id gives 16,777,240, alternating gives 16,777,250 or 16,777,251, and -1 then +1 gives 16,777,251. The test's literal is reproducible only for the order the coder picked, and the TC does not state that order. Spec follow-up: state the order (the first 100,000 roots by ascending key carry +1 and the rest -1) and reword to 'the encoder refuses at 16,777,240 bytes'. Alternatively, drop the literal: AC-4 requires only equality with `QuantityUnit::id`, which the test already asserts. | spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md:31-38 |
| FND-003 | medium | FR-124 and TC-923 still say PLANNED (IR-831). By repo convention the implementing PR flips both. IR-872's implementation PR #45 (02e5cf5) changed FR-121 and TC-921 from PLANNED to 'IMPLEMENTED (IR-872). Five ... tests trace TC-921', and TC-921 records that the AC-6 inspection and the build gates pass. FR-123 and TC-922 read IMPLEMENTED (QSL-899) the same way. AC-6 is untagged Inspection, so the Status line is the only place its outcome is recorded (compare the FR-122-AC-4 note). Flip both to IMPLEMENTED (IR-831), name the five TC-923 tests and the AC-6 result, and keep the RT AD-004 follow-up sentence. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:131-134 |

## Evidence

Gate: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/ci.log (`make ci` EXIT 0: fmt-check, clippy -D warnings host and thumbv7em, 205 tests incl. 5/5 in tests/identified_quantity.rs in 4.91 s debug, build-no-std --locked, cargo deny ok, rustdoc -D warnings); `cargo build --locked` clean. Mutants: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/mutate.py, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/runmut.sh, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/mut.log. Probes: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/probe-identified_quantity.rs, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/probe-results.txt. Matrix: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/matrix.json.
