# Task 07 · merge the reader spec

**Plan:** [plan.md](../plan.md) · **Certificate:** [07-merge_the_reader_spec-certificate.md](07-merge_the_reader_spec-certificate.md)

**Implements:** [`changes/merged/2026-07-26-read_a_brick_in_place.md`](../../../changes/merged/2026-07-26-read_a_brick_in_place.md) §Merge plan, and every one of its `Proposed changes` blocks.
**Depends on:** 01, 02, 06
**Produces:** canonical pages 00, 07, 08 and 09 describe the reader that shipped, with the four corrections the change spec absorbed on 2026-07-26 confirmed present rather than re-authored here.
**Pointers:** `.specs/00-overview.md` §Goals (a numbered list ending at 7) and §Non-goals (the No blog content rendering bullet). `.specs/07-web-client.md:93` §Reactive state and its Decisions list. `.specs/08-wall-and-bricks.md:13` §Responsibilities, `:97` §Cards, `:194` §Sensitive media, `:269` §Accessibility behaviours, `:297` §Implementation layout. `.specs/09-design-system.md` §Motion. `.specs/README.md` pending and merged tables.

## Steps

- [ ] Apply every `Proposed changes` block to its canonical page and bump each edited page's `**Date:**` to the merge date.
- [ ] Confirm each of the four corrections is present in the applied blocks rather than authoring it here. Commit `b55ef455` corrected the change spec itself on 2026-07-26, so all four are now block text: the reader holds the brick and locates it by id rather than storing an index (the `07-web-client.md` Reactive state block); activation is from `BrickShell`'s anchor on post and blog cards, the watch link on a video card and an image anchor on a glaze card, because two of the four cards give `BrickShell` no href (the Cards block); the reader mounts as the wrapper's sibling, outside the inert subtree (the Dialog behaviour block); and the reader claims the video player under `reader:<brick.id>` so the card yields (the same block). Any one missing from the canonical page after the merge means the block was applied incompletely, not that this task failed to invent it.
- [ ] Confirm the same for the three statements the Dialog behaviour and Accessibility blocks now make about the reader's limits, which the proposal once overstated: the root's `overflow: hidden` stops the wall scrolling but not the pump, the sentinel keeps appending behind an open reader (harmless, because an append never moves a laid brick and the reader locates its brick by id), and the reader's left and right arrows cannot collide with the wall's navigation-key freeze handler because **the two key sets are disjoint**: `FeedGrid.svelte:174`'s `NAV_KEYS` is the vertical set (`ArrowDown`, `ArrowUp`, `PageDown`, `PageUp`, `Home`, `End`, space) and the reader steps on `ArrowLeft` and `ArrowRight`. The disjointness is what carries it, not the freeze `reader.open` performs: `feed.freeze()` is async and does not clear `warming` until its fetch resolves, so the wall's listeners are still attached while the reader mounts.
- [ ] Flip the change spec's `**Status:**` to `Merged`, add `**Merged:**`, and move it to `.specs/changes/merged/`.
- [ ] Update `.specs/README.md`: drop the row from the pending table, add it to the merged table. Leave the "Three change specs are pending" sentence for task 27, which reconciles it once.
- [ ] Add a minor changeset describing the reader as a new surface.

## Definition of done

- [ ] Every `Proposed changes` block is applied and every edited page's `**Date:**` is the merge date.
- [ ] The four corrections are present in the canonical prose, carried there by the blocks rather than written by hand. The change spec needs no correction annotation for them: `b55ef455` fixed the spec on 2026-07-26, so the block and the code already agree. Anything found during this merge that they do **not** agree on is annotated in place, per `.specs/README.md`'s rule about merged specs.
- [ ] `.specs/changes/merged/2026-07-26-read_a_brick_in_place.md` exists, `.specs/changes/2026-07-26-read_a_brick_in_place.md` does not, and the README's merged table lists it.
- [ ] A minor changeset exists.
- [ ] Meets the repo definition of done (`just guard-dashes` and `just check` green over the moved and edited prose).
- [ ] Reviewable: read `08-wall-and-bricks.md`'s new brick-reader section against `BrickReader.svelte` and confirm the page describes the component that exists, not the one the proposal imagined.
