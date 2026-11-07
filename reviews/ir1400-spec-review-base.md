---
id: SR-1794040450272172418983
title: IR-1400 PR98 spec-review-base review
type: SpecReview
analysis: base
scope: agent-ix/quire-semantic-value; FR-132, TC-929, src/declaration.rs, changed
  integration tests and allocator fixture
review_set: subset
---

## Summary

Ticket: IR-1400. Reviewed frozen a02c5f71ad6c5111b5bb883645be3c5b62e738be against origin/main, including the preimplementation contract a8671ae. No applicable AssuranceProfile exists.

## Verdict

**PASS** — No defects found within this method scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Public external traversal matches old type_refusal: sealed-handle resolution is preserved; IEEE traversal retains LIFO record/tuple/member ordering, maps/options, visited keys and cycle termination. Pending/visited growth reaches checked reserve_elements and try_reserve_exact; keys/types are borrowed, error construction has no allocating clone, and local vectors drop on all returns. GlobalAlloc fixture unsafe implementation and all four forwarding blocks were audited; System provenance/layouts are preserved and thread-local one-shot denial uses nonallocating Cells. Product unsafe prohibition is unchanged. Real allocator tests assert five measured layout sizes, typed native additional-element request, unchanged environment/type and retry. Capacity test exercises shared production boundary.

Computed matrix examined: FR-132 AC1–4 tagged; AC5 is Inspection/method-without-symbol, verified by direct inspection, so no executable binding is manufactured. Existing unrelated engine grammar warnings are outside this change. EARS engine flags Before/After triggers; its Neither-failure subject warning is a false positive with a defined antecedent. Optional gap semantic lens not invoked separately; code-review checks actual intent/test/code agreement. No separate integrity/object/dependency lenses: no domain-object, structural or dependency-edge changes requiring them. No enforced profile workflow applies.

## Examined scope

```yaml
scope:
- id: FR-132-AC-1
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: 'Public `try_check_type` returns success for admitted leaves, references,
    composites, sealed union handles, nested options, sequences and maps; unknown
    composites and non-object references return `Refused(IllTyped { cause: TypeMismatch
    })`, and populations and IEEE-bearing set/bag/ordered-set elements return `Refused(IllTyped
    { cause: OperatorIneligible })`, preserving the previous external-type traversal''s
    outcomes.'
- id: FR-132-AC-2
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: A dedicated test binary denies a real allocator request while public `try_check_type`
    runs, after fixture construction. The call returns `Allocation` with the failing
    reservation's measured request and unit, and the process completes without aborting.
    Real denials cover the root worklist and nested IEEE discovery's worklist/visited
    storage; policy-only injection is not a substitute.
- id: FR-132-AC-3
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: Checked reservation-size overflow returns `Capacity` with its request and
    native unit, distinct from `Allocation` and `Refused`; controlled tests exercise
    the same reservation boundary used by the public check.
- id: FR-132-AC-4
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: After each controlled storage denial, retrying the same type against the
    same environment after restoring allocation yields its original success or semantic
    refusal; declaration lookups and the supplied type are unchanged and no partial
    checking result is exposed.
- id: FR-132-AC-5
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: Inspection finds fallible checked reservations for every allocation reachable
    from public `try_check_type`, including nested IEEE traversal and error construction;
    in-repository callers consume its typed result and no public `check_type` compatibility
    shim remains.
- id: FR-132
  path: spec/functional/FR-132-check-external-types-with-fallible-storage.md
  role: examined
  excerpt: 'When a caller checks an external parameter or result `ValueType` against
    an

    admitted `TypeEnvironment`, QSV SHALL expose `try_check_type` with fallible

    storage throughout its traversal, including nested IEEE-content discovery.'
- id: TC-929
  path: spec/test-cases/TC-929-fallible-external-type-check.md
  role: examined
  excerpt: 'Verify [FR-132](../functional/FR-132-check-external-types-with-fallible-storage.md)

    through the public `TypeEnvironment::try_check_type` API, with explicit semantic

    oracles and actual allocator denial in a separate test binary.'
```

## Validation evidence

Reviewer independently ran all four changed integration binaries: 57 passed, including actual allocator denial; shared storage reservation unit tests: 12 passed. cargo fmt --all -- --check and cargo clippy --locked --all-targets --all-features -- -D warnings passed. Spec and explicit scratchpad SR validation passed. Full aggregate/no_std/deny/docs gates are lead-owned and were not run in this review. Linear read-back passed: YAML parses, content round-trips (terminal newline stripped by CLI), and finding IDs match. Existing IR has-review label reused.

## Disposition round 1 verification

Latest verdict: **PASS** at 451fd4218092b4a37a08a1697ff8fe5a58a32d6c. All prior findings are fixed; no new findings. Inspected the full fix delta: walk_type_refusal owns external/admission/equality semantic dispatch; borrowed UnknownDeclaration maps without cloning publicly and materializes via try_clone_node for admission. FallibleTypeStorage delegates every public/admission pending and visited reservation to checked reserve_elements; EqualityTypeStorage is selected only by the existing equality checker and uses native reserve_exact with Infallible exhaustion. No new panic adapter, semantic projection of storage failure, public API expansion or unsafe surface was introduced. contains_ieee_walk propagates the selected policy through pending and visited storage while preserving visit ordering, cycles and admission charge points.

FR-132 temporal triggers are canonical When clauses without weakening preallocation or cleanup obligations. Initial findings remain unmodified. Reviewer independently reran the three actual-allocator integration tests and twelve storage reservation tests; all passed. Coder evidence independently inspected: 57 integration and 94 declaration tests passed, one existing performance test ignored; formatting and all-target/all-feature Clippy passed. No aggregate gate run. Base/gap/failure-domain conclusions remain PASS; no new code/criterion/trace gaps introduced.
