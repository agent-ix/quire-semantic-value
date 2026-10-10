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

1. Before admission builds a registry, index, key join, ancestor set, effective
   attribute, or traversal worklist from authored declarations, the admission SHALL
   use a representation whose growth is explicitly fallible before
   mutation. The current `BTreeMap`/`BTreeSet` insertions and collecting,
   cloning, `Arc::new`, and vector growth in that path do not establish this
   property. An implementation may use pre-reserved vectors with checked
   ordering and lookup, or another representation with demonstrably fallible
   construction; it SHALL retain the existing key order and lookup results.
2. When a fallible storage reservation during type-environment admission fails
   because allocation is denied, the admission SHALL
   return a distinct allocation member of `EnvironmentFailure`. It SHALL
   preserve the actual request passed to that reservation in its native unit
   (bytes or additional elements), with the unit identified. The admission SHALL
   measure that request at the failing boundary without deriving it from a
   configured limit, current collection length, or unrelated allocation.
3. If arithmetic needed to size or represent a reservation overflows, then
   admission SHALL return a distinct capacity/size failure rather than label
   it a measured allocator denial. Any conversion to bytes SHALL use checked
   arithmetic; a failed conversion SHALL preserve its own overflow reason.
4. When admission stops for either storage failure, the environment SHALL
   publish no partial descriptor, registry index, or key join. A later
   attempt with the same declarations after removing the denial SHALL produce
   the same complete environment as an attempt with no prior denial.
5. A storage failure SHALL NOT be converted into `InvalidDeclaration`,
   `EnvironmentLimitKind::WorkUnits`, `EnvironmentLimitKind::AncestorSteps`,
   `IdentityRefusal::Allocation`, cancellation, or a checked-invariant cause.
   Existing budget and input failures SHALL retain their original typed
   fields and charge points. Any public helper that projects
   `EnvironmentFailure` SHALL retain the new storage cases as distinct typed
   outcomes rather than manufacture a limit or declaration refusal.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-108-AC-1 | At an actual fallible registry, index, or traversal reservation boundary after the storage redesign, an injected allocator denial returns the allocation member of `EnvironmentFailure` with the exact request amount and its byte or additional-element unit. No configured bound, setting, or declaration cause is present. The same declaration set admits when the denial is removed. | Test |
| FR-108-AC-2 | A reservation-size arithmetic or representation overflow returns a distinct capacity/size failure and is never reported as a measured allocator denial. A representable request preserves its original amount and unit through checked conversion where conversion is needed. | Test |
| FR-108-AC-3 | Denial during descriptor, index, key-join, or traversal construction exposes no partly admitted environment. Retrying the same declarations after removing the denial yields the complete deterministic registry and member links. | Test |
| FR-108-AC-4 | The existing exact `work_units` and `ancestor_steps` N-1/N controls retain their original limit kind, configured bound, and counter; invalid declarations, cancellation, canonical `IdentityRefusal::Allocation`, and released checked-invariant causes retain their established classifications. | Test |
| FR-108-AC-5 | Every allocation-bearing growth path reachable from authored declarations during environment admission uses fallible construction and the new carrier; no ordinary `BTreeMap`/`BTreeSet` insertion, allocating clone, `Arc::new`, or unchecked vector growth remains on those paths. The replacement retains deterministic key order, lookup results, and work accounting. | Inspection |

## Dependencies

- [QSL FR-082](ix://agent-ix/quire-spec-language/FR-082) owns the expression
  checker's admission behavior but does not define registry allocation units
  or an outcome for this carrier. QSL-678 owns specification of the caller's
  public resource payload and its criterion and test case. The intended
  downstream outcome is distinct from malformed input, configured-budget
  exhaustion, and cancellation, with no evaluation or evaluation charge
  before denial. This QSV requirement does not define the QSL result shape.
- [QSL FR-259](ix://agent-ix/quire-spec-language/FR-259) governs canonical
  identity-reader and encoder allocation failure. Its `IdentityRefusal`
  mapping does not classify registry storage failure.
- The QSV admission carrier is limited to type-environment admission storage.
  It does not revise union shape or member-sealing rules, scalar charge
  points, or the released checked-invariant cause catalog.
- QSV's current `TypeEnvironment` uses infallible tree insertion and several
  other infallible allocation sites. Those paths require a storage redesign;
  ordinary `BTreeMap`/`BTreeSet` insertion cannot be treated as an observed
  fallible reservation. The redesign must preserve deterministic iteration,
  lookups, and the existing work accounting.
