# Task 07 · merge the reader spec

**Plan:** [plan.md](../plan.md) · **Certificate:** [07-merge_the_reader_spec-certificate.md](07-merge_the_reader_spec-certificate.md)

**Implements:** [`changes/2026-07-26-read_a_brick_in_place.md`](../../../changes/2026-07-26-read_a_brick_in_place.md) §Merge plan, and every one of its `Proposed changes` blocks.
**Depends on:** 01, 02, 06
**Produces:** canonical pages 00, 07, 08 and 09 describe the reader that shipped, including the three places the change spec's implementation notes were wrong.
**Pointers:** `.specs/00-overview.md` §Goals (a numbered list ending at 7) and §Non-goals (the No blog content rendering bullet). `.specs/07-web-client.md:93` §Reactive state and its Decisions list. `.specs/08-wall-and-bricks.md:13` §Responsibilities, `:97` §Cards, `:194` §Sensitive media, `:269` §Accessibility behaviours, `:297` §Implementation layout. `.specs/09-design-system.md` §Motion. `.specs/README.md` pending and merged tables.

## Steps

- [ ] Apply every `Proposed changes` block to its canonical page and bump each edited page's `**Date:**` to the merge date.
- [ ] Write the four corrections into the canonical prose rather than copying the proposal: the reader holds the brick and locates it by id rather than storing an index; activation is from `BrickShell`'s anchor on post and blog cards, the watch link on a video card and an image anchor on a glaze card, because two of the four cards give `BrickShell` no href; the reader mounts outside the inert wrapper; the reader claims the video player under its own id so the card yields.
- [ ] Correct the overstated claims in the Dialog behaviour block: the root's `overflow: hidden` stops the reader scrolling the wall, the pump may still append bricks behind an open reader (harmless, because appends never move a laid brick and the reader locates its brick by id), and the wall's freeze listeners no longer exist once the wall is frozen.
- [ ] Flip the change spec's `**Status:**` to `Merged`, add `**Merged:**`, and move it to `.specs/changes/merged/`.
- [ ] Update `.specs/README.md`: drop the row from the pending table, add it to the merged table. Leave the "Three change specs are pending" sentence for task 27, which reconciles it once.
- [ ] Add a minor changeset describing the reader as a new surface.

## Definition of done

- [ ] Every `Proposed changes` block is applied and every edited page's `**Date:**` is the merge date.
- [ ] The four corrections are present in the canonical prose, and the change spec carries them annotated in place rather than quietly rewritten, per `.specs/README.md`'s rule about merged specs.
- [ ] `.specs/changes/merged/2026-07-26-read_a_brick_in_place.md` exists, `.specs/changes/2026-07-26-read_a_brick_in_place.md` does not, and the README's merged table lists it.
- [ ] A minor changeset exists.
- [ ] Meets the repo definition of done (`just guard-dashes` and `just check` green over the moved and edited prose).
- [ ] Reviewable: read `08-wall-and-bricks.md`'s new brick-reader section against `BrickReader.svelte` and confirm the page describes the component that exists, not the one the proposal imagined.
