---
id: FR-119
title: "Admit relationship extents and read their reference keys"
type: FR
org: agent-ix
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-364
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-106
    type: references
---
# FR-119: Admit relationship extents and read their reference keys

## Description

`quire-semantic-value` SHALL provide a `no_std` relationship-extent store beside
`ObjectClosure` for model-layer callers supplying full kernel `ObjectReference`s.
The store SHALL implement the admission conditions and read contract owned by
[QSL FR-364](ix://agent-ix/quire-spec-language/FR-364), without owning document
parsing, population bindings, charge accounting or snapshot integration. Those
remain with [QSL FR-106](ix://agent-ix/quire-spec-language/FR-106) and FR-364.

## Inputs

- An admitted `TypeEnvironment` and `ObjectClosure`.
- Navigation-relationship declarations keyed by caller type `K: Ord + Clone`:
  each has a key and source and target ends, each end naming an effective
  object type and its `unique` flag. The model owner (QSL) supplies the table
  and end types from authenticated declarations; QSV checks end-type membership
  in the `TypeEnvironment` but does not authenticate declarations.
- Extents in supplied order, each naming one relationship, a `complete` flag,
  and ordered input instances of full source and target references.
- A population enumerator supplied by QSL, yielding entries naming a
  caller-defined population, its covered effective object type, and its
  explicit completeness state. QSV builds the manifest against the admitted
  `TypeEnvironment`. The QSV boundary is:

  ```rust
  pub trait PopulationEnumerator<P: Ord + Clone> {
      type Error;
      /// Ok(Some(row)): next population; Ok(None): normal exhaustion; Err: enumeration failed.
      fn next_row(&mut self) -> Result<Option<PopulationEntry<P>>, Self::Error>;
  }
  pub struct PopulationEntry<P: Ord> { pub population: P, pub covered_type: EffectiveId, pub complete: bool }
  pub enum ManifestState { Closed, Incomplete }
  pub struct PopulationManifest<P: Ord + Clone> { /* rows sorted by P and retained; state */ }
  impl<P: Ord + Clone> PopulationManifest<P> {
      pub fn from_enumerator<E: PopulationEnumerator<P>>(
          types: &TypeEnvironment, e: &mut E, row_limit: Option<usize>
      ) -> Result<ManifestOutcome<P, E::Error>, ManifestFailure<P>>;
      pub fn state(&self) -> ManifestState;
  }
  pub enum ManifestOutcome<P: Ord + Clone, E> {
      Closed(PopulationManifest<P>),
      Incomplete { manifest: PopulationManifest<P>, stop: StopReason<E> },
  }
  pub enum StopReason<E> { EnumeratorError(E), RowLimit }
  pub enum ManifestFailure<P> {
      DuplicatePopulation(P),
      UnknownCoveredType { population: P, covered_type: EffectiveId },
      Allocation(StorageRequest),
      Capacity(StorageRequest),
  }
  pub enum Resolution { Present, Dangling, Undecidable }
  impl ObjectClosure {
      pub fn resolve<P: Ord + Clone>(&self, x: &ObjectReference, types: &TypeEnvironment, m: &PopulationManifest<P>) -> Resolution;
  }
  ```

## Outputs

- An admitted relationship table and extent store, or a typed table, extent,
  allocation or capacity refusal.
- Pure `complete`, `targets`, `sources` and three-way `resolve` reads.

## Behavior

1. The table SHALL admit navigation relationships fallibly. It SHALL reject a
   repeated key with a typed duplicate-key cause and reject either end whose
   type is not an admitted model object type with a typed unknown-end-type
   cause naming the key, side and type. It SHALL check membership explicitly:
   `TypeEnvironment::conforms` being reflexive for equal keys does not prove
   that a key belongs to the environment. QSV SHALL keep `K` generic.
   QSV SHALL NOT import QSL's `DeclarationKey` or create an `EffectiveId`; see
   [FR-060](./FR-060-the-crate-mints-no-identity.md).
2. For each extent in supplied order, the store SHALL check FR-364 condition
   1 (key is a table navigation relationship), then condition 2 (relationship
   already has an extent). For each instance in supplied order it SHALL check
   condition 3 (source then target reference-type conformance, whether or not
   the referenced object is held in the closure), then condition 4 (duplicate
   full triple unless both ends declare `unique: false`). It SHALL stop at the
   first failure. The four conditions SHALL have distinct typed refusal causes.
   Each SHALL carry the
   relationship and zero-based extent index; instance conditions also carry
   the zero-based instance index. End type mismatch SHALL additionally carry
   side, actual reference object type and declared end type. For each end, QSV SHALL
   explicitly check that the reference's object type is a member of the
   `TypeEnvironment` before checking conformance; an unknown actual type SHALL
   return that same typed `EndTypeMismatch` carrying the unknown type, since
   reflexive `conforms` on an equal unknown key does not prove membership. A
   refusal SHALL admit no part of the refused input; previously admitted
   extents remain unchanged.
   Admission SHALL NOT compare instance counts with lower or upper
   multiplicity bounds.
3. The store SHALL distinguish unlisted, listed incomplete and listed complete
   extents. A listed complete extent may hold zero instances. The `complete(R)`
   read SHALL be true exactly for the listed complete state. `targets(R, r)` and
   `sources(R, r)` SHALL return the references held even by a listed
   incomplete extent, and an empty result for unlisted extents.
4. `targets` SHALL select instances whose full source reference equals `r`;
   `sources` SHALL select instances whose full target reference equals `r`.
   Both SHALL return the opposite full references in ascending whole-reference
   key order (universe, object type, object identity), independent of input
   order, with repeated references adjacent and retained once per instance.
   Equality SHALL use the whole key, including universe and type.
5. `ObjectClosure::resolve(x, types, manifest)` SHALL return `Present` exactly
   when `x` is an admitted object, regardless of manifest completeness. For a
   missing `x`, it SHALL return `Dangling` exactly when the manifest is `Closed`,
   at least one entry's covered type covers `x`'s object type by FR-082
   conformance through the admitted `TypeEnvironment`, every such covering
   entry is complete, and the closure holds no object equal to `x`. Otherwise
   it SHALL return `Undecidable`: this includes an `Incomplete` manifest, no
   covering entry, any incomplete covering entry, or an object type unknown to the
   environment. QSV SHALL check `x`'s type membership explicitly; reflexive
   `conforms` on an unknown equal key is not membership evidence. A covering
   entry for a supertype counts even when a different population's entry
   covers the exact type. `from_enumerator` SHALL retain successfully read rows
   sorted by `P` and issue `Closed` only after `next_row` returns `Ok(None)`.
   An enumerator error or a row limit reached before normal exhaustion SHALL
   return `Incomplete` with the retained rows and typed `EnumeratorError` or
   `RowLimit` stop reason, never `Closed`.
   A duplicate population key or unknown covered type SHALL return distinct
   typed `ManifestFailure<P>` causes, never silently drop or merge a row;
   denied representable storage and capacity overflow SHALL return distinct
   typed `Allocation` and `Capacity` causes. No public constructor SHALL take
   a closed flag or build a `Closed` manifest from rows already in hand, and
   there SHALL be no caller-controlled `close`. QSV cannot verify what the
   enumerator enumerates. QSL-800 owns authenticating the selected population
   set, per-entry completeness and that the enumerator walks all of it. Its
   reviewed contract, not this FR, establishes
   any compatibility claim. QSL-800 MUST confirm the explicit rule that no
   covering entry yields `Undecidable`. QSV SHALL infer no completeness from
   absence and SHALL provide no default or blanket completeness policy.
6. The four reads SHALL be total, pure and allocation-free. They SHALL read no
   object field or population member list, charge nothing, consult no path,
   clock or global state, and leave both stores unchanged. The admitted
   `ObjectClosure` and extent store SHALL have structural equality so each
   compares equal to its clone before and after reads.
7. All table and extent storage growth SHALL be fallible. A denied
   representable reservation SHALL report typed `Allocation` with the exact
   `StorageRequest` amount and native `StorageUnit`; size arithmetic or
   representation overflow SHALL report distinct typed `Capacity`. Neither
   failure SHALL publish a partial table or extent. QSV SHALL impose no
   relationship or instance count limit: QSL owns the `observation.values`
   limit and its accounting under FR-106-AC-27.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-119-AC-1 | The public table admits two distinct caller-defined ordered keys and resolves their ends without a QSL key or newly minted effective identity. Repeating a key returns only typed `DuplicateRelationshipKey` naming that key; an unknown source type and separately an unknown target type return only typed `UnknownEndType` naming key, side and type, including when the unknown key equals itself under `conforms`. Each refusal leaves the prior table equal to its clone. | Test |
| FR-119-AC-2 | At extent index 1 after an admitted index 0, an unknown key and separately a known non-navigation key return typed `NotNavigationRelationship` naming that key and index. No instance of index 1 appears; index 0 retains its reads. | Test |
| FR-119-AC-3 | A second extent of the same relationship at extent index 1 returns typed `DuplicateExtent` naming the key and index, even if its instances are otherwise invalid; the first extent is unchanged. | Test |
| FR-119-AC-4 | At instance index 0, a `Tank` source reference against declared `Sys`, and separately a `Tank` target reference against declared `Pump`, return typed `EndTypeMismatch` with relationship, extent index, instance index, side, actual type and declared type even when the end object is missing from the closure. An unknown actual reference type returns the same typed mismatch carrying that type, even when equal-key `conforms` is true. If both ends mismatch, source wins. A missing end object with a conforming reference type admits and remains available to `resolve`; no refused instance appears. | Test |
| FR-119-AC-5 | Repeating the same relationship/source/target triple at instance index 1 returns typed `DuplicateTriple` with relationship, extent index and instance index when both ends are unique or only one end is non-unique. When both ends are non-unique, both occurrences admit and both reads retain adjacent repeats. A duplicate at instance 1 wins over a type mismatch at instance 2, and a type mismatch in extent 0 wins over a duplicate extent at index 1. | Test |
| FR-119-AC-6 | Four targets through an end declared upper 3, no instance through an end declared lower 1, and a complete empty extent all admit. No multiplicity refusal is returned at admission. | Test |
| FR-119-AC-7 | An unlisted relationship gives `complete=false` and empty reads; a listed incomplete extent gives `complete=false` while retaining its instances in reads; a listed complete empty extent gives `complete=true` and empty reads. None is inferred from another. | Test |
| FR-119-AC-8 | Supplying two relationships and their instances in either order yields identical `targets` and `sources` in ascending universe/type/identity order. A repeated non-unique link appears twice adjacently; inverse `sources` reads the same instances. A same-named object in another universe, and separately another type, does not match a receiver or collapse with it. | Test |
| FR-119-AC-9 | `resolve` returns `Present` for an admitted object even when its population entry is incomplete. | Test |
| FR-119-AC-10 | Repeating every read gives equal results and performs zero allocations and charges; clone comparisons show the closure and store structurally unchanged, including after reads of incomplete and unlisted extents. No field, population member list, path, clock or global is consulted. | Test |
| FR-119-AC-11 | Denying an actual representable table reservation and separately an extent reservation returns typed `Allocation` with the exact requested amount and byte or additional-element unit; capacity arithmetic overflow returns distinct typed `Capacity`. Each failure publishes no partial input, leaves prior admitted state equal to its clone, and the same input admits after denial is removed. No QSV count ceiling rejects it. | Test |
| FR-119-AC-12 | For a missing `x` of a known type, `resolve` returns `Dangling` only with a `Closed` manifest containing at least one covering entry, every covering entry complete, and no object equal to `x` in the closure. Coverage uses FR-082 conformance through `TypeEnvironment`. | Test |
| FR-119-AC-13 | For a missing `x`, `resolve` returns `Undecidable` separately for an `Incomplete` manifest, an incomplete covering entry, no covering entry, and an object type unknown to the environment. No covering entry does not establish completeness. | Test |
| FR-119-AC-14 | A `Closed` manifest with a complete entry covering `x`'s exact type and an incomplete entry for a different population covering a supertype of `x`'s type returns `Undecidable` for missing `x`. | Test |
| FR-119-AC-15 | `from_enumerator` rejects duplicate population keys and unknown covered types with distinct typed `ManifestFailure<P>` causes, never dropping or merging a row, and reports denied representable storage and capacity overflow with distinct typed `Allocation` and `Capacity` causes. Successful rows remain sorted by `P`. Normal exhaustion alone yields a `Closed` manifest equal to its clone; an enumerator error mid-way and a row limit each yield an `Incomplete` manifest with rows retained and the matching typed stop reason. No public constructor accepts a closed flag or makes `Closed` directly from supplied rows; an enumerator stopped by error or row limit cannot make `resolve` return `Dangling`. | Test |

## Dependencies

- [QSL FR-364](ix://agent-ix/quire-spec-language/FR-364) owns the four
  conditions and the observation read contract; this FR binds them to a QSV
  store by reference, without copying QSL's refusal codes or navigation rules.
- QSL FR-364 condition 3 skips an end the observation does not hold; this FR is deliberately stricter by planner ruling, pending a QSL-side amendment.
- [QSL FR-106](ix://agent-ix/quire-spec-language/FR-106) owns document syntax,
  population bindings and closure checks, `observation.values` charging and
  limit enforcement, and snapshot integration. [QSL FR-084](ix://agent-ix/quire-spec-language/FR-084)
  owns population completeness and the reference-key meaning.
- QSL-800 owns the definition of "the population of `x`", the authoritative
  population set, completion and traversal-exhaustion evidence, and review of
  compatibility with this manifest boundary. QSL-800 is open and blocks
  QSL-794; this FR does not assert resolve compatibility acceptance. In
  particular, QSL-800 must confirm that no covering entry yields
  `Undecidable`.
- [FR-106](./FR-106-object-closure-admission.md) supplies `ObjectClosure`;
  [FR-108](./FR-108-report-registry-allocation-failure.md) supplies the
  fallible reservation style and typed storage request units.

## Status

PLANNED/UNRUN; implementation and acceptance tests are pending.
