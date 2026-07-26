# Task 18 · feed picker screen

**Plan:** [plan.md](../plan.md) · **Certificate:** [18-feed_picker_screen-certificate.md](18-feed_picker_screen-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `08-wall-and-bricks.md` → The feed picker (the card, the two entry points, the states table and the dialog clause); implementation note 12. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md), the new picker section.
**Depends on:** 03, 17
**Produces:** the second front door: a screen that stands beside the handle box as a peer, reachable from the landing page and from a laid wall.
**Pointers:** `HandleForm.svelte:67`-`:77` (under the box, the landing-page way in). `SwitchWall.svelte:120`-`:147` (the switch-walls affordance, the wall way in). `BrickReader.svelte` (task 03) is the established dialog language to mirror: `role="dialog"`, `aria-modal`, focus in on open and back to the trigger on close, `inert` behind, Escape and back. `+layout.svelte:110`-`:134` is the wrapper task 03 marks `inert`, with `BrickReader` mounted **after** its closing tag at `:134`; `FeedPicker` mounts in the same place, beside `BrickReader`, and this task **widens** task 03's condition on that wrapper rather than introducing a second one. `FeedCard.svelte` goes in `components/`, **not** `components/cards/`, which is brick renderers.

## Steps

- [ ] Add `FeedPicker.svelte` and `FeedCard.svelte`, opening from both entry points by calling task 17's `openPicker()` and closing with `closePicker()`. **Do not call `pushState` here**: the mutual-exclusion rule with the reader lives in `.ts` so vitest can see both of its halves, and a component copy would put one half back where nothing checks it.
- [ ] Make `page.state.picker` the only source of truth for whether it is open, so the address bar keeps showing whatever is behind it.
- [ ] Wire back, Escape and the close control to close it; focus enters the input on open and returns to the trigger on close; results are a list so a screen reader is told how many there are.
- [ ] Make the content behind `inert` by **widening task 03's condition**, not by adding a second attribute: the wrapper at `+layout.svelte:110` goes inert on `page.state.brick || page.state.picker`, and `FeedPicker` mounts after that wrapper's closing tag at `:134`, beside `BrickReader`. Both halves matter. `inert` is inherited, so a picker mounted inside the wrapper makes itself inert; and a condition **replaced** rather than widened leaves the reader no longer making the wall inert, which nothing in `just check` can see and task 06 would only catch by accident.
- [ ] Make one input serve search, by-creator and paste, deciding from what the reader typed which question is asked. An unparseable pasted value says so in place and navigates nowhere.
- [ ] Give each card the feed's avatar, display name, creator handle, a two-line-clamped description and a like count hidden at zero; activating it navigates to `/?feed=<uri>` and writes to `mason:feeds`.
- [ ] Add `web/tests/feed-picker.test.ts` covering opening from the landing page, Escape closing, the back gesture closing, and a bad paste showing the inline error without navigating, with the AppView list stubbed or the browse-unavailable state used so the case needs no network.

## Definition of done

- [ ] All five rows of the picker's states table render, and the pasted-value error appears in place with no navigation.
- [ ] Every control is at least 44px and the card's hover lift is behind `motion-safe:`.
- [ ] `web/tests/feed-picker.test.ts` is green offline and its header states that Playwright is the **only** lane that can see either new component: tsc drops both and neither vitest suite imports a component.
- [ ] `FeedPicker` mounts outside the subtree it makes inert (after `+layout.svelte:134`, beside `BrickReader`), and the wrapper's `inert` is **one** widened expression, `page.state.brick || page.state.picker`. Verified by reading the markup, which is the only lane there is: `grep -n inert web/src/routes/+layout.svelte` returns a single hit naming both keys, and task 06's reader case is still green, which is what a replaced condition would break.
- [ ] `pnpm knip` is green, which means both new components are reachable from an entry.
- [ ] `grep -n pushState web/src/lib/components/FeedPicker.svelte web/src/lib/components/FeedCard.svelte` returns nothing: every history write goes through task 17's two functions, which are the vitest-covered half of the overlay rule.
- [ ] Meets the repo definition of done (`just guard-autoplay`, `just guard-dashes`, `just check` and `just test-e2e` all green).
- [ ] Reviewable: `just test-e2e`, then open the built site, click into the picker from the landing page, paste a garbage value, and press back.
