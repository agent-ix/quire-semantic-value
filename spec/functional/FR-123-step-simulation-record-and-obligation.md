---
id: FR-123
title: "Share the per-step simulation record and its obligation across QSL and consumers"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-060
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-146
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-020
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-018
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
---
# FR-123: Share the per-step simulation record and its obligation across QSL and consumers

## Description

`quire-semantic-value` SHALL provide the per-step simulation record of
[QSL FR-146](ix://agent-ix/quire-spec-language/FR-146) in a public
`step_simulation` module: `StepSimulationRecord`, `StepObligation` and
`StepRowRef`. QSL's refinement `requirements` hook writes the records and CG
reads them, so QSL ADR-018 LA-5 and ADR-011 X-11 place them here, below both,
with no dependency edge to a QSL layer crate. The crate holds the values and
their canonical encoding; it does not decide which records a refinement
writes, state what an obligation means, or settle one (QSL FR-146).

The refinement and each hypothesis are `quire_exact::NodeKey` values that the
caller minted; QSV neither mints nor decodes one (FR-060). A `StepRowRef` names
one step row of the refinement by its zero-based position in the refinement's
`steps` order, which is declaration order (QSpec FR-375); no sort of any kind
applies. A refinement has exactly one row per concrete
step class (QSL ADR-020 RM-5), so a refinement key and a row position name one
row. The position is a name, not an index QSV dereferences.

## Use case

QSL checks `CasRefinesCounter` and writes one `Initial` record and one `Step`
record per step row, each listing as hypotheses the invariant clauses whose
context is an object type of the concrete model. The `Initial` record carries
the same hypotheses as every `Step` record, as QSpec FR-468 states for the
initial query. CG reads the same values to route them to the SMT backend, and
QSL uses their canonical encodings when it keys each `operation-contract`
record by the refinement node and the obligation.

## Inputs

- A caller-minted `NodeKey` for the refinement and one per hypothesis.
- A `StepObligation`: `Initial`, or `Step` with a `StepRowRef`.
- A `quire_canonical::Limits` when encoding a value.

## Outputs

- A retained `StepSimulationRecord`, or canonical encoding bytes bounded by the
  caller's limit.

## Public API inventory

All items are at `quire_semantic_value::step_simulation`; the module is new and
no existing item changes.

```rust
pub struct StepRowRef(/* u32 */);
impl StepRowRef {
    pub const fn new(index: u32) -> Self;
    pub const fn index(self) -> u32;
}

pub enum StepObligation {
    Initial,                       // ADR-020 RS-2
    Step { row: StepRowRef },      // RS-3 for `stutter`, RS-4 for an abstract operation
}

pub struct StepSimulationRecord {
    pub refinement: NodeKey,
    pub obligation: StepObligation,
    pub hypotheses: Vec<NodeKey>,  // invariant clauses whose context is an object type of the concrete model
}
impl StepSimulationRecord {
    pub const fn new(refinement: NodeKey, obligation: StepObligation,
                     hypotheses: Vec<NodeKey>) -> Self;
}
```

Each value implements `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash`, `Ord` and
`PartialOrd`; `StepRowRef` and `StepObligation` also implement `Copy`. All
three implement `quire_canonical::Encode`; that encoding is the wire form.
There is no `Serialize`, as `NodeKey` has none; the crate's wire form for
identity-bearing values is `Encode` (FR-121).

## Behavior

1. Construction SHALL retain the supplied refinement, obligation and
   hypotheses verbatim. QSV SHALL NOT sort or deduplicate `hypotheses`; QSL
   owns their order.
2. Equality and `Hash` SHALL be field-wise and agree. `Ord` SHALL be the
   derived field order: `Initial` before every `Step`, steps by row position,
   records by refinement, then obligation, then hypotheses.
3. `Encode` of a `StepRowRef` SHALL write its position as a JSON integer.
   `Encode` of a `StepObligation` SHALL write `{"kind":"initial"}` or
   `{"kind":"step","row":<position>}`.
   `Encode` of a `StepSimulationRecord` SHALL write an object with
   `hypotheses` (array of lowercase 64-hex node keys, in order), `obligation`
   and `refinement` (lowercase 64-hex node key) members, in RFC 8785 member
   order.
   Each `Encode` SHALL bound the whole output by the caller's `Limits` and
   report an excess through `quire-canonical`'s typed error path.
4. The module SHALL use only `core`, `alloc` and the crate's existing
   dependencies, call no identity-minting method and add no re-export.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-123-AC-1 | Construction retains the refinement, both obligation forms, row position and hypotheses in supplied order (including a repeated key and an empty list). | Test (TC-922) |
| FR-123-AC-2 | Records differing in refinement, obligation form, row position or any hypothesis are unequal; equal records hash equally; all distinct fixtures stay distinct in a `HashSet`. | Test (TC-922) |
| FR-123-AC-3 | `Ord` agrees with `Eq`; `Initial` orders before `Step`, steps by position; a reversed fixture list sorts to the specified order. | Test (TC-922) |
| FR-123-AC-4 | `quire_canonical::to_vec` yields the exact bytes of Behavior 3 for an `Initial` record with no hypotheses, a `Step` record with two hypotheses, an obligation alone and the maximum row position; changing refinement, row or hypothesis order or count changes the bytes. | Test (TC-922) |
| FR-123-AC-5 | A limit one byte below the encoding returns `quire_canonical::Error::Limit`; the exact length succeeds. | Test (TC-922) |
| FR-123-AC-6 | The change adds no normal dependency, changes no existing public item, calls no `NodeKey::from_digest` in production, and passes `make build-no-std`, `make lint` and `cargo build --locked`. | Inspection and Analysis (TC-922) |

## Status

IMPLEMENTED (QSL-899). Five `src/step_simulation.rs` tests trace TC-922 and
FR-123-AC-1 through AC-5. AC-6 is inspected on the implementing diff.

## Dependencies

- [FR-060](./FR-060-the-crate-mints-no-identity.md) keeps identity minting in
  the caller.
- [QSL FR-146](ix://agent-ix/quire-spec-language/FR-146) owns which records a
  refinement writes and how they settle; QSL ADR-020 RE-2, RS-2 to RS-4 give
  the obligations their meaning; QSL ADR-018 LA-5 and ADR-011 X-11 place the
  values here.
- Blocks QSL-548, which builds the records after the QSL pin moves.
