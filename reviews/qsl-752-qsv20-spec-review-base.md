---
id: SR-9048
title: "base review of FR-116 and TC-916"
type: SpecReview
analysis: base
scope: "agent-ix/quire-semantic-value@346076d6d1009b7019de8f8e97e08428d56d4ea3; spec/functional/FR-116-model-owned-enumeration-case-names.md; spec/test-cases/TC-916-model-owned-enumeration-case-names.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-116
    type: reviews
---
# Base review of FR-116 and TC-916

## Summary

Ticket: QSL-752. This base review folds in the EARS, integrity and
failure-domain checks for the new FR-116 and TC-916.

- The ids are unique. FR-116 and TC-916 exist on no other remote branch, and
  no other branch mentions them. Open PR #12 uses FR-115 and TC-915. The
  FR-113, FR-106 and TC-913 amendments from #19 are on main and do not
  collide.
- `quire validate` passes on both files. The only output is known tooling
  noise: DuplicateArchetype, DuplicateInverseEdge and semantic.inline-data-schema.
- `quire matrix` tags all six FR-116 criteria.
- EARS: the Description is a ubiquitous `SHALL` statement naming the system
  item. The Behavior list and ACs follow the repo's FR-114 layout.
- TC-916 `verifies` FR-116. Its scope lists AC-1 to AC-6, and each procedure
  step has a matching expected result.
- The spec agrees with QSpec FR-141-AC-8 (cited by id only): any non-empty
  case name, distinct by exact bytes, in ascending UTF-8 byte order.
- Public-repo hygiene: no local paths and no private QSpec text.

## Examined scope

- FR-116 (examined): Description, Use case and Behavior items 1 to 4.
- FR-116-AC-1 to FR-116-AC-6 (examined).
- TC-916 (examined): procedure steps 1 to 4 and expected results 1 to 4.

## Verdict

**PASS** with one low finding. The refusal precedence is not stated.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Behavior item 2 and FR-116-AC-4 do not say which cause wins when a list is both malformed and unsorted, for example `["b", "a", "a"]`. The code checks the member-list rule first, as `new` does, and returns `NonCanonicalPreimage`. An implementer who reads the item could instead use a strict-ascending check and return `UnsortedUnorderedMembers`. No TC-916 row pins the order. State "refused with `NonCanonicalPreimage` before the order is checked" and add a duplicate-and-unsorted vector to TC-916 step 2 and to the AC-4 test. | spec/functional/FR-116-model-owned-enumeration-case-names.md:20,31; spec/test-cases/TC-916-model-owned-enumeration-case-names.md:20 |
