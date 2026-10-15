---
id: RV-7892
title: "Reviewer dispositions for SR-7892"
type: Review
---
<!-- reviewer-dispositions repo=agent-ix/quire-semantic-value visibility=public quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-7892 round=1 reviewed=5870800cce45b74a6c549f1da4c855f47d535257 model=claude-opus-5-5 run=db5d20f9-c6c4-4caf-8a3b-b20170c88808 -->

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | still-open | Item 9 now says the walk stops at a composite, but item 8 still lists 'tuple position' among what the walk checks, and a tuple is a composite value (kernel CompositeValue). |
| FND-003 | fixed | 5870800cc |
| FND-004 (new) | medium | Item 11 says record and tuple construction and evaluation refuse a nonconforming member with ConstructionCause::TypeMismatch. |
| FND-005 (new) | low | Item 12 names evaluate_record and evaluate_tuple unqualified. |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  fix_sha: 5870800cc
  after_excerpt: '`evaluate_record` and `evaluate_tuple` SHALL admit each completed member value through
    `TypeEnvironment::admits` and not through the exact kernel `ValueType::admits`, so a conforming child
    member does not stop with `DeferredResultNotAdmitted`. A nonconforming member still stops with that
    checked-invariant cause.'
- fnd: FND-002
  outcome: still-open
  reason: Item 9 now says the walk stops at a composite, but item 8 still lists 'tuple position' among
    what the walk checks, and a tuple is a composite value (kernel CompositeValue). AC-3 and TC-913 steps
    1-2 expect admits to refuse 'the same containers holding a U', and that includes a tuple and a record
    built with from_admitted_slots. Under item 9, admits(Composite(R), r) returns true for such a record,
    because it checks only the key. Drop tuple position from item 8's walk, and drop tuple/record from
    AC-3's admits negatives and TC-913 step 2. Their refusal belongs to construction and evaluation, which
    AC-4 already covers.
- fnd: FND-003
  outcome: fixed
  fix_sha: 5870800cc
  after_excerpt: 'A value whose composite key does not conform to the declared key, at any depth reached
    by the walk, SHALL NOT be admitted. Conformance (item 3) is reflexive on an identical key, so an identical
    key conforms even when the environment does not admit it, and an unequal key the environment does
    not admit never conforms. `admits` returns `false` and carries no cause; the typed refusal belongs
    to the caller: record and tuple construction and evaluation refuse the member with `ConstructionCause::TypeMismatch`
    naming the field or position, and `operand_value` returns `EqualityOperandSourceNotAdmitted`'
new_findings:
- fnd: FND-004
  severity: medium
  confidence: medium
  method: spec-review/integrity
  check: soundness
  artifact_id: FR-113-item-11
  related_ids:
  - FR-113-item-12
  - FR-113-AC-4
  - TC-913
  path: spec/functional/FR-113-composite-value-conformance.md
  excerpt: 'A value whose composite key does not conform to the declared key, at any depth reached by
    the walk, SHALL NOT be admitted. Conformance (item 3) is reflexive on an identical key, so an identical
    key conforms even when the environment does not admit it, and an unequal key the environment does
    not admit never conforms. `admits` returns `false` and carries no cause; the typed refusal belongs
    to the caller: record and tuple construction and evaluation refuse the member with `ConstructionCause::TypeMismatch`
    naming the field or position, and `operand_value` returns `EqualityOperandSourceNotAdmitted`'
  finding: Item 11 says record and tuple construction and evaluation refuse a nonconforming member with
    ConstructionCause::TypeMismatch. Item 12, AC-4 and TC-913 expected result 3 say evaluate_record/evaluate_tuple
    stop with the checked-invariant DeferredResultNotAdmitted. Make item 11 say construction refuses with
    TypeMismatch and evaluation stops with DeferredResultNotAdmitted (item 12).
- fnd: FND-005
  severity: low
  confidence: high
  method: spec-review/integrity
  check: ambiguous
  artifact_id: FR-113-item-12
  related_ids: []
  path: spec/functional/FR-113-composite-value-conformance.md
  excerpt: '`evaluate_record` and `evaluate_tuple` SHALL admit each completed member value through `TypeEnvironment::admits`
    and not through the exact kernel `ValueType::admits`, so a conforming child member does not stop with
    `DeferredResultNotAdmitted`. A nonconforming member still stops with that checked-invariant cause.'
  finding: 'Item 12 names evaluate_record and evaluate_tuple unqualified. quire-exact FR-378 item 4 and
    QSL FR-359 item 5 say the kernel''s evaluate_record/evaluate_tuple stay exact and refuse a descendant
    with DeferredResultNotAdmitted. Item 12 means QSV''s own TypeEnvironment::evaluate_record/evaluate_tuple,
    which sit above the kernel and build through from_admitted_slots (src/declaration.rs:2938-3004), so
    the two repos do not conflict in substance. Qualify the names as TypeEnvironment:: so a reader of
    the two specs does not see a contradiction.'
```

+++
