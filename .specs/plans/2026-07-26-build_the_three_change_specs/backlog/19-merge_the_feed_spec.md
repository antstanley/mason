# Task 19 · merge the feed spec

**Plan:** [plan.md](../plan.md) · **Certificate:** [19-merge_the_feed_spec-certificate.md](19-merge_the_feed_spec-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Merge plan, every one of its `Proposed changes` blocks, and its §Type changes `$defs` fragment.
**Depends on:** 07, 18
**Produces:** the canonical pages and the type schema describe the feed wall that shipped, including the five places the implementation diverged from the proposal.
**Pointers:** Ten pages carry blocks: `00-overview.md` (goals, system shape, scope summary), `01-domain-model.md` (FeedRef, Cursor, query patterns), `02-feed-engine.md` (entry point, the new feed-wall section, the gate), `03-grout-and-mixer.md`, `04-sources-and-moderation.md`, `05-caching-and-persistence.md`, `06-wire-contract.md`, `07-web-client.md`, `08-wall-and-bricks.md`, `canonical-types.schema.json`. Task 07 has already claimed goal 8 in `00-overview.md`, so this task's two goals renumber onto it. **Two edits this build makes necessary carry no block at all**, because the feed spec never wrote one: `.specs/08-wall-and-bricks.md:297` §Implementation layout, which enumerates every component by name (the reader spec adds `BrickReader.svelte` and the refresh spec adds `RefreshWall.svelte`, so the feed spec's two components would be the only ones missing), and `.specs/07-web-client.md:27` §Shape, whose `lib/` tree names modules individually and whose `+page.svelte  actor ? wall : landing form` line stops being true of a feed wall. The feed spec's Shape block only appends a paragraph under the tree; it does not touch the tree.

## Steps

- [ ] Apply every `Proposed changes` block to its canonical page and bump each edited page's `**Date:**` to the merge date, renumbering the goals onto task 07's goal 8.
- [ ] Fold the `$defs` fragment into `canonical-types.schema.json`: add `FeedRef` and `HiddenLabel`, replace `CursorPayload` and `MortarErrorCode`.
- [ ] Write what shipped rather than what was proposed, in five places: the `FeedRef` return shape has two cases rather than an `Option<String>`; the glaze branch does not truncate (fix the ASCII flow, which contradicts the paragraph four lines below it); `+layout.svelte` gates the header on `actor || feed` (a file the implementation notes never mention); the `bad_request` wording names both parameters and appears in two places, one of them the service worker's own copy; and the Reactive state table gets a **row of its own** for `state/feeds.svelte.ts`. The block as drafted extends the `lastHandle` row's storage note to `mason:handle, mason:feeds` and adds no row for the new module, but task 17 puts the recents list in `web/src/lib/state/feeds.svelte.ts`. Applying it verbatim would attribute `mason:feeds` to a module that does not hold it and leave a whole state module off the canonical table. The merged table therefore reads `| state/feeds.svelte.ts | feeds | The feeds opened recently, most recent first | mason:feeds |` and the `lastHandle` row keeps `mason:handle` alone.
- [ ] Make the two edits no block covers, because a canonical page that lists components and modules by name is wrong the moment one is missing:
      - **`08-wall-and-bricks.md:297` §Implementation layout.** Add `FeedPicker.svelte` and `FeedCard.svelte` (task 18) to the `components/` list, both directly under `components/` rather than under `components/cards/`, which is brick renderers.
      - **`07-web-client.md:27` §Shape.** Add `appview.ts` (task 16) to the `lib/` tree as the single client-side AppView base, and widen the `+page.svelte  actor ? wall : landing form` line to say actor or feed, matching the `{#if actor || feed}` task 15 shipped at `+page.svelte:26`.
- [ ] Flip the change spec's `**Status:**` to `Merged`, add `**Merged:**`, and move it to `.specs/changes/merged/`.
- [ ] Update `.specs/README.md`'s pending and merged tables. Leave the "Three change specs are pending" sentence for task 27.
- [ ] Add a minor changeset describing the new surface in mason's voice.

## Definition of done

- [ ] Every `Proposed changes` block is applied, every edited page's `**Date:**` is the merge date, and the goals list numbers correctly after task 07's addition.
- [ ] `canonical-types.schema.json` carries `FeedRef` and `HiddenLabel`, and its `CursorPayload` and `MortarErrorCode` are the replaced versions.
- [ ] The five divergences above are described as the code behaves, not as the proposal read, and the change spec carries each annotated in place.
- [ ] The two blockless edits are made: `08-wall-and-bricks.md` §Implementation layout names `FeedPicker.svelte` and `FeedCard.svelte`, and `07-web-client.md` §Shape names `appview.ts` and no longer says a wall needs an actor. Checked by diffing the two lists against the tree: `ls web/src/lib/components/*.svelte` and `ls web/src/lib/*.ts` have no member the pages omit.
- [ ] `.specs/changes/merged/2026-07-26-lay_a_bluesky_feed.md` exists and the pending table no longer lists it.
- [ ] Meets the repo definition of done (`just guard-dashes`, `just check` and `just test-e2e` green; a changeset exists).
- [ ] Reviewable: run the `spec-reviewer` skill over `.specs/00` to `.specs/08` against the code in mode 2 (canonical versus code) and confirm it reports no divergence on anything this plan built.
