# Task 17 · feed picker state

**Plan:** [plan.md](../plan.md) · **Certificate:** [17-feed_picker_state-certificate.md](17-feed_picker_state-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `08-wall-and-bricks.md` → The feed picker (the five sections and the label filter), and → `07-web-client.md` → The picker is history, not a URL; implementation note 12. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md), the new picker section.
**Depends on:** 01, 14, 16
**Produces:** everything the picker knows, in the one lane a test can see: recents, three queries, their loading and error states, and the hidden-tier filter.
**Pointers:** `web/src/app.d.ts` (`App.PageState`, **created** by task 01, so this task **adds** a member rather than replacing the interface). `state/profile.svelte.ts:26` (the `browser` guard). `lib/appview.ts` (created by task 16). `HiddenLabel` in `types.ts` (created by task 14). `state/handle.svelte.ts` holds `mason:handle` in `localStorage`; `mason:feeds` sits beside it.

## Steps

- [ ] Add `picker?: 'feeds'` to `App.PageState`, alongside task 01's `brick?: string`. Do not replace the interface.
- [ ] Write the one mutual-exclusion rule the reader and feed specs both named: opening either overlay clears the other's key. The picker is a landing surface and the reader is a wall surface, so the case is rare; the rule exists so it is decided rather than emergent.
- [ ] Put the picker's half of that rule in **`feeds.svelte.ts`**, as `openPicker()` and `closePicker()`, not in `FeedPicker.svelte`. Each function pushes a page state carrying only its own key, which is what makes the exclusion structural rather than remembered. The reason is coverage, not tidiness: the reader's half is `reader.svelte.ts`'s `pushState('', { brick })` and vitest runs it, so a picker that called `pushState` from a component would leave one half of a rule this task claims to own in a file neither tsc nor vitest can see. `feeds.test.ts` mocks `$app/navigation` and `$app/state` the way task 01's `reader.test.ts` does.
- [ ] Add `web/src/lib/state/feeds.svelte.ts` owning the `mason:feeds` recents list: most recent first, deduped, capped at a named constant of 12.
- [ ] Add the three queries: popular (`getPopularFeedGenerators`, no query, paged by its cursor), search (the same endpoint with `query`), and by creator (`getActorFeeds` with a bare handle, no resolution hop), each with its own loading and error state.
- [ ] Filter out any feed whose own view carries a hidden label, or whose creator does, driving the cases from a runtime list the `HiddenLabel` type checks for completeness rather than from a retyped array.
- [ ] Add `feeds.test.ts` covering the cap, the dedupe, one case per label, and the browse-unavailable flag.

## Definition of done

- [ ] `App.PageState` carries both `brick?: string` and `picker?: 'feeds'`, and both halves of the mutual-exclusion rule are in `.ts` and vitest-covered: the reader's in `reader.svelte.ts` (task 01), the picker's in `feeds.svelte.ts`'s `openPicker()`. A vitest case asserts `openPicker()` pushes a state whose `brick` key is absent. `grep -n pushState web/src/lib/components/FeedPicker.svelte` returns nothing, which is the check that the rule did not half escape into a component task 18 ships.
- [ ] The recents cap and dedupe each have a vitest case, and the cap is a named constant.
- [ ] There is one vitest case per label in `HiddenLabel`, driven from a **runtime** value the type checks rather than from the type itself, which is erased and can generate nothing: either `const LABELS = [...] as const satisfies readonly HiddenLabel[]` **plus** an exhaustiveness check that every `HiddenLabel` appears in it (a `Record<HiddenLabel, true>` built from `LABELS`, or the equivalent), or a `Record<HiddenLabel, ...>` table iterated with `Object.keys`. `satisfies` alone is not enough: a short array satisfies `readonly HiddenLabel[]` happily. Either spelling makes a label added to mortar's `HIDDEN_LABELS` a compile error here before it is a listing bug.
- [ ] An AppView failure sets a browse-unavailable flag and leaves recents and paste working, rather than throwing or emptying the picker; a vitest case covers it.
- [ ] The module reads its base from `lib/appview.ts` and guards every fetch with SvelteKit's `browser` flag; no fourth hardcoded AppView constant exists in the tree, proven by grep.
- [ ] Meets the repo definition of done (this task is entirely `.ts`, so `cd web && pnpm test` genuinely covers it, the overlay rule included; `pnpm check:ci`, `pnpm knip` and `just check` green).
- [ ] Reviewable: `cd web && pnpm test` runs the real module in node and every state of the picker's states table is reachable from it.
