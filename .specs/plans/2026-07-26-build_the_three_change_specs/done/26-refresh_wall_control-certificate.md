# Done Certificate · Task 26: RefreshWall control

**Task:** [26-refresh_wall_control.md](26-refresh_wall_control.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27

> Verification protocol for Task 26. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. `just check`
> does not cover this task's deliverable at all.

## Definition

DONE(Task 26) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** One header button that closes any open reader and lays the wall again in place,
  disabled while one is in flight.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break the header row's no-wrap constraint at 375px, the three
  controls already in it, task 03's inert wrapper, `FeedGrid`'s warming reflow, task 01's reader
  bookkeeping (the private did-push flag that decides `history.back()` versus `replaceState`), or
  task 25's `FeedState` staying free of DOM and reader imports.

## Obligations

- **O1 · The disabled state is real, because it is the rate limit.**
  - *Claim:* `RefreshWall.svelte` is a plain `<button type="button">` with an accessible name,
    `min-h-11`, and a real `disabled` attribute bound to `feed.loading || feed.warming`, not a
    styled-off look.
  - *Evidence to collect:* read the component. In the browser, inspect the element while the wall is
    warming and confirm the DOM carries `disabled`, and that a screen reader announces it.
  - *Checks:* resolve `feed` to the singleton from `$lib/state/feed.svelte`. A refresh costs one
    hundred-author AppView fan-out and there is no server-side throttle by design, so a stale or
    styled-only disabled turns a double tap into two bursts.
  - *Status:* SATISFIED. `RefreshWall.svelte:115`-`:123` is a plain `<button type="button">`
    carrying `disabled={busy}`, with `busy = $derived(feed.loading || feed.warming)` at `:28`,
    `min-h-11` in the class list at `:119`, and its accessible name in an `sr-only` span at `:122`
    (the icon is `aria-hidden`, from `Icon.svelte:58`). `feed` at `:28` and `:93` resolves by import
    (step 4) to the `feed` singleton exported from `$lib/state/feed.svelte`, not to
    `+layout.svelte`'s local `feed` route parameter, which is a binding in another module: no
    shadowing. Measured in chromium against the built site at 375 by 812, on a preview server of my
    own on port 4291 so no other workspace could answer: before the press the node reports
    `hasAttribute("disabled") === false`; two microtask turns after the press it reports
    `hasAttribute("disabled") === true` and `disabled === true`, with no `aria-disabled` anywhere;
    once the wall settled the attribute was gone from the same node. So it is the platform's own
    disabled state on a native button, which is what a screen reader announces, and not a look. One
    honest note on the evidence: the in-flight window on the demo wall is a few milliseconds, so the
    CDP accessibility tree was read after the wall settled (`{role: button, name: "lay this wall
    again"}`) and the disabled state inside the window was read from the DOM attribute instead.

- **O2 · The handler does not scroll, and both the reason and the reduced-motion path are commented.**
  - *Claim:* the click handler contains **no scroll call of any kind**; the resolution is written in
    the file as a comment covering the symmetric event-delivery coupling **and** the reduced-motion
    path; and the absence is verified by **reading**, because no Playwright case on the demo wall can
    discriminate between the orderings (`feed.rs:60` answers a demo preview with `warming: false`, so
    `#warm` freezes on its first poll in every one of them).
  - *Evidence to collect:* read the handler and confirm it contains no scroll call of any kind, and
    that the comment explains why. Do NOT attempt a Playwright case that distinguishes reflow from
    immediate freeze: `feed.rs:60` answers a demo preview with `warming: false`, so `#warm` freezes
    on its first poll on the demo wall in every ordering, and that assertion has no discriminator.
  - *Checks:* trace **two** chains, not one. First, the scroll coupling, which is symmetric and is
    why the handler must not scroll: `FeedGrid.svelte:182` returns early when `!feed.warming`, so a
    settled wall has no listener to freeze against; `refresh()` then flips `warming` true
    synchronously, the effect re-runs on a microtask and attaches the `once` scroll listener at
    `:191`, and the event queued by `window.scrollTo` arrives after it. `freeze()` at
    `feed.svelte.ts:124` rejects only on `!warming || loading`, so either order commits without
    reflowing. A handler containing any scroll call fails this obligation. Second,
    `FeedGrid.svelte:182`-`:187`
    is a different path entirely: under `prefers-reduced-motion: reduce` the same effect calls
    `freezeOnEngage()` at `:185` immediately and returns, with no listener and no scroll event, so
    **no** scroll-side resolution can reach it. Confirm the comment states what a reduced-motion
    refresh actually is, read out of task 25's code rather than assumed: that freeze is **deferred**
    by task 25's in-flight marker while the flagged cursorless preview is in flight, so exactly
    **one** cursorless request goes out; when the preview lands, `#warm` has adopted its cursor
    (which carries the refreshing snapshot's seed) and freezes from there, so the committed request
    is on the refreshed wall. One refreshed fan-out, and one reflow when the preview lands. A comment
    that describes only the listener chain is incomplete, and one that says "two cursorless requests,
    one of them flagged" describes the mechanism this plan replaced, in which the unflagged request
    commits the pre-refresh wall from warm caches; task 27 copies whichever into `08`.
  - *Status:* SATISFIED, with one deviation recorded and checked rather than waved through. The
    handler is two statements: `if (reader.isOpen) reader.close();` at `:59` and `feed.refresh();`
    at `:93`. `grep -nE 'scrollTo|scrollIntoView|scrollBy|scroll\(|window\.scroll|location\.hash'`
    over the file matches `:61` and `:67` only, both inside the comment that explains the absence,
    and the handler itself contains no call of any kind besides those two. Chain one, the scroll
    coupling, is at `:61`-`:81` and is stated in full: `refresh()` flips `warming` synchronously,
    `FeedGrid`'s effect re-runs on a microtask and arms its `{passive, once}` scroll listener, and
    the event `window.scrollTo` queues is delivered after that microtask, so the coupling was event
    delivery rather than call order. Chain two, the reduced-motion path, is at `:83`-`:92` and I
    checked it against the code rather than against plausibility: `FeedGrid.svelte:181`-`:187` calls
    `freezeOnEngage()` (which is `() => void feed.freeze()`) the instant `warming` flips true, with
    no listener attached; `feed.svelte.ts:250` is `if (this.#refreshInFlight) return;`, so that
    freeze is held with no side effect; the marker is set in `refresh()` at `feed.svelte.ts:143` and
    spent only when the flagged preview is adopted at `:164`-`:176`, where `this.cursor =
    page.cursor` takes the refreshing snapshot's seed; and `feed.test.ts:489` ("holds a freeze that
    beats the refresh preview, and commits on the refreshed wall") pins exactly that shape, one call
    held with `loading` still false, one flagged request, and a commit that carries either the flag
    or a cursor. So the comment's account holds: exactly one cursorless request, and one reflow when
    the preview lands. Neither rejected account appears anywhere in the file: it never says two
    cursorless requests, and it never says no reflow. THE DEVIATION: chain one is written in the past
    tense, followed by the statement that the in-flight marker removed that coupling. I traced the
    counterfactual rather than accepting the claim: with a scrolling handler on this tree the queued
    `scroll` event would reach `freezeOnEngage` after the microtask, `freeze()` would find
    `#refreshInFlight` set and return early, and nothing would commit. The past tense is the accurate
    tense for this tree, and the mechanism the task file asks to be stated is stated.

- **O3 · The Playwright spec asserts all four behaviours at 375px, with a named mechanism for the disabled window.**
  - *Claim:* `web/tests/refresh.test.ts` asserts the control renders with an accessible name at a
    375px viewport; clicking it leaves bricks on the wall (`#wall article` stays visible, the
    twelve-card `initialLoad` grid never appears, while the four-card warming tail does); the control is `disabled` while warming and enabled once settled;
    and a second click while disabled starts nothing.
  - *Evidence to collect:* run `just build && cd web && pnpm test:e2e` and confirm the four
    assertions pass. Read the viewport setting. Read how the disabled assertion observes the window.
  - *Checks:* resolve how long the window is open. `feed.rs:60` answers a demo preview with
    `warming: false`, so `#warm` freezes on its first poll and `feed.warming` is true for roughly two
    service-worker round trips: an assertion that merely awaits `toBeDisabled()` after the click is
    racing a window that may already have closed. Confirm the spec uses the one mechanism that can
    see it: a `page.evaluate` that clicks and reads the `disabled` property in the same evaluated
    function, before yielding. Confirm it does **not** try to hold the window open with
    `context.route`, and that the file says why: `service-worker.ts:290`-`:296` answers `/api/feed`
    with `event.respondWith(serveFeed(...))` out of wasm, and on the demo wall the bricks are
    fixtures compiled into the binary, so there is no network request to delay and a page-to-worker
    request is not routable either way. Run the spec twice; a pass that is not reproducible is not
    evidence for the control that **is** the rate limit.
  - *Status:* SATISFIED. `web/tests/refresh.test.ts:42` is `test.use({ viewport: { width: 375,
    height: 812 } })` at file scope, so all three cases run at that width. Case 1 (`:143`) asserts the
    accessible name (the locator is `getByRole("button", { name: "lay this wall again" })`),
    `tagName === "BUTTON"`, `type="button"`, a rounded height of at least 44px, and zero sideways
    overflow on both the document and the control row. Case 2 (`:176`) installs a MutationObserver
    over `#wall` before the click and then asserts `minCards > 0` and `maxSkeletons === 4`. Both
    assertions can actually fail, which I checked rather than assumed: `#wall` is
    `+page.svelte:37`'s `<main>`, the twelve-card `initialLoad` grid renders inside it, and
    `SkeletonCard`'s root is a `div.animate-pulse` rather than an `article`, so a collapse to the
    grid would drive `minCards` to 0 and `maxSkeletons` to 12. Case 3 (`:223`) is the disabled window
    and the second tap, read synchronously: one `page.evaluate` finds the control by its accessible
    text, records `!button.disabled`, clicks, drains two microtask turns and nothing else, reads
    `button.disabled`, clicks again and reads it once more. A service-worker response is a task, so
    it cannot land inside that function. `context.route` appears nowhere in the file, and `:209`-
    `:219` says why: the worker answers `/api/feed` out of wasm over compiled-in fixtures, so there
    is no request to delay, which `service-worker.ts:301`-`:303` confirms
    (`event.respondWith(serveFeed(event.request))`). "Starts nothing" is counted with a
    `window.fetch` wrapper installed from an init script: exactly two `/api/feed` requests across the
    refresh, exactly one carrying `refresh=1`. My runs, all with `CI=1` so Playwright started its own
    server rather than attaching to another workspace's: `CI=1 just test-e2e` (35 passed, no retries
    consumed), then against that same build `CI=1 playwright test tests/refresh.test.ts --retries=0
    --repeat-each=3` (9 passed) and `--retries=0 --repeat-each=4 --workers=1` (12 passed). Twenty-one
    consecutive clean runs of the three cases with retries switched off: the disabled-window
    assertion did not flake once.

- **O4 · The spec says what it does not cover, and no live region was added.**
  - *Claim:* the spec file notes that the demo wall ignores `refresh` in the engine, so this lane
    covers the client behaviour and not the re-read; and `RefreshWall` announces nothing of its own,
    with the reason recorded in the file.
  - *Evidence to collect:* read the spec's header comment and the component's recorded decision. Run
    `grep -rn 'aria-live' web/src/lib/components/` and confirm the only hit is `FeedGrid.svelte:221`.
  - *Checks:* this is settled by the change spec rather than open to the implementer: the `08`
    Refreshing block says the wall keeps its single polite region, that a refresh is a warm and so
    needs no new announcement, and that `RefreshWall` adds no region of its own. A second region, or
    a refresh-aware branch in `FeedGrid` (which would need `FeedState` to expose that this warm is a
    refresh, a field task 25 does not add), is a divergence from the block and from
    `08-wall-and-bricks.md`'s accessibility section, which states there is exactly one region for the
    whole wall.
  - *Status:* SATISFIED, with one certificate drift noted and validated against the DoD instead.
    `grep -n 'aria-live' web/src/lib/components/RefreshWall.svelte` exits 1 with no output. Across
    `web/src` there are exactly four `aria-live` attributes, and none of them is new: the wall's
    single polite region (`FeedGrid.svelte:239`), the feed picker's (`FeedPicker.svelte:192`), the
    reader dialog's (`BrickReader.svelte:505`) and the glaze carousel's (`GlazeCard.svelte:203`).
    `FeedGrid.svelte` is not in the diff at all (`jj diff --stat` lists six files and it is not one of
    them) and `grep -n refresh web/src/lib/components/FeedGrid.svelte` exits 1, so it gained no
    refresh-aware branch and the count could not have grown. The decision is recorded in the
    component at `RefreshWall.svelte:96`-`:103`, in the shape the change spec settles it. The spec
    file says what it does not cover at `refresh.test.ts:21`-`:28`: the demo wall ignores `refresh`
    in the engine, its bricks are compiled in, `handle_feed` returns from the demo arm before the
    flag reaches anything, and the re-read itself is `cargo nextest`'s in mortar-core. THE DRIFT: this
    obligation's evidence line expects the only `aria-live` under `web/src/lib/components/` to be
    `FeedGrid.svelte:221`. Three of the four are pre-existing regions belonging to other components,
    and the wall's own region now sits at `:239`, so the check that carries the DoD's meaning is
    "unchanged from the parent, and no branch in FeedGrid", which is what I measured.

- **O5 · The reader is closed here, and only here.**
  - *Claim:* the click handler calls `reader.close()` before `feed.refresh()`, `RefreshWall.svelte`
    is where the reader singleton is imported, and `feed.svelte.ts` names no reader at all.
  - *Evidence to collect:* read the handler and confirm the ordering. Run
    `grep -rn "reader" web/src/lib/state/feed.svelte.ts` and expect nothing. Run
    `grep -n "reader" web/src/lib/components/RefreshWall.svelte` and expect the import and the call.
  - *Checks:* resolve why it is here rather than in `FeedState`. `reader.svelte.ts` imports the
    `feed` singleton (`feed.freeze()`, `feed.items.findIndex`), so the reverse import is a cycle
    between two singleton modules and it would drag `$app/navigation` and `$app/state` into
    `feed.test.ts`'s graph, which runs under `environment: "node"`. Then resolve reachability, and do
    not mark this NOT_DONE for being unobservable: with the reader open, task 03 marks the wrapper at
    `+layout.svelte:110` `inert` and this control sits inside it at `:128`, so the state cannot be
    reached through the only trigger today. Confirm the component says so in a comment. The call
    must go through `reader.close()`, not a direct `replaceState`, or the reader's private did-push
    flag is left set and the next close calls `history.back()` on an entry that is not the reader's.
  - *Status:* SATISFIED. `RefreshWall.svelte:59` is `if (reader.isOpen) reader.close();` and
    `feed.refresh()` is at `:93`, so the ordering holds, and the reader singleton is imported at
    `:11`, here and nowhere new. `web/src/lib/state/feed.svelte.ts` is not in the diff, and its
    import list is two lines long, `$lib/api` and `$lib/types` (`:1`-`:2`), so no reader module can
    enter it by any route: the import-shaped `grep -nE "from ['\"][^'\"]*reader"` exits 1, and the
    file names no `reader` identifier in code at all. `close()` resolves (step 4, imported, then step
    2, enclosing class) to `ReaderState.close` at `reader.svelte.ts:156`, not to
    `HTMLDialogElement.close` and not to a direct `replaceState`. The `isOpen` guard is load-bearing
    rather than defensive, and I verified the reasoning against the two files rather than accepting
    it: `isOpen` is `this.showing !== null` (`reader.svelte.ts:85`), `showing` reads the router's
    page state, which the back gesture clears without ever calling `close()`; `#pushed` is set at
    `:133` and cleared only inside `close()` at `:159`. So after a back gesture the reader is shut
    with `#pushed` still true, and an unguarded `close()` would take the `ours` branch at `:160` and
    run `history.back()` on an entry that is not the reader's, leaving the wall altogether.
    `BrickReader.svelte:146`-`:151` declines to call `close()` in its own teardown for exactly that
    reason. Reachability measured rather than assumed: with a brick open on the demo wall at 375px
    the layout wrapper carries `inert`, `wrapper.contains(control)` is true, and `control.focus()`
    leaves `document.activeElement` on a DIV, so no click reaches `:59` today; the component says so
    at `:45`-`:51`. ONE GREP CANNOT PASS LITERALLY, AND THAT IS NOT A DEFECT: `grep -n "reader"
    web/src/lib/state/feed.svelte.ts` returns eleven lines (`:45`, `:99`, `:119`, `:132`, `:149`,
    `:208`, `:220`, `:223`, `:245`, `:341`, `:352`), every one of them task 25's prose about the
    human reader, all pre-existing in a file this task does not touch. The obligation is that
    `FeedState` imports no reader module, so the import-shaped grep is the honest reading of it, and
    the full import list above is stronger evidence than either grep.

- **O6 · Meets the repo definition of done, and the PR says what the gate misses.**
  - *Claim:* the greps and gates pass, knip sees the new component, and the PR states that
    `just check` does not cover this deliverable.
  - *Evidence to collect:* run `just guard-dashes`, `just guard-autoplay`, `cd web && pnpm knip` and
    `just check`. Read the PR body for the statement naming `just build && pnpm test:e2e` as the lane
    and CI's `e2e` job as the enforcement point.
  - *Status:* SATISFIED. `just check` exits 0 on this workspace: oxfmt and `cargo fmt --check`,
    the wasm32 check and the wasm-pack build behind `guard-wasm`, oxlint, knip, clippy, then 154
    nextest tests, both tsc projects and 141 vitest tests. `just guard-dashes` and
    `just guard-autoplay` also exit 0 run on their own, before and after this certificate was
    written. Knip reports nothing unused, so the new component is seen and referenced; the only
    oxlint warnings in the log are pre-existing and outside this diff (`FeedGrid.svelte:117`, `:118`,
    `:134`, `service-worker.ts:281`). The commit message carries the statement in a paragraph of its
    own: "`just check` DOES NOT COVER THIS TASK'S DELIVERABLE", with the reason (tsc cannot parse
    `.svelte`, so no component enters the typecheck program, and no vitest suite renders one),
    `web/tests/refresh.test.ts` named as the only lane that can see it, and CI's e2e job named as the
    enforcement point. `refresh.test.ts:6`-`:15` says the same thing in the file. A minor changeset
    exists at `.changeset/lay-this-wall-again.md`; see the validator's note below the conclusion
    about one sentence in it.

- **O7 · Reviewable: two quick presses at 375px.**
  - *Claim:* on the built site at a 375px viewport, pressing the control twice quickly starts one
    refresh, the wall keeps its bricks throughout, and the header row does not wrap.
  - *Evidence to collect:* run `just build && cd web && pnpm test:e2e`, then perform the sequence in
    a browser at 375px.
  - *Status:* SATISFIED, with the measurement recorded honestly. `CI=1 just test-e2e` builds the
    static site and passes 35 of 35. The sequence was then driven by hand against that build, in
    chromium at 375 by 812, on a preview server of my own on port 4291. Observed: the four controls
    span 137 + 36 + 103 + 44 of a 343px row, `scrollWidth - clientWidth` is 0 on both the row and the
    document, every control's box sits inside the viewport (the switcher's right edge lands at 359 of
    375) and `elementFromPoint` at each control's centre lands inside that control; the wall held its
    24 bricks before, during and after the press and finished with no skeletons on it; the control
    read `disabled === true` two microtask turns after the press and enabled itself again when the
    wall settled; and the press produced exactly two `/api/feed` requests,
    `intent=preview&refresh=1` and an `intent=freeze` carrying a cursor. THE MEASUREMENT: the two
    presses were a 0ms gap, both inside one evaluated function, and that pair is one refresh and two
    requests. I did not measure spaced presses, because on the demo wall the in-flight window is
    milliseconds (the fixtures are compiled in and a preview answers already settled), so a spaced
    pair measures the width of that window rather than the control. The semantic the control
    guarantees is "disabled while one is in flight" rather than a rate limit per unit time, and
    `refresh()`'s own refusal is the second layer under it, unit tested at `feed.test.ts:583`
    ("refuses a second refresh while the first is still warming"). On a live wall the window is a
    hundred-author fan-out rather than milliseconds.

## Regression check

- `+layout.svelte`'s control row now holds four controls. Trace: at 375px the row still does not
  wrap and `LayoutPicker`, `ClientPicker` and `SwitchWall` are all reachable : PRESERVED. Measured in
  chromium against the real build: at 375px the row is 343px of client width holding 137 + 36 + 103 +
  44 of content, so 23px of slack across three gaps; `scrollWidth - clientWidth` is 0 on both the row
  and the document, every control's box is inside the viewport, and `elementFromPoint` at each
  control's centre lands inside that control. Swept 320, 360, 375, 390, 414, 768 and 1280: clean from
  360 up. The re-budget is load-bearing rather than incidental, which I measured as a counterfactual
  in the same browser by putting the old budget back on the live page (row gap 8px, picker segments
  `px-2`): with four controls that row overflows by 21px at 375 and the switcher's right edge lands
  at 380 of 375. At 320px, a width below the one the bar is written against, the row overflows by
  40px now and by 32px under the parent's shape, so a width that was already past its budget is 8px
  further past it. Recorded rather than scored. One trade to name: the layout picker's segments below
  `sm` go from 45, 59 and 44 CSS pixels wide to 37, 51 and 36. Every control on the bar keeps its
  44px height (`min-h-11`), which is the rule the repo states, and the narrowest is still well above
  the 24px floor, but the picker's touch width did shrink on phones.
- Task 03's inert wrapper. Trace: with a reader open, `RefreshWall` is not focusable, which is the
  desirable interaction neither change spec states, and is also why O5's call is a guarantee rather
  than a live path : PRESERVED. Opened a brick on the demo wall at 375px: the layout wrapper carries
  `inert`, `wrapper.contains(control)` is true, and `control.focus()` leaves `document.activeElement`
  on a DIV. The fourth control did not escape the wrapper.
- Task 25's `FeedState`. Trace: `grep -rn "reader" web/src/lib/state/feed.svelte.ts` is still empty
  and `feed.test.ts` still needs no `$app/*` mock, so this task's import did not migrate : PRESERVED.
  The file is not in the diff, its whole import list is `$lib/api` and `$lib/types`, the
  import-shaped grep exits 1, and the 141 vitest tests (`feed.test.ts` among them, under
  `environment: "node"`) pass. See O5 for why the literal word grep cannot be empty on this tree.
- `FeedGrid`'s normal warming reflow on a cold wall. Trace: `/?actor=demo` from a fresh load still
  reflows and freezes as before : PRESERVED. `FeedGrid.svelte` is not in the diff. A fresh load at
  375px lays 24 bricks and settles with no skeletons left on the wall, and the suite's cold-wall
  cases (`service-worker-smoke.test.ts`, `repeated-tags.test.ts`) are green.
- `web/tests/service-worker-smoke.test.ts` and `web/tests/reader.test.ts`. Trace: both green :
  PRESERVED. The whole suite is 35 tests in five files and all 35 passed with no retry consumed: 14
  reader cases including the two inert-wrapper ones and both motion cases, 5 smoke cases including
  "the service worker binds every positional slot", 11 feed-picker cases, 2 repeated-tags cases and
  the 3 new refresh cases.

## Residue

- Pull to refresh is out of scope, per the plan's "what this plan does not do". If the validator
  finds a touch gesture wired here, that is scope creep, not an obligation met.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED on evidence I collected myself, and every regression line traces
PRESERVED: the control is a real disabled `<button>` measured in the DOM at 375px, the handler is two
statements with no scroll call and a reduced-motion comment that matches `feed.svelte.ts` and its
vitest race case rather than plausibility, the reader close is here and `FeedState` still imports no
reader module, no live region was added, and the three-case spec ran twenty-one consecutive times
with retries off without a flake.

## Validator's note

One sentence in `.changeset/lay-this-wall-again.md`, repeated in the commit message, does not survive
measurement: "it fits a 360px phone for the first time too, which it did not before the fourth
control arrived". Restoring the parent's budget on the live page (row gap 8px, picker segments
`px-2`) and hiding the fourth control gives a 328px row holding 324px of content at 360px wide, so
the bar fitted a 360px phone before this change, by about 4px. Everything else in that changeset
measures true, including the 375px re-budget it is really about. This is a changelog sentence rather
than an obligation or a line of code, and no page task 27 copies is affected, so it is recorded here
rather than scored against O6.
