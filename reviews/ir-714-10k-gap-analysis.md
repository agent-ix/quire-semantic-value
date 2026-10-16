---
id: SR-11611
title: "Gap analysis of IR-714 budgeted 10k supplied-admission tests"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-semantic-value; tests/union_staging.rs; spec/functional/FR-109-bound-supplied-value-traversal.md; spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md; QSL FR-321-AC-4; ticket IR-714; branch code/ir-714-budgeted-10k-test at 9f767240d997b776f8328de19ba93dc5a5cd4310"
review_set: subset
---

## Summary

Ticket: IR-714. The diff adds two traced tests to `tests/union_staging.rs`: exact N completion and N-1 denial for a sealed 10,000-link Tree. It changes no production code and no spec. The tests back FR-109-AC-1 with a source-derived N of 60,002, independently re-derived by the reviewer, and with exact typed assertions on the denial record.

## Verdict

**PASS with findings (one medium, one low).** The FR-109-AC-1 and TC-908 step 1 bindings are correct, and all 13 targeted mutants plus one count-preserving mutant were killed by the suite. Two problems remain. The FR-321-AC-4 tag on the denial test claims more than that test exercises. The spec's union qualification and TC-908 status text now contradict the evidence this branch adds under TC-908.

## Scope examined

- `FR-109-AC-1` (examined): A source-enumerated supported membership fixture requiring N logical events completes at N and fails at N-1 before the last event, retaining supplied-admission phase, configured N-1, successful spend N-1, next amount one and the actual denied event/locus. Zero denies the first event. No denied event is performed or spent.
- `FR-109-AC-7` (examined): Each claimed supported helper path has a source-backed input bound ...; a future union path remains an implementation dependency until its owning shape and walk are available.
- `TC-908` (examined): Step 1 N/N-1 procedure; step 4 "Add union cases only after the public owning union API and helper table are reviewed and available"; Status paragraph.
- QSL `FR-321-AC-4` (examined): A supplied `Tree` value 10,000 levels deep is admitted, converted and compared equal to itself under the default run limits without a host stack overflow. With the run's value-occurrence limit set one below the value's occurrence count, admission returns that limit by name with its configured value.
- `FR-109-AC-6` (context_only): The only untagged criterion reported by `quire matrix --strict`. The failure is identical on base 98091ee and head 9f76724.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The N-1 denial test is tagged FR-321-AC-4, but it denies the supplied-membership logical-work ceiling at 60,001 events, not the run's value-occurrence limit one below the 20,001 occurrences, and it performs no conversion or equality; the binding overstates that AC's coverage | tests/union_staging.rs:796 |
| FND-002 | medium | FR-109's helper table still lists the union payload/descriptor family as "Future"; AC-7 still calls the union path an implementation dependency; TC-908's Status assigns deep-Tree exact/one-less controls to QSL and gates union cases on a reviewed helper table. This branch adds a deep-Tree union N/N-1 case under TC-908 without amending that text, so the spec now contradicts its own evidence | spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:96 |

## Coverage

Plan completion: not assessed.

`quire matrix --strict` (quire 0.36.2) exits 1 on head and on base, with exactly one untagged criterion, `FR-109-AC-6`. The only matrix difference is that FR-109-AC-1 gains `tests/union_staging.rs:767` and `:798`, plus a line shift for the existing FR-115-AC-4 binder. Both new tests pass, and the expected values are authored constants, not read back from the counter being tested. Zero-budget denial for AC-1 is covered elsewhere (`src/declaration/membership.rs:3598`), not by this diff. FR-321-AC-4 is a QSL-owned criterion and does not appear in this repository's matrix. Its conversion and equality halves rely on the pre-existing `ten_thousand_supplied_links_follow_sealed_keys_with_internal_position_types`, and its occurrence-limit half relies on QSL TC-830.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The rewritten TC-908 Status says "Other QSV caller and integrated Decimal controls remain PLANNED/UNRUN" and "no QSV Decimal test execution is claimed". Both are false: `bounded_decimal_uses_cumulative_exact_event_boundary` (traced TC-908, FR-109-AC-8) runs through `admit_supplied_bounded_numeric`, and the FR-109-AC-1..5 QSV caller controls are tagged and pass. FR-109's unchanged Status (lines 155-157) makes the same claim. | spec/test-cases/TC-908-supplied-traversal-phase-boundaries.md:101 |
| FND-004 | low | The new "Sealed union payload/descriptor" helper row states no finite-cost premise, unlike the table's other rows and the table's own rule that each premise identifies its input bound. The descriptor lookup's ordered-map gets and its `UnionMember` equality, which compares the member identifier, are not bounded. AC-7 says union helper premises still need inspection, so nothing is overclaimed. | spec/functional/FR-109-bound-supplied-value-traversal.md:108 |


## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | FR-109's rewritten Status says "QSV tests traced to FR-109-AC-1 through FR-109-AC-5 execute", but one of those tests does not run by default: `supplied_path_arena_growth_probe_10k`, traced to AC-1 and AC-5, is `#[ignore]`d. TC-908's rewritten Status places the AC-1..5 caller controls only in `src/declaration/membership.rs` and `tests/union_staging.rs`, which omits binders in `src/supplied_membership.rs`, `src/declaration.rs` and `tests/bounded_numeric_reservation.rs`. Unchanged line 144 of FR-109 Dependencies still says "IR-719 still owns QSV caller qualification", although IR-719 is Done and the new Status calls the caller implemented. | spec/functional/FR-109-bound-supplied-value-traversal.md:156 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The round-3 edit to FR-109 Status left a sentence with no verb: "The integrated Decimal controls traced to FR-109-AC-8 for cumulative N/N-1 and zero budgets, membership and size outcomes, cancellation and typed resource-error projection." The earlier wording "as do ... execute" was cut, so FR-109 no longer says what state those controls are in. TC-908 Status still says they execute. | spec/functional/FR-109-bound-supplied-value-traversal.md:160 |

## Dispositions

Round 1, checked against `git diff 9f76724..abc8f1f` (head `abc8f1f37292bc5a3ce19e2706e050ceef974d4d`). `quire validate` is clean for FR-109 and TC-908 and for all of `spec/**`. `quire matrix --strict` still fails only on the pre-existing untagged FR-109-AC-6. FR-109-AC-1 is tagged by `tests/union_staging.rs:741` and `:772`. No change to `src/`, Cargo or CI, and no `#[allow]` added. QSL-778 (TC-830 step 4, full-size 10,000-level supplied Tree) owns the full-size consumer acceptance. Every changed spec line was checked against the code: the union walk does check shape, final key and member, rejects a payload whose length differs from the position count, reserves worklist capacity before scheduling, and polls at the charged event and after the lookup. No criterion id was removed and no obligation was weakened.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The N-1 denial test now traces only TC-908 and FR-109-AC-1. FR-321-AC-4 stays on the exact-N test as a partial contributor. |
| FND-002 | fixed | The FR-109 union helper row, AC-7's union clause and the Dependencies paragraph now record the implemented union shape and walk. TC-908 step 4 includes sealed-union cases. TC-908 Status names the 10k N/N-1 controls and assigns the consumer limits to QSL-778. New FND-003 covers a remaining false Decimal claim in the rewritten Status. |

Round 2, checked against `git diff abc8f1f..794a2f7` (head `794a2f7b0f09cb6ef307ccb1557f4cf3a5ca4fa3`; Markdown only). `quire validate` is clean for `spec/**`. `quire matrix --strict` still fails only on the pre-existing untagged FR-109-AC-6. FR-109-AC-1 is tagged by `tests/union_staging.rs:741` and `:772`. No change to `src/`, Cargo or CI, and no criterion id or obligation removed.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-003 | fixed | Both Status sections now record the executed controls. `bounded_decimal_uses_cumulative_exact_event_boundary` (membership.rs:3526) covers N/N-1 and the zero budget at :3598. `bounded_decimal_retains_size_and_nonmembership_locus` is at :3619, `decimal_kernel_resource_failures_project_without_partial_admission` at :3730, and `bounded_decimal_maps_original_handle_cancellation_during_helper` at :3778. All four are traced to FR-109-AC-8, run through `admit_supplied_bounded_numeric`, and pass in the lib run. The shift-from-minimum-scale claim matches membership.rs:286-288. TC-920 keeps ownership of the kernel mechanism. The remaining overbroad wording is recorded as new FND-005 (low). |
| FND-004 | fixed | The union row now bounds only payload visits and scheduling by the retained position count: membership.rs:670-693 refuses a payload whose length differs from the position count, then reserves and schedules exactly that many. It states no descriptor cost bound and names the `OrderedIndex` binary-search lookups (declaration.rs:1896, union_handles/union_keys/union_members/composites) and the derived `UnionMember` equality as still to inspect. No bound is invented. |

Round 3, checked against `git diff 794a2f7..d1e859d` (head `d1e859d39f4da0e22d23602dae59cdf0af1ebe9f`; Markdown only). `quire validate` is clean for `spec/**`. `quire matrix --strict` still fails only on the pre-existing untagged FR-109-AC-6. FR-109-AC-1 is tagged by `tests/union_staging.rs:741` and `:772`. No change to `src/`, tests, Cargo or CI, and no criterion id or obligation removed.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-005 | fixed | FR-109 Status now limits the claim to non-ignored tests and names the ignored `supplied_path_arena_growth_probe_10k` (membership.rs:4023-4024), which is the only ignored FR-109-AC-1..5 binder in the matrix. TC-908 Status lists exactly the five files that hold FR-109-AC-1..5 binders, confirmed by grep: membership.rs, supplied_membership.rs, declaration.rs, union_staging.rs and bounded_numeric_reservation.rs. FR-109 Dependencies now describes the delivered IR-719 caller (IR-719 is Done). The explicit `magnitude_bits` with no default and the alignment shift bounded by the declared minimum scale both match membership.rs:282-298 and :388. The verbless sentence this edit left behind is recorded as new FND-006 (low). |


Round 4, checked against `git diff d1e859d..6010bd7` (head `6010bd7caf9f8170b68305a274719ec24e7bae04`). The diff is a one-word change at FR-109 line 162 and nothing else. `quire validate` is clean for `spec/**`. `quire matrix --strict` still fails only on the pre-existing untagged FR-109-AC-6.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-006 | fixed | The sentence is now grammatical ("... also execute."), and it is true. None of the four AC-8 Decimal controls is `#[ignore]`d: membership.rs:3526, :3619, :3730 and :3778 all passed in a `cargo test --lib decimal` run with 0 ignored. |
