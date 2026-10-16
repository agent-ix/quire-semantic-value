---
id: SR-11520
title: "IR-801 relationship extents code review (with Rust review)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; PR #37 diff cca2bc26...09a3641b: src/relationship_extent.rs, src/object_closure.rs, src/declaration.rs, src/lib.rs, tests/relationship_extents.rs, FR-119 and TC-919 status"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-119
    type: reviews
  - target: ix://agent-ix/quire-semantic-value/TC-919
    type: reviews
---

# IR-801 relationship extents code review (with Rust review)

## Summary

Ticket: IR-801. PR: quire-semantic-value#37 at 09a3641ba396a628c981ed03df08435f73f453ea against base cca2bc26242869613b13de387ffcb342fd347219. One pass of code-review with the rust-review lane folded in, plus test-oracle strength, scoped to the PR diff. Focused gates run in a throwaway worktree: `cargo test --locked relationship_extent` (2 unit tests pass), `cargo test --locked --test relationship_extents` (19 pass) and `cargo clippy --locked --all-targets -- -D warnings` (clean). `make ci` was not run.

## Verdict

**CONDITIONAL** — the admission, manifest and resolve logic matches FR-119 on every checked point; no panic, unsafe, std import or catch-all value arm was found. Two medium test-oracle gaps let real defects pass (end-uniqueness on the target side, first-unknown covered type in caller order), one medium unobservable-membership gap, one medium performance defect (quadratic duplicate-triple scan with no QSV count limit), and the AC-10 tag overclaims a zero-allocation assertion that does not exist.

Confirmed clean: Closed is constructed only on `Ok(None)` (relationship_extent.rs:554-563); no public constructor, closed flag, `close` or partial-to-closed conversion exists; `PartialPopulationRows` has no `resolve`; `resolve` decides `Present` first (object_closure.rs:69-71) and compares environments with `core::ptr::eq` only for a missing reference (relationship_extent.rs:721); membership precedes coverage lookup; coverage is precomputed at close with any-member conformance and all-covering-complete (621-648), so child does not cover parent. `row_limit` is a `usize`, the pull precedes the limit check (575-591), over-limit duplicate/unknown rows yield Partial/RowLimit, the failing `Err` pull is not counted. Precedence: duplicate population, then unknown covered type, then reservation (592-607); duplicate key before unknown end type, source before target (94-105); extent conditions 1,2,3(source,target),4 in order with distinct typed causes and position payloads (237-284); duplicate triple refused unless both ends non-unique; the end-type check uses the reference's own type with membership before `conforms` (263). The whole call is atomic (local `admitted`). Sorting is in place (`sort_unstable`), the inverse index is reserved fallibly. ObjectClosure keeps its fields, derive list and `new` signature; equality compares BTreeMap entries in key order and Float by width and bits. Visibility widening in declaration.rs is `pub(crate)` only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Duplicate-triple check is a linear scan of all prior links per instance, so admission is quadratic in instance count while FR-119 forbids any QSV count ceiling; 100k instances with a unique end cost about 5e9 reference comparisons | src/relationship_extent.rs:274-277 |
| FND-002 | medium | AC-5 test never makes only the target end unique or both ends unique (source varies, target always non-unique); a mutation that reads only `source_end.unique` passes every test | tests/relationship_extents.rs:304-311 |
| FND-003 | medium | AC-15 first-unknown test uses [ty(4), ty(5)] where caller order equals EffectiveId order; TC-919 step 6 requires the first type to sort after the second, so an implementation reporting the smallest unknown passes | tests/relationship_extents.rs:679-686 |
| FND-004 | medium | Explicit actual-type membership check before `conforms` in extent admission is not exercised: every test uses the table's own environment, where conforms(unknown, declared) is already false; deleting `types.object_type(actual).is_none() ||` passes all tests. The defect is reachable when the table was admitted against a different environment from the store's `types` (actual == declared, unknown to `types`) | src/relationship_extent.rs:263; tests/relationship_extents.rs:252-280 |
| FND-005 | medium | AC-10 test is tagged FR-119-AC-10 but asserts no allocation count; the computed matrix shows AC-10 tagged although its zero-allocation clause has no failing oracle. A safe dev-dependency counter exists (see gap-analysis SR-11521) | tests/relationship_extents.rs:525-581 |
| FND-006 | low | Capacity cases for table, extent, entry and manifest call the private mapping helpers on a hand-made overflow instead of driving admission; a `map_err` in with_reservation that mapped Capacity to Allocation would still pass | src/relationship_extent.rs:853-868, 905-920 |
| FND-007 | low | `targets`/`sources` scan every link of the extent and filter, ignoring the sorted order they built; each read is O(n) where partition_point would give O(log n + k) | src/relationship_extent.rs:327-361 |
| FND-008 | low | Read signatures bind the query reference and the store to one lifetime `'a`, so returned store references cannot outlive the caller's query key | src/relationship_extent.rs:327-331, 345-349 |
| FND-009 | low | Slice indexing on internal indices (`links[*a]`, `extent.links[*index]`, `coverage[position]`) relies on a by-construction permutation invariant that the type does not enforce; `get` would remove the panic surface | src/relationship_extent.rs:293-294, 357, 728 |
| FND-010 | low | Spec boundary block declares `PartialPopulationRows<'t, P>`; the implementation is `PartialPopulationRows<'t, P, E>` because it stores the stop reason; the FR-119 Inputs block is stale | src/relationship_extent.rs:515; spec/functional/FR-119-admit-and-read-relationship-extents.md |
| FND-011 | low | AC-4 asserts that a conforming missing end admits via `targets` but never calls `resolve` on it, although the criterion says it remains available to `resolve` | tests/relationship_extents.rs:281-297 |
| FND-012 | low | AC-8 order fixtures never distinguish the object-type component of the whole-reference key (sources share type SYS; targets differ only by universe or identity); a (universe, identity) order would pass | tests/relationship_extents.rs:439-507 |
| FND-013 | low | AC-15 does not show duplicate population precedes reservation (no denial combined with a duplicate row), and AC-11 does not compare a prior extent store with its clone after a denied extent reservation | tests/relationship_extents.rs:671-678; src/relationship_extent.rs:844-852 |

## Evidence

- FND-001: for each instance `links.iter().any(...)` compares against all earlier links; there is no QSV count ceiling by Behavior 11. A sort-based check (sort an index permutation by triple, find the smallest second-occurrence index, then compare with the first type-mismatch index, type mismatch winning at an equal index) keeps condition order at O(n log n).
- FND-002: with `unique` in [true,false] for the source and target fixed at `false`, the cases are (true,false) and (false,false); (false,true) and (true,true) are missing.
- FND-004: `TypeEnvironment::conforms` is `sub == sup || ancestry.is_ancestor(sub, sup)` (declaration.rs:2452-2454). `RelationshipExtentStore::new` takes `types` and `table` independently; the table does not retain its environment.
- FND-005: see SR-11521 FND-001 for the AC-10 recommendation; this row records the trace overclaim only.
- Rust lane: no `unsafe`, no `std` import, `use alloc::vec::Vec` and `core::ptr`; no `unwrap`/`expect` in production paths; no bare `as` casts in the diff; the `(Value::X(_), _) => false` arms are per-variant, so a new `Value` variant still fails to compile. Public items carry docs; clippy is clean.

## Examined scope

```yaml
scope:
- id: FR-119
  path: spec/functional/FR-119-admit-and-read-relationship-extents.md
  role: examined
  excerpt: quire-semantic-value SHALL provide a no_std relationship-extent store beside ObjectClosure for model-layer callers supplying full kernel ObjectReferences.
- id: TC-919
  path: spec/test-cases/TC-919-relationship-extents-admit-and-read.md
  role: examined
  excerpt: Relationship extents admit in condition order and read by full reference key
- id: FR-106
  path: spec/functional/FR-106-object-closure-admission.md
  role: context_only
  excerpt: Supplies ObjectClosure.
- id: FR-108
  path: spec/functional/FR-108-report-registry-allocation-failure.md
  role: context_only
  excerpt: Supplies the fallible reservation style and typed storage request units.
```

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-014 | low | The fix round added catch-all arms that map impossible states to values: `_ => instances.len()` treats any non-mismatch refusal as no mismatch, and the inverse comparator `_ => a.cmp(b)` plus `filter_map(... links.get(*index))` silently mis-sort or drop on a broken index invariant instead of being unrepresentable (for example keep the mismatch index as `Option<(usize, ExtentRefusal<K>)>`, and sort an index permutation keyed by borrowed (target, source)) | src/relationship_extent.rs:280-283, 340-346, 423 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-015 | medium | The round-2 inverse index stores cloned (target, source) pairs. `ObjectReference` holds `ObjectId(Box<str>)` (quire-exact d18927e identity.rs:252), so each extent admission makes 2n infallible heap allocations outside the reservation helper. Only the outer Vec goes through `reserve_elements`. Under a denied or failing allocation, admission aborts instead of returning typed `ExtentRefusal::Allocation`, which breaks FR-119 Behavior 11 / AC-11. Reference storage also doubles. The ReservationPolicy-based tests cannot see the clones. The index-based inverse it replaced was fully fallible. A fix that keeps FND-014 closed: keep `inverse: Vec<usize>` reserved fallibly, sorted by indexing `links` with indices taken from `0..links.len()`, and read through the same indexing. That accepts an unreachable bounds panic over a private, construct-once permutation invariant, documented at the field. Alternatively, add a fallible `ObjectReference` clone in quire-exact and use it here. | src/relationship_extent.rs:340-347 |

## Dispositions

Round 1, reviewed head 8445fa44b4338c222e26a0dfb009b14be0d4f18e (diff 09a3641..8445fa4). Gates rerun: unit 2 pass, relationship_extents 19 pass, clippy -D warnings clean, thumbv7em-none-eabi lib build ok, cargo deny ok. Mutations run in a scratch copy: removing the membership check fails type_mismatch_checks_missing_and_unknown_ends_in_source_order; making `targets` collect into a Vec fails repeated_reads_are_pure_and_float_identity_uses_bits ("targets composition").

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Type checks run first to find the first mismatch index m; duplicates are found by sorting (source, target, index) triples over the prefix before m and taking the smallest second-occurrence index, O(n log n). The fallback on a denied scratch reservation is a bounded O(m^2) comparison scan with no allocation that terminates and preserves precedence; it is reached only under reservation denial, documented in code and covered by the denied-reservation DuplicateTriple unit test, so it is acceptable and not a finding. |
| FND-002 | fixed | All four (source_unique, target_unique) combinations are now tested. |
| FND-003 | fixed | The row is now [5, UNKNOWN]; ty(5) is first in caller order and sorts after ty(4), and the test expects ty(5). |
| FND-004 | fixed | A table admitted against the full environment and a store environment holding only Pump yields EndTypeMismatch with actual == declared == Sys; deleting the membership check fails this test (mutation run). |
| FND-005 | fixed | The AC-10 test now measures every read with allocation-counter and fails on an allocating read (mutation run); the trace tag now matches what the test verifies. |
| FND-006 | fixed | A CapacityPolicy test seam drives Capacity through table, extent, entry and manifest admission. |
| FND-007 | fixed | targets and sources use partition_point over the sorted links and inverse index. |
| FND-008 | fixed | The query reference no longer shares the store lifetime; a test keeps a returned reference after the query key is dropped. |
| FND-009 | fixed | Indexing replaced with get; see new FND-014 for the catch-all arms that replaced it. |
| FND-010 | fixed | The FR-119 Inputs block now declares PartialPopulationRows<'t, P: Ord + Clone, E>. |
| FND-011 | fixed | The conforming missing end now resolves Dangling under a closed complete manifest. |
| FND-012 | fixed | A (9, Pump, p1) source sorts after (9, Sys, s1), so the type component is distinguished from identity. |
| FND-013 | fixed | A deny-second policy shows a duplicate population is refused after exactly one reservation; a prior extent store is compared with its clone after a denied reservation. |
| FND-014 | fixed | Round 2 (head 5cba0265ec483fd062631fce8e446add5c3a5c40): the first mismatch is carried as (instance_index, refusal) and checked_len uses map_or, so no catch-all arm remains (grep finds no `_ =>` in the file). The inverse holds sorted (target, source) pairs, which removes the comparator fallback and the filter_map drop. The reads use partition_point and stay O(log n + k). A mutation that makes `sources` collect into a Vec still fails the AC-10 test ("sources composition"). The unit tests (2), relationship_extents (19) and clippy pass. The new storage is a separate finding, FND-015. |
| FND-015 | fixed | Round 3 (head 2f945ec8d8a37c663fb647929142757529d319c4): inverse is a Vec<usize> permutation of 0..links.len(), reserved through reserve_elements and mapped by extent_storage, so Allocation and Capacity stay typed. A new unit test denies the second (inverse) reservation and gets Allocation(1 AdditionalElements). No ObjectReference clone remains on admission; the only remaining clone is the caller key K, unchanged from round 1. Indexing links[i] is the only panic path. It is unreachable because StoredExtent is private, built once, never mutated, and documented at the field. No filter_map(get), catch-all arm or silent drop is reintroduced, so FND-014 stays fixed. sources is two partition_points plus k, so O(log n + k). The sort key (target, source, index) keeps whole-source order within a target and repeats adjacent. In a scratch mutation, making sources collect into a Vec fails the AC-10 test (sources composition). Unit tests (3), relationship_extents (19) and clippy pass. |
