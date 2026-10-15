---
id: SR-9121
title: "IR-714 Enum supplied membership code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value; src/declaration/membership.rs at 5faf70b8; quire-exact d41a1da value.rs; FR-109 and TC-908"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-109
    type: reviews
---

# IR-714 Enum supplied membership code review

## Summary

Ticket: IR-714. Reviewed the sole-file local Enum slice at 5faf70b8e725364fdee52dd3c580b045d0ecbcb6 against f42141336b92745c2b579ef193c67d4bacb5aaa7. This is a partial IR-714 review, not acceptance of numeric, Decimal, union, or QSL cross-phase integration.

## Verdict

**PASS** — no defect in the changed Enum arm or its tagged public controls.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Evidence

- The arm emits DescriptorQuery at the occurrence's descriptor locus before invoking the helper, and AfterHelper before returning Member. The public budget charges and polls the original Cancel on the event, then polls after the helper. A denied query never runs the helper.
- Pinned quire-exact d41a1da39ed43c1ad536b752469953ad3dd7003a implements EnumShape::rank as a linear `.position` over its retained `Vec<VariantId>`, comparing fixed-size identities. `ValueType::admits` calls rank once and compares its result with the member's rank. Thus an admitted shape of E variants has at most E comparisons, no helper allocation, and a cancellation response after that one scan. This is the source-backed E premise; no fixed latency independent of E is claimed. The expected type is caller-supplied under FR-109's admitted-type precondition.
- The changed test enumerates ValueVisit then DescriptorQuery: ceilings 0, 1, and 2 verify the exact denied event/locus and retained spend. Its wrong-kind control verifies that a mismatched value stops after ValueVisit. The earlier member-invalid control now checks the descriptor locus and two-event spend. The scan currently has no cache; because the event precedes it, a future cache hit cannot skip logical charging without changing this arm.
- No unsafe, new public API, copied algorithm, untyped error, panic, or test-only production behavior was added. CI/workflow files are unchanged. This review did not run a full Cargo gate.

## Examined scope

```yaml
scope:
- id: FR-109-AC-1
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: A source-enumerated supported membership fixture requiring N logical events completes at N and fails at N-1 before the last event, retaining supplied-admission phase, configured N-1, successful spend N-1, next amount one and the actual denied event/locus. Zero denies the first event. No denied event is performed or spent.
- id: FR-109-AC-3
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Cancellation through the caller's original handle stops the next permitted response boundary with typed cancellation, no further traversal events and no replacement handle. Cancellation is not reported as work exhaustion or invalid input.
- id: FR-109-AC-4
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Invalid kind/type and member/payload failures permitted by the owning supported-family contracts return their typed membership cause and locus, never a partial admission. Released nested construction custody is preserved; a helper cannot manufacture extra payload visits.
- id: FR-109-AC-7
  path: spec/functional/FR-109-bound-supplied-value-traversal.md
  role: examined
  excerpt: Each claimed supported helper path has a source-backed input bound, finite comparison/arithmetic and temporary-storage bound, cancellation response boundary and fallible storage route where needed. Unqualified Decimal and future union paths remain implementation dependencies; logical-event totals alone do not establish those helper premises.
```
