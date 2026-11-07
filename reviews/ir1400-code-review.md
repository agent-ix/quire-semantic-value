---
id: SR-1794040450272172418981
title: IR-1400 PR98 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-semantic-value; FR-132, TC-929, src/declaration.rs, changed
  integration tests and allocator fixture
review_set: subset
---

## Summary

Ticket: IR-1400. Reviewed frozen a02c5f71ad6c5111b5bb883645be3c5b62e738be against origin/main, including the preimplementation contract a8671ae. No applicable AssuranceProfile exists.

## Verdict

**FAIL** — Findings below require correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | New try_check_type duplicates the live type_refusal semantic dispatch for named types, populations, options, collections and maps; consolidate these rules in one traversal/decision owner with caller-specific failure projection. Search found type_refusal at src/declaration.rs:3466 and its live caller at 5454, plus type_refusal_admission at 3617. | src/declaration.rs:3280 |

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
- id: TypeEnvironment::try_check_type
  path: src/declaration.rs
  role: examined
  excerpt: 'pub fn try_check_type(&self, value_type: &ValueType) -> Result<(), TypeCheckError>'
- id: src/declaration.rs
  path: src/declaration.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! The QSpec FR-143 declared record, tuple, union and model object-type registry,
    and

    //! the QSpec FR-149 checked equality layer over it.

    //!

    //! It owns the registry (`TypeEnvironment`, `ObjectTypeDecl'
- id: tests/fallible_type_check.rs
  path: tests/fallible_type_check.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! External checking under\
    \ real allocator denial in its own integration binary.\nmod common;\n\nuse ix_trace_rs::trace;\n\
    use quire_exact::{\n    CollectionKind, CollectionType, FloatType, IeeeWidth,\
    \ IllTyped,"
- id: tests/failing-allocator/src/lib.rs
  path: tests/failing-allocator/src/lib.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Thread-local real allocation denial for the external-check test binary.

    //! Safe code cannot implement `GlobalAlloc`; all unsafe operations below only

    //! forward the allocator contract unchanged to `'
- id: tests/failing-allocator/Cargo.toml
  path: tests/failing-allocator/Cargo.toml
  role: examined
  excerpt: '[package]

    name = "qsv-failing-allocator"

    version = "0.0.0"

    edition = "2021"

    publish = false

    license = "AGPL-3.0-or-later"


    # A separate test fixture owns the audited GlobalAlloc boundary. The product

    # manifest continues to forbid unsafe code, includ'
- id: Cargo.toml
  path: Cargo.toml
  role: examined
  excerpt: '[package]

    name = "quire-semantic-value"

    version = "0.1.0"

    edition = "2021"

    rust-version = "1.98"

    description = "The shared no_std semantic-value leaf: runtime semantic values
    over the quire-exact kernel."

    authors = ["Agent IX <peter@agent-ix.ai>"]

    li'
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: '# This file is automatically @generated by Cargo.

    # It is not intended for manual editing.

    version = 4


    [[package]]

    name = "allocation-counter"

    version = "0.8.1"

    source = "registry+https://github.com/rust-lang/crates.io-index"

    checksum = "beb9e990c0a'
- id: tests/map_and_bounded_quantity.rs
  path: tests/map_and_bounded_quantity.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! `Map` and `BoundedQuantity` in declaration admission, equality checking,

    //! type resolution and supplied membership, through the public API (TC-927).

    //! Fixture keys are producer-supplied identities'
- id: tests/union_registry.rs
  path: tests/union_registry.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Shared registry/admission tests, distinct from trusted kernel construction.

    //! Structural fixture keys are already-verified inputs, not FR-441 minting evidence.


    mod common;


    use ix_trace_rs::trace;

    '
- id: tests/union_staging.rs
  path: tests/union_staging.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Resolved topology and verified sealing through the real shared registry.

    //! Fixture keys are producer-supplied identities, not a digest-minting oracle.


    mod common;


    use ix_trace_rs::trace;

    use quire'
```

## Validation evidence

Reviewer independently ran all four changed integration binaries: 57 passed, including actual allocator denial; shared storage reservation unit tests: 12 passed. cargo fmt --all -- --check and cargo clippy --locked --all-targets --all-features -- -D warnings passed. Spec and explicit scratchpad SR validation passed. Full aggregate/no_std/deny/docs gates are lead-owned and were not run in this review. Linear read-back passed: YAML parses, content round-trips (terminal newline stripped by CLI), and finding IDs match. Existing IR has-review label reused.
