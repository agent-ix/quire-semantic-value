<!-- reviewer-dispositions repo=agent-ix/quire-semantic-value visibility=public quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-7891 round=1 reviewed=5870800cce45b74a6c549f1da4c855f47d535257 model=claude-opus-5-5 run=db5d20f9-c6c4-4caf-8a3b-b20170c88808 -->

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | fixed | 5870800cc |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  fix_sha: 5870800cc
  after_excerpt: 'With a chain parent `P`, intermediate `M`, child `C` (a grandchild of `P`) and an unrelated
    record `U`, built with the trusted constructors (`from_admitted`, `form`, `from_admitted_slots`) because
    the kernel''s checked constructors are exact, `admits` accepts an `Option<P>`, a `Sequence`, `Set`,
    `Bag` or `OrderedSet` of `P`, a tuple and a record holding a `C` or an `M`, each keeping its key;
    the top-level `C` at `P` and `M` already passes. It refuses the same containers holding a `U`, a `P`
    at a `C`-typed or `M`-typed version, and `Option<C>` or `Sequence<C>` at `Option<P>` or `Sequence<P>`
    as '
- fnd: FND-002
  outcome: fixed
  fix_sha: 5870800cc
  after_excerpt: '`operand_value` admits a composite operand of source type `P` holding a `C` or an `M`
    unchanged and with its own key, and an `Option<P>` or `Sequence<P>` operand holding one. It refuses
    a `P` operand holding `U`, and a `P` value at source type `C`, with `EqualityOperandSourceNotAdmitted`.
    A `Reference<T>` operand is still checked by kind only.'
```

+++
