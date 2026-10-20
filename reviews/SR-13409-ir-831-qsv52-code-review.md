---
id: SR-13409
title: "IR-831 quire-semantic-value PR 52 code review: Conversion::identified_quantity handoff spec"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@f0cc3120be0a6f15eac62c9e2ca45e8b3630aca0 vs origin/main 9f1453a; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: reviews
---

## Summary

Ticket: IR-831. Spec-only PR: `git diff origin/main...f0cc312` adds FR-124 and TC-923 and changes no
source. This code review checks that the specified API is grounded in the current source and can be
built as written.

Source checks, all at the PR head (quire-exact pinned at d18927e in `Cargo.lock`):

- `convert_quantity` returns `Result<Outcome<Conversion>, IllTyped>`. `Conversion` has the fields
  `source: Quantity`, `canonical: Rational`, `value: ConvertedValue` and `unit: QuantityUnit`, with
  read accessors only (`src/quantity.rs`).
- `UnitScope::resolve` takes `&Quantity` and `UnitScope::form` takes an `IdentifiedUnit`. No public
  item turns a `Conversion` into either one, so the gap the FR states is real.
- Everything the method needs is already public: `quire_exact::Quantity::new(Rational, UnitId)`,
  `UnitId::declared` and `UnitId::compound` (quire-exact `src/quantity.rs:32` and
  `src/identity.rs:153,162`), and `IdentifiedUnit::new` and `QuantityUnit::id` in QSV. The crate
  already builds `Quantity::new` in production (`evaluate`, and `operand_value` in
  `src/declaration.rs:5678`). The method needs no new dependency, no re-export and no `NodeKey` mint.
- "No charge" holds by construction. The exact arm of `convert` already charges `unit.target-domain`
  and `unit.result-retain`, and the specified method takes no `Meter`.
- `HandoffRefusal` deriving `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq` and `thiserror::Error` is
  sound, because `IdentityRefusal` is `Copy` (`QuantityRefusal` already derives `Copy` around it).

A scratch probe was run in a copy of the tree, not in the PR (`tests/zz_probe_ir831.rs`, release, 3/3
pass). The method was written outside the crate from public items only. A compound quantity returned
by it was formed in a `UnitScope` and resolved back to `Conversion::unit()` with the same id. A
declared unit keyed by a compound unit's digest bytes kept a distinct `UnitId` (declared arm against
compound arm). A conversion into a 200,000-term dimensionless compound target completed, and the
handoff then refused with `IdentityRefusal::InputBytes { bound: 16777216, required: 16777240 }`.
So the NotExact case and the Identity case can both be reached.

## Examined scope

- FR-124 (examined): `quire-semantic-value` SHALL turn a completed exact quantity conversion into the pair a later quantity operation needs: a kernel `Quantity` and the `IdentifiedUnit` that carries the quantity's unit id. `convert_quantity` returns a `Conversion` that holds the converted value and a bare `QuantityUnit`. A later operation reads a `Quantity` through `UnitScope::resolve`, which needs the quantity's `UnitId` and the unit retained under it. Without the handoff a consumer would compute a compound unit's id and keep its own table, which the runtime's single-core rule forbids: RT FR-275-AC-23 gives compo
- FR-124#api (examined): impl Conversion {
    /// The converted value as a kernel quantity in the target unit, with the
    /// target unit identified once.
    pub fn identified_quantity(&self) -> Result<(Quantity, IdentifiedUnit), HandoffRefusal>;
}

pub enum HandoffRefusal {
    NotExact,                   // the conversion's value is a decimal or an integer
    Identity(IdentityRefusal),  // the target compound unit has no id
}
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

## Verdict

**PASS with one low finding.** The signature can be built as written from the crate's existing
dependencies. The ownership split holds: quire-exact owns `Quantity` and `UnitId`, QSV builds them,
and QSV owns `QuantityUnit`, `CompoundUnit`, `IdentifiedUnit`, `UnitScope` and `Conversion`. The
owner decisions (`&self`, refusing a non-exact conversion, a new refusal type) are accepted as given.
The method has a single, coherent path to the code. The one finding is a gap between the specified
refusal and the source's refusal set.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 3 covers only a refusal by the identity byte limit. But `QuantityUnit::id` can also return `IdentityRefusal::Allocation` (a failed heap reservation) and `NonCanonical` (`src/semantic_node.rs:122-155`), and the API's `Identity(IdentityRefusal)` arm carries them too. Behavior 3 should say that any `IdentityRefusal` from the target's id goes to `HandoffRefusal::Identity`, so the allocation case has specified behaviour. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:85-87 |

## Evidence

Probe source and output: /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/zz_probe_ir831.rs (copy of the tree at /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/qsv-probe).
