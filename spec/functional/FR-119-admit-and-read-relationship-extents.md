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
  - target: ix://agent-ix/quire-spec-language/FR-082
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-084
    type: references
---
# FR-119: Admit relationship extents and read their reference keys

## Description

`quire-semantic-value` SHALL provide a `no_std` relationship-extent store beside
`ObjectClosure` for model-layer callers supplying full kernel `ObjectReference`s.
The behaviors below define QSV's admission and read contract. They relate to
[QSL FR-364](ix://agent-ix/quire-spec-language/FR-364); document parsing,
population bindings, charge accounting and snapshot integration remain with
[QSL FR-106](ix://agent-ix/quire-spec-language/FR-106) and QSL FR-364.

## Inputs

- An admitted `TypeEnvironment` and `ObjectClosure`.
- Navigation-relationship declarations keyed by caller type `K: Ord + Clone`:
  each has a key and source and target ends, each end naming an effective
  object type and its `unique` flag, with no multiplicity bounds. The model owner
  (QSL) supplies the table
  and end types from authenticated declarations; QSV checks end-type membership
  in the `TypeEnvironment` but does not authenticate declarations.
- One call supplies a sequence of extents in order, each naming one relationship,
  a `complete` flag,
  and ordered input instances of full source and target references.
- A population enumerator supplied by QSL, yielding entries naming a
  caller-defined population `P`, its non-empty, caller-ordered set of covered
  effective object types, and its
  explicit completeness state. QSV builds the manifest against the admitted
  `TypeEnvironment`. The QSV boundary is:

  ```rust
  pub trait PopulationEnumerator<P: Ord + Clone> {
      type Error;
      /// Ok(Some(row)): next population; Ok(None): normal exhaustion; Err: enumeration failed.
      fn next_row(&mut self) -> Result<Option<PopulationEntry<P>>, Self::Error>;
  }
  pub struct PopulationEntry<P: Ord> { /* private population, covered_types, complete */ }
  // try_new fallibly stores the non-empty, duplicate-free covered_types sequence.
  // EmptyCoveredTypes, DuplicateCoveredType, Allocation and Capacity are typed causes.
  pub struct ClosedPopulationManifest<'t, P: Ord + Clone> { /* private rows and &'t TypeEnvironment */ }
  pub struct PartialPopulationRows<'t, P: Ord + Clone, E> { /* private rows, source_rows, StopReason<E>, &'t TypeEnvironment */ }
  impl<'t, P: Ord + Clone> ClosedPopulationManifest<'t, P> {
      pub fn from_enumerator<E: PopulationEnumerator<P>>(
          types: &'t TypeEnvironment, e: &mut E, row_limit: usize
      ) -> Result<ManifestOutcome<'t, P, E::Error>, ManifestFailure<P>>;
  }
  pub struct ManifestOutcome<'t, P: Ord + Clone, E> { /* private state and &'t TypeEnvironment */ }
  // Read-only accessors expose state, closed/partial views, rows in P order,
  // source_rows and stop reason. No accessor permits mutation or state conversion.
  pub enum StopReason<E> { RowLimit { limit: usize }, EnumeratorError(E) }
  pub enum ManifestFailure<P> {
      DuplicatePopulation(P),
      UnknownCoveredType { population: P, covered_type: EffectiveId },
      Allocation(StorageRequest),
      Capacity(StorageRequest),
  }
  pub enum Resolution { Present, Dangling, Undecidable }
  impl ObjectClosure {
      pub fn resolve<'t, P: Ord + Clone, E>(&self, x: &ObjectReference, types: &'t TypeEnvironment, m: &ManifestOutcome<'t, P, E>) -> Resolution;
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
   For a declaration with competing faults, duplicate key SHALL precede
   unknown end type; source SHALL precede target.
2. The store SHALL admit one call's entire supplied extent sequence
   transactionally through one one-shot constructor, like `ObjectClosure::new`.
   It SHALL have no incremental add operation. A later constructor call SHALL
   build a new independent store, even when it supplies a relationship present
   in an earlier store; cross-call duplication is not condition 2. For each
   extent in that call's order, the store SHALL check
   FR-364 condition
   1 (key is a table navigation relationship), then condition 2 (relationship
   already has an extent). For each instance in supplied order it SHALL check
   condition 3 (source then target reference-type conformance against the
   authenticated declared end type, whether or not the referenced object is
   held in the closure), then condition 4 (duplicate
   full triple unless both ends declare `unique: false`). It SHALL stop at the
   first failure. The four conditions SHALL have distinct typed refusal causes.
   Each SHALL carry the
   relationship and zero-based position in that call as extent index; instance
   conditions also carry
   the zero-based instance index. End type mismatch SHALL additionally carry
   side, actual reference object type and declared end type. For each end, QSV SHALL
   explicitly check that the reference's object type is a member of the
   `TypeEnvironment` before checking conformance; an unknown actual type SHALL
   return that same typed `EndTypeMismatch` carrying the unknown type, since
   reflexive `conforms` on an equal unknown key does not prove membership. A
   store SHALL admit no extents from a refused call.
   QSV SHALL leave any earlier independent store unchanged.
   The table input contains no lower or upper multiplicity bounds. QSV SHALL
   perform no instance-count comparison at admission. QSL passes only each end's type
   and `unique` flag through this boundary.
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
5. `ObjectClosure::resolve(x, types, manifest)` SHALL first return `Present`
   whenever `x` is an admitted object, regardless of the manifest's state or
   whether its environment reference is identical to `types`. This preserves
   [QSL FR-364](ix://agent-ix/quire-spec-language/FR-364)'s rule that a held
   object is present. Only when `x` is missing SHALL the manifest determine
   `Dangling` or `Undecidable`. The manifest types SHALL borrow the
   `TypeEnvironment` against which they were built;
   `ObjectClosure` SHALL retain no environment reference and gain no lifetime
   parameter. `resolve` SHALL use the same `'t` for `types` and the manifest,
   and SHALL compare their environment references with `core::ptr::eq` at run
   time for a missing `x`. The shared lifetime constrains the borrow scope but
   does not establish that both references identify the same value; Rust may
   shorten compatible borrows. A mismatch for a missing `x` SHALL return
   `Undecidable`. This compares reference identity
   of the environment value, without a hash, digest or pin; it does not prove
   the closure was admitted against that environment. QSV SHALL NOT authenticate
   that a closure and manifest describe the same checked unit or source. A
   `Dangling` result therefore depends on the caller's cross-input pairing: a
   manifest for source A can misclassify a missing reference from source B even
   when both use the same environment value. For a
   missing `x`, it SHALL return `Dangling` exactly when the outcome is `Closed`,
   the manifest and supplied `types` have identical environment references,
   `x`'s type is admitted, at least one population entry
   covers that type, and every covering entry is complete. Otherwise it SHALL
   return `Undecidable`. QSV SHALL explicitly check type membership before
   conformance; reflexive `conforms` on an unknown equal key is not membership
   evidence. A population entry covers type T if and only if T conforms,
   including reflexively, to at least one of that entry's declared member
   types under [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) through
   `TypeEnvironment`. Declaring only a child type SHALL NOT cover its parent.
   An incomplete covering entry SHALL prevent `Dangling` even when a different
   complete entry covers the exact type.
   QSV SHALL infer no completeness from absence and provide no default or blanket
   completeness policy.
6. `from_enumerator` SHALL require `row_limit: usize`. QSV SHALL pull at most
   the limit's number of retained rows plus one probe row, used to distinguish normal
   exhaustion from a row-limit stop. QSV SHALL call `next_row` before checking
   the limit. `Ok(None)` SHALL construct `Closed`, including at the first pull
   when `row_limit == 0` and immediately after exactly `row_limit` retained rows.
7. When `next_row` returns `Ok(Some(row))`, QSV SHALL count the pulled row in `source_rows` and check
   the row limit before validation or storage growth. An excess row SHALL
   return `Partial` with the previously retained rows and `RowLimit { limit }`,
   even if that row repeats `P` or names an unknown covered type. Its pull is
   counted, so N retained rows followed by an excess pull yield `source_rows == N + 1`.
   `Err(error)` SHALL return `Partial` with `EnumeratorError(error)` and
   `source_rows == N` for N successful `Some` pulls; the failing pull is not counted.
   The outcome and partial-row accessors SHALL report the same count and stop.
8. Within a retained row, QSV SHALL check duplicate `P` before unknown covered
   type and before reservation. `P` SHALL be the population identity, and a
   repeated `P` SHALL return `DuplicatePopulation`. Every covered type SHALL
   belong to the admitted environment; otherwise QSV SHALL return
   `UnknownCoveredType` naming the population and type. When several covered
   types of one row are unknown, the refusal SHALL name the first unknown in
   caller-supplied covered-type order. QSV SHALL require each
   `PopulationEntry` to have at least one covered type, preserve caller order, and reject a
   repeated covered type with a typed cause during fallible construction.
   Manifest rows SHALL be readable in ascending `P` order. Denied representable
   storage and capacity overflow SHALL have distinct typed `Allocation` and
   `Capacity` causes; neither SHALL silently drop or merge a row.
9. QSV SHALL give `ManifestOutcome`, `ClosedPopulationManifest` and
   `PartialPopulationRows` private fields and read-only accessors for state, ordered rows,
   `source_rows` and stop reason as applicable. Only normal exhaustion SHALL
   construct `Closed`. No public constructor SHALL accept a closed flag, make
   `Closed` from supplied rows, or make `Partial` with caller-chosen count or
   stop. No caller-controlled `close`, partial-to-closed conversion or
   `PartialPopulationRows::resolve` SHALL exist.
10. The four reads SHALL be total, pure and allocation-free. They SHALL read no
   object field or population member list, charge nothing, consult no path,
   clock or global state, and leave both stores unchanged. The admitted
   `ObjectClosure` and extent store SHALL have structural equality so each
   compares equal to its clone before and after reads. `ObjectClosure` SHALL
   change only by adding `resolve` and hand-written structural `PartialEq`/`Eq`;
   its fields, lifetime shape and `new` signature SHALL remain unchanged.
   Closure equality SHALL compare entries in ascending full reference-key order,
   then corresponding field slots by exact structural identity: the same slot
   state, `Value` variant, stored type and identity metadata, and recursively
   the same ordered children and leaf payloads. A `Float` payload SHALL compare
   its IEEE-754 width and bit pattern, including NaN payload and sign; identical
   NaNs compare equal and positive and negative zero compare unequal. Thus
   closure equality SHALL be reflexive for every admitted closure, including
   one with a Float attribute. This identity equality serves clone and purity
   checks; it is not the model's numeric `=` and SHALL NOT be used for model
   equality. In particular, `quire-exact::compare_keys` excludes `Float` from
   `=`. The extent store
   holds no environment reference and SHALL compare its stored state by value.
   Manifest equality SHALL compare rows and state by value and environment
   references by reference identity. `ManifestOutcome<'t, P, E>` equality SHALL
   require `E: PartialEq` (and `E: Eq` for `Eq`).
11. All table and extent storage growth SHALL be fallible. A denied
   representable reservation SHALL report typed `Allocation` with the exact
   `StorageRequest` amount and native `StorageUnit`; size arithmetic or
   representation overflow SHALL report distinct typed `Capacity`. Neither
   failure SHALL publish a partial table or extent. QSV SHALL impose no
   relationship or instance count limit: QSL owns the `observation.values`
   limit and its accounting under FR-106-AC-27.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-119-AC-1 | The public table admits two distinct caller-defined ordered keys and resolves their ends without a QSL key or newly minted effective identity. Repeating a key returns only typed `DuplicateRelationshipKey` naming that key, even when an end type is unknown; an unknown source type and separately an unknown target type return only typed `UnknownEndType` naming key, side and type, including when the unknown key equals itself under `conforms`. With both ends unknown, source wins. Each refusal leaves the prior table equal to its clone. | Test |
| FR-119-AC-2 | In one call containing valid extent 0 and an extent 1 whose key is absent from the table, `NotNavigationRelationship` names that key and index 1; neither extent appears afterward. A store built by an earlier successful call remains equal to its clone. | Test |
| FR-119-AC-3 | A second extent of the same relationship at extent index 1 in one constructor call returns typed `DuplicateExtent` naming the key and index, even if its instances are otherwise invalid; neither extent from that call appears. A later constructor call over the same relationship succeeds independently and leaves the earlier store unchanged; no incremental add operation exists. | Test + Inspection |
| FR-119-AC-4 | At instance index 0, a `Tank` source reference against declared `Sys`, and separately a `Tank` target reference against declared `Pump`, return typed `EndTypeMismatch` with relationship, extent index, instance index, side, actual type and declared type even when the end object is missing from the closure. An actual reference type unknown to the declared environment also returns that mismatch carrying the unknown type. If both ends mismatch, source wins. A missing end object with a conforming reference type admits and remains available to `resolve`; no refused instance appears. | Test |
| FR-119-AC-5 | Repeating the same relationship/source/target triple at instance index 1 returns typed `DuplicateTriple` with relationship, extent index and instance index when both ends are unique or only one end is non-unique. When both ends are non-unique, both occurrences admit and both reads retain adjacent repeats. A duplicate at instance 1 wins over a type mismatch at instance 2, and a type mismatch in extent 0 wins over a duplicate extent at index 1. | Test |
| FR-119-AC-6 | The public table input contains only type and `unique` for each end, no lower or upper bound, and the admission path performs no count comparison. For one source, 0, 1 and a large number of instances each admit; a complete empty extent admits. | Inspection + Test |
| FR-119-AC-7 | An unlisted relationship gives `complete=false` and empty reads; a listed incomplete extent gives `complete=false` while retaining its instances in reads; a listed complete empty extent gives `complete=true` and empty reads. Listing an incomplete extent with instances does not make it complete; an unlisted relationship stays incomplete when a sibling is listed complete. | Test |
| FR-119-AC-8 | Supplying two relationships and their instances in either order yields identical `targets` and `sources` in ascending universe/type/identity order. A repeated non-unique link appears twice adjacently; inverse `sources` reads the same instances. A same-named object in another universe, and separately another type, does not match a receiver or collapse with it. | Test |
| FR-119-AC-9 | `resolve` returns `Present` for an admitted object with a `Partial` outcome and with a `Closed` outcome whose covering population entry is incomplete, including when the manifest was built against a different environment value from the supplied `types`. | Test |
| FR-119-AC-10 | Test: repeating every read gives equal results, zero allocations and charges, and closure and store equal to their pre-read clones, including after incomplete and unlisted reads. A closure with a Float attribute, including NaN, equals itself and its clone; two otherwise identical closures whose Float attribute differs only between positive and negative zero compare unequal. Inspection: read signatures and bodies access no object field, population member list, path, clock or global and allocate no storage; closure equality uses exact structural identity, not model `=`. | Test + Inspection |
| FR-119-AC-11 | Denying an actual representable table reservation and separately an extent reservation returns typed `Allocation` with the exact requested amount and byte or additional-element unit; capacity arithmetic overflow returns distinct typed `Capacity`. Each failure publishes no partial input, leaves prior admitted state equal to its clone, and the same input admits after denial is removed. No QSV count ceiling rejects it. | Test |
| FR-119-AC-12 | For a missing `x` of known type, with identical supplied and manifest environment references, `resolve` returns `Dangling` exactly when a `Closed` manifest has at least one covering entry and every covering entry is complete. A population entry covers type T if and only if T conforms, including reflexively, to at least one declared member type under [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) through `TypeEnvironment`. Positive cases include exact-type coverage, supertype-only coverage, and coverage through one member of a multi-type population. | Test |
| FR-119-AC-13 | For a missing `x`, `resolve` returns `Undecidable` separately for a `Partial` outcome, a `Closed` outcome with an incomplete covering entry, no covering entry, and an actual object type unknown to the supplied environment. Test unknown-type membership with identical supplied and manifest environment references and a `Closed` manifest whose entries completely cover an admitted control type: the missing control reference resolves `Dangling`, while a missing reference of unknown actual type resolves `Undecidable`. Separately, a row naming a covered type unknown to that manifest's environment fails intake with typed `ManifestFailure::UnknownCoveredType`. Inspect `resolve` for explicit actual-type membership; the result alone can also follow from absence of coverage. No covering entry establishes no completeness. | Test + Inspection |
| FR-119-AC-14 | A `Closed` manifest with a complete entry covering `x`'s exact type and an incomplete entry for a different population covering a supertype of `x`'s type returns `Undecidable` for missing `x`. The same result holds when the incomplete covering type is one member of a multi-type population. | Test |
| FR-119-AC-15 | `PopulationEntry` construction refuses empty or duplicate covered types with distinct typed causes and preserves a non-empty caller-ordered set using fallible storage. `from_enumerator` rejects duplicate population keys before unknown covered type or reservation, and rejects unknown covered types with distinct typed `ManifestFailure<P>` causes, naming the first unknown in caller order when several occur in one row; it never drops or merges a row. It reports denied representable storage and capacity overflow with distinct typed `Allocation` and `Capacity` causes. Read accessors return retained rows sorted by `P`, `source_rows`, state and stop. Inspection confirms private manifest fields and absence of public constructors for arbitrary closed or partial outcomes, state flags, `PartialPopulationRows::resolve` and partial-to-closed conversion. | Test + Inspection |
| FR-119-AC-16 | With N retained rows and one further `Some(row)` beyond `row_limit == N`, `from_enumerator` returns `Partial` retaining only N sorted rows and both `source_rows` accessors equal N + 1, with `RowLimit { limit: N }`. An excess duplicate-population row and separately an excess unknown-covered-type row yield that same partial outcome before validation. A missing object covered by a retained complete row resolves `Undecidable`, while an admitted object resolves `Present`. | Test |
| FR-119-AC-17 | With `row_limit` exactly equal to the source row count, the next pull returns `Ok(None)` and `from_enumerator` returns `Closed`, equal to its clone by row/state value and environment reference identity. `ManifestOutcome` equality requires `E: PartialEq`. With `row_limit == 0`, `Ok(None)` on the first pull returns empty `Closed`; `Ok(Some(row))` on the first pull returns `Partial` with no retained rows, `source_rows == 1` and `StopReason::RowLimit { limit: 0 }`. Each case checks the pull before the limit check. | Test |
| FR-119-AC-18 | After N successful `Some` pulls followed by `Err`, `from_enumerator` returns `Partial` with N rows retained in sorted order, both `source_rows` accessors equal N, and `EnumeratorError` carrying the original error. The failing pull is not counted; a missing object covered by a retained complete row still resolves `Undecidable`. | Test |
| FR-119-AC-19 | With a missing known-type `x`, passing `types` as a different environment value from the one borrowed by a closed manifest returns `Undecidable` even when its complete rows would cover `x`. The same mismatch does not prevent `Present` for a held `x`, even with a `Partial` outcome. Inspection confirms the manifest types borrow their environment, `resolve` shares `'t` between `types` and the outcome and compares references by `core::ptr::eq` only for missing `x`, while `ObjectClosure` has no environment field or lifetime and retains its `new` signature. QSV does not authenticate the closure and manifest's checked-unit/source pairing. | Test + Inspection |
| FR-119-AC-20 | With only a child type declared in a complete population entry, a missing parent-typed reference remains `Undecidable` in a `Closed` manifest with matching environments: the parent does not conform to the child. A missing child-typed reference is `Dangling` under that same entry, demonstrating reflexive coverage. | Test |

## Dependencies

- [QSL FR-364](ix://agent-ix/quire-spec-language/FR-364) owns the four
  conditions and the observation read contract; the behaviors above specify
  QSV's input and output boundary without copying QSL's refusal codes or navigation rules.
- NON-NORMATIVE: To preserve QSL FR-364's at-most-one-extent-per-relationship
  rule for an observation, QSL supplies all of that observation's extents in
  one QSV constructor call. QSV checks only the sequence supplied in that call.
- QSL FR-364 condition 3 currently says an absent end skips type checking
  without limiting that skip to snapshot document ends. FR-119 applies the QSL
  owner's clarification: the skip applies only to snapshot document ends
  (`population`, `key`) whose object is absent and thus supplies no type.
  Model-layer extent ends carry full `ObjectReference`s with their own types;
  FR-119 checks each type against the authenticated declared end type even when
  its object is absent. This tightens FR-364's present wording; QSL-800 owns
  amending FR-364.
- [QSL FR-106](ix://agent-ix/quire-spec-language/FR-106) owns document syntax,
  population bindings and closure checks, `observation.values` charging and
  limit enforcement, and snapshot integration. [QSL FR-084](ix://agent-ix/quire-spec-language/FR-084)
  owns population completeness and the reference-key meaning.
- NON-NORMATIVE (QSL-800 confirmation): QSL owns the definition of "the
  population of `x`", completion and traversal-exhaustion evidence. The
  authoritative set is the population declaration in the admitted domain
  package, checked through the admitted model index; QSL-800 owns that check.
  Its producer will issue one matched bundle of population manifest,
  `ObjectClosure`/observation and `TypeEnvironment`, bound to the same whole
  `RawSourceRef`/checked-unit source identity and selection digest, with a
  typed refusal on mismatch. QSL, not QSV, owns this binding; external resolve
  acceptance waits on it. QSV cannot authenticate what an enumerator
  enumerates. The multi-type `covered_types` row follows QSL's population
  member-type list: [QSL TC-1033](ix://agent-ix/quire-spec-language/TC-1033)
  gives Q the list `[Sys, Pump, Tank]`.
- [FR-106](./FR-106-object-closure-admission.md) supplies `ObjectClosure`;
  [FR-108](./FR-108-report-registry-allocation-failure.md) supplies the
  fallible reservation style and typed storage request units.

## Status

IMPLEMENTED/TESTS PASSED: FR-119-AC-1 through FR-119-AC-9 and FR-119-AC-11 through FR-119-AC-20.

IMPLEMENTED/TESTS PASSED: FR-119-AC-10. Repeated `complete`, `targets`, `sources`, and `resolve` reads were measured with `allocation-counter`; every measured read allocated zero times. An allocating control confirmed the counter was active. Charge, purity and Float-identity assertions also passed.
