# Done Certificate · Task 25: FeedState.refresh

**Task:** [25-feed_state_refresh.md](25-feed_state_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 25. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task
> carries the coverage for the whole client half of the refresh. It deliberately does **not** own
> the reader obligation both change specs assigned to whoever merges second: that sits at the
> control, in task 26, because the import would otherwise run backwards.

## Definition

DONE(Task 25) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** A new wall in place, with the outgoing one left on screen and no twelve-card initial
  grid, in a module vitest still runs for real in node.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break `reset`, `#warm`, `freeze`, `loadMore`, the generation
  bookkeeping that makes a superseded loop bow out, or the session cache's back/forward rehydration.

## Obligations

- **O1 · The outgoing wall stays on screen.**
  - *Claim:* after `refresh()` and before the first preview resolves, `items` is unchanged,
    `initialLoad` is false and `warming` is true.
  - *Evidence to collect:* run the named vitest case. Read `refresh()` and confirm it touches
    neither `items` nor `initialLoad`; `FeedGrid.svelte:222` shows the 12-card skeleton grid only
    when `initialLoad` is true, so setting it would make a refresh look like a failure.
  - *Status:* unverified

- **O2 · One fan-out per refresh, and the refresh reaches the wall that commits.**
  - *Claim:* at most one request per refresh carries the flag; the second cursorless request is
    **deferred rather than issued unflagged**, so the request that commits (the `"freeze"` one)
    either carries the flag or carries a non-null cursor onto the refreshing snapshot; and after the
    wall settles a later `reset` carries none.
  - *Evidence to collect:* run the three named cases. Read the guard and confirm it is two pieces of
    state, not one: an armed flag, disarmed when a cursorless response is adopted, when the wall
    settles (`freeze`'s `finally` with the generation matching) or on the next `reset`, **and** an
    in-flight marker, set by `refresh()` and cleared when the flagged request settles whether or not
    its page was adopted. Confirm the marker makes `freeze()` **return early with no side effect**,
    beside the existing guard at `feed.svelte.ts:124` and ahead of `#generation++` and
    `loading = true`; a freeze that bumps the generation before bowing out supersedes `#warm` and
    strands the wall. Read the race case and confirm it asserts the committed request, not only a
    count over the mock call list.
  - *Checks:* trace the race and price it. `refresh()` calls `#warm`, whose first `fetchFeed` is
    issued synchronously with `this.cursor === null`; `freeze()` at `feed.svelte.ts:121` proceeds
    while `warming` is true and `loading` is false, and its own request is cursorless too. Under
    reduced motion this is not a race but the default path: `FeedGrid.svelte:181`-`:187` is an
    `$effect` that calls `freezeOnEngage()` the instant `feed.warming` flips true, with no scroll
    event at all. Two cursorless **flagged** requests means `handle_feed` rolls two `fresh_seed`s,
    inserts two snapshots and spawns two fills, each bypassing `author_feed` and `image_feed`: two
    hundred-author fan-outs from one tap, against a spec whose only rate limit is the disabled
    button. But stripping the flag off the second one is the **other** failure, and it is the one a
    call-counting test cannot see: a cursorless unflagged request rolls its own seed, builds its own
    snapshot and spawns a fill that `sources/fetch.rs:143` answers from the still-valid five-minute
    `author_feed` entries before any network, so it clears `FIRST_PAINT_AUTHORS` at once and commits
    the **pre-refresh** wall while the flagged fill is still working through up to a hundred
    rate-limited AppView calls. Deferral is what avoids both: the held freeze returns early, the
    preview resolves, `#warm` adopts its cursor (which carries the seed, `feed.rs:90`) and freezes
    from there. If the implementation strips instead of defers, this obligation fails even with the
    count green.
  - *Status:* unverified

- **O3 · The session cache entry is dropped before the generation bumps.**
  - *Claim:* refresh, settle, then `reset` on the same target rehydrates the **new** wall; and a
    `reset` issued while the refresh is still warming does not resurrect the old one.
  - *Evidence to collect:* run both named cases. Read `refresh()` and confirm the `#cache.delete`
    happens before `#generation++`.
  - *Checks:* resolve the key passed to `#cache.delete`. It must be the same `#key(target, mode)`
    task 15 changed the shape of; a stale key spelling would delete nothing and pass silently.
  - *Status:* unverified

- **O4 · Refresh refuses when it must.**
  - *Claim:* `refresh()` returns without side effects while `loading`, while `warming`, or with no
    target, so a double call cannot start two fan-outs.
  - *Evidence to collect:* run the named case. Read the guard and compare it to `loadMore`'s at
    `feed.svelte.ts:155`.
  - *Checks:* resolve what "without side effects" covers. An early return that has already deleted
    the `#cache` entry or already bumped `#generation` is not a refusal; confirm the guard is the
    first statement in the method.
  - *Status:* unverified

- **O5 · `FeedState` touches no DOM, and does not reach one through an import either.**
  - *Claim:* `refresh()` performs no scroll and no DOM access, and `feed.svelte.ts` imports no
    reader, which together are what keep the module runnable in vitest's node environment.
  - *Evidence to collect:* run
    `grep -nE 'document|window|scrollTo|matchMedia|history' web/src/lib/state/feed.svelte.ts` and
    expect no hits. Then run `grep -n "reader" web/src/lib/state/feed.svelte.ts` and expect no hits
    either. Confirm `pnpm test` runs in node without a DOM shim for this file, and that
    `feed.test.ts` gained no `vi.mock('$app/navigation')` or `vi.mock('$app/state')`.
  - *Checks:* the first grep alone passes vacuously if the module calls `reader.close()`, because
    `reader.close()` is a `history.back()` one module away and the grep cannot see through an
    import. The second grep is the one that matters, and a new `$app/*` mock in `feed.test.ts` is
    the symptom that it was needed. Separately, `reader.svelte.ts` imports **this** module, so an
    import back is a cycle between two singletons; closing the reader on refresh is task 26's job,
    at the control, and its absence here is the obligation rather than a gap.
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci` and `just check`. Confirm
    `feed.test.ts:106`-`:110` and `:159` were updated for the new argument and `loadMore`'s
    assertion kept its length.
  - *Status:* unverified

- **O7 · Reviewable: vitest covers all of it.**
  - *Claim:* `cd web && pnpm test` runs the real module in node, and every claim above is an
    assertion rather than an inspection.
  - *Evidence to collect:* the test run, plus a count of the cases against O1 to O5.
  - *Status:* unverified

## Regression check

- `reset(target, mode)` rehydration from the session cache. Trace: lay `/?actor=demo`, navigate away,
  go back; same arrangement, no skeletons : (PRESERVED / REGRESSION)
- `#warm` and `freeze` generation bookkeeping. Trace: a superseded preview loop still bows out; the
  existing `feed.test.ts` cases pass : (PRESERVED / REGRESSION)
- `loadMore` always carries a cursor and does not take the flag. Trace: its recorded call tuple is
  still three elements : (PRESERVED / REGRESSION)
- Task 01's reader. Trace: `pnpm vitest run src/lib/state/reader.test.ts` green, and
  `web/tests/reader.test.ts` green under `just test-e2e` : (PRESERVED / REGRESSION)
- `feed.test.ts`'s module graph. Trace: it still mocks only `$lib/api`, still runs under
  `environment: "node"`, and still needs no `$app/*` mock : (PRESERVED / REGRESSION)

## Residue

- The live region is settled by the change spec rather than left to either task: the wall keeps its
  single polite region, a refresh is a warm, and `RefreshWall` adds none. So `FeedState` exposes
  nothing about this warm being a refresh, and a field added here for a refresh-aware branch in
  `FeedGrid` is a divergence rather than extra credit.
- Closing an open reader on refresh is task 26's obligation, discharged at the control. If the
  validator finds it here instead, that is the import cycle this certificate exists to keep out, not
  extra credit.
- In the freeze-beats-preview race the flagged preview is **not** superseded by an unflagged
  survivor: the marker holds the freeze back, the preview resolves, and the commit that follows rides
  the cursor `#warm` adopted from it. Record whether the module says in a comment why the second
  cursorless request is deferred rather than stripped of its flag, because the stripped version looks
  equivalent, is green on a call count, and commits the pre-refresh wall. Task 26 and task 27 both
  restate this, so a silent version of it invites the next reader to "simplify" the guard.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
