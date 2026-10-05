---
id: FR-107
title: "Set each checking limit on its own"
type: FR
relationships: []
---
# FR-107: Set each checking limit on its own

## Description

`CheckingLimits` SHALL give each of its three ceilings (`nodes`, `input_bytes`, `work_budget`) a consuming builder that sets that ceiling and keeps the other two. A caller changes one checking limit without restating the others. QSL names the node ceiling `s3.nodes`.

## Use case

A driver sets the node ceiling (QSL's `s3.nodes`) alone from a request and keeps the published defaults for the other checking limits.

## Behavior

1. `with_nodes`, `with_input_bytes` and `with_work_budget` each return the limits with only their own ceiling replaced.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-107-AC-1 | Starting from a base with distinct node, input-byte and work-budget ceilings, each builder applied alone changes its own ceiling and leaves the other two equal to the base's. | Test (TC-905) |

## Status

Implemented.

## Dependencies

- None.
