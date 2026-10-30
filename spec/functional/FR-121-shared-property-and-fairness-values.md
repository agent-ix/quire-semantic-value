---
id: FR-121
title: "Share checked property forms and fairness constraints across QSL and consumers"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-060
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-123
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-129
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
---
# FR-121: Share checked property forms and fairness constraints across QSL and consumers

## Description

`quire-semantic-value` SHALL provide the checked property-form and fairness
value types consumed by QSL, IR and CG in a public `fairness` module. QSL
FR-123 classifies a checked temporal clause and resolves its fairness
constraints; QSL FR-129 adds strong fairness. QSL ADR-011 X-11 places shared
values below those consumers. This crate holds their values and canonical
encoding, but does not parse a clause, resolve an operation, select a temporal
profile, classify a formula or evaluate fairness.

An operation in a checked fairness constraint is the pair of the declaring
type's caller-minted `quire_exact::EffectiveId` and the resolved operation
name. The pair is necessary because two declaring types can have operations
with the same name. QSV retains that pair; it neither mints nor decodes an
`EffectiveId` (FR-060). The companion QSL FR-123 amendment (QSL PR #955)
uses this same pair for the checked fairness operation. That amendment
remains a separate QSL-owned change.

## Use case

QSL checks an infinite-trace clause with a weak whole-operation fairness
constraint on `Counter::increment` and a strong each-invocation constraint on
`OtherCounter::increment`. It emits the two resolved values in source order.
IR and CG read the same types without importing a QSL layer crate, and QSL
uses their canonical encodings when it includes the fairness list in the
checked clause identity.

## Inputs

- A property form chosen by QSL's temporal checker.
- For each fairness constraint, a resolved `FairnessKind`, a caller-minted
  declaring-type `EffectiveId`, the resolved operation name as `String`, and
  a resolved `FairnessGranularity`.
- A `quire_canonical::Limits` when encoding a value.

## Outputs

- The five shared property-form variants and the four combinations of weak or
  strong fairness with whole-operation or each-invocation granularity.
- A retained fairness operation pair and constraint, or canonical encoding
  bytes bounded by the caller's limit.

## Public API inventory

All items are under `quire_semantic_value::fairness` and use `core` and `alloc`.
The fields are private; the listed constructors and accessors are public.

```rust
pub enum PropertyForm {
    ReachableInvariant, BoundedMltl, Safety, Liveness,
    Refinement { liveness: bool },
}
pub enum FairnessKind { Weak, Strong }
pub enum FairnessGranularity { Whole, Each }

pub struct FairnessOperation { /* declaring_type: EffectiveId, name: String */ }
impl FairnessOperation {
    pub fn new(declaring_type: EffectiveId, name: String) -> Self;
    pub fn declaring_type(&self) -> EffectiveId;
    pub fn name(&self) -> &str;
}

pub struct FairnessConstraint {
    /* kind: FairnessKind, operation: FairnessOperation,
       granularity: FairnessGranularity */
}
impl FairnessConstraint {
    pub fn new(kind: FairnessKind, operation: FairnessOperation,
               granularity: FairnessGranularity) -> Self;
    pub fn kind(&self) -> FairnessKind;
    pub fn operation(&self) -> &FairnessOperation;
    pub fn granularity(&self) -> FairnessGranularity;
}
```

Each public value implements `Clone`, `Debug`, `Eq`, `PartialEq`, `Hash`, `Ord`
and `PartialOrd`; all three enums implement `Copy`. The two structs
implement `quire_canonical::Encode`. `PropertyForm` implements `Encode` as
specified below. Neither QSL nor another consumer re-exports these items
under an old module path.

## Behavior

1. `PropertyForm` SHALL contain exactly `ReachableInvariant`, `BoundedMltl`,
   `Safety`, `Liveness` and `Refinement { liveness: bool }`. `Refinement` is a
   distinct property form whose Boolean `liveness` field is retained exactly.
   `FairnessKind` SHALL contain exactly `Weak` and `Strong`; the absence of a
   source kind is resolved by QSL to `Weak` before construction. Likewise,
   `FairnessGranularity` SHALL contain exactly `Whole` and `Each`; QSL resolves
   an absent source granularity to `Whole` before construction.
2. Constructors SHALL retain their supplied values exactly. Two operations
   with the same name but different declaring-type identities are unequal.
   Two constraints are equal exactly when their kind, operation pair and
   granularity are equal. QSV SHALL not deduplicate or sort a fairness list:
   QSL FR-123 and FR-129 own first-occurrence deduplication and source order.
3. `Hash` SHALL agree with equality. `Ord` SHALL be a total order consistent
   with equality and independent of insertion or source order: enum variants
   order as listed above; `FairnessOperation` compares the supplied structural
   `EffectiveId` and then operation-name UTF-8 bytes; `FairnessConstraint` compares
   kind, operation and granularity in that order. This order is for maps and
   sets, not the QSL fairness list's source order.
4. `Encode` of a `FairnessOperation` SHALL produce RFC 8785 bytes of
   `{"declaring_type":<EffectiveId>,"name":"<name>"}`, where the
   `EffectiveId` uses FR-201's `owner_effective_type` and `original` members.
   `Encode` of a `FairnessConstraint` SHALL produce RFC 8785 bytes of an
   object with `granularity`, `kind` and `operation` members in that order;
   `granularity` is `"whole"` or `"each"`, `kind` is `"weak"` or `"strong"`,
   and `operation` is the operation object. The first four `PropertyForm`
   variants SHALL encode as one JSON string: `"reachable-invariant"`,
   `"bounded-mltl"`, `"safety"` or `"liveness"`. `Refinement { liveness }`
   SHALL encode as `{"refinement":{"liveness":<bool>}}`, with the Boolean
   equal to the retained field. Strings SHALL use RFC 8785 escaping. The caller's
   `Limits` bounds the complete output through `quire-canonical`'s normal
   typed error path; no partial bytes are a successful result.
5. The module SHALL retain QSV's `#![no_std]` boundary and its existing
   production dependency set. It SHALL call no identity-minting method, own
   no temporal-profile or model-operation lookup, and add no compatibility
   re-export. `PropertyForm` SHALL expose exactly the five variants listed in
   Behavior 1; no other existing item or export changes.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-121-AC-1 | Public construction and access through `quire_semantic_value::fairness` round-trip `ReachableInvariant`, `BoundedMltl`, `Safety`, `Liveness`, `Refinement { liveness: false }` and `Refinement { liveness: true }`, both `FairnessKind` variants, both `FairnessGranularity` variants and all four kind/granularity pairs; an operation retains its supplied declaring-type `EffectiveId` and name. | Test (TC-921) |
| FR-121-AC-2 | Two `FairnessOperation` values with the same name and different declaring-type IDs are unequal, while equal IDs and names are equal; two `FairnessConstraint` values are equal exactly when kind, operation pair and granularity all match. Equal values have equal hashes, and unequal variants or fields remain distinct in a `HashSet`. | Test (TC-921) |
| FR-121-AC-3 | `Ord` and `PartialOrd` agree with `Eq`: property forms order `ReachableInvariant < BoundedMltl < Safety < Liveness < Refinement { liveness: false } < Refinement { liveness: true }`, kinds `Weak < Strong`, granularities `Whole < Each`; operations order by structural declaring-type identity then name bytes, and constraints by kind, operation, granularity. A sort of fixtures in reverse insertion order produces the same order and retains both same-named operations of different declaring types. | Test (TC-921) |
| FR-121-AC-4 | Under a limit that admits the output, `quire_canonical::to_vec` of a weak-whole constraint on operation `tick` with root declaring type `{owner_effective_type: null, original: {package: "qsv-test", node: "Type000"}}` equals the RFC 8785 bytes of `{"granularity":"whole","kind":"weak","operation":{"declaring_type":{"original":{"node":"Type000","package":"qsv-test"},"owner_effective_type":null},"name":"tick"}}`; changing kind, granularity, declaring type or name changes those bytes, and a name containing a quote is escaped canonically. | Test (TC-921) |
| FR-121-AC-5 | `quire_canonical::to_vec` of each fieldless `PropertyForm` yields its specified JSON string spelling, and `Refinement { liveness: false }` and `Refinement { liveness: true }` yield `{"refinement":{"liveness":false}}` and `{"refinement":{"liveness":true}}`; encoding a `FairnessOperation` embeds its structural declaring-type components and name, including a non-null owner chain, and an output limit one byte below the required bytes returns a `quire_canonical::Error::Limit` rather than a truncated successful value. | Test (TC-921) |
| FR-121-AC-6 | The implementing change adds no normal dependency, adds only the specified `PropertyForm::Refinement { liveness }` variant and its encoding, changes no other public item or re-export, calls no identity constructor in production, and passes `make build-no-std`, `make lint` and `cargo build --locked` with the existing dependency set. | Inspection and Analysis (TC-921) |

## Dependencies

- [FR-060](./FR-060-the-crate-mints-no-identity.md) keeps identity minting in
  the caller.
- [QSL FR-123](ix://agent-ix/quire-spec-language/FR-123) and
  [QSL FR-129](ix://agent-ix/quire-spec-language/FR-129) own temporal checks,
  default resolution, deduplication and source order. QSL PR #955 aligns
  FR-123's checked-value wording with the operation identity above.
- [QSL ADR-011 X-11](ix://agent-ix/quire-spec-language/ADR-011) places shared
  values in QSV below QSL, IR and CG.
