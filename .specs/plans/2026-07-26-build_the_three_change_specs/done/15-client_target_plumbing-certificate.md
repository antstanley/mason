# Done Certificate · Task 15: client target plumbing

**Task:** [15-client_target_plumbing.md](15-client_target_plumbing.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Status:* **SATISFIED.** Ran `cd web && pnpm test` → 4 files, 47 tests, all pass. Both new
    assertions are whole-string `toHaveBeenCalledWith` against the single fetch argument, not
    substring matches: `"/api/feed?actor=demo&cursor=cur1&mode=glaze&intent=preview"` and
    `"/api/feed?feed=at%3A%2F%2Fdid%3Aplc%3Aabc%2Fapp.bsky.feed.generator%2Fhot&cursor=cur1&mode=glaze&intent=freeze"`,
    so a stray second parameter fails them. The params are built at `api.ts:67-69` as
    `new URLSearchParams("feed" in target ? { feed: target.feed } : { actor: target.actor })`, one
    single-key object literal per branch, never a spread and never a seed-then-delete, so a widened
    object cannot put both on the wire. `warmFeed`'s signature line is not in `jj diff` at all (only
    the doc comment above it is), and `web/src/lib/components/HandleForm.svelte` does not appear in
    `jj st`, so `HandleForm.svelte:21`'s `warmFeed(... || 'demo')` is byte-identical. `FeedTarget` is
    added by **this** diff at `api.ts:52`, and knip is green inside `just check`, so the export has
    its consumer here. Corroborated on the wire in chromium against the built site: `?actor=demo`
    walls sent `?actor=demo&intent=preview`, feed walls sent `?feed=nonsense&intent=preview` and
    `?feed=nonsense&mode=glaze&intent=preview`; no request carried both parameters, and mortar's own
    precedence (`feed.rs:73-79`, feed wins when both arrive) matches the client's.

- **O2 · The session cache key is a stable string and separates the two wall kinds.**
  - *Claim:* `#key` derives a stable string from target plus mode, and `feed.test.ts` proves a graph
    wall and a feed wall do not rehydrate into each other.
  - *Evidence to collect:* read `#key` in `feed.svelte.ts`. Run the new `feed.test.ts` case.
  - *Checks:* an object interpolated into a template literal yields `[object Object]`. Read the
    interpolation and confirm the target is destructured or stringified explicitly; the failure mode
    is every feed wall collapsing onto one cache entry, which no type would catch.
  - *Status:* **SATISFIED.** `feed.svelte.ts:56-59` reads the union arm explicitly and prefixes the
    kind (`feed\u{1f}<value>` or `actor\u{1f}<value>`), joined to the mode with the same separator.
    The object is never interpolated. The new case
    `a graph wall and a feed wall never rehydrate into each other` passes. It was mutation-tested
    rather than read: two scratch copies of `feed.svelte.ts` were built beside their own copies of
    `feed.test.ts`, one with `#key` interpolating the target object (the `[object Object]` failure)
    and one keying on the value alone with the kind dropped. **Both mutants were killed by the new
    case** (`expected false to be true` at `expect(feed.warming).toBe(true)`); the object mutant was
    additionally killed by the pre-existing rehydration case. The kind-dropping mutant is killed by
    the new case and by nothing else, which is exactly the obligation it was authored to hold. The
    scratch copies were deleted and `jj st` confirmed back to the same ten files.

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
  - *Status:* **SATISFIED.** Five call sites, each converted or deliberately untouched:
    `+page.svelte:32` `feed.reset(current, currentMode)` with `current: FeedTarget`;
    `FeedGrid.svelte:64` `feed.reset({ actor: handle }, currentMode)`; `FeedGrid.svelte:272`
    `feed.reset(currentTarget, currentMode)`; `LandingWall.svelte:16` `fetchFeed({ actor: 'demo' })`;
    `HandleForm.svelte:21` `warmFeed(...)`, unchanged by design. Resolution: `feed` at both FeedGrid
    sites and at `+page.svelte:32` is the singleton imported from `$lib/state/feed.svelte` (step 4,
    imported), no local of that name in either file; `+page.svelte` names its URL parameter
    `feedUri` precisely to avoid shadowing it. `+layout.svelte` **does** bind a local
    `const feed = $derived(...)` at `:19`, and that is checked and clear: nothing named `feed` is
    imported into that file, so the local shadows nothing. Header gate `+layout.svelte:139` is
    `{#if actor || feed}`, padding `:137` is `{actor || feed ? 'pb-24 md:pb-0' : ''}`, title `:26-28`
    branches feed-first. Confirmed by reading the built site's DOM at `/?feed=nonsense`: the
    `Wall layout` fieldset (LayoutPicker), the `Open posts in Bluesky` control (ClientPicker), the
    `Switch wall…` button (SwitchWall), the `skip to the wall` link, `pb-24` on the wrapper and the
    document title `a feed · mason` are all present. The layout picker is not merely rendered but
    live: selecting Glaze on a feed wall re-requested `?feed=nonsense&mode=glaze&intent=preview`.
    The heading was **read, not inferred**, on a wall that actually lays: with the worker blocked and
    `/api/feed` answered by the driver, `/?feed=at%3A%2F%2F…%2Fwhats-hot` rendered one article and
    `#wall h1` had count 1 and text exactly `a bluesky feed, laid on mason`. `@'s wall on mason`
    appeared on no page probed (`/`, `/?actor=demo`, `/?feed=nonsense`, `/?actor=demo&feed=nonsense`,
    the laid feed wall). Zero `pageerror` events across every probe.

- **O4 · Both Playwright cases pass.**
  - *Claim:* `/?actor=demo` still lays bricks (the existing case, unedited) and `/?feed=nonsense`
    renders the header with the layout picker present plus an error panel rather than a blank page.
  - *Evidence to collect:* run `just test-e2e`. Confirm the existing smoke test's body is unchanged
    in the diff.
  - *Status:* **SATISFIED.** `just test-e2e` → 5 passed in chromium, including
    `a feed wall renders the chrome a graph wall gets` and
    `a laid feed wall › lays bricks under a heading of its own`. The existing demo-wall case is
    unedited: the diff's hunks in `web/tests/service-worker-smoke.test.ts` are the `FeedResponse`
    import, the `layoutPicker()` helper, and an append starting after that test's body, which appears
    only as unchanged context. The new chrome case asserts the layout picker specifically (the canary
    for the single `{#if}` gating the whole header) and asserts `#wall h1` does not contain
    `wall on mason`, then repeats for `/?actor=demo&feed=nonsense` with zero articles, pinning the
    feed-wins precedence.

- **O5 · The PR states the coverage gap plainly.**
  - *Claim:* the PR body says this task ships no tsc coverage for any of the five `.svelte` edits
    and that the two Playwright cases are the whole of their coverage.
  - *Evidence to collect:* read the PR body.
  - *Status:* **SATISFIED.** This run lands a commit rather than a PR, so the drafted commit message
    is the PR body. Its fifth paragraph says it outright: TypeScript 7 cannot parse `.svelte`, zero
    component files enter the program, a green `pnpm check:ci` proves exactly nothing about
    `+page.svelte`, `+layout.svelte`, `FeedGrid.svelte` or `LandingWall.svelte`, and the Playwright
    cases are the whole of their coverage. The claim was verified rather than taken on trust:
    `tsc -p tsconfig.json --listFiles` emits **0** files ending in `.svelte`. The third paragraph
    also discharges the task's open question, saying the `warmFeed` decision is made by default
    rather than on evidence and that task 18's picker prefetch can land later without touching this
    diff. Note for the record: the revision itself still reads `(no description set)`; the message
    exists as drafted prose and must actually be set when the task is committed.

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `cd web && pnpm check:ci`, `just test-e2e` and
    `just check`. Expect all clean.
  - *Status:* **SATISFIED.** All four run by the validator, all green. `cd web && pnpm test` → 4
    files / 47 tests. `cd web && pnpm check:ci` → clean (svelte-kit sync + tsc on both projects).
    `just test-e2e` → 5 passed. `just check` → guard-dashes, guard-autoplay, guard-toolchain,
    fmt-check, guard-wasm, lint (oxlint + knip + clippy) and test (149 rust tests, 47 vitest, tsc)
    all pass. A changeset is present (`.changeset/plain-walls-lay-feeds.md`, minor) for the
    user-visible change. No Rust changed, so no `just wasm` and no wire, fixture or spec updates were
    owed.

- **O7 · Reviewable: a feed wall has chrome and a feed-shaped error.**
  - *Claim:* on the built site, `/?feed=nonsense` shows the header controls and an error panel that
    names the feed rather than the handle.
  - *Evidence to collect:* run `just test-e2e`, then serve `web/build` and open the URL.
  - *Status:* **SATISFIED on its load-bearing content, with one clause deferred to task 16.**
    `just test-e2e` green, then `web/build` served on :4173 and driven in chromium. `/?feed=nonsense`
    renders the full chrome, skip link, header, `Wall layout` fieldset, `Open posts in Bluesky`,
    `Switch wall…`, `pb-24` bottom padding, title `a feed · mason`, over an error panel, not a
    blank page, and the panel's `try again` button re-requests `?feed=nonsense`, never `actor=`.
    The panel reads "the wall wouldn't load / mason could not reach the network. check your
    connection and try again." It **does not name the handle**, which is the half of this clause that
    could have regressed and the half a widened route puts at risk. It also does not name the feed,
    and that is by the plan's own decomposition rather than an omission here: `?feed=nonsense` fails
    `feedref::parse` and mortar answers `bad_request` (`feed.rs:300`), which `#fail` maps to
    `feed-unavailable`; the feed-shaped panel is task 16's step ("Add a `feed-not-found` panel to
    `FeedGrid` reading 'no such feed'"), and `bad_request` would not reach even that panel. Task 15's
    Steps contain no step touching `FeedGrid`'s error copy, so this clause of the Reviewable line
    asks for work no step in this task produces. Recorded rather than silently dropped.

## Regression check

- `web/src/lib/state/feed.svelte.ts` `reset` is called from `FeedGrid.svelte:55` on mount and from
  `:263` on the error-panel retry. Trace: `/?actor=demo` still lays and the retry still re-lays :
  **PRESERVED**. `/?actor=demo` laid 24 bricks with `?actor=demo&intent=preview` then
  `?actor=demo&cursor=…&intent=freeze` on the wire. The retry form on
  `/?actor=definitely-not-a-real-handle.invalid` re-requested that same handle; the `try again`
  button on `/?feed=nonsense` re-requested `?feed=nonsense` and never `actor=`.
- `LandingWall.svelte:16` calls `fetchFeed('demo')` today. Trace: the landing wall still renders its
  demo bricks : **PRESERVED**. `/` rendered 24 articles behind the form and put one bare
  `?actor=demo` on the wire.
- `HandleForm.svelte:21` calls `warmFeed(handle || 'demo')`. Trace: typing a handle still warms the
  engine : **PRESERVED**. The file is untouched (absent from `jj st`) and `/` fired a second bare
  `?actor=demo`, which is the warm.
- Session cache rehydration: lay `/?actor=demo`, navigate away, go back. Expect the same arrangement
  and no skeletons : **PRESERVED**. Client-side navigation to `/` via the header's home link and
  `goBack()` returned the identical 24-brick arrangement with **zero** `/api/feed` requests for the
  wall; the only two requests in that window were the landing page's own `?actor=demo` pair.
- Additional, beyond the authored list, because this diff makes `+page.svelte`'s wall selector an
  **object**-valued `$derived` where it was a string: a same-URL `pushState` must not re-reset the
  wall. Trace: opening the reader from tasks 03/04/05 over `/?actor=demo` (which `pushState`s on the
  same URL), stepping with ArrowRight (`replaceState`), then Escape → dialog opened and closed, wall
  arrangement byte-identical before and after, **zero** `/api/feed` requests during. So the derived
  does not re-propagate when its string dependencies recompute to equal values. `+layout.svelte`'s
  `overlayOpen = $derived(reader.isOpen)` is untouched by this diff : **PRESERVED**.

## Residue

- `warmFeed` stays actor-only and a feed target skips it, decided by default rather than on
  evidence. If the validator finds a feed target reaching `warmFeed`, note it: it would be a no-op
  fan-out request against a target with no follow graph. The alternative nobody has evidence for is
  having task 18's picker prefetch the first `getFeed` page into `feed_pages`; that is a new
  behaviour, not a signature change, so it can land later without reopening this diff.
  - *Validator:* no feed target reaches `warmFeed`. Its only call site is `HandleForm.svelte:21`,
    which is untouched and passes a handle string; grep finds no other caller. The signature is
    unchanged.
- Two cosmetic gaps this diff opens and task 16 closes, both observed and neither blocking:
  `SwitchWall` still receives `actor={actor ?? ''}` (`+layout.svelte:157`), so a feed wall's switcher
  reads `@` with an aria-label of `Switch wall, currently viewing @`, and on
  `/?actor=demo&feed=nonsense` it reads `@demo` while a feed is the wall laid. It is not a crash and
  not a dead end: `profile.load('')` returns at `profile.svelte.ts:26` before any network call, and
  the panel still opens and still switches. Task 16's steps name this state explicitly ("an
  aria-label naming the feed rather than an empty handle"). Second, `FeedGrid`'s empty-wall state
  still says "start from another handle" on a feed wall; task 16 owns that copy too.
- The base blob of `web/src/lib/state/feed.svelte.ts` held a raw NUL byte inside `#key`'s template
  literal, which is why `jj diff` reports the file as binary. The new text is plain ASCII escapes.
  Verified by decoding both revisions; the file is not corrupt mid-edit.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED with evidence the validator collected itself, the four gates run
green, the cache key's new test was mutation-tested and kills both the `[object Object]` and the
kind-dropping breaks, and all five `.svelte` call sites plus the sr-only heading were exercised and
read out of a real browser rather than inferred, with the four named regression traces PRESERVED
plus a fifth for the new object-valued `$derived`; the only unmet fragment is O7's "names the feed"
clause, which task 16's steps own and which `?feed=nonsense` (a `bad_request`) would not reach even
after them.
