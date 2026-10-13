<!-- reviewer-dispositions repo=agent-ix/quire-semantic-value visibility=public quoin=0.28.3 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-7894 round=1 reviewed=5870800cce45b74a6c549f1da4c855f47d535257 model=claude-opus-5-5 run=db5d20f9-c6c4-4caf-8a3b-b20170c88808 -->

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5870800cc |
| FND-002 | deferred | The FR-359 references edge still does not resolve against QSL main. |
| FND-003 (new) | low | The frontmatter adds depends_on only for quire-exact FR-376. |

+++ [reviewer data]

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  fix_sha: 5870800cc
  after_excerpt: A `Set<P>` or `Bag<P>` holding a `C` and a `P` of equal shared fields has two elements
    and is admitted. This vector requires the quire-exact collection and equality changes of FR-376 to
    FR-378 and cannot pass before they land.
- fnd: FND-002
  outcome: deferred
  reason: The FR-359 references edge still does not resolve against QSL main. It resolves when impl/business-model-meanings
    merges to QSL main. FR-113 Dependencies now states that the document lives on that branch. quire validate
    passes, and naming a branch of a public repo leaks nothing.
new_findings:
- fnd: FND-003
  severity: low
  confidence: high
  method: spec-review/dependency
  check: other
  artifact_id: FR-113
  related_ids:
  - FR-113-AC-6
  path: spec/functional/FR-113-composite-value-conformance.md
  excerpt: 'id: FR-113 title: "Admit composite values under declared composite supertypes" type: FR relationships:
    - target: ix://agent-ix/quire-specification/FR-208 type: references - target: ix://agent-ix/quire-spec-language/FR-359
    type: references - target: ix://agent-ix/quire-exact/FR-376 type: depends_on'
  finding: The frontmatter adds depends_on only for quire-exact FR-376. The Dependencies section and AC-6
    name FR-376 to FR-378, and AC-6's Set/Bag vector needs FR-377 (canonical key order across declarations
    in form) and FR-378 (the caller-admitted form path). Add depends_on edges for FR-377 and FR-378.
```

+++
