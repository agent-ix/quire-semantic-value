---
id: SR-9046
title: "Code review of quire-semantic-value PR #20: model-owned enumeration case names"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@346076d6d1009b7019de8f8e97e08428d56d4ea3; src/enumeration.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-116
    type: reviews
---
# Code review of quire-semantic-value PR #20

## Summary

Ticket: QSL-752 (and QSL-738 variant types). Code review with the rust-review
lane folded in, scoped to the PR diff: `EnumDeclaration::model_owned`, the
private `sorted_if_unordered` helper shared with `EnumDeclaration::new`, and four
tests tagged TC-916.

- Purely additive. The only new `pub` item is `EnumDeclaration::model_owned`.
  `new` keeps its signature and behaviour: the unsorted check moved into the
  private helper unchanged. `is_member_list`, `member`, `EnumValue`,
  `EnumMemberIndex` and `compare_enum` are untouched. QSL's `branch=main`
  dependency needs no lock-step adaptation.
- Refusals are typed with existing causes and match `new`: empty list, empty
  name or repeated name give `NonCanonicalPreimage` (the same cause `new` uses
  for its member-list rule), checked before the sort, as in `new`. Unsorted
  cases give `UnsortedUnorderedMembers`. `Vec<String>::is_sorted` uses `String`'s
  `Ord`, which is byte-lexicographic, so it matches QSL's
  `cases.sort_by(|l, r| l.1.cmp(&r.1))` (ascending UTF-8 byte order) and QSpec
  FR-141-AC-8's ordering.
- Names are held verbatim. Nothing in QSV trims, normalizes or checks a
  `model_owned` member: `is_identifier` is reached only through
  `is_member_list` from `new`. `member()` compares with `==` on `String`, so
  matching is byte-exact.
- No identity is computed in QSV for a declaration (FR-060). The canonical
  preimage encoding lives in QSL. There, model-owned variant types are keyed
  by their FR-322 checked node and `mint_variant_id`, and are never encoded
  as `quire.enum-declaration-node/v1` or `quire.enum-member-node/v1` graph nodes
  (`enum_nodes` runs only for source `EnumBinding`s). So the identifier-only
  strict readers there are not reached. No latent break was found in QSV.
- quire-exact side: `EnumValue` identity is the member key and `variant()`
  retypes it. Values of two declarations compare as `DistinctEnumDeclarations`.
  An ordering operator on a model-owned (unordered) value is
  `UnorderedEnumOrdering`. This is unchanged and correct.
- Gate evidence covers the head. The red log has six E0599 errors for the
  missing `model_owned`. The green log has six passed. The gate log is
  `make ci`: fmt-check, clippy (all targets, all features, and no_std),
  59 tests, doc, and deny (advisories, bans, licenses and sources ok). It was
  written after the last commit, which touched only one line of the FR-116
  spec.

## Verdict

**PASS** with one low finding. The API shape fits: an associated constructor
next to `new`, with `ordered` fixed to false, the same `Vec<String>` argument
and the same error type. The tests would fail without the change (compile
error, then each refusal). The assertions are typed on the exact cause. The
diff has no local paths and no private QSpec text, which is cited by id only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `SemanticGraphCause::NonCanonicalPreimage` is documented as "The preimage does not satisfy `node-identity-preimage.schema.json`". `model_owned` now returns it for an empty list, an empty name or a repeated name of a model-owned declaration, which has no such preimage and whose names that schema's identifier rule would not admit. Widen the variant doc to cover a malformed member list of a model-owned declaration. | src/semantic_node.rs:35-36; src/enumeration.rs:78-89 |
