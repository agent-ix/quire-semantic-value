---
id: FR-129
title: "Bound exponent copying and decimal emission in compound unit identity"
type: FR
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-128
    type: depends_on
  - target: ix://agent-ix/quire-semantic-value/FR-124
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-259
    type: references
---
# FR-129: Bound exponent copying and decimal emission in compound unit identity

## Description

When `compound_unit_id` measures and owns an admitted compound unit's ordered
root terms, `quire-semantic-value` SHALL use `quire-exact`'s fallible `Integer`
copy and bounded canonical decimal emission for every exponent. A failed
reservation SHALL become `IdentityRefusal::Allocation { requested }`, naming
the additional bytes of that reservation. This is QSV's consumer obligation at
the existing [FR-128](./FR-128-consume-structural-identities.md) structural
identity boundary; [QSL FR-259](ix://agent-ix/quire-spec-language/FR-259)
governs the upstream encoder's allocation classification.

## Inputs

- Ordered, admitted `(root NodeKey, Integer exponent)` terms.
- The caller's `quire_canonical::Limits` byte bound.

## Outputs

- The structural `UnitId::Compound` retaining each root and mathematical
  exponent, or one typed `IdentityRefusal` with no partial id.

## Behavior

1. When the compound term preimage fits the caller's byte bound, QSV SHALL
   encode each exponent as its exact canonical signed decimal string and
   return a `UnitId::Compound` whose ordered terms and exponents equal the
   inputs. The canonical encoding is only a bounded measurement: its bytes and
   any digest of them SHALL NOT replace the structural id.
2. If the canonical term preimage exceeds the caller's byte bound, then QSV SHALL
   return `IdentityRefusal::InputBytes { bound, required }`, where
   `bound` is that limit and `required` is the reached output byte count of
   `quire-canonical` at refusal, including the emitted structural context
   before the over-limit exponent bytes. `required` SHALL be greater than
   `bound`; it need not equal the length of the complete final preimage when
   the encoder refuses early. No partial id is returned.
3. If the fallible copy or decimal emission of an exponent cannot reserve
   storage, then QSV SHALL return `IdentityRefusal::Allocation { requested }`
   with the additional bytes requested by that failed reservation. It SHALL
   return no id and SHALL NOT classify the failure as `InputBytes` or
   `NonCanonical`.
4. QSV SHALL use `Integer::try_clone` and
   `Integer::try_to_decimal_string` at this boundary. It SHALL NOT use
   `Integer::clone` or `Integer::to_string` for these two operations, or add a
   second integer implementation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-129-AC-1 | For ordered roots with one inline positive exponent and one large negative exponent, `compound_unit_id` returns the same structural `UnitId::Compound` terms, roots, signs and mathematical exponents supplied; its bounded RFC 8785 term encoding contains the full canonical decimal exponent strings in the same term order and no digest or encoded byte array is the returned id. | Test |
| FR-129-AC-2 | For the same term preimage, a limit equal to its complete canonical byte length succeeds and a limit one byte shorter returns `InputBytes { bound: length - 1, required: length }`, with no id. With an earlier limit crossed while writing an exponent, `required` equals the canonical encoder's reached output count including preceding structural bytes, and exceeds `bound`; it is not just that exponent's decimal length or necessarily the complete final preimage length. | Test |
| FR-129-AC-3 | On controlled refusal of an exponent-copy reservation and of an exponent-decimal reservation, each call returns `Allocation { requested }` with the exact additional bytes requested by the refused reservation, returns no id, and does not yield `InputBytes` or `NonCanonical`. | Test |
| FR-129-AC-4 | Inspection of the compound identity construction and encoding path finds `Integer::try_clone` and `Integer::try_to_decimal_string`, maps their allocation errors to `IdentityRefusal::Allocation { requested }`, retains the structural `UnitId` output, and finds no infallible exponent `clone`/`to_string`, digest conversion or duplicate integer implementation on that path. | Inspection |

## Dependencies

- [FR-128](./FR-128-consume-structural-identities.md) owns QSV's structural
  unit identity; [FR-124](./FR-124-identified-quantity-from-an-exact-conversion.md)
  propagates its refusal through the exact quantity handoff.
- `quire-exact` supplies fallible `Integer::try_clone` and bounded
  `Integer::try_to_decimal_string`. `quire-canonical` owns RFC 8785 encoding
  and the caller's byte limit.

## Status

Planned under IR-1149. The exact kernel APIs are merged in quire-exact #109;
QSV's consumer path remains to be implemented.
