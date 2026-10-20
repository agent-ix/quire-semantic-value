---
id: FR-125
title: "Own the closed refusal vocabulary of definition-closure admission"
type: FR
relationships:
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
---
# FR-125: Own the closed refusal vocabulary of definition-closure admission

## Description

`quire-semantic-value` SHALL own the refusal vocabulary that definition-closure
admission reports: the lock's eight `selection_refusal_codes`
(`SelectionRefusalCode`), the one diagnostic code `invalid_package`
(`PackageRefusalCode::InvalidPackage`) and the three causes it pairs with here
(`PackageCause`). A consumer such as the runtime consumes these types and keeps
no copy of the vocabulary.

QSpec FR-272 ("typed diagnostic cause") is the upstream authority for the
`invalid_package` spellings and for which cause pairs with which code. QSpec
`proposals/quire-v1/definitions/value-complete.md` is the upstream authority for
the eight selection codes and their check order. This requirement restates
the subset that definition-closure admission raises and closes it: the crate
declares no `invalid_package` cause and no diagnostic code that FR-272 does not
list. The eight selection codes are not FR-272's; they come from
`value-complete.md`.

## Use case

The runtime admits a package's `div`, `rem` and `mod` definitions (the
integer-division closure) and an IEEE profile (`admit_ieee_profile`, the IEEE
profile closure), and must name each refusal exactly as the QSpec catalog does.
These two closures are the raise sites of the three causes. It matches on `PackageCause`
without a wildcard arm, so a cause added later is a compile error at every
consumer, not a silent fallthrough.

## Inputs

- A definition-closure admission outcome that is a refusal.

## Outputs

- `SelectionRefusalCode`, `PackageRefusalCode`, `PackageCause` and
  `PackageRefusal { code, cause }`, with `as_str` on each, and
  `SelectionRefusalCode::from_code` and `SelectionRefusalCode::ALL`.

## Behavior

- `PackageCause` SHALL have exactly three variants: `MissingMember`,
  `ConflictingDefinition` and `IncompatibleDefinition`.
- `PackageCause::as_str` SHALL return `missing-member`,
  `conflicting-definition` and `incompatible-definition` respectively, the
  spellings of FR-272's `invalid_package` row, byte for byte.
- `PackageRefusalCode` SHALL have exactly one variant, `InvalidPackage`, whose
  `as_str` is `invalid_package`.
- `PackageRefusal::invalid_package` SHALL pair every `PackageCause` with
  `PackageRefusalCode::InvalidPackage`.
- Every `PackageRefusal` SHALL carry `PackageRefusalCode::InvalidPackage`.
- The crate SHALL NOT declare a cause of any other code family, including the
  `stale_dependency` and `source_digest_mismatch` causes `byte-digest-mismatch`,
  `digest-domain-mismatch` and `content-mismatch`; the QSpec package-read and
  replay layers own them.
- `PackageCause` SHALL NOT be marked `#[non_exhaustive]`, because the set is the
  three causes that the two closures raise, and a consumer's exhaustive
  `match` is the check that it handles all of them. A new cause is a change to
  this requirement, and it is a cause FR-272's `invalid_package` row already
  lists or one FR-272 adds first.
- `SelectionRefusalCode::ALL` SHALL list the eight codes in normative check
  order: `selection_unknown_trigger`, `selection_duplicate_trigger`,
  `selection_unknown_role`, `selection_duplicate_role`,
  `selection_required_missing`, `selection_alternative_conflict`,
  `selection_trigger_unsatisfied` and `selection_untriggered_profile`.
- `SelectionRefusalCode::as_str` SHALL return each code's spelling, and
  `from_code` SHALL return the code for exactly those eight spellings and
  `None` for any other string, `invalid_package` included.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-125-AC-1 | `PackageCause::as_str` returns `missing-member`, `conflicting-definition` and `incompatible-definition`, each byte-equal to the spelling in QSpec FR-272's `invalid_package` row, and `PackageRefusalCode::InvalidPackage.as_str()` is `invalid_package`. | Test (TC-924) |
| FR-125-AC-2 | `PackageRefusal::invalid_package(cause)` returns `code` `InvalidPackage` and the given `cause` for each of the three causes, and `PackageRefusalCode` has the one variant `InvalidPackage`. | Test and Inspection (TC-924) |
| FR-125-AC-3 | `PackageCause` has exactly three variants and is not `#[non_exhaustive]`: an exhaustive `match` over it with no wildcard arm, written in an integration test under `tests/` (a crate other than the defining one, where `#[non_exhaustive]` takes effect), compiles, and none of `byte-digest-mismatch`, `digest-domain-mismatch` or `content-mismatch` is the spelling of any variant. | Test and Inspection (TC-924) |
| FR-125-AC-4 | `SelectionRefusalCode::ALL` holds the eight codes in the order above, `as_str` returns their spellings, `from_code` round-trips each, and `from_code` returns `None` for `invalid_package`, an empty string and a differently cased spelling. | Test (TC-924) |

## Dependencies

- QSpec FR-272 (spec/objects/foundation/FR-272-typed-diagnostic-cause.md) owns
  the causes and their pairing with diagnostic codes.
- QSpec `proposals/quire-v1/definitions/value-complete.md` owns the selection
  codes and their check order (the lock's `selection_refusal_codes` array in
  `complete-value-lock.json` is sorted alphabetically, not in check order).

## Status

Implemented in `src/definition.rs`; the TC-924 tests are pending.

## References

- Linear IR-882.
