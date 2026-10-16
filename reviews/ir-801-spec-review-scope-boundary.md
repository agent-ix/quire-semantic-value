---
id: SR-11504
title: "IR-801 scope-boundary review of QSV PR #36"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-semantic-value@171c84f48d3f436103ae12a773e668f37873f468; spec/functional/FR-119-admit-and-read-relationship-extents.md; spec/test-cases/TC-919-relationship-extents-admit-and-read.md"
review_set: subset
---

## Summary

Ticket: IR-801. PR quire-semantic-value#36, reviewed head 171c84f48d3f436103ae12a773e668f37873f468, base d6fcf31ccf1898deb222904771cb53bdac0a2287. Scope-boundary analysis: allocation of responsibility between QSV FR-119, QSL FR-364/FR-106/FR-084 and QSL-800.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-119 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | `quire-semantic-value` SHALL provide a `no_std` relationship-extent store beside `ObjectClosure` for model-layer callers supplying full kernel `ObjectReference`s. The store SHALL implement the admission conditions and read contract owned by [QSL FR-364](ix://agent-ix/quire-spec-language/FR-364), without owning document parsing, population bindings, charge accounting or snapshot integration. Those remain with [QSL FR-106](ix://agent-ix/quire-spec-language/FR-106) and FR-364. |
| FR-119-AC-1 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | The public table admits two distinct caller-defined ordered keys and resolves their ends without a QSL key or newly minted effective identity. Repeating a key returns only typed `DuplicateRelationshipKey` naming that key; an unknown source type and separately an unknown target type return only typed `UnknownEndType` naming key, side and type, including when the unknown key equals itself under `conforms`. Each refusal leaves the prior table equal to its clone. |
| FR-119-AC-2 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | At extent index 1 after an admitted index 0, an unknown key and separately a known non-navigation key return typed `NotNavigationRelationship` naming that key and index. No instance of index 1 appears; index 0 retains its reads. |
| FR-119-AC-3 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | A second extent of the same relationship at extent index 1 returns typed `DuplicateExtent` naming the key and index, even if its instances are otherwise invalid; the first extent is unchanged. |
| FR-119-AC-4 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | At instance index 0, a `Tank` source reference against declared `Sys`, and separately a `Tank` target reference against declared `Pump`, return typed `EndTypeMismatch` with relationship, extent index, instance index, side, actual type and declared type even when the end object is missing from the closure. An unknown actual reference type returns the same typed mismatch carrying that type, even when equal-key `conforms` is true. If both ends mismatch, source wins. A missing end object with a conforming reference type admits and remains available to `resolve`; no refused instance appears. |
| FR-119-AC-5 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | Repeating the same relationship/source/target triple at instance index 1 returns typed `DuplicateTriple` with relationship, extent index and instance index when both ends are unique or only one end is non-unique. When both ends are non-unique, both occurrences admit and both reads retain adjacent repeats. A duplicate at instance 1 wins over a type mismatch at instance 2, and a type mismatch in extent 0 wins over a duplicate extent at index 1. |
| FR-119-AC-6 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | Four targets through an end declared upper 3, no instance through an end declared lower 1, and a complete empty extent all admit. No multiplicity refusal is returned at admission. |
| FR-119-AC-7 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | An unlisted relationship gives `complete=false` and empty reads; a listed incomplete extent gives `complete=false` while retaining its instances in reads; a listed complete empty extent gives `complete=true` and empty reads. None is inferred from another. |
| FR-119-AC-8 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | Supplying two relationships and their instances in either order yields identical `targets` and `sources` in ascending universe/type/identity order. A repeated non-unique link appears twice adjacently; inverse `sources` reads the same instances. A same-named object in another universe, and separately another type, does not match a receiver or collapse with it. |
| FR-119-AC-9 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | `resolve` returns `Present` for an admitted object with a `Partial` outcome and with a `Closed` outcome whose covering population entry is incomplete. |
| FR-119-AC-10 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | Repeating every read gives equal results and performs zero allocations and charges; clone comparisons show the closure and store structurally unchanged, including after reads of incomplete and unlisted extents. No field, population member list, path, clock or global is consulted. |
| FR-119-AC-11 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | Denying an actual representable table reservation and separately an extent reservation returns typed `Allocation` with the exact requested amount and byte or additional-element unit; capacity arithmetic overflow returns distinct typed `Capacity`. Each failure publishes no partial input, leaves prior admitted state equal to its clone, and the same input admits after denial is removed. No QSV count ceiling rejects it. |
| FR-119-AC-12 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | For a missing `x` of a known type, `resolve` returns `Dangling` only with a `Closed` manifest containing at least one covering entry, every covering entry complete, and no object equal to `x` in the closure. Coverage uses FR-082 conformance through `TypeEnvironment`. |
| FR-119-AC-13 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | For a missing `x`, `resolve` returns `Undecidable` separately for a `Partial` outcome, a `Closed` outcome with an incomplete covering entry, no covering entry, and an object type unknown to the environment. No covering entry does not establish completeness. |
| FR-119-AC-14 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | A `Closed` manifest with a complete entry covering `x`'s exact type and an incomplete entry for a different population covering a supertype of `x`'s type returns `Undecidable` for missing `x`. |
| FR-119-AC-15 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | `from_enumerator` rejects duplicate population keys and unknown covered types with distinct typed `ManifestFailure<P>` causes, never dropping or merging a row, and reports denied representable storage and capacity overflow with distinct typed `Allocation` and `Capacity` causes. Successful retained rows are sorted by `P`. No public constructor accepts a closed flag or makes `Closed` directly from supplied rows; no type exposes a state flag, and `PartialPopulationRows` has neither `resolve` nor a conversion to `ClosedPopulationManifest`. |
| FR-119-AC-16 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | With `row_limit` smaller than the source row count, `from_enumerator` pulls one row beyond the limit, returns `Partial` with only the rows within the limit retained in sorted order, equal `source_rows` counts in the outcome and `PartialPopulationRows` including the excess pulled row, and `StopReason::RowLimit { limit }`. `resolve` returns `Undecidable` for a missing object covered by a retained complete row, but `Present` for an admitted object. |
| FR-119-AC-17 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | With `row_limit` exactly equal to the source row count, the next pull returns `Ok(None)` and `from_enumerator` returns `Closed`, equal to its clone. With `row_limit == 0`, `Ok(None)` on the first pull returns empty `Closed`; `Ok(Some(row))` on the first pull returns `Partial` with no retained rows, `source_rows == 1` and `StopReason::RowLimit { limit: 0 }`. Each case checks the pull before the limit check. |
| FR-119-AC-18 | examined | spec/functional/FR-119-admit-and-read-relationship-extents.md | An enumerator error after one or more rows returns `Partial` with those rows retained in sorted order, equal `source_rows` counts in the outcome and `PartialPopulationRows`, and `StopReason::EnumeratorError` carrying the original error; a missing object covered by a retained complete row still resolves `Undecidable`. |
| TC-919 | examined | spec/test-cases/TC-919-relationship-extents-admit-and-read.md | Verify [FR-119](../functional/FR-119-admit-and-read-relationship-extents.md) through QSV's public table, extent and `ObjectClosure` APIs, using full references supplied by a model-layer caller. Scope: FR-119-AC-1 through FR-119-AC-18. This is a planned QSV counterpart of [QSL TC-1033](ix://agent-ix/quire-spec-language/TC-1033). |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior 5 contains 'QSL-800 MUST confirm the explicit rule that no covering entry yields Undecidable' and statements of what QSL-800 owns. These bind another repository's ticket, are unverifiable by any QSV test, and sit among QSV's SHALLs. Keep them only in Dependencies as non-normative notes. | spec/functional/FR-119-admit-and-read-relationship-extents.md:154-160; spec/functional/FR-119-admit-and-read-relationship-extents.md:212-217 |

## Verdict

Approve with one low finding. The boundary is drawn as intended: QSV keeps K and P generic and mints no EffectiveId or PopulationId (consistent with FR-060); QSL owns parsing, population bindings, charges, the observation.values limit (FR-106-AC-27) and snapshot integration; QSL-800 owns the population set and completion evidence; and the FR claims no resolve-compatibility acceptance. One placement issue: obligations on QSL-800 are written as normative statements inside QSV's Behavior list.

## Dispositions

Round 1 on reviewed head `144d71cef1086c8e3f9e4ff0ceeeae214ca87b85` against the review-pass head `171c84f48d3f436103ae12a773e668f37873f468`, scoped to `git diff 171c84f..144d71c` of FR-119 and TC-919. Each outcome was checked against the new text; the original finding rows above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The QSL-800 MUST and ownership statements moved to a NON-NORMATIVE Dependencies note; no QSV SHALL binds QSL-800. |
| FND-002 | fixed | ObjectClosure keeps its fields, lifetime shape and new signature (Behavior 10). Only the transient manifest types borrow &'t TypeEnvironment, and resolve compares by core::ptr::eq at run time. The author's caveat is correct: a shared 't only bounds the borrows, because variance lets two distinct environments unify to one 't, so the runtime ptr::eq check is what establishes identity. QSL holders such as qsl-semantics ObjectEnvironment { objects: ObjectClosure } gain no lifetime. |
| FND-003 | fixed | The 'QSL SHALL supply all extents of one observation in one call' sentence is gone from Behavior 2. It is now a NON-NORMATIVE Dependencies note, and QSV checks only the sequence supplied in one call. |

Round 2 on reviewed head `483bb13c5f02e6bc5bdabe2c7416e9602132c950`, scoped to `git diff 144d71c..483bb13`: the final 1 row(s) of the table above are round 2 and are the latest outcomes for the disposition-pass-1 findings they name.

Round 3 on reviewed head `d5bb6fc162c6f94c00df0abeebd60d3c8d4bbb6b`, scoped to `git diff 483bb13..d5bb6fc`: the final 1 row(s) of the table above are round 3 and are the latest outcomes for the disposition-pass-2 findings they name; rows are in round order, so the round-2 row(s) noted above now sit immediately before them. No regression was found in the changed text; every finding in this artifact now has a latest outcome of fixed.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | FR-119 mandates that ObjectClosure keep a lifetime-bound &TypeEnvironment. That adds a lifetime parameter to a public type QSL consumes by value: QSL's ObjectEnvironment stores objects: ObjectClosure (qsl-semantics/src/model/object_environment.rs:41), and qsl-eval/qsl-replay build closures from package.graph().scope().types(). Every holder gains the lifetime, any QSL type owning both environment and observation becomes self-referential, and the current derive(Default) on ObjectClosure cannot hold. The FR records only that ObjectClosure::new must change; it neither assigns nor scopes the QSL change, nor weighs an owned handle or another ptr-identity form. State the cross-repo impact and owner, or specify identity pairing without fixing the borrow form. | spec/functional/FR-119-admit-and-read-relationship-extents.md:81-92; src/object_closure.rs:55-58; quire-spec-language qsl-semantics/src/model/object_environment.rs:41,51 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | Behavior 2 now carries 'QSL SHALL supply all extents of one observation in one call', a normative obligation on another repository inside QSV's numbered behavior. QSV cannot test it: a QSV store cannot tell whether two constructor calls belong to one observation. This is the pattern SR-11504 FND-001 moved out of Behavior. Make it a non-normative Dependencies note on what QSL must do to keep FR-364's 'at most one extent per relationship'. | spec/functional/FR-119-admit-and-read-relationship-extents.md:103-104 |
