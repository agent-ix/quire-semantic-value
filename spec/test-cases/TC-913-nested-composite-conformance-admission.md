---
id: TC-913
title: "Nested composite conformance admission"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: verifies
---
# TC-913: Nested composite conformance admission

## Description

Verify that the QSV type environment and equality operand check admit a
composite value by conformance at every nested depth, without covariance over
type constructors, and refuse nonconforming values with typed causes.

## Test Procedure

1. Declare parent `P`, intermediate `M` (supertype `P`), child `C` (supertype
   `M`) and unrelated `U`, each a record with distinct keys. Build one value of
   each. Query `admits` for `C` at `Option<P>`, `Option<M>`, and `Sequence`,
   `Set`, `Bag` and `OrderedSet` of `P`; for a tuple position and a record
   field typed `P`; and for `Option<Sequence<Option<P>>>` with `C` innermost.
2. Query the negative set: `P` at `C`-typed and `M`-typed versions of those
   slots, `U` at `P`-typed versions, an `Option<C>` value and a `Sequence<C>`
   value at `Option<P>` and `Sequence<P>`, and a nested value with one
   nonconforming element among conforming ones. Build a `Set` of `P` holding a
   `C` and a `P` of equal shared fields.
3. Construct a record and a tuple with a conforming child member and with an
   unrelated member.
4. Call `operand_value` with source type `P` and values `C`, a grandchild,
   `Option<P>` and `Sequence<P>` holding a `C`, then with `U` and with a `P`
   value at source type `C`. Call it with a `Reference<T>` operand.

## Expected Results

1. Every positive query is admitted and the admitted values keep key `C`.
2. Every negative query returns `false`. The `Set` holds both elements.
3. The conforming construction succeeds with the child kept. The unrelated
   member is refused with `TypeMismatch` naming the field or position.
4. Conforming operands are returned unchanged with their own keys. `U` and the
   reverse case return `EqualityOperandSourceNotAdmitted`. The `Reference<T>`
   operand is still checked by kind only.
