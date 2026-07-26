# Done Certificate · Task 15: client target plumbing

**Task:** [15-client_target_plumbing.md](15-client_target_plumbing.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 15. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. Five of this
> task's edits are in files tsc cannot parse; a green typecheck proves nothing about them.

## Definition

DONE(Task 15) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** `/?feed=<uri>` lays a wall with the header, both pickers, the bottom padding and
  the document title a graph wall gets.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break the graph wall: `/?actor=demo` must still lay bricks, warm,
  freeze, paginate and rehydrate from the session cache on back/forward.

## Obligations

- **O1 · Exactly one target parameter reaches the query string, and only `fetchFeed` changed.**
  - *Claim:* `api.ts` exports `FeedTarget`, `fetchFeed` takes one and writes exactly one of `actor`
    and `feed`, asserted against the exact URL for both kinds. **`warmFeed(actor: string)` is
    unchanged**, decided here rather than left open: a feed has no follow graph to warm, so a feed
    target skips the call.
  - *Evidence to collect:* run `cd web && pnpm test` and read the two exact-URL assertions in
    `api.test.ts`; confirm they compare the whole query string, following the existing style at
    `api.test.ts:51`. Read the diff for `warmFeed` and for `HandleForm.svelte:21`; both must be
    untouched.
  - *Checks:* resolve which branch builds the params. A `URLSearchParams` seeded with both keys and
    one deleted would also pass a substring test but not an exact-URL one; confirm the assertion is
    exact. Then confirm the `FeedTarget` export lands **here** and not at task 14, where it had no
    consumer and knip would have failed it.
  - *Status:* unverified

- **O2 · The session cache key is a stable string and separates the two wall kinds.**
  - *Claim:* `#key` derives a stable string from target plus mode, and `feed.test.ts` proves a graph
    wall and a feed wall do not rehydrate into each other.
  - *Evidence to collect:* read `#key` in `feed.svelte.ts`. Run the new `feed.test.ts` case.
  - *Checks:* an object interpolated into a template literal yields `[object Object]`. Read the
    interpolation and confirm the target is destructured or stringified explicitly; the failure mode
    is every feed wall collapsing onto one cache entry, which no type would catch.
  - *Status:* unverified

- **O3 · All five `.svelte` call sites are updated, the header is un-gated, and the heading is not broken.**
  - *Claim:* `+page.svelte`, `FeedGrid.svelte` (twice), `LandingWall.svelte` and
    `HandleForm.svelte` all compile and run against the new signatures; `+layout.svelte`'s
    header condition is `actor || feed`; and `+page.svelte`'s `h1` has a feed-wall branch, so a laid
    feed wall never renders `@'s wall on mason`.
  - *Evidence to collect:* run
    `grep -rn 'feed.reset\|fetchFeed(\|warmFeed(' web/src --include=*.svelte` and confirm five hits,
    each passing a target rather than a bare string. Read `+layout.svelte` around `:13`, `:101`,
    `:110`, `:111` and `:129`, and `+page.svelte` around `:26` and `:31`. Then **read the rendered
    text** of `#wall h1` on a laid feed wall, in the browser or in the Playwright case; do not infer
    it from the source.
  - *Checks:* `{#if actor}` at `:111` today gates the skip link, the whole header, the layout
    picker, the client picker and `SwitchWall`. Confirm all five are inside the widened condition
    and that the bottom-padding ternary at `:110` and the title at `:101` were widened too. Then
    resolve the second `{#if actor}`, at `+page.svelte:26`, which gates the `#wall` main containing
    the page's only `h1` at `:31`. Widening `:26` without giving `:31` a branch produces
    `@'s wall on mason`, and neither of this task's Playwright cases reaches it: `/?actor=demo` is
    unchanged and `/?feed=nonsense` hands the `h1` to the error panel by the existing condition at
    `:30`. This is a read, and it is the one this obligation most easily passes without doing.
  - *Status:* unverified

- **O4 · Both Playwright cases pass.**
  - *Claim:* `/?actor=demo` still lays bricks (the existing case, unedited) and `/?feed=nonsense`
    renders the header with the layout picker present plus an error panel rather than a blank page.
  - *Evidence to collect:* run `just test-e2e`. Confirm the existing smoke test's body is unchanged
    in the diff.
  - *Status:* unverified

- **O5 · The PR states the coverage gap plainly.**
  - *Claim:* the PR body says this task ships no tsc coverage for any of the five `.svelte` edits
    and that the two Playwright cases are the whole of their coverage.
  - *Evidence to collect:* read the PR body.
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `cd web && pnpm check:ci`, `just test-e2e` and
    `just check`. Expect all clean.
  - *Status:* unverified

- **O7 · Reviewable: a feed wall has chrome and a feed-shaped error.**
  - *Claim:* on the built site, `/?feed=nonsense` shows the header controls and an error panel that
    names the feed rather than the handle.
  - *Evidence to collect:* run `just test-e2e`, then serve `web/build` and open the URL.
  - *Status:* unverified

## Regression check

- `web/src/lib/state/feed.svelte.ts` `reset` is called from `FeedGrid.svelte:55` on mount and from
  `:263` on the error-panel retry. Trace: `/?actor=demo` still lays and the retry still re-lays :
  (PRESERVED / REGRESSION)
- `LandingWall.svelte:16` calls `fetchFeed('demo')` today. Trace: the landing wall still renders its
  demo bricks : (PRESERVED / REGRESSION)
- `HandleForm.svelte:21` calls `warmFeed(handle || 'demo')`. Trace: typing a handle still warms the
  engine : (PRESERVED / REGRESSION)
- Session cache rehydration: lay `/?actor=demo`, navigate away, go back. Expect the same arrangement
  and no skeletons : (PRESERVED / REGRESSION)

## Residue

- `warmFeed` stays actor-only and a feed target skips it, decided by default rather than on
  evidence. If the validator finds a feed target reaching `warmFeed`, note it: it would be a no-op
  fan-out request against a target with no follow graph. The alternative nobody has evidence for is
  having task 18's picker prefetch the first `getFeed` page into `feed_pages`; that is a new
  behaviour, not a signature change, so it can land later without reopening this diff.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
