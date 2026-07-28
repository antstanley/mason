# Done Certificate · Task 25: FeedState.refresh

**Task:** [25-feed_state_refresh.md](25-feed_state_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26, re-validated 2026-07-27 (second pass, after the error-path retry)

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
  - *Collected:* `refresh > leaves the outgoing wall up, with no skeleton grid, while the new one
    warms` (`feed.test.ts:390`) run under `pnpm vitest run src/lib/state/feed.test.ts`: PASS. It lays
    a settled alice wall, wires the preview to a deferred that has not resolved, calls `refresh()`
    and asserts `warming` true, `initialLoad` false, `ids` still `["alice-1"]`, `cursor` null,
    `done` false, and one issued request `[{actor:"alice"}, null, undefined, "preview", true]`. It
    then resolves the preview and asserts the outgoing arrangement reflowed into `["fresh-1"]`.
    `refresh()` at `feed.svelte.ts:127`-`:146` writes `cursor`, `done`, `error`, `warming`, the two
    private refresh fields and `#generation`, and names neither `items` nor `initialLoad`.
    `FeedGrid.svelte:240` gates the twelve-card grid on `feed.initialLoad`, which stays false.
  - *Status:* SATISFIED

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
  - *Collected:* five cases now carry this obligation and all five PASS.
    `carries the flag on exactly one request across a preview and its freeze` (`:415`) pins the whole
    call list: the cursorless preview carries `true`, the commit rides the preview's cursor
    `"alice-p"` with `false`. `holds a freeze that beats the refresh preview, and commits on the
    refreshed wall` (`:489`) is the race. `cannot leak the flag into the next wall` (`:524`) settles a
    refresh, clears the call log and shows `reset` to bob issuing no flagged call. The two error-path
    cases added by the second pass, `still commits, and flags nothing twice, when the refresh preview
    fails` (`:432`) and `sends one flagged request when the reader engages and the refresh preview
    then fails` (`:462`), both assert over the whole call list rather than by counting: exactly
    `[preview, null, true]` then `[freeze, null, false]`, and `flagged()` equal to the single preview
    call.
    The guard is two fields, not one. `#refreshPending` (`feed.svelte.ts:53`) is armed at `:142` and
    disarmed at FOUR points: on adopt in the poll loop (`:171`), in `#warm`'s catch once the flagged
    request has settled by throwing (`:211`), in `freeze`'s `finally` under a matching generation
    (`:279`), and at the top of `reset` (`:93`). `#refreshInFlight` (`:57`) is set at `:143` and
    cleared on adopt (`:172`), in `#warm`'s catch (`:195`) and in `reset` (`:94`). The marker check
    sits at `freeze`'s line `:250`, after the existing guard at `:228` and BEFORE
    `const gen = ++this.#generation` at `:253` and `this.loading = true` at `:254`, so the held commit
    mutates nothing. The race case asserts the committed request at `feed.test.ts:521`:
    `expect(committed[0]?.[4] === true || committed[0]?.[1] != null).toBe(true)`.
  - *Checks:* traced. `refresh()` calls `#warm`, whose first `fetchFeed` is issued synchronously with
    `this.cursor === null` and `flagged` true (`feed.svelte.ts:164`-`:166`), which the test observes
    as one recorded call before any timer advance. `freeze()` called while that request is in flight
    passes the `:228` guard (target set, warming true, loading false, generation matching) and returns
    at `:250` with `loading` still false and the generation unbumped. When the preview resolves,
    `#warm` adopts its cursor, both fields disarm, and the commit goes out as
    `[{actor:"alice"}, "fresh-p", undefined, "freeze", false]`: a non-null cursor onto the refreshing
    snapshot. `fetchFeed` resolves to the import at `feed.svelte.ts:1` from `$lib/api`, whose fifth
    positional parameter is `refresh`; no shadowing, and the local `flagged` bindings at `:164` and
    `:263` shadow nothing at module or class scope.
    The error path was traced and then priced. Trace: `refresh()` arms both fields and nulls the
    cursor; the poll issues `[target, null, undefined, "preview", true]`; the promise rejects;
    `#warm`'s catch passes the generation check at `:188`, sees the marker still set at `:189`, and
    clears the marker (`:195`) and the flag (`:211`) together with no `await` between them; the
    commit at `:215` then reaches `freeze`, is not held, reads `flagged` false at `:263` and goes out
    as `[target, null, undefined, "freeze", false]`. Pricing, in this second pass, by mutation with
    the file restored by checksum afterwards (`f6ce452e6050...`): deleting only
    `this.#refreshPending = false;` from that catch block makes exactly the two error-path cases FAIL,
    27 passed, and the recorded second call is `[..., "freeze", true]`, the two-flagged-cursorless
    shape itself. Re-running the strip-the-flag mutation of the first pass (replace the hold at `:250`
    with a `held` local and let the freeze proceed with the flag stripped) still makes the race case
    and the second error-path case FAIL. The first pass additionally ran that mutation with the
    "held, not sent" assertions blinded and the race case still failed on the committed-request
    assertion alone (`expected false to be true`), and the second pass changed no line of that case,
    so the half that matters still holds the obligation without riding a count. The invariant the
    fourth disarm point buys is `#refreshPending` implies `#refreshInFlight`: the flag is armed only
    at `:142`, beside the marker at `:143`, and every clear of the marker (`:94`, `:172`, `:195`)
    clears the flag in the same straight-line block, so the read at `:263` cannot be true while the
    hold at `:250` lets a commit through. `freeze`'s read is therefore a backstop rather than a live
    path, which the comment at `:256`-`:262` says in those words.
  - *Status:* SATISFIED

- **O3 · The session cache entry is dropped before the generation bumps.**
  - *Claim:* refresh, settle, then `reset` on the same target rehydrates the **new** wall; and a
    `reset` issued while the refresh is still warming does not resurrect the old one.
  - *Evidence to collect:* run both named cases. Read `refresh()` and confirm the `#cache.delete`
    happens before `#generation++`.
  - *Checks:* resolve the key passed to `#cache.delete`. It must be the same `#key(target, mode)`
    task 15 changed the shape of; a stale key spelling would delete nothing and pass silently.
  - *Collected:* both cases run and PASS. `drops the session entry, so coming back finds the
    refreshed wall` (`feed.test.ts:537`) refreshes to `["fresh-2","fresh-1"]` / cursor `"fresh-c"`,
    navigates to bob and back to alice, and asserts the rehydration is the refreshed wall with no
    further request. `does not resurrect the outgoing wall when a reset lands mid-refresh` (`:558`)
    calls `refresh()` then `reset({actor:"alice"})` while the flagged request is still in flight, and
    asserts `initialLoad` true with `items` empty (laid afresh, not rehydrated), then that the
    superseded refresh landing last changes nothing. In `refresh()` the delete is at
    `feed.svelte.ts:133` and `const generation = ++this.#generation` at `:144`, so the drop precedes
    the bump.
  - *Checks:* the key resolves to `this.#key(target, this.#mode)`, the same private method at
    `feed.svelte.ts:69` that `#save` (`:78`) and `reset` (`:95`) use, with task 15's
    kind-plus-unit-separator shape; `target` is the captured `this.#target` and the mode is the
    instance's, so the spelling matches the entry `#save` wrote. Priced again in this second pass:
    deleting the `#cache.delete` line makes `does not resurrect the outgoing wall when a reset lands
    mid-refresh` FAIL at `feed.test.ts:572` (`expected false to be true`, the old arrangement
    rehydrated), 28 passed, and nothing else fails. Restored by checksum.
  - *Status:* SATISFIED

- **O4 · Refresh refuses when it must.**
  - *Claim:* `refresh()` returns without side effects while `loading`, while `warming`, or with no
    target, so a double call cannot start two fan-outs.
  - *Evidence to collect:* run the named case. Read the guard and compare it to `loadMore`'s at
    `feed.svelte.ts:155`.
  - *Checks:* resolve what "without side effects" covers. An early return that has already deleted
    the `#cache` entry or already bumped `#generation` is not a refusal; confirm the guard is the
    first statement in the method.
  - *Collected:* both cases run and PASS. `refuses a second refresh while the first is still warming`
    (`feed.test.ts:583`) calls `refresh()` three times against a pending preview and asserts one
    request and one flagged request, so a double tap cannot start two fan-outs. `refuses with no wall
    at all, and while a page is in flight` (`:601`) covers the other two rejection paths: a
    `FeedState` nothing was ever reset on issues no request, and a `refresh()` during `loadMore`
    leaves `warming` false and the cursor at `"alice-c"`, with the page that was in flight landing
    normally afterwards.
  - *Checks:* the guard is `if (this.loading || this.warming || !target) return;` at
    `feed.svelte.ts:129`, the first statement after the `const target` capture at `:128` and ahead of
    the `#cache.delete` at `:133` and the generation bump at `:144`, so a refusal mutates nothing. It
    mirrors `loadMore`'s at `:294` minus `done`, which the doc comment at `:124`-`:126` explains.
  - *Status:* SATISFIED

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
  - *Collected:* `grep -nE 'document|window|scrollTo|matchMedia|history'
    web/src/lib/state/feed.svelte.ts` returns nothing (exit 1). The narrower
    `grep -nE "reader\.|from ['\"].*reader"` on the same file returns nothing (exit 1), so there is
    no import of `./reader.svelte` and no member access on a reader; the plain `reader` grep matches
    only prose in comments about the human reader. The file's only imports are `$lib/api` and
    `$lib/types` (`:1`-`:2`). `feed.test.ts` still carries exactly one `vi.mock`, for `$lib/api`
    (`:11`), and gained no `$app/*` mock; `web/vitest.config.ts:11` still sets `environment: "node"`,
    and the suite runs green there with no DOM shim, so a DOM global would throw when `refresh()`
    actually ran. The test lane also pins both halves itself: `names no DOM global and imports no
    reader` (`feed.test.ts:623`) reads the module off disk and asserts no `from "...reader.svelte"`
    and no match for the DOM identifier alternation.
  - *Status:* SATISFIED

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci` and `just check`. Confirm
    `feed.test.ts:106`-`:110` and `:159` were updated for the new argument and `loadMore`'s
    assertion kept its length.
  - *Collected:* re-run in the task's workspace by the validator on the second pass, not read from the
    report. `cd web && pnpm test`: 7 files, 130 tests, all passed. `cd web && pnpm check:ci`: exit 0
    (`svelte-kit sync` plus `tsc --noEmit` on both projects). `just check`: exit 0, covering
    `guard-dashes`, `guard-autoplay`, `guard-toolchain`, `fmt-check`, `guard-wasm`, lint (oxlint,
    knip, clippy) and test (154 Rust, 130 vitest). The four oxlint warnings printed are pre-existing,
    three `no-await-in-loop` in `FeedGrid.svelte` and one `preserve-caught-error` in
    `service-worker.ts`, neither file touched by this diff. The three-call array is now
    `feed.test.ts:115`-`:119` with `false` appended to each tuple, and the `toHaveBeenLastCalledWith`
    is now `:168`-`:174` with a fifth `false`; `loadMore`'s assertion at `:296` is still
    `[{ actor: "alice" }, "c2", undefined]`, three elements, and green. No changeset was added, which
    is right: `refresh()` has no caller yet, so nothing a reader can do has changed, and task 27's own
    definition of done owns the minor changeset for this spec. The commit message drafted for this
    work carries the statement the second pass asked for, a paragraph naming the error path as a
    correction to the specification and as a FOURTH disarm point the change spec's three do not cover.
  - *Status:* SATISFIED

- **O7 · Reviewable: vitest covers all of it.**
  - *Claim:* `cd web && pnpm test` runs the real module in node, and every claim above is an
    assertion rather than an inspection.
  - *Evidence to collect:* the test run, plus a count of the cases against O1 to O5.
  - *Collected:* the Reviewable action was exercised on this pass: `cd web && pnpm test` in
    `/Users/ant/code/mason-ws2`, 130 tests green, `src/lib/state/feed.test.ts` contributing 29 (18
    pre-existing, 11 new under `describe("refresh")`). The change is entirely `.ts`, so this lane sees
    all of it, and the eleven cases map onto the obligations: O1 one case, O2 five (one flagged
    request, the freeze-beats-preview race, the leak, and the two error-path cases), O3 two, O4 two,
    O5 one. Every claim above is an assertion in that lane rather than an inspection, except the
    code-ordering reads named in O2, O3 and O4, which are inspections by design and which three
    mutations priced independently on this pass.
  - *Status:* SATISFIED

## Regression check

- `reset(target, mode)` rehydration from the session cache. Trace: lay `/?actor=demo`, navigate away,
  go back; same arrangement, no skeletons : PRESERVED. `reset` gained only the two field resets at
  `feed.svelte.ts:93`-`:94`, ahead of the cache lookup and touching nothing the cache branch reads;
  the session-cache cases in `feed.test.ts` and the new `drops the session entry` case both show a
  rehydration with no further request.
- `#warm` and `freeze` generation bookkeeping. Trace: a superseded preview loop still bows out; the
  existing `feed.test.ts` cases pass : PRESERVED. All 18 pre-existing cases pass, `a failed preview
  still commits the wall through the freeze` among them, which is the unrefreshed twin of the new
  error-path cases. `#warm`'s catch was rewritten from
  `if (generation === this.#generation) await this.freeze(...)` to an early return on mismatch plus
  the same call, which is the same branch inverted, and the disarm block it now wraps is itself
  guarded on the marker, so on an unrefreshed wall it is skipped entirely and the catch behaves
  exactly as before. On such a wall both fields are false, `flagged` is false everywhere and the
  marker guard at `:250` never fires.
- `loadMore` always carries a cursor and does not take the flag. Trace: its recorded call tuple is
  still three elements : PRESERVED. `loadMore` is unchanged, its `fetchFeed` call at `:299` still
  passes three arguments, and `feed.test.ts:296` still asserts a three-element tuple, green.
- Task 01's reader. Trace: `pnpm vitest run src/lib/state/reader.test.ts` green, and
  `web/tests/reader.test.ts` green under `just test-e2e` : PRESERVED. The unit lane is green inside
  the 130. `CI=1 just test-e2e` re-run on this pass (its own server on 4173 with `--strictPort`, so it
  cannot attach to another workspace's build) is 20 tests green across the three specs,
  `reader.test.ts` among them.
- `feed.test.ts`'s module graph. Trace: it still mocks only `$lib/api`, still runs under
  `environment: "node"`, and still needs no `$app/*` mock : PRESERVED. The one new import is
  `node:fs`, for the source-text assertions in the last case, which the node environment provides.

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

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED with evidence collected on this second pass, every regression line
is PRESERVED, and the three failure modes this certificate exists to price (two flagged cursorless
requests, a stripped flag on the commit, and a surviving session entry) were each bought with a
mutation that made the named cases fail and was then restored by checksum.

Recorded for the Residue, which asked. The module does say in a comment why the second cursorless
request is deferred rather than stripped: `feed.svelte.ts:230`-`:249`, twenty lines directly above the
marker guard, naming the fresh seed, the second snapshot, the untouched five-minute author-feed cache,
the twelve-author first-paint gate, the hundred rate-limited calls, why flagging both is not the
alternative, and why `prefers-reduced-motion: reduce` makes this the ordinary path rather than a race.
`#refreshPending` and `#refreshInFlight` carry their own doc comments at `:45`-`:57`.

The first pass recorded two observations, neither an obligation. The first is now closed. It read: when
a refresh's own preview THROWS, the flag was still armed and the cursor still null, so the commit that
followed carried the flag too, two flagged cursorless requests under one tap, sequentially. The second
pass added a fourth disarm point in `#warm`'s catch (`feed.svelte.ts:189`-`:212`), which clears the
flag in the same guarded block that releases the marker, and two vitest cases that assert over the
whole call list rather than by counting (`feed.test.ts:432` and `:462`). Deleting only that one line
makes both cases fail and reproduces the two-flagged-cursorless call list exactly, so the closure is
priced rather than asserted. The price of the closure is named in the comment and is the cheaper half:
a refresh whose first request never answers commits an unrefreshed wall, and the control is live again
the moment that commit settles, so asking again costs one more tap rather than a second fan-out.

The second observation stands and is still not a defect: a held freeze is not re-issued the moment the
marker clears; `#warm` freezes when the snapshot settles or the ceiling hits, which is what this task's
step 5 specifies word for word, so a reduced-motion reader can see the wall reflow more than once
during a refresh rather than the single reflow the prose in step 6 pictures. The second pass recorded
the consequence in the hold comment (`feed.svelte.ts:245`-`:249`) rather than changing the behaviour,
which is what the retry asked for. Worth a sentence in task 26's or 27's reading, not a defect here.

One note for whoever merges the spec (task 27). With the fourth disarm point in place, `#refreshPending`
implies `#refreshInFlight` on every path, so `freeze`'s own read of the flag at `feed.svelte.ts:263` can
no longer evaluate true. It is kept because the change spec's step 9 asks both cursorless requests to
read the flag, and the comment at `:256`-`:262` labels it a backstop rather than a live path so it is
neither mistaken for one nor quietly deleted. The change spec itself still names three disarm points
and has not been amended; the sentence it needs is that a flagged cursorless request settling WITHOUT
being adopted, a throw included, spends the flag too.
