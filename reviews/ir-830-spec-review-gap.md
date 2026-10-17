---
id: SR-11978
title: "IR-830 gap analysis of QSV PR #40"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@a22519b3f16a10e9158082892c8eccf36aae6456; FR-120-AC-1..AC-11 against TC-920 (spec-only; no code in the diff)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-920
    type: references
---

## Summary

Ticket: IR-830. PR quire-semantic-value#40, reviewed head a22519b3f16a10e9158082892c8eccf36aae6456. Planless gap analysis. Plan completion: not assessed. The computed matrix (`quire matrix --scope . --format json`, quire 0.36.2) lists FR-120-AC-1..AC-8 as `untagged` and AC-9..AC-11 as `method-without-symbol`. That is expected for a spec slice, and FR-120 Status says so honestly ("PLANNED ... Spec only; the code and TC-920 tests follow"). The diff adds no production code, so there is no orphan code. The semantic check here asks whether each TC-920 procedure and expected result can actually test its criterion against the QSV and quire-canonical code that exists today.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-120-AC-3 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation, a bad receiver and a bad argument it reports the operation, with a bad receiver and a bad argument it reports the receiver, and with two bad arguments it reports the lower index. |
| FR-120-AC-2 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value; a receiver that is not one canonical JCS document is refused the same way with site `Receiver`. |
| FR-120-AC-8 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` and `canonical_bytes` under a `Limits` smaller than the input or output return a typed refusal naming the limit with site `Operation`, `Receiver` or `Argument(i)` as applicable, and a denied reservation returns `Allocation`; none panics or aborts. |
| FR-120-AC-1 | examined | spec/functional/FR-120-model-transition-identity-type.md | `ModelTransition::new` over a canonical operation text, receiver `"c1"` and arguments `true`, `[1,2]`, `{"a":1}` returns a value whose `operation()`, `receiver()` and `arguments()` equal the supplied text, receiver and sequence in the supplied order; an absent receiver reads as `None`; an empty argument sequence and a sequence of 64 arguments are both admitted, and no arity check refuses either. |
| FR-120-AC-4 | examined | spec/functional/FR-120-model-transition-identity-type.md | For operation `{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}` receiver `"c1"` and the single argument `true`, `canonical_bytes` equals exactly `{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\",\"node\":\"test/counters/Counter/increment\",\"package\":\"test/counters\"}","receiver":"c1","type":"transition"}`; with an absent receiver the member is `"receiver":null`; with no arguments the array is `[]`; with arguments `1` then `2` they appear in that order. |
| TC-920 | examined | spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md | Step 3: Call `new` with a valid operation and arguments `true`, `{"b":1,"a":2}`, `false` and again with `[1, 2]` and `{"b":1,"a":2}`; then a bad operation with a bad receiver and a bad argument, and a bad receiver with a bad argument. Expected: Step 3: `Argument(1)` first case, `Argument(1)` second case; the first pair reports `Operation`, the second `Receiver`. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-920 Step 3's second case passes arguments `[1, 2]` and `{"b":1,"a":2}`. Both are non-canonical: `[1, 2]` has insignificant whitespace and re-encodes as `[1,2]`. FR-120-AC-3 ("with two bad arguments it reports the lower index") therefore requires `Argument(0)`, but the Expected Results say "`Argument(1)` second case". A correct implementation fails the TC as written. Change the expected result to `Argument(0)`. | spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md:28-30,51-52 |
| FND-002 | medium | The `Allocation` clause of FR-120-AC-8 cannot be tested as specified. TC-920 Step 8 denies a reservation "through the crate's reservation seam". QSV's seam is `ReservationPolicy` (src/declaration.rs), but `new` and `canonical_bytes` take no policy. `new` takes owned `String`s, so QSV reserves nothing itself, and every reservation on the path is a `try_reserve` inside `quire_canonical::read` or the writer, with no injection point. No test can deny one. Add a policy parameter (or a `*_with_reservations` variant) to the inventory, or reword the clause as a tested mapping of `quire_canonical` `Allocation` into `TransitionCause::Allocation`. | spec/functional/FR-120-model-transition-identity-type.md:82-91,156 |
| FND-003 | low | TC-920 Step 2 ends with two receiver cases (`c1` and `{"b":1,"a":2}`), but the Expected Results say "`Receiver` for the last case", as if there were one. Say "`Receiver` for the two receiver cases". | spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md:25-27,49-50 |

## Verdict

FAIL for this method. All eleven criteria are mapped to TC-920 steps, and the matrix state is the honest spec-slice state. But one expected result contradicts its criterion (FND-001, high), and AC-8's allocation clause has no seam to test it through (FND-002, medium). The Ord/Eq collision in SR-11975 FND-002 also needs an AC-5/AC-6 case once it is fixed.
