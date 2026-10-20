---
id: SR-13407
title: "IR-882 quire-semantic-value PR 51 code review: planned_equality_with_plan in the CheckedEquality plan arm"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-semantic-value@c7c38d4e1f87c24d0bfe3eb5a7965133b5342537 vs origin/main 9fd1375; src/declaration.rs, Cargo.lock"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-semantic-value/FR-113
    type: reviews
---

## Summary

Ticket: IR-882. This is a code review with the rust-review lane folded in. The diff
(`git diff origin/main...c7c38d4`) is one commit with three hunks:

- `src/declaration.rs:5019-5023`. In `CheckedEquality::run`, the `EqualitySchedule::Plan` arm now calls
  `outcome_into_stop(quire_exact::planned_equality_with_plan(&left, &right, meter))?.equal()`.
  It used to call `outcome_into_stop(quire_exact::planned_equality(..))?`.
- `src/declaration.rs:22`. The module doc now names `planned_equality_with_plan`.
- `Cargo.lock`. quire-exact moves from d18927e to 1c6cbcb6 (`cargo update -p quire-exact` only).

The equivalence claim was checked independently, at three levels:

1. **Source.** In quire-exact at 1c6cbcb6, `planned_equality` is a `match` over
   `planned_equality_with_plan(..)`. It maps `Completed(r)` to `Completed(r.equal())` and passes
   `Undefined`, `Refused` and `Incomplete` through unchanged. `planned_equality_with_plan` is
   `Outcome::from_stop(planned_equality_stop(..))`. Between d18927e and 1c6cbcb6, `src/equality.rs`
   changed only in `#[cfg(test)]` code: TC-932 trace tags and a strengthened charge-prefix test.
   `outcome_into_stop` maps the same `Outcome` variants to the same `Stop`s, whatever the payload type.
2. **Kernel differential** (scratch test, not part of the PR). The test builds 59 values through
   QSV's checked constructors: integers, booleans, `Shape` unions (Rect/Circle/Empty with reordered
   payloads), records with required and optional slots, tuples, sequence collections of 0 to 2
   tuples, and `Option` of collection. Every ordered pair was run, cross-kind pairs included, which
   gives `ValueKindMismatch` refusals. Each pair ran under these meters:
   - an unlimited meter;
   - every `work_units` limit from 0 to W+1;
   - every `value_occurrences` limit from 0 to V+1;
   - `result_units` 0 and 1;
   - an injected denial at every occurrence of `EqualityPlanForm`, `EqualityPlan`, `EqualityPair`
     and `EqualityResultRetain`.

   On each run, `planned_equality` and `planned_equality_with_plan(..).equal()` were compared on
   their `Outcome` Debug output and on the full meter state: admission count, all ten counters and
   the `test-support` ordered charge log. The two agreed on all 148,499 runs.
3. **End-to-end differential.** The test dumps `CheckedEquality::evaluate` traces for every
   same-typed pair, both operators and all of the meter configurations above: 91,638 lines,
   including 46,574 `Incomplete` lines at four distinct charge points. The dump was taken in three
   trees:
   - PR head;
   - origin/main's `declaration.rs` (the old `planned_equality` call) on the new lock;
   - PR head on the old d18927e lock.

   All three dumps are byte-identical (sha1 dcce8646). Verdicts, `Incomplete` charge points and
   charge schedules are unchanged, and the lock bump changes nothing that QSV can observe.

The rest of the lock bump was audited too: twelve quire-exact commits, d18927e to 1c6cbcb6.

- **`NonZeroInteger` division admission (#49).** `exact_div`, `div_rem_truncating` and
  `div_mod_floor` now take `NonZeroInteger`. This is a breaking public signature change. QSV calls
  none of the three.
- **Single-pass `Meter::charge` (#52).** The first failure is still the earliest field kind and,
  within that kind, the first request inserted. High-water maxima and cumulative work/result totals
  are unchanged. The test-support record is the same per-kind maximum.
- **`ChargePoint::TemporalVisit` (#54).** It is appended last, so `ALL` grows from 62 to 63 and
  `as_str` gains "temporal.visit". The enum is not `#[non_exhaustive]`, and QSV has no exhaustive
  match over it.
- **`Meter::cancel()` accessor (#59).** Additive.
- **Equality tests (#57) and CI, README and review files.** No behaviour change.

No canonical bytes, `Value` types, error causes or limits change.

QSL (agent-ix/quire-spec-language) main is 70c7092b. Its Cargo.lock pins quire-exact at 2134917
(#52) and QSV at 48e1bc3, and it holds one quire-exact entry. QSV and QSL both declare quire-exact
as the same source, `git = ".../quire-exact", branch = "main"`. Cargo therefore unifies it to the
one rev in QSL's own lock, and QSV's Cargo.lock is never read downstream. A QSL lock bump of QSV
alone cannot produce two quire-exact revs in one build. QSV at this head needs only a quire-exact
that exports `planned_equality_with_plan`, which has been true since 10b628d (#34). The full QSV
suite at head passes with quire-exact pinned to QSL's 2134917.

Gates at head, run in a reviewer clone with its own target dir:

- `make ci` passes: fmt-check, clippy on host and thumbv7em, test, no_std build, deny and
  `-D warnings` docs.
- tests/union_registry.rs passes 16 tests and tests/union_staging.rs passes 15. The lib unit suite
  passes 137, with 1 pre-existing ignored resource probe.

There is no compatibility layer and no new public item.

Commit hygiene:

- The author and committer are `Peter Krenesky <kreneskyp@gmail.com>`, set through env vars because
  gitgh's git config has no identity. This matches every commit on origin/main.
- The message ends with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. That trailer
  appears on 10 of the last 20 main commits, and CONTRIBUTING.md, AGENTS.md and CLAUDE.md carry no
  rule against it. The PR body ends with the Claude Code attribution line.

## Reviewed units

- src/declaration.rs:5019-5023 (examined): `(EqualitySchedule::Plan, _, _) => { let equal = outcome_into_stop(quire_exact::planned_equality_with_plan(&left, &right, meter))?.equal(); return Ok(equal == (self.operator == EqualityOperator::Equal)); }`
- src/declaration.rs:20-25 (examined): "The occurrence-pair walk it schedules is [`quire_exact::planned_equality_with_plan`]/[`quire_exact::plan_equality`], called directly by `CheckedEquality::run`: the kernel's own `plan_pairs` is `pub(crate)` there, and this layer never needs the lower-level pair count that an evaluator gets straight from `quire_exact::member_equal`."
- Cargo.lock:156 (examined): quire-exact `source = "git+https://github.com/agent-ix/quire-exact?branch=main#1c6cbcb6d523fe1e07f05510918b00047a48233e"` (was d18927e).
- Commit c7c38d4 message and PR #51 body (examined): "Bumps the quire-exact lock to main 1c6cbcb, which carries planned_equality_with_plan." / "the old rev lacked `planned_equality_with_plan`".
- FR-113 Behavior 13 (context_only): "whether two composites of different declarations are equal belongs to the kernel's composite equality".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The commit message and PR body give a false rationale for the lock bump. They say d18927e "lacked `planned_equality_with_plan`", but quire-exact has defined and re-exported it since 10b628d (#34). That commit is an ancestor of d18927e, where `src/lib.rs:120` exports `planned_equality_with_plan`. The PR head builds and passes on the d18927e lock, with byte-identical evaluate traces. So the bump is not required by this change. It pulls in twelve unrelated quire-exact commits, which include a breaking public signature change (#49, `NonZeroInteger` divisors), all verified inert for QSV. The merge-commit history will keep the wrong claim. Fix: correct the commit or PR text. Either keep the bump as a deliberate refresh to current main, or drop the Cargo.lock hunk. | Cargo.lock:156 |
| FND-002 | low | The edited doc line still says `quire_exact::plan_equality` is "called directly by `CheckedEquality::run`". QSV has no call to `plan_equality`: the only kernel equality call in `src/` is the `planned_equality_with_plan` call at line 5020. This is pre-existing on main, but this PR rewrote the same line and kept the claim. Fix: drop "/[`quire_exact::plan_equality`]" or reword it. | src/declaration.rs:22-23 |

## Verdict

The behaviour change is correct and its equivalence claim holds: verdicts, `Incomplete` charge
points, meter charge schedules and error causes are identical by source reading, by a 148,499-run
kernel differential and by a byte-identical 91,638-line end-to-end differential across old call,
new call and old lock. The lock bump is behaviour-neutral for QSV. It cannot skew quire-exact
across QSL's graph, because QSL resolves its own single branch=main entry. Both findings are text
accuracy only, with no runtime effect. Mergeable as-is; FND-001 and FND-002 are optional wording
fixes.
