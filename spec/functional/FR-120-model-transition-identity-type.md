---
id: FR-120
title: "ModelTransition carries an operation and ordered arguments as a canonical transition identity"
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
# FR-120: ModelTransition carries an operation and ordered arguments as a canonical transition identity

## Description

`quire-semantic-value` SHALL provide a public `no_std` type `ModelTransition`:
one applied model operation, named by its declaration identity text, with the
ordered arguments it was applied to. It yields the transition identity of
[QSpec FR-181](ix://agent-ix/quire-specification/FR-181) in RFC 8785 JCS bytes,
and orders by those bytes, as [QSL FR-101](ix://agent-ix/quire-spec-language/FR-101)
requires of successor order. QSL ADR-018 LA-5 and ADR-011 X-11 place the type
here so that QSL's evaluator and the contract-lowering stage share one
definition without a dependency edge to a QSL layer crate.

The type holds exactly `operation` and `arguments`. It carries no step
probability, reward or other field; those belong to later statistical work in
QSL and add to this type only by a later requirement. QSV interprets neither
the operation nor an argument: the declaration identity text is minted by QSL
([FR-060](./FR-060-the-crate-mints-no-identity.md)), and each argument arrives
already in QSL's typed canonical form. The receiver, when the operation has
one, is argument 0, as in [QSL FR-120](ix://agent-ix/quire-spec-language/FR-120).

This is the spec slice. The code lands in a later pull request against the
criteria below.

## Use case

QSL's `ModelSystem` applies `increment` to counter `c1` and builds a
`ModelTransition` from the operation's declaration identity text and the encoded
argument vector. The engine orders a state's successors by
`transition.canonical_bytes()`, and contract lowering reads the same value
without importing a QSL layer crate.

## Inputs

- `operation`: the operation's declaration identity text, a string that is
  itself one canonical JCS document.
- `arguments`: an ordered sequence of argument encodings, each one canonical
  JCS document (QSL's typed canonical form of one value), receiver first.
- For bytes: a `quire_canonical::Limits`.

## Outputs

- A `ModelTransition`, or a typed refusal.
- The canonical identity bytes.

## Public API inventory

All items are at `quire_semantic_value::model_transition`; the module is new and
no existing item changes.

```rust
pub struct ModelTransition { /* private: operation, arguments */ }

impl ModelTransition {
    pub fn new(
        operation: String,
        arguments: Vec<String>,
        limits: Limits,
    ) -> Result<Self, InvalidModelTransition>;
    pub fn operation(&self) -> &str;
    pub fn arguments(&self) -> &[String];
    pub fn canonical_bytes(&self, limits: Limits) -> Result<Vec<u8>, quire_canonical::Error>;
}

pub struct InvalidModelTransition { /* private */ }
impl InvalidModelTransition {
    pub fn site(&self) -> TransitionSite;
    pub fn cause(&self) -> TransitionCause;
}
pub enum TransitionSite { Operation, Argument(usize) }
pub enum TransitionCause { NotCanonicalJson, Limit, Allocation }

// Clone, Debug, Eq, PartialEq, Hash, serde::Serialize derived or implemented;
// Ord and PartialOrd implemented over canonical_bytes, not derived.
```

Names are the proposed ones and may change in review; the criteria bind the
behavior. The type has no `Deserialize`, like every other type of this crate.

## Behavior

1. `new` SHALL retain the operation text and the argument sequence verbatim and
   in the supplied order. It SHALL admit an empty argument sequence and any
   number of arguments, and SHALL NOT check arity, which is QSL's declaration.
2. `new` SHALL check the operation first, then each argument in order, and stop
   at the first failure. A text is valid when it is one JSON value that equals
   its own RFC 8785 re-encoding by `quire-canonical`. A failure SHALL carry the
   site, `Operation` or the zero-based `Argument` index, and a typed cause, and
   no value is built. The check SHALL stop at the supplied `Limits`, and a
   reservation failure SHALL be a typed `Allocation`, never an abort or panic.
3. `canonical_bytes` SHALL return the RFC 8785 UTF-8 bytes of the JSON object
   with members `type` equal to `"transition"`, `operation` equal to the
   operation text as a JSON string, and `arguments` equal to the array of the
   argument documents in order, each embedded as the JSON value it encodes.
   Members appear in RFC 8785 order (`arguments`, `operation`, `type`). The
   bytes are the one definition of transition identity.
4. Two values are equal exactly when their operation texts are equal and their
   argument sequences are equal element by element. Argument order and count are
   significant. Equality agrees with equality of `canonical_bytes`.
5. The order SHALL be the ascending unsigned-lexicographic order of
   `canonical_bytes` (QSL FR-101 successor order). It is a total order
   consistent with equality. It is not the field-wise order of operation then
   arguments, which differs from byte order wherever JSON escaping reorders two
   operation texts.
6. `Serialize` SHALL emit the same JSON object as `canonical_bytes` when
   encoded with `quire-canonical`.
7. The type SHALL use only `core` and `alloc` and the crate's existing
   dependencies. The change that adds it SHALL keep `make build-no-std` (target
   `thumbv7em-none-eabi`) green and SHALL NOT alter the crate's `[dependencies]`
   table, so QSL's dependency assertion over QSV's table
   ([QSL TC-390](ix://agent-ix/quire-spec-language/TC-390)) keeps passing.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-120-AC-1 | `ModelTransition::new` over a canonical operation text and arguments `true`, `[1,2]`, `{"a":1}` returns a value whose `operation()` and `arguments()` equal the supplied text and sequence in the supplied order; an empty argument sequence and a sequence of 64 arguments are both admitted, and no arity check refuses either. | Test (TC-920) |
| FR-120-AC-2 | `new` refuses an operation text that is not one canonical JCS document (non-JSON text, `{"b":1,"a":2}`, a document with insignificant whitespace, a text with a trailing second value) with site `Operation` and cause `NotCanonicalJson`, and builds no value. | Test (TC-920) |
| FR-120-AC-3 | `new` refuses an argument that is not one canonical JCS document with site `Argument(i)` for its zero-based index and cause `NotCanonicalJson`; with a bad operation and a bad argument it reports the operation, and with two bad arguments it reports the lower index. | Test (TC-920) |
| FR-120-AC-4 | For operation `{"digest_domain":"sha256-jcs","node":"test/counters/Counter/increment","package":"test/counters"}` and the single argument `true`, `canonical_bytes` equals exactly `{"arguments":[true],"operation":"{\"digest_domain\":\"sha256-jcs\",\"node\":\"test/counters/Counter/increment\",\"package\":\"test/counters\"}","type":"transition"}`; with no arguments the array is `[]`; with arguments `1` then `2` they appear in that order. | Test (TC-920) |
| FR-120-AC-5 | Values with the same operation and the same arguments are equal and have equal `canonical_bytes`; a different operation, a different argument, the same arguments in swapped order and a prefix of the arguments are each unequal with unequal bytes. | Test (TC-920) |
| FR-120-AC-6 | Over operation texts whose nodes are `x` and `x/y` in one package, the order of the two values equals the order of their `canonical_bytes`, the opposite of the order of the two operation texts compared as raw strings; over a set of at least 20 values the sorted order equals the order of sorted bytes, and `cmp` is antisymmetric and agrees with `==`. | Test (TC-920) |
| FR-120-AC-7 | `Serialize` encoded with `quire-canonical` yields bytes equal to `canonical_bytes` for the AC-4 values. | Test (TC-920) |
| FR-120-AC-8 | `new` and `canonical_bytes` under a `Limits` smaller than the input or output return a typed refusal naming the limit with site `Operation` or `Argument(i)` as applicable, and a denied reservation returns `Allocation`; none panics or aborts. | Test (TC-920) |
| FR-120-AC-9 | The public fields of `ModelTransition` and its inventory hold exactly `operation` and `arguments`: no probability, reward, weight, receiver or other member exists, and `ModelTransition` is the only new public transition type. | Inspection (public API of `model_transition`) |
| FR-120-AC-10 | With `ModelTransition` present, `make build-no-std` (`cargo build --locked --target thumbv7em-none-eabi`) and `cargo clippy --lib --target thumbv7em-none-eabi -- -D warnings` succeed, and the module names no `std` path. | Analysis (`make build-no-std`, `make lint`) |
| FR-120-AC-11 | The `[dependencies]` table of `Cargo.toml` is unchanged by the change that adds `ModelTransition`: exactly `quire-exact`, `quire-canonical` (default features off), `serde` and `thiserror`, each with its prior source and features; `[dev-dependencies]` and the `Cargo.lock` package set are unchanged, and `cargo build --locked` succeeds. | Inspection (`git diff` of `Cargo.toml` and `Cargo.lock`) |

## Status

PLANNED: FR-120-AC-1 through FR-120-AC-11. Spec only; the code and TC-920 tests
follow in a later pull request.

Open design points, pending the QSL lane (ticket IR-830): the receiver form of
the identity, the argument element representation, the validation depth, the
member order of the example bytes and the decode path. Behavior 1 to 3 and
AC-1 to AC-8 are written to the recommended answer of each.

## Dependencies

- [FR-060](./FR-060-the-crate-mints-no-identity.md): the operation text is
  supplied, not minted.
- [FR-108](./FR-108-report-registry-allocation-failure.md): reservation failure
  is a typed refusal.
- [QSL FR-101](ix://agent-ix/quire-spec-language/FR-101) and
  [QSL FR-120](ix://agent-ix/quire-spec-language/FR-120) consume the type;
  QSL ADR-018 LA-5, ADR-011 X-11 and ADR-016 ID-9 place and describe it.
- Blocks QSL-493, which bumps its QSV dependency after the code lands.
