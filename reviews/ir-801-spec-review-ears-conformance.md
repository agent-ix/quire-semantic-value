---
id: SR-11503
title: "IR-801 ears-conformance review of QSV PR #36"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-semantic-value@171c84f48d3f436103ae12a773e668f37873f468; spec/functional/FR-119-admit-and-read-relationship-extents.md; spec/test-cases/TC-919-relationship-extents-admit-and-read.md"
review_set: subset
---

## Summary

Ticket: IR-801. PR quire-semantic-value#36, reviewed head 171c84f48d3f436103ae12a773e668f37873f468, base d6fcf31ccf1898deb222904771cb53bdac0a2287. EARS conformance of FR-119's requirement statements (Description and Behavior 1-7).

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
| FND-001 | low | Behavior 5 is one item of about 41 lines with over twenty SHALL/MUST clauses spanning resolve semantics, from_enumerator pull/limit ordering, ManifestFailure typing, API prohibitions and QSL-800's obligations. Split it into atomic statements (resolve; manifest construction and pull order; manifest refusals; API surface prohibitions; external ownership note) so each AC traces to one statement. | spec/functional/FR-119-admit-and-read-relationship-extents.md:120-160 |
| FND-002 | low | The Description's 'The store SHALL implement the admission conditions and read contract owned by QSL FR-364' is a requirement by reference: it imports every FR-364 clause, including the condition-3 skip that Behavior 2 deliberately overrides. Reword as a non-normative pointer and let Behavior 1-7 carry the normative content. | spec/functional/FR-119-admit-and-read-relationship-extents.md:22-24 |

## Verdict

Approve with low findings. Behavior 1-4, 6 and 7 are ubiquitous or event-driven SHALL statements with a single responsible subject. Behavior 5 bundles many independent requirements, and the Description's SHALL delegates its content by reference.

## Dispositions

Round 1 on reviewed head `144d71cef1086c8e3f9e4ff0ceeeae214ca87b85` against the review-pass head `171c84f48d3f436103ae12a773e668f37873f468`, scoped to `git diff 171c84f..144d71c` of FR-119 and TC-919. Each outcome was checked against the new text; the original finding rows above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The former Behavior 5 is split into Behavior 5 (resolve), 6 (pull order), 7 (row counting and stops), 8 (row validation) and 9 (API surface). |
| FND-002 | fixed | The Description no longer imports FR-364 by a SHALL; it says the behaviors define QSV's contract and relate to FR-364. |
| FND-003 | fixed | The normative SHALLs and the implementation-status note are gone from Inputs, which now holds only the boundary block; the borrow, ptr::eq comparison and no-authentication statements live in Behavior 5 and Behavior 10. |

Round 2 on reviewed head `483bb13c5f02e6bc5bdabe2c7416e9602132c950`, scoped to `git diff 144d71c..483bb13`: the final 1 row(s) of the table above are round 2 and are the latest outcomes for the disposition-pass-1 findings they name.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | Normative statements (ObjectClosure SHALL retain a borrow and expose admitted_types(); resolve SHALL compare environments by ptr::eq; QSV SHALL NOT authenticate pairing) and an implementation-status note ('The current ObjectClosure::new does not yet retain this borrow') sit in the Inputs section, outside the numbered Behavior list the ACs trace to. Move the SHALLs into Behavior and drop the status note. | spec/functional/FR-119-admit-and-read-relationship-extents.md:81-92 |
