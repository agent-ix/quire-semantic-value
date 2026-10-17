---
id: SR-11977
title: "IR-830 integrity review of QSV PR #40"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-semantic-value@a22519b3f16a10e9158082892c8eccf36aae6456; spec/functional/FR-120-model-transition-identity-type.md; spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-920
    type: references
---

## Summary

Ticket: IR-830. PR quire-semantic-value#40, reviewed head a22519b3f16a10e9158082892c8eccf36aae6456. This review checks completeness, consistency and atomicity of FR-120 and TC-920 against each other and against the sibling QSV API conventions on origin/main: `semantic_node::IdentityRefusal`, `IDENTITY_LIMITS` and the `ReservationPolicy` seam in `declaration`. `quire validate --scope . <FR-120> <TC-920>` passes (quire 0.36.2). Numbering holds: FR-120 follows FR-119, TC-920 follows TC-919, and neither id is used elsewhere in QSV or by any open PR. The new module `model_transition` follows the crate's one-module-per-concept layout (`src/lib.rs` `pub mod` list). TC-920 covers every criterion: AC-1 to AC-8 by steps 1-8, and AC-9 to AC-11 by step 9.

## Reviewed units

| ID | Role | Path | Excerpt |
| --- | --- | --- | --- |
| FR-120 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` SHALL check the operation first, then the receiver when present, then each argument in order, and stop at the first failure. A text is valid when it is one JSON value that equals its own RFC 8785 re-encoding by `quire-canonical`. A failure SHALL carry the site, `Operation`, `Receiver` or the zero-based `Argument` index, and a typed cause, and no value is built. The check SHALL stop at the supplied `Limits`, and a reservation failure SHALL be a typed `Allocation`, never an abort or panic. |
| FR-120-AC-2 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value; a receiver that is not one canonical JCS document is refused the same way with site `Receiver`. |
| FR-120-AC-3 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation, a bad receiver and a bad argument it reports the operation, with a bad receiver and a bad argument it reports the receiver, and with two bad arguments it reports the lower index. |
| FR-120-AC-8 | examined | spec/functional/FR-120-model-transition-identity-type.md | `new` and `canonical_bytes` under a `Limits` smaller than the input or output return a typed refusal naming the limit with site `Operation`, `Receiver` or `Argument(i)` as applicable, and a denied reservation returns `Allocation`; none panics or aborts. |
| TC-920 | examined | spec/test-cases/TC-920-model-transition-identity-ordering-and-build.md | Verify FR-120 through QSV's public `model_transition` API. Scope: FR-120-AC-1 through FR-120-AC-8 as tests; FR-120-AC-9 by inspection of the public items; FR-120-AC-10 by `make build-no-std` and `make lint`; FR-120-AC-11 by the `Cargo.toml` and `Cargo.lock` diff of the implementing change. Planned; the tests are written with the code. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Behavior 2 says "the check SHALL stop at the supplied `Limits`" but does not say what the limit bounds. `quire_canonical::read` takes its own `max_input_bytes: u64`, and the re-encoding runs under `Limits::max_bytes`. The spec does not say whether one `Limits` bounds each text on its own (read input and re-encoded output) or the operation, receiver and arguments together, or how it relates to the `canonical_bytes` limit. Two implementers would refuse different inputs. State that `limits.max_bytes()` is the per-text read and re-encode bound, or give a cumulative rule. | spec/functional/FR-120-model-transition-identity-type.md:114-120 |
| FND-002 | low | The proposed refusal vocabulary departs from its sibling `semantic_node::IdentityRefusal` (`InputBytes { bound, required }`, `Allocation { requested }`, `NonCanonical`). `TransitionCause::{NotCanonicalJson, Limit, Allocation}` carries no bound, required or requested amount, and `canonical_bytes` returns raw `quire_canonical::Error` where the siblings convert into their own refusal. Align the names and payloads with `IdentityRefusal`, or reuse it, so callers map one vocabulary. | spec/functional/FR-120-model-transition-identity-type.md:91-100 |
| FND-003 | low | Behavior 5 says the field-wise order of operation then arguments "differs from byte order wherever JSON escaping reorders two operation texts". That explanation is incomplete: RFC 8785 puts `arguments` before `operation`, so the field-wise order also differs whenever both the operations and the arguments differ. State both causes, or simply say the order is byte order and nothing else. | spec/functional/FR-120-model-transition-identity-type.md:132-136 |

## Verdict

Approve with findings for this method. FR-120 and TC-920 agree on scope, numbering and verification methods, and every criterion has a TC-920 step. The one medium gap is the unstated meaning of `Limits` in `new`. The refusal-vocabulary drift and the incomplete ordering rationale are low.

## Dispositions

Round 1 on reviewed head `b6004d37b13eff375c7ee7b4e6fc44b5c30ee9f8` against the review-pass head `a22519b3f16a10e9158082892c8eccf36aae6456`, scoped to `git diff a22519b..b6004d3` of FR-120 and TC-920. Each outcome was checked against the new spec text and, for the Encode claims, against quire-canonical at the locked revision 5dc4e12; the original finding rows above remain unchanged.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Behavior 4 and Inputs: `limits.max_bytes()` bounds each text on its own as read input bound and re-encoding bound, not the texts together; AC-8 tests the together-over case. |
| FND-002 | fixed | TransitionCause gains `InputBytes { bound, required }` and `Allocation { requested }` with IdentityRefusal's payloads, and canonical_bytes returns `semantic_node::IdentityRefusal`. `NotCanonicalJson` vs `NonCanonical` remain distinct names for distinct causes, which is correct. |
| FND-003 | fixed | Behavior 8 states that `arguments` sorts before `operation`, that a longer sequence orders before its prefix, and that escaping can reverse operations; AC-6 tests the prefix case (verified: `,` 0x2C < `]` 0x5D). |
