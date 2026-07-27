# Task 01 · reader rune and page state

**Plan:** [plan.md](../plan.md) · **Certificate:** [01-reader_rune_and_page_state-certificate.md](01-reader_rune_and_page_state-certificate.md)

**Implements:** [`changes/merged/2026-07-26-read_a_brick_in_place.md`](../../../changes/merged/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `07-web-client.md` → Reactive state → The reader is history, not a URL; implementation notes 1 and 5. Targets [`07-web-client.md`](../../../07-web-client.md) §Reactive state.
**Depends on:** none
**Produces:** `App.PageState` exists with `brick?: string`, and every open, close, step and modifier-key decision the reader makes lives in one `.svelte.ts` module that vitest runs for real.
**Pointers:** `web/src/app.d.ts:8` (the interface is a comment today, so it is created rather than extended); `web/src/lib/state/feed.svelte.ts:121` (`freeze()` is already a no-op on a settled wall); `web/src/lib/api.test.ts:9` (the `vi.mock` style to mirror); SvelteKit 2.70.1 types `pushState(url, state: App.PageState)`.

## Steps

- [ ] Uncomment `interface PageState` in `web/src/app.d.ts` and declare exactly one member, `brick?: string`. Leave `Error`, `Locals`, `PageData` and `Platform` commented.
- [ ] Add `web/src/lib/state/reader.svelte.ts`: a singleton class instance exporting `brick` (`$state<Brick | null>`), `index` (derived by `feed.items.findIndex(b => b.id === brick.id)`, never stored), `open(brick, opener)`, `close()`, `next()`, `prev()`, `canNext`, `canPrev`, and `activate(event, brick): boolean`.
- [ ] Make `open` call `feed.freeze()` fire and forget before `pushState('', { brick: brick.id })`, record the opening element for focus return, and set a private did-push flag.
- [ ] Make `close()` call `history.back()` only when the private flag is set, and `replaceState('', {})` otherwise; clear the flag either way. Page state does not survive a reload, so a reload lands closed.
- [ ] Add `web/src/lib/state/reader.test.ts` with **three** mocks, not two: `$app/navigation`, `$app/state`, and `$lib/api`. The third is not about this module, it is about its graph: `reader.svelte.ts` imports the `feed` singleton for `feed.freeze()` and `feed.items`, `feed.svelte.ts:1` imports `$lib/api`, and `api.ts:1` imports `$app/environment`, which has nothing to answer in vitest's `node` environment. `feed.test.ts:10` already mocks it for exactly this reason and says so in a comment. Cover the modifier-key predicate, the freeze-before-push order, clamping at both ends, and both close paths.

## Definition of done

- [ ] `pushState('', { brick: id })` typechecks under `exactOptionalPropertyTypes`, and `App.PageState` declares exactly one member.
- [ ] `activate` returns false and calls neither `preventDefault` nor `open` when `event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0`; it returns true and calls `preventDefault` exactly once otherwise. Both branches are vitest cases.
- [ ] `next()` and `prev()` move within `feed.items` by the id-derived index, clamp at 0 and `items.length - 1`, and call neither `feed.loadMore()` nor `fetchFeed`. When the open brick's id is absent from `feed.items`, `index` is -1 and both `canNext` and `canPrev` are false.
- [ ] The module's only SvelteKit imports are `$app/navigation` and `$app/state`, both replaceable by `vi.mock`, and it imports no `.svelte` component. Its **graph** reaches one more, through `feed.svelte.ts` to `$lib/api` to `$app/environment`, so the test mocks `$lib/api` as well; with all three mocks `pnpm vitest run src/lib/state/reader.test.ts` runs the real module in node.
- [ ] Meets the repo definition of done (tests at the right tier, negative-space cases for every rejected activation, named constants for any new bound, `just check` green, no em dash).
- [ ] Reviewable: run `cd web && pnpm vitest run src/lib/state/reader.test.ts` and read the module; every decision the reader makes is in it, and no `.svelte` file was touched.
