---
id: SR-20261102072325
title: spec-review/ears-conformance review of IR-1281 PR 88
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-semantic-value; spec/functional/FR-126-own-the-evaluation-result-vocabulary.md;
  spec/test-cases/TC-926-catalog-vocabulary-and-supplied-stop-records.md
review_set: subset
---

## Summary

Ticket: IR-1281. The six edited/new SHALL statements name Category or the code fold and concrete observable responses. Installed ISO validation reports both changed documents grammar-clean with zero findings. The enum table supplies finite outputs; no trigger ambiguity or vague response occurs.

## Verdict

PASS for the specification stage only. Implementation, production tests, QSL adoption and qualification are not claimed. No applicable active AssuranceProfile was found in this repository.

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
- id: FR-126-AC-6
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`render_identity(b"c9")` is `c9`; `render_identity(&[0xFF, 0xFE])` is
    `identity bytes 0xfffe (not UTF-8)`; `render_identity(&[0xFF])` and `render_identity(&[0xFE])`
    differ.'
- id: FR-126-AC-7
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: An `UndefinedCoded` implementation whose prefix is `STATE_MODEL_CATALOG_PREFIX`
    renders a `catalog_code` that begins with `state-model/`.
- id: FR-126-AC-8
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`SuppliedLimits::default()` reports 16777216, 16777216 and 4096 for A,
    H and B; each of `with_admission_work_units`, `with_conversion_work_units` and
    `with_numeric_magnitude_bits` changes exactly its own field and leaves the other
    two unchanged.'
- id: FR-126-AC-9
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: A `SuppliedRun` maps an allocator-denied bounded-admission error to a `Storage`
    cause and an unrepresentable size to a `Capacity` cause, each with its own unit,
    no limit and zero consumed.
- id: FR-126/Description
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: '`quire-semantic-value` SHALL own the native-diagnostics vocabulary, the

    supplied-value stop records that an evaluation result names, and the

    supplied-value run (`SuppliedRun`) that produces them, so the shared

    evaluation interface (`quire-eval`, QSL FR-401) carries them without depending

    on a compiler crate. QSL FR-401''s placement table assigns the vocabulary and

    records here, and QSL FR-403''s disposition table assigns the run and

    `render_identity` here.'
- id: FR-126/Outputs
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: "- `catalog`: `Category`, `CatalogCode`, `CatalogCoded`, `UndefinedCoded`,\n\
    \  `UndefinedRecord`, `UndefinedReason` and `InternalFault`; the function\n  `render_identity`;\
    \ and the constant `STATE_MODEL_CATALOG_PREFIX`.\n- `supplied_stop`: `SuppliedFailure`,\
    \ `SuppliedStop`, `SuppliedContext`,\n  `SuppliedCause`, `SuppliedPhase`, `SuppliedEvent`,\
    \ `SuppliedLocus`,\n  `SuppliedLimit`, `SuppliedLimitKind`, `AttemptedCount`,\n\
    \  `SuppliedNumericOperand`, `SuppliedStorage`, `SuppliedStorageUnit`,\n  `UnsupportedHelperStop`,\
    \ and the limit-field names\n  `ADMISSION_WORK_UNITS`, `CONVERSION_WORK_UNITS`\
    \ and `NUMERI"
- id: FR-126/Behavior
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: "- `Category` SHALL have the ten ADR-013 O-16 values in the following\n\
    \  `Category::ALL` order, with these stable `as_str` and `Display` spellings\n\
    \  and `exit_code` results:\n\n  | Category | Spelling | Exit code |\n  | ---\
    \ | --- | --- |\n  | `Success` | `success` | 0 |\n  | `Violation` | `violation`\
    \ | 10 |\n  | `Undefined` | `undefined` | 10 |\n  | `Refusal` | `refusal` | 20\
    \ |\n  | `Unsupported` | `unsupported` | 21 |\n  | `Incomplete` | `incomplete`\
    \ | 22 |\n  | `Pending` | `pending` | 0 |\n  | `Indeterminate` | `indeterminate`\
    \ | 11 |\n  | `Inconclusive` | `inconclusive` | 22 |\n  | `InternalFailure` |\
    \ `int"
- id: FR-126/Dependencies
  path: spec/functional/FR-126-own-the-evaluation-result-vocabulary.md
  role: examined
  excerpt: "- IR-1281 implements the ten-category contract above; this specification\n\
    \  update precedes its implementation and test changes. Consumer adoption in\n\
    \  QSL is separate work.\n\n- QSL FR-401 places these types; QSL FR-262 names\
    \ them in the interface."
- id: TC-926/Description
  path: spec/test-cases/TC-926-catalog-vocabulary-and-supplied-stop-records.md
  role: examined
  excerpt: 'Verify FR-126-AC-1 to AC-5 through the public `catalog` and `supplied_stop`

    API, as an integration test of the crate (`tests/evaluation_vocabulary.rs`):

    the order, spellings, exit codes and severity fold of `Category`, the identity

    and rendering of `CatalogCode`, `InternalFault`, the rendering of an

    `UndefinedCoded` cause, and the field names and conversions of the

    supplied-value stop records.


    The expanded category checks below are planned for IR-1281 implementation;

    this specification update does not claim the existing tests pass them.'
- id: TC-926/Test Procedure
  path: spec/test-cases/TC-926-catalog-vocabulary-and-supplied-stop-records.md
  role: examined
  excerpt: "1. Read `Category::ALL`; map every value through `as_str`, `Display`,\n\
    \   `exit_code` and `trace_exit_code`. For `Category::most_severe`, fold no\n\
    \   codes; each pair of distinct known codes in both input orders; each\n   known\
    \ code with unknown code 99 in both orders; `[99, 98]` and\n   `[98, 99]`; and\
    \ repeated equal codes. Check that shared-code pairs\n   (`Success`/`Pending`,\
    \ `Violation`/`Undefined`,\n   `Incomplete`/`Inconclusive`) yield only their shared\
    \ code in either\n   order (FR-126-AC-1).\n2. Build `CatalogCode::new(\"ill_typed\"\
    , \"x\")` and the same code with prefix\n   `value`; compare them fo"
- id: TC-926/Expected Results
  path: spec/test-cases/TC-926-catalog-vocabulary-and-supplied-stop-records.md
  role: examined
  excerpt: "- Step 1: `Category::ALL` is `[Success, Violation, Undefined, Refusal,\n\
    \  Unsupported, Incomplete, Pending, Indeterminate, Inconclusive,\n  InternalFailure]`\
    \ in that order. Both string renderings are respectively\n  `success`, `violation`,\
    \ `undefined`, `refusal`, `unsupported`,\n  `incomplete`, `pending`, `indeterminate`,\
    \ `inconclusive` and\n  `internal-failure`. Both exit methods return respectively\n\
    \  0, 10, 10, 20, 21, 22, 0, 11, 22 and 30. The severity fold returns the\n  earlier\
    \ code in `30 > 20 > 21 > 22 > 10 > 11 > 0` for every known pair,\n  regardless\
    \ of input order, and returns 99 against ev"
- id: FR-285
  path: QSL main spec/functional/FR-285-map-every-outcome-category-to-one-exit-code.md
  role: context_only
  excerpt: When an outcome holds several items, the frontend shall exit with the most
    severe item code under the order 30, 20, 21, 22, 10, 11, 0.
- id: ADR-013/O-16
  path: QSL main spec/decisions/ADR-013-canonical-type-package-conversion-ownership.md
  role: context_only
  excerpt: has the ten values in the first column
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
