# Done Certificate · Task 05: brick reader bodies and stepping

**Task:** [05-brick_reader_bodies_and_stepping.md](05-brick_reader_bodies_and_stepping.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 05. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 05) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The reader earns the click: all three kinds render what the card left out, and
  arrow keys step along the laid wall without paginating it.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break the one-video-at-a-time rule enforced by `player.claim` and
  `VideoCard.svelte:47`'s collapse effect, the `Sensitive` reveal sharing from task 02, or
  `just guard-autoplay`'s single-`.play(`-site rule.

## Obligations

- **O1 · Each kind renders the fields the card had to leave out.**
  - *Claim:* post shows every image at its own aspect ratio, unclamped text, the external embed with
    its whole description and `createdAt`; blog shows a full-width cover, publication chip, whole
    `description`, every tag, `publishedAt` and a read-at-publication primary control; video shows
    poster, title, activity or viewer count, runtime and author.
  - *Evidence to collect:* read `BrickReader.svelte`'s kind switch and compare field by field
    against `PostCard.svelte` (which renders only `images[0]` at `:13`), `BlogCard.svelte` (which
    clamps and slices) and `VideoCard.svelte`. Confirm no `truncate`, `line-clamp-*` or `.slice(0,`
    appears in the reader's own markup.
  - *Status:* unverified

- **O2 · The reader claims the player under a distinct id, so the card yields.**
  - *Claim:* the reader passes `VideoPlayer` an id derived from but not equal to `brick.id`, so
    `VideoCard.svelte:47` sees `player.activeId !== brick.id` and tears its own player down.
  - *Evidence to collect:* read the `VideoPlayer` invocation in `BrickReader.svelte` and record the
    exact id expression. Confirm it is not `brick.id`.
  - *Checks:* resolve `player` to the singleton in `$lib/state/player.svelte`, not a local. Trace:
    with the card's player active under `brick.id`, opening the reader must set `activeId` to the
    reader's id, which makes the card's `$effect` condition true and collapses it. Passing the same
    id would leave both mounted and both playing, one behind a scrim, and neither `inert` nor
    `overflow: hidden` pauses either.
  - *Status:* unverified

- **O3 · The reader fetches nothing.**
  - *Claim:* `BrickReader.svelte` contains no `fetch`, no `fetchFeed` and no import from `$lib/api`,
    and shows the selected brick with no replies, thread or parent post.
  - *Evidence to collect:* run
    `grep -nE "fetch|\\\$lib/api" web/src/lib/components/BrickReader.svelte`, expect no hits.
  - *Status:* unverified

- **O4 · Stepping stops at the ends and never paginates.**
  - *Claim:* left and right arrow keys and two visible controls step within `feed.items`, both stop
    at the ends, and neither triggers `feed.loadMore()`.
  - *Evidence to collect:* confirm the controls call `reader.next()` and `reader.prev()` from task
    01 rather than reimplementing the step. Run
    `grep -n loadMore web/src/lib/components/BrickReader.svelte`, expect no hits. Drive the demo
    wall and press right arrow from the last laid brick.
  - *Checks:* resolve the arrow-key handler's scope: with the wall inert, `FeedGrid`'s window-level
    `keydown` freeze handler must not also fire. Confirm the reader's handler does not bubble into a
    second freeze.
  - *Status:* unverified

- **O5 · The autoplay guard still passes.**
  - *Claim:* no second `.play(` call site exists anywhere in `web/src`.
  - *Evidence to collect:* run `just guard-autoplay`, expect clean. Confirm the reader reuses
    `VideoPlayer.svelte`, the one sanctioned site.
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, and the PR states that the three renderings are covered only by
    task 06 and the double-player claim only by reading the id plus one manual pass.
  - *Evidence to collect:* run `just check`. Read the PR body for the statement.
  - *Status:* unverified

- **O7 · Reviewable: exactly one video plays.**
  - *Claim:* on `/?actor=demo`, opening a video brick's card player and then opening the reader on
    that same brick from another card leaves exactly one video playing.
  - *Evidence to collect:* perform the sequence in a browser with audio on. Record that this claim
    has no automated lane: playing a demo video needs the external m3u8 at `test-streams.mux.dev`
    and the Playwright lane is otherwise network-free.
  - *Status:* unverified

## Regression check

- `VideoCard.svelte:47` collapse effect: with the reader closed, pressing play on a card still
  claims under `brick.id` and mounts the inline player : (PRESERVED / REGRESSION)
- `Sensitive.svelte` from task 02: a brick uncovered on the card is uncovered in the reader and the
  reverse : (PRESERVED / REGRESSION)
- `web/tests/service-worker-smoke.test.ts`: run `just test-e2e` and expect it green :
  (PRESERVED / REGRESSION)

## Residue

- The change spec's per-kind table also names "Post (from glaze)" as reusing the post rendering.
  That falls out of the kind switch rather than being a separate obligation; confirm it incidentally.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
