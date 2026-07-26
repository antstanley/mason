# Task 16 · feed identity and the fourth error

**Plan:** [plan.md](../plan.md) · **Certificate:** [16-feed_identity_and_fourth_error-certificate.md](16-feed_identity_and_fourth_error-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `07-web-client.md` → Reactive state (`feedInfo`) and → Error classification, and → `08-wall-and-bricks.md` → Wall states; implementation note 11. Targets [`07-web-client.md`](../../../07-web-client.md) §Reactive state and §Error classification, and [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md) §Wall states.
**Depends on:** 15
**Produces:** the generator's own face in the header, and "no such feed" instead of telling somebody with a bad feed link to fix their handle.
**Pointers:** `state/profile.svelte.ts:8` (the hardcoded AppView base to hoist), `:26` (the `browser` guard to mirror). `feed.svelte.ts:197` (`#fail`), `feed.test.ts:218` (the `it.each` table). `SwitchWall.svelte:12`, `:20`, `:85`, `:88` (where the owner's face and aria-label live). `+layout.svelte:129` (the prop `SwitchWall` takes). `FeedGrid.svelte:224`-`:277` (the three existing error panels), `:278`-`:317` (the empty state).

## Steps

- [ ] Add `web/src/lib/appview.ts` and hoist the AppView base out of `state/profile.svelte.ts:8`, deleting the constant there rather than duplicating it. There are three client-side AppView readers after this change and one hardcoded constant each is two too many.
- [ ] Add `web/src/lib/state/feedinfo.svelte.ts`, modelled on `profile.svelte.ts`, reading `app.bsky.feed.getFeedGenerator` behind the `browser` guard. It never blocks the wall, and on a miss leaves the header showing the feed's rkey.
- [ ] Add `feedinfo.test.ts`, including a case for the miss.
- [ ] Map `feed_not_found` to `"feed-not-found"` in `#fail` with a `satisfies MortarErrorCode` comparison, and add the row to `feed.test.ts:218`'s table.
- [ ] Give `SwitchWall` the generator's avatar and display name where it shows the owner's face, with an aria-label naming the feed rather than an empty handle.
- [ ] Add a `feed-not-found` panel to `FeedGrid` reading "no such feed", and change the empty state to "this feed has no bricks yet" on a feed wall.

## Definition of done

- [ ] `lib/appview.ts` is the single client-side AppView base, proven by grep: no fourth hardcoded constant, and `profile.svelte.ts`'s own is deleted.
- [ ] `#fail` uses `satisfies MortarErrorCode`, so a code renamed in mortar and the regenerated fixture fails tsc here, and the `it.each` table covers the new row.
- [ ] The three existing `FeedGrid` error panels and both handle-box recoveries are unchanged on a graph wall.
- [ ] The PR states that the `.svelte` half of this task (`SwitchWall`, `FeedGrid`) ships **with no automated coverage at all**: no offline e2e case can reach a live `getFeedGenerator` or a `getFeed` 404, and neither vitest suite imports a component. The `.ts` half is vitest-covered and that is the whole of the lane.
- [ ] Meets the repo definition of done (`pnpm test`, `pnpm check:ci`, `just test-e2e` as a regression only, `just check`).
- [ ] Reviewable: `cd web && pnpm test` covers `appview`, `feedinfo` and `#fail`; the header and panel are read in the diff and exercised by hand against a real feed uri.
