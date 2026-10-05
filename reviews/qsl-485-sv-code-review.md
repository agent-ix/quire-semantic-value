---
id: SR-1334
title: "Code review of quire-semantic-value PR #4: ancestor_steps as an edge count, IDENTITY_LIMITS at 16 MiB"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@b7d0a2450fb066a592e181ef1d017df8ff464cdb; git diff origin/main...HEAD (PR #4): src/declaration.rs, src/semantic_node.rs, src/unit.rs; context: agent-ix/quire-spec-language qsl-semantics/src/model/key.rs, qsl-semantics/src/value/semantic_node.rs, qsl-eval/src/simulation/key.rs"
review_set: subset
---
# Code review of quire-semantic-value PR #4

## Summary

Ticket: QSL-485 (B4). Branch task/485-edge-steps, head b7d0a24. The Rust lane
(rust-review) is folded into this file. No build was run; the coder's gate is
queued.

What the change does, checked against the code:

- `DEFAULT_ANCESTOR_STEPS` is 16777216.
- `TypeEnvironment::check_ancestor_steps` sums, for each object type, its own
  declared `supertypes` edges plus the declared edges of every distinct proper
  ancestor in `Ancestry`. That sum is the number of edges in the type's
  closure, and each edge is counted once, because each ancestor appears once
  in the sorted, deduplicated ancestor set. It is the most edges the model's
  walk from that type can follow, so check time stays the stricter side
  (FR-082-AC-6). The refusal's actual counter is limit + 1, as FR-082 states.
- `Ancestry::count` is deleted because nothing uses it any more.
- `IDENTITY_LIMITS` is now `Limits::new(16_777_216)`, which is the decided
  default of `identity.input_bytes` (FR-259 Behavior 3).

The edge counting is correct. Positions index `object_types` in key order, the
same order `edges` is built in, so `edges[position]` belongs to the right type.

## Verdict

Changes requested. The edge count is right, and the chain test discriminates
the old node count from the new edge count (it admits at 3 for 4 types).
However, a finite `IDENTITY_LIMITS` turns `compound_unit_id`'s encode
`panic!` into a reachable path (FND-001), and it does the same to
consumers in QSL (FND-002). The diamond test does not tell an edge count
apart from a node count (FND-003).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `compound_unit_id` still `panic!`s on any encoder error. The new comment says a compound-unit preimage is "far below" the 16 MiB ceiling, but nothing bounds it. Each term is about 100 bytes (a 64-hex id plus the exponent's decimal string), and `Integer` exponents have no size limit, so roughly 170k distinct unit terms, or one very large exponent, push the preimage past 16 MiB. Callers may raise the S1 and S3 limits with no ceiling, so this input is reachable, and it now panics where before it hashed. FR-259 Behavior 4 says a byte error must be reported as the stage's input-bytes limit. Fix: return the byte error, for example `Result<UnitId, _>` mapped by the caller to `identity.input_bytes`, or prove a bound on terms and exponent size upstream and cite it. | src/unit.rs:670-675; src/semantic_node.rs:111 |
| FND-002 | medium | The doc comment this PR deletes named a cross-repo hazard: QSL's `preimage_digest` reports every encoder error as `NonCanonicalPreimage`, so a finite bound turns a byte overflow into a malformed-value refusal (against FR-259 Behavior 4 and AC-2). That is still true in QSL at fc90c46a1. QSL's `model::key::sha256_and_len` (its doc says "LIMITS sets no byte ceiling") and qsl-eval's `plain_digest` both `panic!` on encoder errors. When QSL bumps its pin to this commit, a model identity preimage over 16 MiB panics, and a node preimage over 16 MiB is refused as non-canonical. The model's cumulative `hashed_bytes` default is 256 MiB, so that limit does not stop a single preimage at 16 MiB. Fix: in the QSL pin-bump change for B4, map the byte error at those sites to the `identity.input_bytes` limit, or pass the stage's own byte budget (FR-259 Behavior 3). | src/semantic_node.rs:104-111; qsl-semantics/src/model/key.rs:500-516; qsl-semantics/src/value/semantic_node.rs:205-215; qsl-eval/src/simulation/key.rs:83-87 |
| FND-003 | low | `ancestor_steps_counts_the_closure_edges_not_the_chain_depth` uses a diamond of 4 types and 4 edges. The old node count gives the same verdicts (admit at 4, stop at 3), so the test rules out a chain depth but not a node count. Its doc also says the refusal names "the edge count the walk would have reached", but `actual` is always limit + 1; it equals 4 here only by coincidence. Fix: add an edge `D -> A` (5 edges, 4 types), assert that 4 stops and 5 admits, and describe `actual` as limit + 1. | src/declaration.rs:2737-2760 |
