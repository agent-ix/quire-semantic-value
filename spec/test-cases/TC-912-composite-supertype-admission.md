---
id: TC-912
title: "Composite supertype admission and bounded ancestry"
type: TC
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: verifies
---
# TC-912: Composite supertype admission and bounded ancestry

## Description

Verify that the QSV type environment admits record or tuple values through an
acyclic composite-supertype chain without changing their most-specific
identity, and refuses invalid or over-limit ancestry.

## Test Procedure

1. Construct three distinct record declarations with producer-assigned keys
   ordered as parent, intermediate and child. Declare the intermediate's
   parent and the child's intermediate as direct supertypes. Construct the
   child value and query both its immediate parent and the transitive
   grandparent conformance/admission. Also query the parent in the child
   direction and an unrelated record in the parent direction.
2. Inspect the admitted `CompositeValue` declaration key and check that
   `admits_equality_conversion` still rejects conversion between the child's
   and parent's distinct `ValueType::Composite` keys.
3. Attempt a two-node composite-supertype cycle and a declaration with an
   unknown supertype. Then admit a one-edge chain with `ancestor_steps` set to
   zero and a valid chain with insufficient `work_units`.

## Expected Results

1. The intermediate conforms to the parent. The child conforms to both its
   immediate parent and grandparent and is admitted at both boundaries; the
   parent does not conform to the child, and the unrelated record is refused.
2. The accepted value retains the child's original declaration key. Equality
   conversion between child and parent type keys remains unavailable.
3. The cycle and missing key are rejected before an environment is returned.
   Each exhausted ceiling returns its corresponding typed limit failure with
   no partial environment.
