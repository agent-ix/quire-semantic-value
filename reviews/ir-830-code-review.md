---
id: SR-12010
title: "IR-830 code review (rust lane) of QSV PR #41"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@a43b13fc941d12ca397fe10717e92cf759fd5ee7; src/model_transition.rs, src/lib.rs, tests/model_transition.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-120
    type: references
  - target: ix://agent-ix/quire-semantic-value/TC-920
    type: references
---

# IR-830 code review (rust lane) of QSV PR #41

## Summary

Ticket: IR-830, PR quire-semantic-value#41 at a43b13fc941d12ca397fe10717e92cf759fd5ee7. `/rust-review` lane folded into this file. Diff `origin/main...HEAD`: new `src/model_transition.rs` (328 lines), `pub mod model_transition;` in `src/lib.rs`, `tests/model_transition.rs` (12 tests) and the FR-120/TC-920 status rows. Gates run at head with one target dir: `make fmt-check`, `make lint` (host and thumbv7em clippy, -D warnings), `make test` (all suites; 12/12 model_transition), `make build-no-std`, `make docs`: all green. Cargo.toml and Cargo.lock untouched. 29 hand mutants were run in a throwaway worktree (since removed); results are in the gap-analysis artifact SR-12011.

## Reviewed scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| src/model_transition.rs | examined | ModelTransition::new / check / embed / Encode / identity_bytes / Ord / Escaped |
| src/lib.rs | examined | pub mod model_transition; |
| tests/model_transition.rs | examined | 12 #[trace("TC-920", ...)] tests for FR-120-AC-1..8, 12, 13, 15, 16 |
| FR-120-B6 | examined | 6. `Encode` SHALL write the same bytes as `canonical_bytes` under the same limits, by pushing events from an explicit stack. It SHALL read each stored text again inside `encode_into` and embed the resulting document through its own non-recursive `quire-canonical` encoding, and SHALL map a `ReadEr... |
| FR-120-AC-14 | examined | Inspection of `new` and `canonical_bytes` confirms every `quire_canonical` allocation refusal (read tree in `new`, the re-read inside `encode_into`, writer buffer) is mapped to `Allocation{requested}`, `IdentityRefusal::Allocation` or `quire_canonical::Error::Allocation` respectively, and neither... |
| FR-120-AC-9 | examined | The private fields of `ModelTransition` and its public accessors hold exactly `operation`, `receiver` and `arguments`: no probability, reward, weight or other member exists, and `ModelTransition` is the only new public value type of the module besides the refusal types of the inventory. |
| FR-120-AC-10 | examined | With `ModelTransition` present, `make build-no-std` (`cargo build --locked --target thumbv7em-none-eabi`) and `cargo clippy --lib --target thumbv7em-none-eabi -- -D warnings` succeed, and the module names no `std` path. |
| FR-120-AC-11 | examined | The `[dependencies]` table of `Cargo.toml` is unchanged by the change that adds `ModelTransition`: exactly `quire-exact`, `quire-canonical` (default features off), `serde` and `thiserror`, each with its prior source and features; `[dev-dependencies]` and the `Cargo.lock` package set are unchanged... |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `check` maps both wildcard arms to a value. On `read`, `_` stands for the two variants that exist today (`Malformed`, `NumberOutOfRange`) and they are not named. On `to_vec`, `_` sends `Error::Protocol` and `Error::Internal` to `NotCanonicalJson`. quire-canonical documents `Internal` as "a bug in this crate, never a property of the input", so a library fault is reported as the caller's bad text. Because both enums are `#[non_exhaustive]`, a new read or encode refusal, for example a new resource limit, would also be reported as `NotCanonicalJson` with no compile signal. Name `Malformed` and `NumberOutOfRange` explicitly, and keep `_` only for variants that do not exist yet, with a comment that says so. The `embed` wildcard to `Error::Internal` is honest: it can only fire if the private-field invariant from `new` fails. Keep it. | src/model_transition.rs:198-224 |

## Verdict

PASS with one low finding. Recorded clean: (1) Behavior 6 / AC-14. `Encode` writes the fixed outer shape (object, `arguments` array, `operation` string, `receiver`, `type`) as events and embeds each stored text by `read(.., u64::MAX)` + `Document::encode_into`. quire-canonical's reader and its document encoder both use explicit heap stacks (read.rs, value.rs), so no step recurses per nesting level. A hand-written recursive `embed` aborts the AC-16 test with a stack overflow on the 2 MiB thread. The re-read maps `ReadError::Allocation` to `Error::Allocation`, `ReadError::Limit` to `Error::Limit` (unreachable at u64::MAX but mapped as Behavior 6 requires), and anything else to `Error::Internal`, which is the honest variant for a broken invariant. `new` maps the read-tree and re-encode reservations to `TransitionCause::Allocation`, and `canonical_bytes` uses `?` through `From<Error> for IdentityRefusal`. There is no unwrap, expect, slice indexing or abort on these paths. The only indexing is `HEX[..]` on a nibble, which is in `cmp`, not in `new` or `canonical_bytes`, and cannot panic. `Escaped::next` uses `get`. (2) Ord. `identity_bytes` matches quire-canonical's escape.rs byte for byte (`"`, `\\`, short forms, lowercase `\u00xx`, literal U+007F and non-ASCII). Stored texts are embedded verbatim, which is correct because each one equals its own re-encoding. An absent receiver is `null`, and `Some("null")` is refused, so the byte string is injective and `cmp` agrees with the derived `Eq`. A throwaway probe compared `cmp` and `==` against `canonical_bytes` for all 180x180 pairs of 5 operations (including `\\`, `\"`, `\u001f`, `é`), 9 receivers (absent, false, true, 1, -1, [1], {"a":1}, strings) and 4 argument lists: all agree. The C0-control arms of `Escaped` are unreachable, because a canonical text holds no raw control byte, but they are correct. (3) Public API matches the FR-120 inventory: the four public items, `site()`/`cause()`, and `Clone, Debug, Eq, PartialEq` with Ord/PartialOrd written by hand. There is no Serialize/Deserialize and no std path. `new` checks the operation, then the receiver, then each argument, in index order, against a separate per-text limit; an exact-limit text is admitted and limit minus 1 is refused (probe).
