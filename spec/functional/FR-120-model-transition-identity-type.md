---
id: FR-120
title: "ModelTransition carries an operation, receiver and ordered arguments as a canonical transition identity"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-060
    type: references
  - target: ix://agent-ix/quire-semantic-value/FR-108
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-101
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-120
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-018
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-specification/FR-181
    type: references
---
# FR-120: ModelTransition carries an operation, receiver and ordered arguments as a canonical transition identity

## Description

`quire-semantic-value` SHALL provide a public `no_std` type `ModelTransition`:
one applied model operation, named by its declaration identity text, with its
optional receiver and the ordered arguments it was applied to. It yields the
transition identity of [QSpec FR-181](ix://agent-ix/quire-specification/FR-181)
in RFC 8785 JCS bytes, encodes itself through `quire_canonical::Encode`, and
orders by those bytes, as [QSL FR-101](ix://agent-ix/quire-spec-language/FR-101)
requires of successor order. QSL ADR-018 LA-5 and ADR-011 X-11 place the type
here so that QSL's evaluator and the contract-lowering stage share one
definition without a dependency edge to a QSL layer crate.

The type holds exactly `operation`, `receiver` and `arguments`. It carries no
step probability, reward or other field; those belong to later statistical work
in QSL and add to this type only by a later requirement. QSV interprets neither
the operation, the receiver nor an argument: the declaration identity text is
minted by QSL ([FR-060](./FR-060-the-crate-mints-no-identity.md)), and the
receiver and each argument arrive already in QSL's typed canonical form.

The identity shape is QSpec FR-181's, which owns it on its main branch: the
receiver is its own member, `null` for an operation with no receiver, and is not
argument 0; arguments are in parameter declaration order. QSL FR-120 currently
writes the receiver as argument 0, and QSL FR-101 and ADR-016 ID-9 write the
identity without a receiver member. The amendment of those three to the QSpec
form is requested of the QSL lane and has no ticket named here yet; this
requirement does not claim it is done.

This is the spec slice. The code lands in a later pull request against the
criteria below.

## Use case

QSL's `ModelSystem` applies `increment` to counter `c1` and builds a
`ModelTransition` from the operation's declaration identity text, the receiver
`c1` and the encoded argument vector. QSL's `TransitionSystem` binds its
transition id to `quire_canonical::Encode`, so the engine encodes the value
directly, orders a state's successors by it, and contract lowering reads the
same value without importing a QSL layer crate.

## Inputs

- `operation`: the operation's declaration identity text, a string that is
  itself one canonical JCS document.
- `receiver`: absent, or the receiver reference's encoding, one canonical JCS
  document other than `null`.
- `arguments`: an ordered sequence of argument encodings in parameter
  declaration order, each one canonical JCS document (QSL's typed canonical
  form of one value).
- A `quire_canonical::Limits`, whose `max_bytes` bounds each text separately in
  `new` and the whole identity in `canonical_bytes`.

## Outputs

- A `ModelTransition`, or a typed refusal.
- The canonical identity bytes, or the sibling `IdentityRefusal`.

## Public API inventory

All items are at `quire_semantic_value::model_transition`; the module is new and
no existing item changes.

```rust
pub struct ModelTransition { /* private: operation, receiver, arguments */ }

impl ModelTransition {
    pub fn new(
        operation: String,
        receiver: Option<String>,
        arguments: Vec<String>,
        limits: Limits,
    ) -> Result<Self, InvalidModelTransition>;
    pub fn operation(&self) -> &str;
    pub fn receiver(&self) -> Option<&str>;
    pub fn arguments(&self) -> &[String];
    pub fn canonical_bytes(&self, limits: Limits)
        -> Result<Vec<u8>, semantic_node::IdentityRefusal>;
}

impl quire_canonical::Encode for ModelTransition { /* explicit stack */ }

pub struct InvalidModelTransition { /* private */ }
impl InvalidModelTransition {
    pub fn site(&self) -> TransitionSite;
    pub fn cause(&self) -> &TransitionCause;
}
pub enum TransitionSite { Operation, Receiver, Argument(usize) }
pub enum TransitionCause {
    NotCanonicalJson,
    NullReceiver,
    InputBytes { bound: u64, required: u64 },
    Allocation { requested: usize },
}

// Clone, Debug, Eq, PartialEq implemented or derived; Ord and PartialOrd
// implemented over the canonical bytes, not derived.
```

`InputBytes` and `Allocation` carry the same payloads as the sibling
`IdentityRefusal` variants, and `canonical_bytes` reuses `IdentityRefusal`, so a
caller maps one vocabulary. Names are the proposed ones and may change in
review; the criteria bind the behavior. The type has no `Serialize` or
`Deserialize`: `quire-canonical` takes serde values only through `FixedShape`,
which cannot state a depth that follows the arguments.

## Behavior

1. `new` SHALL retain the operation text, the receiver (present or absent) and
   the argument sequence verbatim and in the supplied order. It SHALL admit an
   empty argument sequence and any number of arguments, and SHALL NOT check
   arity, which is QSL's declaration.
2. `new` SHALL check the operation first, then the receiver when present, then
   each argument in order, and stop at the first failure. A text is valid when
   it is one JSON value that equals its own RFC 8785 re-encoding by
   `quire-canonical`. A failure SHALL carry the site, `Operation`, `Receiver` or
   the zero-based `Argument` index, and a typed cause, and no value is built.
3. `new` SHALL refuse a receiver whose document is `null` with cause
   `NullReceiver`, because an absent receiver is the only `null` receiver in the
   identity and a second spelling would give two unequal values one byte string.
4. `new` SHALL apply `limits.max_bytes()` to each text on its own, as the input
   byte bound of its read and the byte bound of its re-encoding, not to the texts
   together. `new` SHALL refuse an excess with cause `InputBytes` at that
   text's site, and SHALL map a reservation that `quire-canonical` fails to cause
   `Allocation`, never an abort or panic.
5. `canonical_bytes` SHALL return the RFC 8785 UTF-8 bytes of the JSON object
   with members `type` equal to `"transition"`, `operation` equal to the
   operation text as a JSON string, `receiver` equal to the receiver document
   as the JSON value it encodes or `null` when absent, and `arguments` equal to
   the array of the argument documents in order, each embedded as the JSON
   value it encodes. Members appear in RFC 8785 order (`arguments`, `operation`,
   `receiver`, `type`). The bytes are the one definition of transition
   identity. `limits.max_bytes()` SHALL bound the whole identity; an excess is
   `IdentityRefusal::InputBytes` and no bytes are returned.
6. `Encode` SHALL write the same bytes as `canonical_bytes` under the same
   limits, by pushing events from an explicit stack. It SHALL read each stored text
   again inside `encode_into` and embed the resulting document through its own
   non-recursive `quire-canonical` encoding, and SHALL map a `ReadError::Allocation`
   from that read to `quire_canonical::Error::Allocation`, a read limit to
   `Error::Limit` and any other read refusal to a `quire_canonical::Error`
   variant, never an unwrap, expect or abort. No step of
   `new`, `Encode`, equality, comparison, `Clone`, `Debug` or `Drop` SHALL
   recurse once per nesting level of an operation, receiver or argument.
7. Two values SHALL be equal exactly when their operation texts are equal, their
   receivers are equal (absent equals only absent) and their argument sequences
   are equal element by element. Argument order and count are significant, and
   two values that differ only in receiver are unequal. Equality agrees with
   equality of `canonical_bytes`.
8. The order SHALL be the ascending unsigned-lexicographic order of
   `canonical_bytes` (QSL FR-101 successor order). It is a total order
   consistent with equality. The order is not field-wise: `arguments` sorts before
   `operation` in RFC 8785, a longer argument sequence orders before its own
   prefix (`,` sorts before `]`), and JSON escaping can reverse two operation
   texts.
9. `cmp` and `partial_cmp` SHALL take no limits, fail never and allocate
   nothing, and SHALL agree with `canonical_bytes` under any limit that admits
   both values.
10. The type SHALL use only `core` and `alloc` and the crate's existing
   dependencies. The change that adds it SHALL keep `make build-no-std` (target
   `thumbv7em-none-eabi`) green and SHALL NOT alter the crate's `[dependencies]`
   table, as ADR-011 X-11 requires of the crate.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-120-AC-1 | `ModelTransition::new` over a canonical operation text, receiver `"c1"` and arguments `true`, `[1,2]`, `{"a":1}` returns a value whose `operation()`, `receiver()` and `arguments()` equal the supplied text, receiver and sequence in the supplied order; an absent receiver reads as `None`; an empty argument sequence and a sequence of 64 arguments are both admitted, and no arity check refuses either. | Test (TC-920) |
| FR-120-AC-2 | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value; a receiver that is not one canonical JCS document is refused the same way with site `Receiver`. | Test (TC-920) |
| FR-120-AC-3 | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation, a bad receiver and a bad argument it reports the operation, with a bad receiver and a bad argument it reports the receiver, and with two bad arguments it reports the lower index. | Test (TC-920) |
| FR-120-AC-4 | For operation `{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}`, receiver `"c1"` and the single argument `true`, `canonical_bytes` equals exactly `{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\",\"node\":\"test/counters/Counter/increment\",\"package\":\"test/counters\"}","receiver":"c1","type":"transition"}`; with an absent receiver the member is `"receiver":null`; with no arguments the array is `[]`; with arguments `1` then `2` they appear in that order. | Test (TC-920) |
| FR-120-AC-5 | Values with the same operation, receiver and arguments are equal and have equal `canonical_bytes`; a different operation, a different argument, the same arguments in swapped order, a prefix of the arguments, a different receiver, and an absent receiver against the receiver document `"null"` (a string) are each unequal with unequal bytes; two values differing only in receiver have identities differing only in `receiver`. | Test (TC-920) |
| FR-120-AC-6 | With equal receiver and arguments, the values over operation texts whose nodes are `x` and `x/y` in one package order as their `canonical_bytes` do, the opposite of the order of the two operation texts compared as raw strings; with equal operation and receiver, arguments `[1]` order after arguments `[1,2]`, as their bytes do. | Test (TC-920) |
| FR-120-AC-7 | `quire_canonical::to_vec` of a `ModelTransition` under the same `Limits` yields bytes equal to `canonical_bytes` for the AC-4 values, and, when the limit is too small, an error whose `IdentityRefusal::from` equals the `IdentityRefusal` that `canonical_bytes` returns (equal `bound` and `required`). | Test (TC-920) |
| FR-120-AC-8 | `new` under a `Limits` smaller than one text refuses with that text's site (`Operation`, `Receiver` or `Argument(i)`) and cause `InputBytes{bound, required}` with `bound` equal to the limit and `required` greater than it; texts each within the limit are admitted although together over it. | Test (TC-920) |
| FR-120-AC-9 | The private fields of `ModelTransition` and its public accessors hold exactly `operation`, `receiver` and `arguments`: no probability, reward, weight or other member exists, and `ModelTransition` is the only new public value type of the module besides the refusal types of the inventory. | Inspection (public API of `model_transition`) |
| FR-120-AC-10 | With `ModelTransition` present, `make build-no-std` (`cargo build --locked --target thumbv7em-none-eabi`) and `cargo clippy --lib --target thumbv7em-none-eabi -- -D warnings` succeed, and the module names no `std` path. | Analysis (`make build-no-std`, `make lint`) |
| FR-120-AC-11 | The `[dependencies]` table of `Cargo.toml` is unchanged by the change that adds `ModelTransition`: exactly `quire-exact`, `quire-canonical` (default features off), `serde` and `thiserror`, each with its prior source and features; `[dev-dependencies]` and the `Cargo.lock` package set are unchanged, and `cargo build --locked` succeeds. | Inspection (`git diff` of `Cargo.toml` and `Cargo.lock`) |
| FR-120-AC-12 | `new` refuses the receiver text `null` with site `Receiver` and cause `NullReceiver`, and builds no value; for every pair of values built over the AC-5 and AC-6 fixtures, `cmp` returns `Equal` exactly when `==` is true. | Test (TC-920) |
| FR-120-AC-13 | `canonical_bytes` under a `Limits` smaller than the identity returns `IdentityRefusal::InputBytes` with `bound` equal to the limit and `required` greater than it, and returns no bytes; under exactly the identity length it succeeds. | Test (TC-920) |
| FR-120-AC-14 | Inspection of `new` and `canonical_bytes` confirms every `quire_canonical` allocation refusal (read tree in `new`, the re-read inside `encode_into`, writer buffer) is mapped to `Allocation{requested}`, `IdentityRefusal::Allocation` or `quire_canonical::Error::Allocation` respectively, and neither contains an `unwrap`, `expect`, indexing panic or `abort` path; QSV reserves no storage itself on these paths. | Inspection (`model_transition` source) |
| FR-120-AC-15 | `cmp` and `partial_cmp` allocate zero bytes (measured with `allocation-counter`), take no `Limits`, and over at least 20 values mixing operations, receivers (present, absent) and argument counts, the sorted order of the values equals the sorted order of their `canonical_bytes`, and `cmp` is antisymmetric and transitive over the set. | Test (TC-920) |
| FR-120-AC-16 | A value whose receiver and an argument are each an array nested 100,000 levels deep is built by `new`, encoded by `quire_canonical::to_vec` and `canonical_bytes`, compared, cloned, formatted with `Debug` and dropped without stack overflow, and the two encodings are equal. | Test (TC-920) |

## Status

IMPLEMENTED/TESTS PASSED: FR-120-AC-1 through FR-120-AC-8, FR-120-AC-12, FR-120-AC-13,
FR-120-AC-15 and FR-120-AC-16 (`tests/model_transition.rs`, traced to TC-920).

IMPLEMENTED/TESTS PASSED: FR-120-AC-10. `cargo build --locked --target
thumbv7em-none-eabi` (`make build-no-std`) and `cargo clippy --lib --target
thumbv7em-none-eabi -- -D warnings` both succeed with `ModelTransition` present,
and `src/model_transition.rs` names no `std` path.

IMPLEMENTED/INSPECTED: FR-120-AC-11. No test backs this criterion. Inspection of
`git diff origin/main` shows no change to `Cargo.toml` or `Cargo.lock`, and
`cargo build --locked` succeeds.

IMPLEMENTED/INSPECTED: FR-120-AC-9. No test backs this criterion. Inspection of
`src/model_transition.rs` found the private fields `operation`, `receiver` and
`arguments` only, the accessors of the same three, and the new public items
`ModelTransition`, `InvalidModelTransition`, `TransitionSite` and
`TransitionCause`; no probability, reward or weight member exists.

IMPLEMENTED/INSPECTED: FR-120-AC-14. No test backs this criterion. Inspection of
`new`, `check`, `canonical_bytes` and `embed` found that a `ReadError::Allocation`
or `Error::Allocation` maps to `TransitionCause::Allocation`,
`IdentityRefusal::Allocation` or `quire_canonical::Error::Allocation`
respectively, with no `unwrap`, `expect`, indexing or `abort` on those paths, and
no storage reserved by QSV itself. `check` names an encoder fault
(`Error::Protocol`, `Error::Internal`) and reports it as `NotCanonicalJson`: the
IR leader settled that FR-120's cause list stays at four causes, because such a
fault is unreachable by construction for a document `read` has just produced.

The design points below were ruled by the QSL planner (terra:QSL:plan, relayed
by the IR planner to this author); the ticket IR-830 records only the narrower
scope comment, so this is not a ticket record. The authority for the identity
shape is QSpec FR-181 on its main branch. Ruled: the QSpec FR-181 identity with
a separate receiver, canonical-JCS text elements, validation by RFC 8785
re-encoding, RFC 8785 member order and byte-order `Ord`. Round 2 of review
changed the encoding route to `quire_canonical::Encode`, which QSL's
`TransitionSystem::TransitionId` binds (qsl-eval `simulation/explore.rs`), and
dropped `Serialize`. The QSL planner (terra:QSL:plan) confirmed the `Encode`
route (the `TransitionId` bound at qsl-eval `simulation/explore.rs:35` and
ADR-030 D-4.7) and the null-receiver refusal, relayed by the IR lane; that is
not a ticket record.

## Dependencies

- [FR-060](./FR-060-the-crate-mints-no-identity.md): the operation text is
  supplied, not minted.
- [FR-108](./FR-108-report-registry-allocation-failure.md): reservation failure
  is a typed refusal.
- [QSL FR-101](ix://agent-ix/quire-spec-language/FR-101) and
  [QSL FR-120](ix://agent-ix/quire-spec-language/FR-120) consume the type;
  QSL ADR-018 LA-5 and ADR-011 X-11 place it; QSL FR-101, FR-120 and ADR-016
  ID-9 are to be amended to QSpec FR-181's receiver member (pending, no ticket
  named here).
- Blocks QSL-493, which bumps its QSV dependency after the code lands.
