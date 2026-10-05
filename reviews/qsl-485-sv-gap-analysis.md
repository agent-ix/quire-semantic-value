---
id: SR-1335
title: "Gap analysis of quire-semantic-value PR #4: FR-082-AC-6 edge count and FR-259 identity default against tests"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value@b7d0a2450fb066a592e181ef1d017df8ff464cdb; src/declaration.rs (TypeEnvironment::check_ancestor_steps, ancestor_steps_tests), src/semantic_node.rs (IDENTITY_LIMITS, tests); spec: agent-ix/quire-spec-language FR-082 (Behavior 'Ancestor and conformance walks are bounded', AC-6), NFR-012, FR-259 Behavior 3 and 4"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-spec-language/FR-082
    type: reviews
  - target: ix://agent-ix/quire-spec-language/FR-259
    type: reviews
---
# Gap analysis of quire-semantic-value PR #4

## Summary

Ticket: QSL-485 (B4). Manual check of acceptance criteria against tests (no plan
bundle in this repo).

Examined:

- FR-082-AC-6 (the qsv side): type-environment admission is never less strict
  than the model's walk, and a chain one edge past `ancestor_steps` refuses at
  check time with an edge-count limit. Covered by
  `a_chain_of_n_edges_admits_at_n_and_stops_at_n_minus_one`, which tells an
  edge count apart from the old node count, and by the diamond test, which
  tells an edge count apart from a chain depth.
- NFR-012 / FR-082: the published default is 16777216 edges. Covered by
  `the_default_ceiling_is_sixteen_million_edges`.
- FR-259 Behavior 3: `IDENTITY_LIMITS` is the 16777216-byte default.
  Covered by `the_identity_limits_default_is_sixteen_mebibytes`.
- FR-259 Behavior 4: a byte error is reported as the stage's input-bytes
  limit. Not covered; see FND-001.

## Verdict

Changes requested. The edge-count criterion is backed by a test that
discriminates. The new finite identity byte limit has no test of what
happens when it is reached (FND-001). The new tests carry no trace tags, which
this crate's own tests use (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Nothing tests what happens when `IDENTITY_LIMITS` is reached. Before this PR the limit could not be reached, so the behaviour is new: a compound unit or node preimage over 16 MiB. FR-259 Behavior 4 requires an input-bytes limit outcome and no malformed-value refusal. Today `compound_unit_id` panics and QSL's `preimage_digest` refuses `NonCanonicalPreimage` (SR-1334 FND-001 and FND-002). Fix: add a test that encodes a compound unit over the byte limit and asserts the limit outcome, which needs the error path SR-1334 FND-001 asks for. | src/semantic_node.rs:104-111,155-163; src/unit.rs:670-675 |
| FND-002 | low | The three new `ancestor_steps_tests` and `the_identity_limits_default_is_sixteen_mebibytes` have no `#[trace(...)]`. Other tests in this crate carry one (for example `src/object_closure.rs:239`). Their doc comments name QSL FR-082-AC-6 and FR-259 B3 in prose only. Fix: tag them with `#[trace("TC-220", "FR-082-AC-6")]` and with the FR-259 test case. | src/declaration.rs:2706-2783; src/semantic_node.rs:158-162 |
