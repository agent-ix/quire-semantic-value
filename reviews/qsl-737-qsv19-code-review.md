---
id: SR-9023
title: "Code review of quire-semantic-value PR #19: union payload arm in object closure"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@e2627f96f9f76250b56a13d75ed9b61c916bcf39; src/object_closure.rs, Cargo.lock, src/declaration.rs (TypeEnvironment::admits), spec/functional/FR-106-object-closure-admission.md, spec/test-cases/TC-904-object-closure-refuses-duplicates-unknown-types-and-ambiguous-find.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-106
    type: reviews
---
# Code review of quire-semantic-value PR #19

## Summary

Ticket: QSL-737. The PR adds `Value::Union(union) => pending.extend(union.payload())`
to `ObjectClosure::check_closed`, locks quire-exact main at d41a1da3, and adds the
test `union_payload_references_are_closed_over`. The code-review and rust-review
lanes ran together. The arm is correct. The catch-all arm in
`TypeEnvironment::admits` now absorbs `Value::Union` without checking it, and the
new test is traced to an acceptance criterion it does not assert.

## Verdict

**CONDITIONAL**: one medium finding and one low finding.

What holds:

- **Arm semantics.** In quire-exact d41a1da3, `UnionValue::payload()` returns
  `&[Value]`, the positional payload. It holds plain `Value`s, the same kind as an
  `Option` payload or a collection's elements. Composite slots are `FieldValue`s,
  so they are filtered by `present`. Pushing the payload onto the worklist is the
  right closure walk, and nested unions are reached through it. QSV PR #12
  (`task/qsl-500-sv-union` at 41da06d) has the same arm with a charge added.
- **Test oracle.** The nested-dangling case is the only discriminating case. If
  the arm ignored the payload (`Value::Union(_) => {}`), it would return `Ok(())`
  and the assertion would fail. The closed case and the tolerated case pass
  whether or not the payload is walked. Calling the private `check_closed`
  directly is acceptable here, because building the case through
  `ObjectClosure::new` would depend on the defect in FND-001.
- **Lock scope.** The only `Cargo.lock` change is the quire-exact `source` line
  (4c27c08e to d41a1da3). No other package and no dependency list changed.
- **Gates.** The logs come from worktree `qsv-union-arms` at e2627f9, with a clean
  tree. fmt-check, clippy `--all-targets --all-features -D warnings`,
  `cargo test --all-features` (53 passed, 0 failed, 0 ignored), no_std lint and
  build, `cargo doc -D warnings` and `cargo deny check` all exit 0. The baseline
  E0004 is recorded in the check log. The reviewer did not re-run them.
- **Rust idioms.** The match stays exhaustive and has no catch-all. The new
  `#[allow(clippy::disallowed_methods)]` carries a reason. There is no new panic
  surface in library code and no `unsafe`.
- **Public-repo hygiene.** The diff and the PR text contain no local paths and
  no private text.
- **Duplication and vendoring.** None found.

The PR text claims that `ValueType::admits` rejects a union value, so the arm
cannot be reached through `ObjectClosure::new`. That claim is not accurate. See
FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The catch-all `_` arm of `TypeEnvironment::admits` now silently absorbs `Value::Union`. It sends `(ValueType::Composite(k), Value::Union(u))` to the kernel's `ValueType::admits`, which in quire-exact d41a1da3 accepts on `u.declaration() == k` alone (value.rs:434-436). QSV registers only record and tuple composites, so a record-typed or tuple-typed slot admits a union value carrying that record's key, and the union payload is never checked. This skips the `Reference<T>` conformance check and the position check that FR-113 item 8 requires. Example: an object type has a field typed `Composite(K)` for a record `K`. The attribute is `UnionValue::from_admitted(UnionMember::from_admitted(K, m, id), vec![Value::Reference(<a reference to an unrelated type>)])`. `fill_slots`, and so `ObjectClosure::new`, admits that non-record value. The same path also reaches `check_closed`, which the PR says it cannot. The trigger requires breaking the `from_admitted` contract, which is why this is medium and not high. QSV #12 replaces this arm with a member and position check. Until then, an explicit `Value::Union` arm that refuses would close it. | src/declaration.rs:1772-1776 |
| FND-002 | low | The new test is tagged `#[trace("TC-904", "FR-106-AC-10")]`, but FR-106-AC-10 and TC-904 rows 1 to 4 cover only a duplicate identity triple, an unknown object type and an ambiguous `find`. The test asserts dangling-reference refusal and the `tolerated` set, which neither of them states. The computed matrix counts it as evidence for AC-10, which inflates that criterion's coverage. | src/object_closure.rs:360 |

## New findings (disposition pass 1)

Reviewed at agent-ix/quire-semantic-value@7616456f7379e25a9a14e515b7dfe2df47dc88b2.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | `union_value_is_refused_at_record_and_tuple_slots` is tagged `#[trace("TC-913", "FR-113-AC-4")]`. FR-113-AC-4 states nested `Option`/`Sequence` admission by conformance, `evaluate_record`/`evaluate_tuple`, and construction refusal for a nonconforming composite `U`. It says nothing about a union value. No FR-113 criterion or TC-913 row states that a union value is refused at a record or tuple slot, so the computed matrix counts this test as AC-4 evidence for behaviour AC-4 does not state. | src/declaration.rs:3992 |
| FND-004 | low | The record and tuple construction assertions in the same test cannot detect the defect they claim to cover. The union sits in a `String` field and an `Integer` position, so `TypeMismatch` comes from the kernel's exact leaf `admits` whether or not the new `(Composite(_), Union(_))` arm exists. Only the direct and `Option`-wrapped `admits` assertions turn red without the arm; `qsv19-red.log` shows the first of them failing. To exercise the arm, the construction case needs a member typed `Composite(K)` holding a union carrying `K`. | src/declaration.rs:4040-4050 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 7616456 | `(ValueType::Composite(_), Value::Union(_)) => return false` added before the catch-all in `TypeEnvironment::admits`. The direct and `Option`-wrapped record and tuple cases go red with the arm removed (`qsv19-red.log`, exit 101 at the first assertion). |
| FND-002 | fixed 7616456 | `union_payload_references_are_closed_over` is retagged `#[trace("TC-904", "FR-106-AC-11")]`, and `quire matrix` binds it to FR-106-AC-11. FR-106-AC-11 and TC-904 rows 5 to 7 state its cases. |
