---
id: SR-8688
title: "Code review of quire-semantic-value PR #16: nested conformance admission"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@abb629f5342f6ca4c695acf74f525e2bdc4adddf; src/declaration.rs (TypeEnvironment::admits, admitted, evaluate_record, evaluate_tuple, operand_value, CheckedEquality::evaluate, nested_conformance_tests)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: reviews
---
# Code review of quire-semantic-value PR #16

## Summary

Ticket: QSL-737. Code review with the Rust lane (rust-review) folded in, over
the one changed file `src/declaration.rs` against base `020d52d`.
`Cargo.toml` and `Cargo.lock` are untouched. Gates were not run: the brief
is reviewer-only with no builds.

## Verdict

PASS with three low findings. The implementation matches FR-113 items 8 to 14.

- **Walk.** The explicit `Vec` worklist pops each value occurrence once. It
  stops at a composite, uses no recursion and charges no work. Each `Option`
  or collection node also compares its recorded type with `==`, so the cost
  is O(occurrences x declared type size). That is linear in the value for a
  fixed declaration, which item 14 accepts. The walk terminates because
  values are finite trees.
- **No covariance.** The `Option` and collection arms first call the kernel
  `ValueType::admits`. At quire-exact `a58033d` (`src/value.rs:249-255`)
  that compares the recorded payload or collection type exactly. So
  `Sequence<C>` at `Sequence<P>`, `Option<C>` at `Option<P>`, and `Set` at
  `Sequence` are refused before any element is pushed.
- **Identical keys.** `composite_conforms` and `conforms` test `sub == sup`
  first, so an identical key conforms even when the environment does not
  admit it.
- **Refusals.** `evaluate_record` and `evaluate_tuple` still map a failed
  `admits` to `DeferredResultNotAdmitted` through `admitted`. Construction
  still refuses with `ConstructionCause::TypeMismatch`, naming the field
  (`fill_slots`) or the position (`tuple`).
- **`CheckedEquality::evaluate` signature.** The change is accepted. `run` is
  the only caller of `operand_value` for a checked equality. The alternative
  was to capture conformance at checking time, as `enum_members` does. That
  would retain each operand key's descendant set per equality node, which is
  the retention the `enum_members` doc rules out. QSL calls `operand_value`
  directly and does not call `evaluate`. The missing spec sentence is
  recorded in the gap-analysis review, SR-8689.

**Test oracle strength.** Each test was checked by reasoning about the revert
of the behaviour it covers:

- `evaluate_record_and_tuple_admit_members_by_conformance`: reverting
  `admitted` to the kernel `value_type.admits` refuses the `C` member at `P`.
  Kernel composite admission is exact (`src/value.rs:250-252`). The
  `let Outcome::Completed .. else panic!` then fires, so the test goes red
  for both the record and the tuple.
- `operand_value_checks_composite_operands_by_conformance`: reverting to
  `source.admits` refuses `C` at `P`, so the `.expect` panics. The
  `Option<P>`-holding-`U` case is admitted on revert, so its `matches!(Err)`
  fails. Red.
- `operand_value_checks_reference_operand_by_kind_only`: passes before and
  after the change. That is right for AC-5's "still" clause. It kills the
  mutation that routes `Reference<T>` through `types.admits`: object type 62
  at declared 61 has no ancestry, so it would be refused.
- `construction_refuses_nonconforming_member_naming_the_component`: the
  `Option<P>` field holding `U` is admitted on revert, so the test goes red.
  The top-level `U` record and tuple cases were already refused before this
  PR. They are non-discriminating but harmless.
- Container tests:
  - Reverting the composite arm turns the positive cases red.
  - Not pushing payloads or elements turns the nested-`U` cases red.
  - Removing the container `declared.admits(value)` check turns the
    covariance and collection-kind cases red.
  - A one-level walk turns the `Option<Sequence<Option<P>>>` mixed case red.

The coder's note that the tests never went red describes their process, not
the tests. On these mutations, every oracle is discriminating.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The key assertions in the `admits` container test are tautological. `TypeEnvironment::admits(&self, &ValueType, &Value) -> bool` cannot change the value, so `assert_eq!(key_of(inner.payload()), fx.c)` and the `keys == [m, c]` check only read the fixture back. No source change can make them fail. "Keeps its key" is meaningful only for the `operand_value` and `evaluate_*` results, which assert it. Delete these assertions. | src/declaration.rs:5160-5163; src/declaration.rs:5173-5177 |
| FND-002 | low | One fact, two places. `evaluation_environment` re-declares the `P`/`M`/`C`/`U` chain verbatim from `fixture()`, about 40 lines, so a change to the chain must be made twice. `units_scope()` and `Fixture::composite_type` are one-line wrappers around `UnitTable::default()` and `ValueType::Composite`. Build the evaluation environment from the fixture's declaration list plus the two holders, and inline the wrappers. | src/declaration.rs:5059-5083; src/declaration.rs:5274-5340; src/declaration.rs:5111-5113; src/declaration.rs:5467-5469 |
| FND-003 | low | The new `types` parameter of `operand_value` and `CheckedEquality::evaluate` is undocumented. Nothing says it must be the environment that checked the equality. Scenario: a caller passes `TypeEnvironment::default()`, as every pre-existing test now does, to a checked composite equality. A conforming `C` operand at `P`, or an `Option<Reference<T>>` holding a subtype reference, is then refused with `EqualityOperandSourceNotAdmitted`, a checked-invariant cause that should be unreachable. Add one doc line on each function. | src/declaration.rs:3681-3695; src/declaration.rs:4187-4195 |
