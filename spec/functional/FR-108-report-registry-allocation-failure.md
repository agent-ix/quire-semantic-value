---
id: FR-108
title: "Type-environment admission reports a failed storage reservation"
type: FR
relationships: []
---
# FR-108: Type-environment admission reports a failed storage reservation

## Description

When storage for the type-environment registry, an index, or an admission
traversal cannot be reserved, `quire-semantic-value` SHALL stop admission with
a typed allocation failure that retains the request made at the failing
reservation boundary. This failure is distinct from an invalid declaration,
a configured environment limit, and canonical identity-preimage allocation.

## Inputs

- Authored record, tuple, union, and object-type declarations and their keys.
- Configured `TypeEnvironmentLimits` and the caller's cancellation handle.

## Outputs

- One complete admitted `TypeEnvironment`, or an `EnvironmentFailure` naming
  the reason admission stopped.

## Behavior

1. When a fallible storage reservation for a registry, index, key join, or
   traversal worklist fails because allocation is denied, the admission SHALL
   return a distinct allocation member of `EnvironmentFailure`. It SHALL
   preserve the actual request passed to that reservation in its native unit
   (bytes or additional elements), with the unit identified. The admission SHALL
   measure that request at the failing boundary without deriving it from a
   configured limit, current collection length, or unrelated allocation.
2. If arithmetic needed to size or represent a reservation overflows, then
   admission SHALL return a distinct capacity/size failure rather than label
   it a measured allocator denial. Any conversion to bytes SHALL use checked
   arithmetic; a failed conversion SHALL preserve its own overflow reason.
3. When admission stops for either storage failure, the environment SHALL
   publish no partial descriptor, registry index, or key join. A later
   attempt with the same declarations after removing the denial SHALL produce
   the same complete environment as an attempt with no prior denial.
4. A storage failure SHALL NOT be converted into `InvalidDeclaration`,
   `EnvironmentLimitKind::WorkUnits`, `EnvironmentLimitKind::AncestorSteps`,
   `IdentityRefusal::Allocation`, cancellation, or a checked-invariant cause.
   Existing budget and input failures SHALL retain their original typed
   fields and charge points. Any public helper that projects
   `EnvironmentFailure` SHALL retain the new storage cases as distinct typed
   outcomes rather than manufacture a limit or declaration refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-108-AC-1 | At an actual fallible registry, index, or traversal reservation boundary, an injected allocator denial returns the allocation member of `EnvironmentFailure` with the exact request amount and its byte or additional-element unit. No configured bound, setting, or declaration cause is present. The same declaration set admits when the denial is removed. | Test |
| FR-108-AC-2 | A reservation-size arithmetic or representation overflow returns a distinct capacity/size failure and is never reported as a measured allocator denial. A representable request preserves its original amount and unit through checked conversion where conversion is needed. | Test |
| FR-108-AC-3 | Denial during descriptor, index, key-join, or traversal construction exposes no partly admitted environment. Retrying the same declarations after removing the denial yields the complete deterministic registry and member links. | Test |
| FR-108-AC-4 | The existing exact `work_units` and `ancestor_steps` N-1/N controls retain their original limit kind, configured bound, and counter; invalid declarations, cancellation, canonical `IdentityRefusal::Allocation`, and released checked-invariant causes retain their established classifications. Storage growth reachable from authored declarations uses the new failure carrier rather than infallible growth. | Test |

## Dependencies

- [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) owns the expression
  checker's consumer result. Its implementation must map this carrier to a
  typed resource failure with the measured request and no invented limit,
  distinct from malformed input, configured-budget exhaustion, and
  cancellation. QSL must deny before evaluation or evaluation charge. The
  QSL owner must bind those consumer assertions to its own criterion and test
  case; this QSV artifact does not assign a new QSL requirement ID.
- [QSL FR-259](ix://agent-ix/quire-spec-language/FR-259) governs canonical
  identity-reader and encoder allocation failure. Its `IdentityRefusal`
  mapping does not classify registry storage failure.
- The QSV admission carrier is limited to registry/index/traversal storage.
  It does not revise union shape or member-sealing rules, scalar charge
  points, or the released checked-invariant cause catalog.
