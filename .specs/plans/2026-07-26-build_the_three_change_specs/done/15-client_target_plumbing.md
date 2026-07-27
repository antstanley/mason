# Task 15 · client target plumbing

**Plan:** [plan.md](../plan.md) · **Certificate:** [15-client_target_plumbing-certificate.md](15-client_target_plumbing-certificate.md)

**Implements:** [`changes/merged/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/merged/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `07-web-client.md` → Responsibilities, → Shape, → The feed state machine; implementation notes 11 and 11b. Targets [`07-web-client.md`](../../../07-web-client.md) §Responsibilities and §The feed state machine.
**Depends on:** 03, 13, 14
**Produces:** `/?feed=<uri>` lays a wall with the header, both pickers, the bottom padding and the document title a graph wall gets.
**Pointers:** `api.ts:36` (`fetchFeed`, first parameter `actor: string`), `:45` (the params build), `:66` (`warmFeed`). `api.test.ts:36`, `:48`, `:51` (the exact-URL assertion), `:78`, `:101`. `feed.svelte.ts:35` (`#actor`), `:41` (`#cache`), `:43` (`#key`), `:59` (`reset`), `:97`, `:131`, `:160` (the three fetch sites). `feed.test.ts:90`-`:322` (every `reset` call). **Five `.svelte` call sites tsc cannot see:** `routes/+page.svelte:22`, `FeedGrid.svelte:55`, `FeedGrid.svelte:263`, `LandingWall.svelte:16`, `HandleForm.svelte:21`. `+layout.svelte:13` (the `actor` derive), `:101` (the title ternary), `:110` (the padding ternary), `:111` (`{#if actor}`, which gates the **entire** header), `:129` (the `SwitchWall` prop). And two more lines in `+page.svelte` that are not call sites and are easy to miss: `:26` (`{#if actor}`, which gates the whole `#wall` main) and `:31`, the page's **only** `h1`, `<h1 class="sr-only">@{actor}'s wall on mason</h1>`, which renders as `@'s wall on mason` the moment `:26` widens.

## Steps

- [ ] Export `FeedTarget` (`{ actor: string } | { feed: string }`) from `api.ts` here rather than at task 14, where it would have had no consumer and knip fails an unused exported type. `FeedTargetKind` is already there from task 14.
- [ ] Change `fetchFeed` to take a `FeedTarget`, writing exactly one of `actor` and `feed` into the query string. Update `api.test.ts` to assert the built URL for both target kinds. **`warmFeed(actor: string)` keeps its signature**, decided once here and not revisited: warming fills the follow-graph caches and a feed target has no graph to warm, so a feed wall simply skips the call. `HandleForm.svelte:21`'s `warmFeed(handle || 'demo')` therefore does not change, which matters because it is one of the five `.svelte` call sites nothing typechecks.
- [ ] Give `FeedState` a target field, change `reset(target, mode)`, and derive `#key` from a **stable string** built from target plus mode. An object interpolated into a template literal yields `[object Object]` and would collapse every feed wall onto one cache entry.
- [ ] Update every `reset` call in `feed.test.ts`, and add a case proving a graph wall and a feed wall do not rehydrate into each other from the session cache.
- [ ] Change `+page.svelte` to derive both parameters with `feed` winning, and to show `HandleForm` only when neither is present. Widening `{#if actor}` at `:26` to `actor || feed` also widens the `h1` at `:31`, so give it a feed-wall branch beside the title branch `+layout.svelte:101` already owns. Left alone it reads `@'s wall on mason`, and it is the page's only `h1`. Nothing in the plan's lanes catches it by accident: it is in a `.svelte` file, and of this task's two Playwright cases one is `/?actor=demo` (unchanged) and the other is `/?feed=nonsense`, where the existing condition at `:30` hands the `h1` to the error panel instead.
- [ ] Change `+layout.svelte` to gate the header, the bottom padding and the document title on `actor || feed`, and give the title a feed-wall branch. Without this a feed wall renders with no `LayoutPicker`, no `ClientPicker` and no `SwitchWall` at all.
- [ ] Update the other four `.svelte` call sites and add the two Playwright cases named below.

## Definition of done

- [ ] Exactly one of `actor` and `feed` reaches the query string, asserted for both target kinds against the exact URL.
- [ ] `warmFeed`'s signature is unchanged, proven by the diff, and `HandleForm.svelte:21` is untouched.
- [ ] `#key` derives a stable string from target plus mode, and `feed.test.ts` proves a graph wall and a feed wall keep separate session-cache entries.
- [ ] All five `.svelte` call sites compile and run, and `+layout.svelte:111`'s condition is `actor || feed`.
- [ ] `+page.svelte`'s `h1` has a feed-wall branch and never renders `@'s wall on mason`. The `#wall h1` text is **read**, on a laid feed wall, not assumed: a widened `{#if}` with an un-widened heading is a silent accessibility regression on the page's only landmark heading.
- [ ] Playwright: `/?actor=demo` still lays bricks (the existing case, unchanged) and `/?feed=nonsense` renders the header with the layout picker present plus an error panel rather than a blank page.
- [ ] The PR states plainly that this task ships **no tsc coverage** for any of the five `.svelte` edits: `pnpm check:ci` passing proves nothing about them, and the two Playwright cases above are the whole of their coverage.
- [ ] Meets the repo definition of done (`cd web && pnpm test`, `pnpm check:ci`, `just test-e2e` and `just check` all green).
- [ ] Reviewable: `just test-e2e`, then open `/?feed=nonsense` in the built site and confirm the chrome is there and the error panel names the feed rather than the handle.

## Open questions

- The `warmFeed` decision above is made by default rather than on evidence; say so in the PR. The alternative, not taken, is having task 18's picker prefetch the first `getFeed` page into `feed_pages` under the card the reader is about to activate. That is a new behaviour rather than a signature change, so it can land later without touching this task's diff.
