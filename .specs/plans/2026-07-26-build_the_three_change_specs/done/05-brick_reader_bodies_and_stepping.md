# Task 05 · brick reader bodies and stepping

**Plan:** [plan.md](../plan.md) · **Certificate:** [05-brick_reader_bodies_and_stepping-certificate.md](05-brick_reader_bodies_and_stepping-certificate.md)

**Implements:** [`changes/merged/2026-07-26-read_a_brick_in_place.md`](../../../changes/merged/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `08-wall-and-bricks.md` → The brick reader (the per-kind table and the arrow-key rule). Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md), the new brick-reader section.
**Depends on:** 02, 03, 04
**Produces:** the reader earns the click: all three kinds render what the card left out, and arrow keys step along the laid wall without paginating it.
**Pointers:** `FeedGrid.svelte:204` is the kind switch to mirror. `PostCard.svelte:13` renders only `images[0]`; `BlogCard` clamps `description` to three lines and slices `tags` to four; the external embed is truncated. `VideoCard.svelte:47` collapses the card when `player.activeId !== brick.id`, and `player.claim(id)` at `VideoCard.svelte:89` sets it. `VideoPlayer.svelte` is the one sanctioned `.play(` call site.

## Steps

- [ ] Add the kind switch to `BrickReader.svelte`, reusing `AuthorChip`, `Sensitive`, `Icon` and `VideoPlayer`.
- [ ] Post: render every image at its own aspect ratio, the text unclamped, `external` with its whole description, and `createdAt`.
- [ ] Blog: cover at full width, publication chip, whole `description`, every entry of `tags`, `publishedAt`, and "read at <publication.name>" as the primary control.
- [ ] Video: poster plus a play button that mounts the existing `VideoPlayer` under a player id **distinct** from the card's, for example `reader:<brick.id>`, so `VideoCard.svelte:47` sees a losing claim and tears its own player down.
- [ ] Wrap all reader media in `Sensitive` with `id={brick.id}`.
- [ ] Add left and right arrow key handling and two visible previous/next controls, both stepping within `feed.items` and stopping at the ends.

## Definition of done

- [ ] Each kind renders the fields the change spec's table names, and none of them is clamped, truncated or sliced the way the card clamps it.
- [ ] The reader passes `VideoPlayer` an id derived from but not equal to `brick.id`, so opening the reader over a playing card tears the card's player down instead of leaving two elements playing. Passing the same id would leave both mounted.
- [ ] The reader shows the selected brick and nothing around it: `BrickReader.svelte` contains no `fetch`, no `fetchFeed` and no import from `$lib/api`, proven by grep.
- [ ] Stepping stops at both ends of the laid wall and never calls `feed.loadMore()`.
- [ ] `just guard-autoplay` still passes: no second `.play(` call site anywhere in `web/src`.
- [ ] Meets the repo definition of done (`just check` green, and the PR states that the three renderings are covered only by task 06 and the double-player claim by reading the id plus one manual pass).
- [ ] Reviewable: on `/?actor=demo`, open a video brick's card player, then open the reader on that same brick from another card, and confirm exactly one video is playing.

## Open questions

- The double-player claim has no automated lane at all: playing a demo video needs the external m3u8 at `test-streams.mux.dev` and the Playwright lane is otherwise network-free.
