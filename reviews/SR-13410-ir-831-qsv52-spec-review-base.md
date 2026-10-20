---
id: SR-13410
title: "IR-831 quire-semantic-value PR 52 base spec review of FR-124 and TC-923"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@f0cc3120be0a6f15eac62c9e2ca45e8b3630aca0 vs origin/main 9f1453a; spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md, spec/test-cases/TC-923-identified-quantity-from-an-exact-conversion.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-923
    type: reviews
---

## Summary

Ticket: IR-831. Base spec review of FR-124 and TC-923, compared with the consumer's merged
contract in quire-contract-runtime main 65febe7 (RT PR #141): AD-004 step 4, FR-275-AC-23 and TC-201
step 8.

How FR-124 meets what RT asks for:

- Exact magnitude plus the target's `UnitId`, paired with `IdentifiedUnit`: Behavior 1 and AC-1.
- `Conversion::unit()` equals the identified unit: AC-1.
- `form` and `resolve` through `UnitScope`, then the second U25 step to exact `1 J`: AC-2.
- No charge for the handoff: Behavior 4 and AC-2.
- Declared and compound arms stay distinct for equal bytes: AC-5.
- An identity refusal gives no quantity and no unit: AC-4.
- A non-exact conversion has no quantity: AC-3.

So every clause of FR-275-AC-23 has a matching QSV criterion. The ownership text matches AD-004's
single owner and the unified-core rule. The two kernel types are built by QSV and never redeclared,
and RT derives its own representation at its boundary.

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
- FR-124#use-case (examined): The runtime converts a quantity into a compound target unit, then converts that result into joules (RT TC-187 U25, FR-007-AC-23). After the first conversion it asks for the identified quantity, retains the unit in its `UnitScope`, resolves the quantity, and converts the resolved quantity to joules. The second conversion yields exact `1 J`.
- FR-124-AC-1 (examined): An exact conversion of a declared quantity into a compound target unit returns a quantity whose magnitude is the converted exact value, whose unit id equals the `IdentifiedUnit`'s id, and whose `IdentifiedUnit::unit()` equals the conversion's `unit()`; an exact conversion into a declared unit does the same with the declared unit's id.
- FR-124-AC-2 (examined): After `UnitScope::form` of the returned `IdentifiedUnit`, the returned id equals the quantity's unit id and `UnitScope::resolve` of the quantity returns the conversion's `QuantityUnit`; converting the resolved quantity to the units of RT TC-187 U25's second step yields the exact value `1`, and the meter after the two conversions with the handoff between them equals the meter after the two `convert_quantity` calls charged back to back.
- FR-124-AC-3 (examined): A conversion to a decimal target and one to an integer target each return `HandoffRefusal::NotExact` and no quantity.
- FR-124-AC-4 (examined): A compound target whose id the identity byte limit refuses returns `HandoffRefusal::Identity` carrying the same `IdentityRefusal` as `QuantityUnit::id`, with no quantity and no unit, and a following `UnitScope::resolve` of any quantity of that unit finds nothing.
- FR-124-AC-5 (examined): A declared unit and a compound unit whose digest bytes are equal produce distinct unit ids from `identified_quantity`, and resolving the compound quantity returns the compound unit.
- FR-124-AC-6 (examined): The change adds no normal dependency, changes no existing public item, calls no `NodeKey::from_digest` in production, and passes `make build-no-std`, `make lint` and `cargo build --locked`.
- TC-923 (examined): Verify [FR-124](../functional/FR-124-identified-quantity-from-an-exact-conversion.md) through the public `quire_semantic_value::quantity` API: the pair that `Conversion::identified_quantity` returns, its retention and resolution in a `UnitScope`, its refusals, and that it adds no charge. Steps 1 to 5 are executable tests; step 6 is the implementing PR's inspection and build gate. The fixtures are RT TC-187 U25's units (a declared unit converted into a compound target, then into joules).

## Verdict

**PASS with findings (2 medium, 0 high).** The requirement is coherent and covers RT's need. The
medium findings are traceability defects between repos: RT's merged text still states a different
signature, and the U25 fixture source is attributed to the wrong repository. Neither blocks the
design. Both should be fixed before the implementing PR binds tests to the wrong source.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Shape mismatch with RT's merged contract. RT AD-004 step 4 (main 65febe7, lines 425-437) still states `Conversion::exact_quantity(&self) -> Result<Option<(Quantity, IdentifiedUnit)>, IdentityRefusal>` and says a non-exact result returns `Ok(None)`. It calls these semantics 'the required boundary'. FR-124 specifies `identified_quantity`, returning `Err(HandoffRefusal::NotExact)` and `HandoffRefusal::Identity(..)`. The owner decisions behind the difference (refuse a non-exact conversion, use a new refusal type) are not disputed, and AD-004 itself leaves the name and signature to the QSV owner. But FR-124 never says that it settles and replaces AD-004's proposed signature, and RT's text was not updated. The two repos now state different boundaries. FR-275-AC-23 ('a non-exact conversion has no exact-quantity result') is still compatible. Fix: have FR-124 say that it settles RT AD-004's proposed `exact_quantity`, and file the RT AD-004 text update (name, `Err(NotExact)` in place of `Ok(None)`, `HandoffRefusal`) as RT follow-up work under IR-831. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:57-71 |
| FND-002 | medium | TC-187 is attributed to the wrong repository. FR-124's Use case, AC-2 and Dependencies, and TC-923's Description, all say 'RT TC-187 U25'. TC-187 is QSpec's (`ix://agent-ix/quire-specification/TC-187`, `spec/test-cases/TC-187-quantity-and-unit-conversion.md`, row U25: `1 N_m` to compound `kg*m^2*s^-2` under `L(1,0,0,1,3,1)`, then to `J`). RT has no TC-187, and its own FR-007 calls it 'QSpec TC-187'. Someone writing the test who looks in RT for the AC-2 fixture will not find it. Fix: say 'QSpec TC-187 U25'. The RT criteria FR-007-AC-23 and FR-275-AC-23 stay as RT references. | spec/functional/FR-124-identified-quantity-from-an-exact-conversion.md:39, 105, 116; TC-923:18 |

## Evidence

RT clone at /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/rt (65febe7); QSpec clone at /private/tmp/claude-501/-Users-peter-dev/be6a486a-dbff-4734-b40c-5d7654187d9c/scratchpad/sr-qsv52/qspec.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | round 1 head aa62a43e1 |
| FND-002 | still-open | Fixed at three of the four sites the finding named: Use case line 49, AC-2 line 117 and TC-923 line 18, plus a new relationship edge to QSpec TC-187. Dependencies line 128 still reads 'RT FR-275-AC-23 and TC-187 U25 are the consumer's criteria that this handoff serves'. That sentence still makes TC-187 RT's, a consumer criterion. Fix: 'RT FR-275-AC-23 is the consumer's criterion; QSpec TC-187 U25 is the fixture'. |
