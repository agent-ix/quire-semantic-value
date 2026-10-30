---
id: FR-124
title: "Hand an exact quantity conversion to the next operation as an identified quantity"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-122
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-060
    type: references
  - target: ix://agent-ix/quire-specification/FR-142
    type: references
  - target: ix://agent-ix/quire-contract-runtime/FR-275
    type: references
  - target: ix://agent-ix/quire-exact/FR-088
    type: references
  - target: ix://agent-ix/quire-specification/TC-187
    type: references
---
# FR-124: Hand an exact quantity conversion to the next operation as an identified quantity

## Description

`quire-semantic-value` SHALL turn a completed exact quantity conversion into
the pair a later quantity operation needs: a kernel `Quantity` and the
`IdentifiedUnit` that carries the quantity's unit id. `convert_quantity` returns
a `Conversion` that holds the converted value and a bare `QuantityUnit`. A
later operation reads a `Quantity` through `UnitScope::resolve`, which needs
the quantity's `UnitId` and the unit retained under it. Without the handoff a
consumer would compute a compound unit's id and keep its own table, which the
runtime's single-core rule forbids: RT FR-275-AC-23 gives compound-unit
identity one owner, this crate.

This specification settles the name and the refusal shape that RT AD-004 step 4
left to this crate when it proposed `Conversion::exact_quantity(&self) ->
Result<Option<(Quantity, IdentifiedUnit)>, IdentityRefusal>`: the method is
`identified_quantity`, and a non-exact conversion is a `HandoffRefusal::NotExact`
refusal, not `Ok(None)`.

The types keep their owners. `Quantity` and `UnitId` are kernel values of
`quire-exact`, one authoritative declaration that this crate constructs and
never redeclares. `QuantityUnit`, `CompoundUnit`, `IdentifiedUnit`,
`UnitScope` and `Conversion` are this crate's. A consumer such as the runtime
derives its own representation from the returned pair at its boundary and
declares no compound-unit id, unit table or conversion adapter.

## Use case

The runtime converts a quantity into a compound target unit, then converts that
result into joules (QSpec TC-187 U25, FR-007-AC-23). After the first conversion it
asks for the identified quantity, retains the unit in its `UnitScope`,
resolves the quantity, and converts the resolved quantity to joules. The
second conversion yields exact `1 J`.

## Inputs

- A `Conversion` from `convert_quantity` (QSpec FR-142), whose `ConvertedValue`
  is `Exact`.

## Outputs

- The pair (`Quantity`, `IdentifiedUnit`), or a typed refusal.

## Public API inventory

All items are at `quire_semantic_value::quantity`; the method and the refusal
type are new and no existing item changes.

```rust
impl Conversion {
    /// The converted value as a kernel quantity in the target unit, with the
    /// target unit identified once.
    pub fn identified_quantity(&self) -> Result<(Quantity, IdentifiedUnit), HandoffRefusal>;
}

pub enum HandoffRefusal {
    NotExact,                   // the conversion's value is a decimal or an integer
    Identity(IdentityRefusal),  // the structural unit handoff exceeded its resource boundary
}
```

`HandoffRefusal` implements `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq` and
`thiserror::Error`, as `QuantityRefusal` does.

## Behavior

1. When the conversion's value is `ConvertedValue::Exact(v)`,
   `identified_quantity` SHALL return the quantity with magnitude `v` and the
   unit id of the conversion's target unit, paired with the `IdentifiedUnit`
   of that unit, whose `unit()` equals the conversion's `unit()`.
2. When the conversion's value is a decimal or an integer representation,
   `identified_quantity` SHALL return `HandoffRefusal::NotExact` and form no
   quantity: such a value is a number in the target representation, not a
   quantity of the unit. Exactness is checked first: for such a value the
   target unit's id is never computed, so a decimal or integer conversion into
   a target whose id would be refused is `NotExact`, not `Identity`.
3. When the conversion's value is exact, QSV SHALL check the structural compound-term
   representation against the existing 16 MiB `identity.input_bytes` ceiling before forming
   `UnitId::Compound`. The check uses bounded RFC 8785 encoding of QSpec FR-201's `UnitId`
   `terms` arm, including each root `NodeId`'s structural components and each exponent's canonical
   decimal spelling, solely to measure work and allocation; neither its bytes nor a digest of
   them becomes the unit identity. If that check returns any `IdentityRefusal` (`InputBytes`, `Allocation` or
   `NonCanonical`), `identified_quantity` SHALL return `HandoffRefusal::Identity` carrying the
   same refusal as `QuantityUnit::id`, with no quantity and no unit. For `InputBytes`, `bound` is
   16,777,216 and `required` is greater than `bound`.
4. `identified_quantity` SHALL charge no meter and read no clock or
   environment: the conversion has charged its work, and the result is a
   function of the conversion.
5. When a caller records the returned `IdentifiedUnit` with `UnitScope::form`,
   `UnitScope::form` SHALL return the quantity's unit id and `UnitScope::resolve`
   of the quantity SHALL return the same `QuantityUnit` as the conversion's
   `unit()`.
6. Because the structural `UnitId` compares its `Declared` arm before its `Compound` arm
   (QSpec FR-201), `identified_quantity` SHALL give a compound target the compound-arm id and
   a declared target the declared-arm id, so a compound quantity never resolves to a declared
   unit. A declared root node and the one-term compound value `(same root node, exponent 1)`
   remain distinct even though they carry that node as a component.
7. The method SHALL use only the crate's existing dependencies, call no
   identity-minting method of `NodeKey`, and add no re-export (FR-060).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-124-AC-1 | An exact conversion of a declared quantity into a compound target unit returns a quantity whose magnitude is the converted exact value, whose unit id equals the `IdentifiedUnit`'s id, and whose `IdentifiedUnit::unit()` equals the conversion's `unit()`; an exact conversion into a declared unit does the same with the declared unit's id. | Test (TC-923) |
| FR-124-AC-2 | After `UnitScope::form` of the returned `IdentifiedUnit`, the returned id equals the quantity's unit id and `UnitScope::resolve` of the quantity returns the conversion's `QuantityUnit`; converting the resolved quantity to joules as in QSpec TC-187 U25 yields the exact value `1`, and each of the two conversions makes one `unit.identity-read`, one `unit.target-domain` and one `unit.result-retain` and no `unit.edge` charge (RT FR-007-AC-23). | Test (TC-923) |
| FR-124-AC-3 | A conversion to a decimal target and one to an integer target each return `HandoffRefusal::NotExact` and no quantity. | Test (TC-923) |
| FR-124-AC-4 | A compound target whose dimensions cancel and whose ordered structural term encoding exceeds the 16 MiB `identity.input_bytes` ceiling returns `HandoffRefusal::Identity(InputBytes { bound: 16_777_216, required })` with `required > bound`, the same refusal as `QuantityUnit::id`, with no quantity and no unit, and the caller's `UnitScope` holds no new unit afterwards. The encoding is measured but neither hashed nor retained as identity. | Test (TC-923) |
| FR-124-AC-5 | A declared root target with node `r` and a compound target whose sole root term is `(r, 1)` produce distinct unit ids from `identified_quantity` (`Declared(r)` before `Compound([(r, 1)])` in structural order), and resolving the compound quantity returns the compound unit rather than the declared root. | Test (TC-923) |
| FR-124-AC-6 | The handoff adds no normal dependency or duplicate public identity type, retains the existing public `identified_quantity` signature, calls no `NodeKey::new` or retired digest constructor in production, and passes `make build-no-std`, `make lint` and `cargo build --locked`. | Inspection and Analysis (TC-923) |

## Dependencies

- [FR-122](FR-122-supplied-membership-owner-obligations.md) names the
  equality and quantity-conversion producers whose causes this handoff does not
  change; [FR-060](FR-060-the-crate-mints-no-identity.md) forbids minting.
- QSpec FR-142 owns quantity conversion; RT FR-275-AC-23 is the consumer's criterion;
  QSpec TC-187 U25 is the fixture.

## Status

The existing handoff is implemented (IR-831; RT AD-004 step 4). IR-1142 PR #73
carries the structural identity resource and cross-arm controls in
FR-124-AC-4/5; its focused tests pass, with independent review and the final
gate pending. IR-1149 separately owns the preexisting infallible `Integer`
clone and decimal formatting in compound identity construction under QSL
FR-259 Behavior 6. Updating RT AD-004's proposed `exact_quantity` signature
to this one is a follow-up under IR-831. The RT cutover follows.
