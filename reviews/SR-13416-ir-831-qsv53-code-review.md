---
id: SR-13416
title: "IR-831 quire-semantic-value PR 53 code review: Conversion::identified_quantity"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@175f21fc53067f94374731116bfb9fdbd372d192 vs origin/main 9270383; src/quantity.rs, tests/identified_quantity.rs, spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: references
---

## Summary

Ticket: IR-831. Code review with the rust-review lane folded in. Scope is `git diff origin/main...175f21f`:
`src/quantity.rs` (+32 lines) and `tests/identified_quantity.rs` (+348 lines). `Cargo.toml`, `Cargo.lock` and
`src/lib.rs` are unchanged.

The implementation is three lines. It does a let-else on `ConvertedValue::Exact`, which returns `NotExact`.
It then calls `IdentifiedUnit::new(self.unit.clone())?`, which lifts `IdentityRefusal` through `#[from]`, and
returns `Quantity::new(value.clone(), unit.id())`. The unit id is computed once and reused for the quantity.
`HandoffRefusal` derives the set the spec lists and follows the shape of `QuantityRefusal`: `#[error(transparent)]`
and `#[from]` on the Identity arm. The method takes `&self` and no `Meter`, and it reads no clock or environment
(grep: no `env`, `Instant` or `SystemTime`). It calls no `NodeKey::from_digest` in production; the test file's
`clippy::disallowed_methods` allow is scoped to fixtures with a reason, as other tests/*.rs files do. The doc
comment has an `# Errors` section. Tests use the repo's `/// Trace:` plus `#[trace("TC-923", "FR-124-AC-n")]` idiom.

Mutation check, run on a copy of the tree (`mut.log`). Every mutant except one is killed:
- quantity id on the declared arm: AC-1, AC-2 and AC-5 fail.
- quantity id always on the compound arm: AC-1, AC-2 and AC-5 fail.
- inexact value accepted: AC-3 fails.
- IdentityRefusal swallowed into NotExact: AC-4 fails.
- a different IdentityRefusal carried: AC-4 fails.
- wrong magnitude: AC-1 and AC-2 fail.
- identity checked before exactness: survives. This is the gap-analysis finding on Behaviors 2 and 3.

Two mutants from the brief cannot be written. Mutating the caller's `UnitScope` or taking or charging a meter
would need a different signature; `&self` gives the method no path to either one, so the compiler enforces those
two properties.

Step 4 takes 4.9 s in debug. About 110,000 roots would be the floor for crossing 16 MiB at about 152.5 bytes per
term (30,500,052 bytes over 200,000 terms), so 200,000 has about 1.8x margin. That cost is acceptable for a
non-ignored test and is not a finding.

## Examined scope

- src/quantity.rs#identified_quantity (examined): pub fn identified_quantity(&self) -> Result<(Quantity, IdentifiedUnit), HandoffRefusal> { let ConvertedValue::Exact(value) = &self.value else { return Err(HandoffRefusal::NotExact); }; let unit = IdentifiedUnit::new(self.unit.clone())?; Ok((Quantity::new(value.clone(), unit.id()), unit)) }
- src/quantity.rs#HandoffRefusal (examined): /// Why [`Conversion::identified_quantity`] produced no quantity. #[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)] pub enum HandoffRefusal { /// The conversion's value is a decimal or an integer, not an exact /// rational. #[error("the conversion's value is not an exact quantity")] NotExact, /// The target compound unit has no id. #[error(transparent)] Identity(#[from] IdentityRefusal),
- tests/identified_quantity.rs#ac2-charges (examined): // One identity read, one target domain and one result retain per // conversion, and no edge: the source and target are roots. for meter in [&first, &second] { assert_eq!(meter.admission_count(), 3); assert_eq!(meter.consumed(quire_exact::LimitKind::UnitEdges), 0); }
- tests/identified_quantity.rs#ac4-scope (examined): // The refusal formed nothing: only the source unit, which the caller // inserted into the package table, resolves. assert_eq!(scope.resolve(&source).unwrap().unit(), &source_unit); assert_eq!(scope.into_formed().ids().count(), 0);
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

## Verdict

**PASS with one medium and one low finding.** The implementation is correct and minimal, adds no
public surface beyond FR-124's inventory, adds no dependency or re-export, and follows the crate's refusal idiom.
Behaviors 1-3 and 5-6 are each pinned by a test that kills its mutant. Behavior 4 and AC-6 hold by signature and
inspection. The findings are both in the tests: AC-2's charge oracle is weaker than the AC, and AC-4's scope
assertion is vacuous. The commit convention matches main: author and committer Peter Krenesky
<kreneskyp@gmail.com> and a `Co-Authored-By: Claude Sonnet 5.5` trailer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-2 requires each conversion to make one `unit.identity-read`, one `unit.target-domain`, one `unit.result-retain` and no `unit.edge`. The test asserts only `admission_count() == 3` and `consumed(UnitEdges) == 0`, so a conversion that made two identity reads and one retain would still pass. The PR body says naming the points needs quire-exact's `test-support` feature, but it does not. `quire_exact::InjectedDenial` and `Meter::with_injected_denial` are public with no feature gate. Denying occurrence 1 of each point returns `Outcome::Incomplete` with that `charge_point`, occurrence 2 never fires, and `UnitEdge` occurrence 1 never fires, for both conversions. A reviewer probe (probe-identified_quantity.rs, zz_probe_charge_points_via_injected_denial) passes with no new dependency or feature. Strengthen the test this way. The AC wording can stay. | tests/identified_quantity.rs:213-218 |
| FND-002 | low | The AC-4 clause 'the caller's UnitScope holds no new unit afterwards' is asserted with `scope.into_formed().ids().count() == 0` on a scope that nothing under test ever receives. `identified_quantity(&self)` has no path to it, so the assertion cannot fail and no mutant can reach it. Either say in the test comment that the `&self` signature guarantees this (the assertion then documents the guarantee and does not test it), or drop it. Spec side: the clause is true by signature and could move from Test to Inspection. | tests/identified_quantity.rs:315-318 |

## Evidence

Gate: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/ci.log (`make ci` EXIT 0: fmt-check, clippy -D warnings host and thumbv7em, 205 tests incl. 5/5 in tests/identified_quantity.rs in 4.91 s debug, build-no-std --locked, cargo deny ok, rustdoc -D warnings); `cargo build --locked` clean. Mutants: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/mutate.py, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/runmut.sh, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/mut.log. Probes: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/probe-identified_quantity.rs, /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/probe-results.txt. Matrix: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv53/matrix.json.
