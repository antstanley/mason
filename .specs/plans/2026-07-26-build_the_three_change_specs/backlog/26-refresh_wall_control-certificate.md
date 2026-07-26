# Done Certificate · Task 26: RefreshWall control

**Task:** [26-refresh_wall_control.md](26-refresh_wall_control.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · The scroll-versus-freeze interaction is resolved and commented on both paths, and the motion path is asserted.**
  - *Claim:* the resolution is written in the file as a comment covering the event path **and** the
    reduced-motion path, and the event path is proven by the Playwright case rather than left to the
    next implementer.
  - *Evidence to collect:* read the comment and confirm it names both paths. Run the Playwright case
    that distinguishes the two outcomes: after a click, the wall must reflow (warming stays true
    through at least one preview) rather than freezing immediately.
  - *Checks:* trace **two** chains, not one. First, `FeedGrid.svelte:188`-`:201` re-arms one-shot
    `wheel`, `scroll`, `touchmove`, `keydown` and `focusin` listeners whenever `feed.warming` flips
    true, and `refresh()` flips it true synchronously, so a programmatic `scrollTo` dispatches a
    scroll event that lands on `freezeOnEngage` and freezes the reflow the click just started;
    confirm the chosen resolution actually breaks that chain. Second, `FeedGrid.svelte:182`-`:187`
    is a different path entirely: under `prefers-reduced-motion: reduce` the same effect calls
    `freezeOnEngage()` at `:185` immediately and returns, with no listener and no scroll event, so
    **no** scroll-side resolution can reach it. Confirm the comment states what a reduced-motion
    refresh actually is, read out of task 25's code rather than assumed: **two** cursorless requests
    go out (the preview `refresh()` issues synchronously, and the freeze the effect triggers), task
    25's in-flight marker lets only the first carry `refresh=1`, the preview's page is discarded when
    the freeze bumps the generation, and the wall moves exactly once. One refreshed fan-out, no
    reflow. A comment that describes only the listener chain is incomplete, and one that says "a
    single cursorless freeze" is wrong; task 27 copies whichever into `08`.
  - *Status:* unverified

- **O3 · The Playwright spec asserts all four behaviours at 375px, with a named mechanism for the disabled window.**
  - *Claim:* `web/tests/refresh.test.ts` asserts the control renders with an accessible name at a
    375px viewport; clicking it leaves bricks on the wall (`#wall article` stays visible, the
    skeleton grid never appears); the control is `disabled` while warming and enabled once settled;
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
  - *Status:* unverified

- **O4 · The spec says what it does not cover, and the live region question is decided.**
  - *Claim:* the spec file notes that the demo wall ignores `refresh` in the engine, so this lane
    covers the client behaviour and not the re-read; and the live-region question is answered one way
    or the other, recorded in the file.
  - *Evidence to collect:* read the spec's header comment and the component's live-region decision.
    Confirm no second polite live region was added: `FeedGrid.svelte:221` is the wall's single one,
    and `08-wall-and-bricks.md`'s accessibility section states there is exactly one.
  - *Checks:* if the decision was to give `FeedGrid` a refresh-aware branch, resolve how it learns
    the warm is a refresh; `FeedState` does not expose that today, so either it gained a field or the
    decision was the other one.
  - *Status:* unverified

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
  - *Status:* unverified

- **O6 · Meets the repo definition of done, and the PR says what the gate misses.**
  - *Claim:* the greps and gates pass, knip sees the new component, and the PR states that
    `just check` does not cover this deliverable.
  - *Evidence to collect:* run `just guard-dashes`, `just guard-autoplay`, `cd web && pnpm knip` and
    `just check`. Read the PR body for the statement naming `just build && pnpm test:e2e` as the lane
    and CI's `e2e` job as the enforcement point.
  - *Status:* unverified

- **O7 · Reviewable: two quick presses at 375px.**
  - *Claim:* on the built site at a 375px viewport, pressing the control twice quickly starts one
    refresh, the wall keeps its bricks throughout, and the header row does not wrap.
  - *Evidence to collect:* run `just build && cd web && pnpm test:e2e`, then perform the sequence in
    a browser at 375px.
  - *Status:* unverified

## Regression check

- `+layout.svelte`'s control row now holds four controls. Trace: at 375px the row still does not
  wrap and `LayoutPicker`, `ClientPicker` and `SwitchWall` are all reachable :
  (PRESERVED / REGRESSION)
- Task 03's inert wrapper. Trace: with a reader open, `RefreshWall` is not focusable, which is the
  desirable interaction neither change spec states, and is also why O5's call is a guarantee rather
  than a live path : (PRESERVED / REGRESSION)
- Task 25's `FeedState`. Trace: `grep -rn "reader" web/src/lib/state/feed.svelte.ts` is still empty
  and `feed.test.ts` still needs no `$app/*` mock, so this task's import did not migrate :
  (PRESERVED / REGRESSION)
- `FeedGrid`'s normal warming reflow on a cold wall. Trace: `/?actor=demo` from a fresh load still
  reflows and freezes as before : (PRESERVED / REGRESSION)
- `web/tests/service-worker-smoke.test.ts` and `web/tests/reader.test.ts`. Trace: both green :
  (PRESERVED / REGRESSION)

## Residue

- Pull to refresh is out of scope, per the plan's "what this plan does not do". If the validator
  finds a touch gesture wired here, that is scope creep, not an obligation met.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
