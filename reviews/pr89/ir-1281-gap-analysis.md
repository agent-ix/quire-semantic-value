---
id: SR-20261102072328
title: gap-analysis review of IR-1281 PR 89
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-semantic-value; src/catalog.rs; tests/evaluation_vocabulary.rs;
  FR-126; TC-926
review_set: subset
---

## Summary

IR-1281; PR 89; QSV current main version 0.1.0. Only src/catalog.rs and tests/evaluation_vocabulary.rs change. Reviewed the merged FR-126 / TC-926 contract and all seven integration-test bodies. No applicable AssuranceProfile found.

## Verdict

PASS for the authorized changed-code scope. No findings.

The ten variants retain the old relative order and match FR-126 exactly. Stable as_str/Display labels, ordinary and trace codes match all ten rows. Inconclusive trace is 22, Pending 0, Indeterminate 11. Explicit severity uses 30 > 20 > 21 > 22 > 10 > 11 > 0; min_by_key retains first equal minimum, preserving unknown priority/ties and empty None. Pair tests assert concrete winning codes in both directions, including 10 over 11 and 20 over 21/22; they do not reproduce the production ranking algorithm. Unknown values 1, 12 and 255 exercise the same unknown-code contract TC-926 illustrates with 99/98. Repeated 0/10/22 cover shared-code pairs. No extra category precedence is representable in the code-only fold.

Read-only consumer inspection: QSL qsl-replay/src/outcome.rs category_str serializes Category::as_str, and OutcomeDocument uses as_str for its category field. The stable labels are therefore serialized labels; adding serde to Category is unnecessary. Exhaustive consumer matches and the old eight-value diagnostic expectation require separate QSL adoption; this review does not claim that lane is done. No QSL shim, duplicate owner, dependency/lockfile, workflow or gate change appears in the diff.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined scope

```yaml
scope:
- id: FR-126-AC-1
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`Category::ALL` is exactly `[Success, Violation, Undefined, Refusal, Unsupported,
    Incomplete, Pending, Indeterminate, Inconclusive, InternalFailure]`, in that order;
    `as_str` and `Display` return the spellings in the Behavior table; `exit_code`
    and `trace_exit_code` both return 0, 10, 10, 20, 21, 22, 0, 11, 22, 30 respectively.
    `most_severe` follows `30 > 20 > 21 > 22 > 10 > 11 > 0`, ranks unknown codes above
    known codes with first-input ties, and returns `None` for empty input; shared-code
    categories have no additional precedence.'
- id: FR-126-AC-2
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: Two `CatalogCode`s with equal code and cause are equal and hash equal whatever
    their prefix; `Display` renders `prefix/code/cause` with a prefix and `code/cause`
    without.
- id: FR-126-AC-3
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`InternalFault::new(stage, invariant)` reports the runtime-invariant code,
    category `InternalFailure`, its stage and invariant, and the named values it was
    given.'
- id: FR-126-AC-4
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: An `UndefinedCoded` implementation without a prefix renders the reason
    spelling; with a prefix it renders `prefix/reason`.
- id: FR-126-AC-5
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`SuppliedLimitKind::field` and `SuppliedPhase::field` return the field
    names above and `work_units`, `value_occurrences` and `scale_expansion` for the
    three kernel counters; a `SuppliedStop` converts to `SuppliedFailure::Stop`, and
    `SuppliedStop::limit` is `Some` exactly for a `Limit` cause.'
bindings:
- test_id: category_vocabulary_has_stable_order_labels_and_exit_codes
  ac_id: FR-126-AC-1
  trace: correct
- test_id: category_severity_uses_the_explicit_order_in_both_input_directions
  ac_id: FR-126-AC-1
  trace: correct
- test_id: category_severity_keeps_unknown_codes_and_first_input_ties
  ac_id: FR-126-AC-1
  trace: correct
- test_id: catalog_code_identity_ignores_the_rendering_prefix
  ac_id: FR-126-AC-2
  trace: correct
- test_id: internal_fault_is_one_code_and_one_category
  ac_id: FR-126-AC-3
  trace: correct
- test_id: undefined_cause_renders_its_prefix_then_the_reason
  ac_id: FR-126-AC-4
  trace: correct
- test_id: supplied_stop_names_its_field_and_wraps_as_a_stop
  ac_id: FR-126-AC-5
  trace: correct
```

## Coverage

Existing focused logs were inspected: fmt and scoped library/integration Clippy exit 0; evaluation_vocabulary seven passed. Gates were not repeated, per the authorized scoped qualification. No broad suite, proof, consumer qualification or aggregate gate is claimed.

Reconciliation: quire matrix --scope . --format json; static trace axis only, no run-evidence store read. FR-126 AC-1 through AC-5 are tagged, none untagged or ignored-only; AC-1 has three real public-API tests. Changed behavior inventory: ALL, as_str/Display, exit_code, trace_exit_code and most_severe; all owned by FR-126-AC-1. No changed-source/test stubs, uncovered changed behavior or coverage inflation found. Scope follows the user's changed-code-only instruction; no repository-wide gap verdict is asserted. Semantic alignment was included as explicitly requested in this dispatch.

Plan completion: not assessed
