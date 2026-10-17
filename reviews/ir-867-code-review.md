---
id: SR-12365
title: "IR-867 code review (Rust lane) of QSV PR #42"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@f2c92ce08573f10b6244d304162fc07deee3bae3; tests/model_transition.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-920
    type: references
---

# IR-867 code review (Rust lane) of QSV PR #42

## Summary

Ticket: IR-867. The diff deletes 12 redundant `/// Trace:` lines from `tests/model_transition.rs`. Each deleted criterion id is present in the immediately following `#[trace("TC-920", "FR-120-AC-*")]` on the same test function. No Rust item, attribute, assertion, or production code changes. The Rust review checked the whole diff against the repository conventions. `git diff --check` and `cargo fmt --check` pass.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| new_retains_operation_receiver_and_arguments_verbatim | examined | `FR-120-AC-1` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| new_refuses_a_non_canonical_operation_or_receiver | examined | `FR-120-AC-2` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| new_checks_operation_then_receiver_then_arguments_in_order | examined | `FR-120-AC-3` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| canonical_bytes_are_the_jcs_transition_object | examined | `FR-120-AC-4` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| equality_is_field_wise_and_agrees_with_the_identity_bytes | examined | `FR-120-AC-5` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| order_is_the_byte_order_not_the_field_order | examined | `FR-120-AC-6` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| encode_writes_the_canonical_bytes_and_the_same_refusal | examined | `FR-120-AC-7` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| new_bounds_each_text_on_its_own | examined | `FR-120-AC-8` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| a_null_receiver_is_refused_and_cmp_agrees_with_equality | examined | `FR-120-AC-12` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| canonical_bytes_refuses_a_limit_below_the_identity | examined | `FR-120-AC-13` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| cmp_allocates_nothing_and_sorts_like_the_bytes | examined | `FR-120-AC-15` remains in its `#[trace]` attribute after the adjacent doc line is removed. |
| deep_nesting_does_not_overflow_the_stack | examined | `FR-120-AC-16` remains in its `#[trace]` attribute after the adjacent doc line is removed. |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

**PASS.** All 12 trace bindings remain attached to the same tests. Quire base and head matrices are identical after source-line normalization. Strict coverage is unchanged at 88/97 backed, 9 unbacked, and 0 contradicted; the pre-existing unbacked rows are outside this deletion. No runtime tests were rerun for this doc-only diff.

## Acceptance criteria examined

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-120-AC-1 | examined | `ModelTransition::new` over a canonical operation text, receiver `"c1"` and arguments `true`, `[1,2]`, `{"a":1}` returns a value whose `operation()`, `receiver()` and `arguments()` equal the supplied text, receiver and sequence in the supplied order; an absent receiver reads as `None`; an empty argument sequence and a sequence of 64 arguments are both admitted, and no arity check refuses either. |
| FR-120-AC-2 | examined | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value; a receiver that is not one canonical JCS document is refused the same way with site `Receiver`. |
| FR-120-AC-3 | examined | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation, a bad receiver and a bad argument it reports the operation, with a bad receiver and a bad argument it reports the receiver, and with two bad arguments it reports the lower index. |
| FR-120-AC-4 | examined | For operation `{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}`, receiver `"c1"` and the single argument `true`, `canonical_bytes` equals exactly `{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\",\"node\":\"test/counters/Counter/increment\",\"package\":\"test/counters\"}","receiver":"c1","type":"transition"}`; with an absent receiver the member is `"receiver":null`; with no arguments the array is `[]`; with arguments `1` then `2` they appear in that order. |
| FR-120-AC-5 | examined | Values with the same operation, receiver and arguments are equal and have equal `canonical_bytes`; a different operation, a different argument, the same arguments in swapped order, a prefix of the arguments, a different receiver, and an absent receiver against the receiver document `"null"` (a string) are each unequal with unequal bytes; two values differing only in receiver have identities differing only in `receiver`. |
| FR-120-AC-6 | examined | With equal receiver and arguments, the values over operation texts whose nodes are `x` and `x/y` in one package order as their `canonical_bytes` do, the opposite of the order of the two operation texts compared as raw strings; with equal operation and receiver, arguments `[1]` order after arguments `[1,2]`, as their bytes do. |
| FR-120-AC-7 | examined | `quire_canonical::to_vec` of a `ModelTransition` under the same `Limits` yields bytes equal to `canonical_bytes` for the AC-4 values, and, when the limit is too small, an error whose `IdentityRefusal::from` equals the `IdentityRefusal` that `canonical_bytes` returns (equal `bound` and `required`). |
| FR-120-AC-8 | examined | `new` under a `Limits` smaller than one text refuses with that text's site (`Operation`, `Receiver` or `Argument(i)`) and cause `InputBytes{bound, required}` with `bound` equal to the limit and `required` greater than it; texts each within the limit are admitted although together over it. |
| FR-120-AC-12 | examined | `new` refuses the receiver text `null` with site `Receiver` and cause `NullReceiver`, and builds no value; for every pair of values built over the AC-5 and AC-6 fixtures, `cmp` returns `Equal` exactly when `==` is true. |
| FR-120-AC-13 | examined | `canonical_bytes` under a `Limits` smaller than the identity returns `IdentityRefusal::InputBytes` with `bound` equal to the limit and `required` greater than it, and returns no bytes; under exactly the identity length it succeeds. |
| FR-120-AC-15 | examined | `cmp` and `partial_cmp` allocate zero bytes (measured with `allocation-counter`), take no `Limits`, and over at least 20 values mixing operations, receivers (present, absent) and argument counts, the sorted order of the values equals the sorted order of their `canonical_bytes`, and `cmp` is antisymmetric and transitive over the set. |
| FR-120-AC-16 | examined | A value whose receiver and an argument are each an array nested 100,000 levels deep is built by `new`, encoded by `quire_canonical::to_vec` and `canonical_bytes`, compared, cloned, formatted with `Debug` and dropped without stack overflow, and the two encodings are equal. |
