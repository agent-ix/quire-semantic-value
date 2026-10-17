---
id: SR-12011
title: "IR-830 gap analysis of QSV PR #41"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@a43b13fc941d12ca397fe10717e92cf759fd5ee7; spec/functional/FR-120-model-transition-identity-type.md, spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md, src/model_transition.rs, tests/model_transition.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-920
    type: references
---

# IR-830 gap analysis of QSV PR #41

## Summary

Ticket: IR-830, PR quire-semantic-value#41 at a43b13fc941d12ca397fe10717e92cf759fd5ee7. Planless gap analysis. Plan completion: not assessed. Authority: FR-120 AC-1..16 and TC-920 as merged in spec PR #40 (origin/main 7ad17ad). Computed matrix (quire 0.36.2, `quire matrix --strict`): exit 1 at head with 1 untagged criterion (FR-109-AC-6). It also exits 1 on origin/main, where it has 13 untagged (FR-109-AC-6 plus 12 FR-120). So the PR's tags close all 12 FR-120 test criteria, the one remaining failure was there before this PR, and FR-120-AC-9/10/11/14 read method-without-symbol (Inspection/Analysis), as the spec says. Each of the 12 bindings was read against its test body. 29 hand mutants were run in a throwaway worktree with one target dir (both removed): 22 killed, 7 survived (classified in Coverage).

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-120-AC-1 | examined | `ModelTransition::new` over a canonical operation text, receiver `"c1"` and arguments `true`, `[1,2]`, `{"a":1}` returns a value whose `operation()`, `receiver()` and `arguments()` equal the supplied text, receiver and sequence in the supplied order; an absent receiver reads as `None`; an empty a... |
| FR-120-AC-2 | examined | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value; a receiver that is not one canonical JCS ... |
| FR-120-AC-3 | examined | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation, a bad receiver and a bad argument it reports the operation, with a bad receiver and a bad argument it reports the receiver, and wit... |
| FR-120-AC-4 | examined | For operation `{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}`, receiver `"c1"` and the single argument `true`, `canonical_bytes` equals exactly `{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\",\"node\":\"test/counters/Counter/... |
| FR-120-AC-5 | examined | Values with the same operation, receiver and arguments are equal and have equal `canonical_bytes`; a different operation, a different argument, the same arguments in swapped order, a prefix of the arguments, a different receiver, and an absent receiver against the receiver document `"null"` (a st... |
| FR-120-AC-6 | examined | With equal receiver and arguments, the values over operation texts whose nodes are `x` and `x/y` in one package order as their `canonical_bytes` do, the opposite of the order of the two operation texts compared as raw strings; with equal operation and receiver, arguments `[1]` order after argumen... |
| FR-120-AC-7 | examined | `quire_canonical::to_vec` of a `ModelTransition` under the same `Limits` yields bytes equal to `canonical_bytes` for the AC-4 values, and, when the limit is too small, an error whose `IdentityRefusal::from` equals the `IdentityRefusal` that `canonical_bytes` returns (equal `bound` and `required`). |
| FR-120-AC-8 | examined | `new` under a `Limits` smaller than one text refuses with that text's site (`Operation`, `Receiver` or `Argument(i)`) and cause `InputBytes{bound, required}` with `bound` equal to the limit and `required` greater than it; texts each within the limit are admitted although together over it. |
| FR-120-AC-9 | examined | The private fields of `ModelTransition` and its public accessors hold exactly `operation`, `receiver` and `arguments`: no probability, reward, weight or other member exists, and `ModelTransition` is the only new public value type of the module besides the refusal types of the inventory. |
| FR-120-AC-10 | examined | With `ModelTransition` present, `make build-no-std` (`cargo build --locked --target thumbv7em-none-eabi`) and `cargo clippy --lib --target thumbv7em-none-eabi -- -D warnings` succeed, and the module names no `std` path. |
| FR-120-AC-11 | examined | The `[dependencies]` table of `Cargo.toml` is unchanged by the change that adds `ModelTransition`: exactly `quire-exact`, `quire-canonical` (default features off), `serde` and `thiserror`, each with its prior source and features; `[dev-dependencies]` and the `Cargo.lock` package set are unchanged... |
| FR-120-AC-12 | examined | `new` refuses the receiver text `null` with site `Receiver` and cause `NullReceiver`, and builds no value; for every pair of values built over the AC-5 and AC-6 fixtures, `cmp` returns `Equal` exactly when `==` is true. |
| FR-120-AC-13 | examined | `canonical_bytes` under a `Limits` smaller than the identity returns `IdentityRefusal::InputBytes` with `bound` equal to the limit and `required` greater than it, and returns no bytes; under exactly the identity length it succeeds. |
| FR-120-AC-14 | examined | Inspection of `new` and `canonical_bytes` confirms every `quire_canonical` allocation refusal (read tree in `new`, the re-read inside `encode_into`, writer buffer) is mapped to `Allocation{requested}`, `IdentityRefusal::Allocation` or `quire_canonical::Error::Allocation` respectively, and neither... |
| FR-120-AC-15 | examined | `cmp` and `partial_cmp` allocate zero bytes (measured with `allocation-counter`), take no `Limits`, and over at least 20 values mixing operations, receivers (present, absent) and argument counts, the sorted order of the values equals the sorted order of their `canonical_bytes`, and `cmp` is antis... |
| FR-120-AC-16 | examined | A value whose receiver and an argument are each an array nested 100,000 levels deep is built by `new`, encoded by `quire_canonical::to_vec` and `canonical_bytes`, compared, cloned, formatted with `Debug` and dropped without stack overflow, and the two encodings are equal. |
| TC-920 | examined | Steps 1-13 and expected results; Status sentence edited by this PR |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | In the AC-15 order-agreement set (and the AC-5/6 fixtures), every present receiver is a JSON string (`"c1"`, `"c2"`, `"null"`, `"r0"`..`"r7"`). So `cmp`'s receiver bytes and its spelling of an absent receiver are only ever compared with strings, which all start with `"` (0x22). Two wrong `identity_bytes` mutants survive all 12 tests: absent receiver as `unwrap_or("zz")`, and the receiver escaped as a string (`receiver.bytes().flat_map(Escaped::new)`). Either one changes the successor order (QSL FR-101) as soon as a receiver is `false`, a number, an array or an object, for example `[` (0x5b) against an escaped `\"` (0x5c). A throwaway probe shows the shipped code is correct for those receivers. The test does not pin it. Add non-string receivers (`false`, a number, an array or object) next to absent ones in the AC-15 set. | tests/model_transition.rs:321-372; src/model_transition.rs:169-185 |
| FND-002 | low | The Status rows keep FR-120-AC-9, AC-10, AC-11 and AC-14 as PLANNED and say their evidence is "recorded on the implementing pull request". This reviewer re-measured that evidence at head and it holds: four public items, fields operation/receiver/arguments, thumbv7em build and clippy green, no Cargo.toml/Cargo.lock diff, every allocation refusal mapped. After merge, though, the spec will report four met criteria as planned, and the only record of the evidence will be a PR body outside the repo. The repo's precedent (FR-119 Status) records measured inspection evidence in the Status section itself. Record AC-9/10/11/14 as verified by inspection/analysis, with the evidence, in FR-120 Status and the TC-920 description. | spec/functional/FR-120-model-transition-identity-type.md:204-211; spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md:17-19 |

## Coverage

- Reconciliation: `quire matrix --strict` (quire 0.36.2). Head: FR-120-AC-1..8, 12, 13, 15, 16 tagged; AC-9/10/11/14 method-without-symbol; 1 untagged repo-wide (FR-109-AC-6). origin/main 7ad17ad: 13 untagged (FR-109-AC-6 plus the 12 FR-120 test criteria). The exit-1 is not caused by this PR.
- Plan completion: not assessed
- Bindings: all 12 tags sit on tests that assert their clause. AC-15 measures `cmp` and `partial_cmp` with `allocation_counter::measure` (count_total == 0) and has a live allocating control (`canonical_bytes` count_total > 0). AC-16 runs on a 2 MiB `thread::Builder` stack at depth 100,000. A recursive-embed mutant aborts it with `has overflowed its stack` (SIGABRT), so the test can fail. AC-13 checks one byte short and exactly the identity length. AC-3 checks operation over receiver over argument, and the lower of two bad argument indexes.
- Mutants killed (22): quote not escaped; non-ASCII escaped; absent receiver as `""`; separator always; separator `^`; arguments reversed in cmp; `]` as `}`; cmp reversed; cmp allocates (collect); field-wise cmp; null receiver accepted; receiver checked before operation; arguments checked highest index first; argument index +1; new read limit minus 1; per-text limit summed across texts; read Limit as NotCanonicalJson; `required` set to `bound`; canonical check skipped; Encode absent receiver as string "null"; Encode arguments reversed; recursive embed.
- Mutants survived (7): absent receiver as `zz` and receiver escaped in cmp (FND-001). Backslash not escaped in cmp is equivalent for order: a 200,000-pair random check over canonical JSON strings found no order difference between escaping `\\` and not escaping it, and `cmp` exposes nothing but order. `\n` as `\u000a` and uppercase hex are dead arms: a canonical text has no raw C0 byte. Read and embed Allocation remapped: AC-14 is inspection-only by the spec, and inspection confirms the mapping.
- AC-9/10/11/14 inspection evidence in the PR body was re-measured and is honest: public items ModelTransition, InvalidModelTransition, TransitionSite, TransitionCause; private fields operation/receiver/arguments; `make build-no-std` and thumbv7em clippy green; no `std::` in the module; `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` empty; allocation mapping as in SR-12010's verdict.
- Untraced behaviors / stubs: none. Every production item in the module serves FR-120. No stub, no tautological assertion.
- Semantic review: FR-120 behavior 1-10 intent was checked against code and tests. Equality and order are injective over the stored fields because `Some("null")` is refused and canonical texts cannot hold a top-level comma.

## Verdict

CHANGES REQUESTED: one medium (a test gap in the receiver kinds of the AC-15 order set), one low (status wording). Recorded clean: AC-1 (verbatim accessors, None, empty, 64 arguments), AC-2 (four bad operations, two bad receivers), AC-3 (site order and lowest index), AC-4 (exact bytes, null receiver, [], argument order), AC-5 (the 14-fixture equality/bytes matrix; the difference is only the receiver member), AC-6 (escape reversal x vs x/y, `[1,2]` before `[1]`), AC-7 (to_vec == canonical_bytes, same refusal one byte short), AC-8 (per-site InputBytes with bound and required; texts that fit one by one but not together are admitted), AC-12 (NullReceiver; cmp Equal iff ==), AC-13, AC-16. The production code is correct. The finding is about test strength.

## Dispositions

Round 1, reviewed at 3b73897e0d134190bce7b29d8f248b83705a5756 (fix commits 52e46bc, 3b73897). `quire matrix --strict` still exits 1 on FR-109-AC-6 only, which predates this PR; FR-120 rows unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `fixtures()` adds 17 receivers: false, true, 0, 1, -1, 1.5, [], [1], [1,2], [[1]], {}, {"a":1}, a nested object holding null, and strings with `\"`, `\\`, `\n` and `é`. The set feeds AC-5, AC-12 and AC-15, and AC-15 now also asserts `a.cmp(b) == bytes(a).cmp(&bytes(b))` for every pair. Re-run at head: `unwrap_or("zz")` for an absent receiver, `receiver.bytes().flat_map(Escaped::new)`, and the absent receiver as `""` are each KILLED by `cmp_allocates_nothing_and_sorts_like_the_bytes`. Not escaping backslashes still survives; it is equivalent for order, as before. |
| FND-002 | fixed | FR-120 Status now records the evidence for AC-9, 10, 11 and 14 in the repo, in the FR-119 style, and TC-920 points to it. I re-measured it at head and it holds. `IMPLEMENTED/INSPECTED` is accepted: the ecosystem has no existing token for inspection-only evidence, and this one is unambiguous next to `IMPLEMENTED/TESTS PASSED`. Non-blocking: AC-11 is verified by inspection of a `git diff`, so `IMPLEMENTED/INSPECTED` would fit it better than `TESTS PASSED`. AC-10 is a build and lint run, so `TESTS PASSED` is fine there. |
