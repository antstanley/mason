# Done Certificate · Task 05: brick reader bodies and stepping

**Task:** [05-brick_reader_bodies_and_stepping.md](05-brick_reader_bodies_and_stepping.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27, second pass

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
  - *Status:* **SATISFIED.** Read `BrickReader.svelte:182-404` (three snippets: `postBody`,
    `blogBody`, `videoBody`) against `PostCard.svelte:13`, `BlogCard.svelte` and `VideoCard.svelte`.
    `grep -nE "truncate|line-clamp|\.slice\(0,"` over the reader returns exactly one hit, at `:229`,
    inside a comment; no markup class carries either. Driven in chromium against the real static
    build (`just build`, `pnpm preview :4173`):
    · **post**, `/?actor=demo` brick `fixture-post-9`: card renders 1 image, reader renders 5
    `<figure>`s / 5 `<img>`s / 5 `<figcaption>`s, `whiteSpace: pre-wrap` on the text, a
    `<time datetime="2026-07-10T14:03:00Z">`, and "open the post". On a synthetic wall (service
    worker blocked, `/api/feed` fulfilled by the driver) a 3-image post rendered 3 distinct
    ratios `["800 / 400","600 / 900","500 / 500"]`, 2 captions (the empty-alt image correctly gets
    none), the whole external title, all 8 repeats of the embed description and 6 kept line breaks,
    where its card carried `truncate` + `line-clamp-2` on the same embed.
    · **blog**: cover at the panel's full inner width (616px of 617px), publication chip, whole
    description (6 of 6 repeats), `<time>` published date, and "read at &lt;publication&gt;" as the
    filled primary control. Tag counts card-vs-reader on a 7-tag blog: **4 on the card, 7 in the
    reader**.
    · **video**: poster at `16 / 9`, a "Play video" control, title, `1h 22m` runtime badge on an
    archived stream, `37 watching` + `live` on a live one, activity, timestamp, watch link, and the
    `AuthorChip` in the panel header. Zero clamped nodes in any brick body; the only two in the
    dialog at all are `AuthorChip`'s own name and handle, a shared component the reader did not
    touch.

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
  - *Status:* **SATISFIED.** The id expression is `BrickReader.svelte:67`,
    ``const playerId = $derived(brick ? `reader:${brick.id}` : '')``, passed as `id={playerId}` at
    `:322`. Derived from `brick.id`, never equal to it. Resolution: `player` at `:19` is
    `import { player } from '$lib/state/player.svelte'`, the `PlayerState` singleton (step 4,
    imported); no local, parameter or snippet binding of that name exists in the file. Trace, as
    written: card playing under `fixture-video-13` → the reader's play button calls
    `player.claim('reader:fixture-video-13')` synchronously → `VideoCard.svelte:43`'s effect reads
    `playRequested && player.activeId !== brick.id` → true → `playRequested = false` → the card's
    `VideoPlayer` tears down, pausing the element and calling `player.release('fixture-video-13')`,
    which is a **no-op** because `release` guards on an id match, so the reader's claim survives its
    loser's teardown. Under the same id both branches would be false and both players would stay
    mounted and playing. Verified dynamically, with the external m3u8 reachable from this machine so
    the assertions are real playback and not element counts alone: card playing
    `{inDialog:false, paused:false, t:2.60}` → reader opened on that brick, still exactly one video,
    still the card's, still running `{t:4.16}`, reader showing poster + play button and 0 `<video>`
    → play in the reader: exactly one video, `{inDialog:true, paused:false, t:3.32}`, and
    `#wall article video` count 0. Stepping off tears it down (0 videos); stepping back shows the
    poster and the play button again rather than a live player; closing while playing leaves 0.
    Note on the trace's wording: the collapse lands on the reader's **play click**, not on open,
    because the spec's per-kind table asks for "the poster with a play button" and `guard-autoplay`
    forbids the reader mounting a player by itself. At no point were two videos playing.

- **O3 · The reader fetches nothing.**
  - *Claim:* `BrickReader.svelte` contains no `fetch`, no `fetchFeed` and no import from `$lib/api`,
    and shows the selected brick with no replies, thread or parent post.
  - *Evidence to collect:* run
    `grep -nE "fetch|\\\$lib/api" web/src/lib/components/BrickReader.svelte`, expect no hits.
  - *Status:* **SATISFIED.** `grep -nE "fetch|\$lib/api" web/src/lib/components/BrickReader.svelte`
    exits 1 with no output (not even a `fetchpriority` attribute). The file's whole import list is
    `$lib/types` (type-only), `state/reader.svelte`, `state/player.svelte`, `state/client.svelte`,
    `$lib/format`, and four components (`AuthorChip`, `Icon`, `Sensitive`, `VideoPlayer`). It
    renders `reader.showing` and nothing else: no replies, no thread, no parent post appear in the
    markup, and the driven panels contained only the one brick's own fields.

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
  - *Status:* **SATISFIED.** Both controls (`:461`, `:477`) and both arrow keys (`:132`) go through
    one local `step(delta)` at `:91`, which calls `reader.next()` / `reader.prev()` from task 01 and
    reimplements no stepping of its own; the end guard is `ReaderState.#step`'s
    `const to = feed.items[at + delta]; if (!to) return;`. `grep -n loadMore` over the reader exits 1
    with no output, and `reader.test.ts` pins that `feed.loadMore` and `fetchFeed` are never called
    at either end. Driven on the demo wall: at "brick 1 of 24" the previous control is `disabled` and
    `ArrowLeft` leaves the position at 1 of 24; **200** consecutive `ArrowRight` presses land on
    "brick 24 of 24", the next control goes `disabled`, `#wall article` is still **24** (nothing
    paged), the URL is still `/?actor=demo`, the scroll lock still reads `hidden` and the wall is
    still `inert`. Controls step too: previous → 23 of 24, next → 24 of 24.
  - *Checks (resolved):* `FeedGrid.svelte:174`'s `NAV_KEYS` is
    `{ArrowDown, ArrowUp, PageDown, PageUp, Home, End, ' '}` and `freezeOnKey` matches on membership,
    so the horizontal pair can never reach it; the two sets are disjoint, which is the property that
    carries this rather than the async `feed.freeze()`. Measured: `ArrowDown` and `PageDown` inside
    an open reader do not step, and `Alt+ArrowRight` / `Shift+ArrowRight` are declined (a modified
    arrow is browser history), while a plain `ArrowRight` steps. A focused `<video>` keeps its own
    arrows: with the reader's player focused and running, `ArrowRight` left the position unchanged.
    The only other `document` `keydown` listeners in the tree (`SwitchWall`, `GlazeCard`'s alt panel)
    match `Escape` alone and are attached only while their own surface is open.

- **O5 · The autoplay guard still passes.**
  - *Claim:* no second `.play(` call site exists anywhere in `web/src`.
  - *Evidence to collect:* run `just guard-autoplay`, expect clean. Confirm the reader reuses
    `VideoPlayer.svelte`, the one sanctioned site.
  - *Status:* **SATISFIED.** `just guard-autoplay` exits 0 (and again inside `just check`).
    `grep -rn "\.play(" web/src` returns exactly one line, `VideoPlayer.svelte:69`. The reader mounts
    that same component and calls `player.claim(...)`, never `play()`; the word "autoplay" appears
    nowhere in `web/src`, comments included.

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, and the PR states that the three renderings are covered only by
    task 06 and the double-player claim only by reading the id plus one manual pass.
  - *Evidence to collect:* run `just check`. Read the PR body for the statement.
  - *Status:* **SATISFIED.** `just check` run by this gate: **exit 0** (guard-dashes,
    guard-autoplay, guard-toolchain, fmt-check, guard-wasm, oxlint, knip, clippy, 146 cargo tests,
    both `tsc` projects, 5 vitest files / **58** tests). The four oxlint lines are pre-existing
    warnings in `FeedGrid.svelte` and `service-worker.ts`, untouched here. `just test-e2e`: 3 passed
    (second pass, after the duplicate-tag fix landed: **5 passed**, the two new `repeated-tags` cases
    included).
    Tests at the right tier with their negative space: `web/src/lib/format.test.ts` pins
    `runtimeLabel` (42s / 10m / 1h 22m) and its rejections (zero, negative, `NaN`, `Infinity` → "")
    and `dateLabel` (shape per locale) and its rejections (empty, prose, truncated → ""), and
    `reader.test.ts` gained the `total` case plus end-state assertions on the singleton. A minor
    changeset exists at `.changeset/warm-bricks-read-themselves.md`. Judging the drafted commit
    message as the PR body (this run lands one commit per task rather than opening a PR), both
    required statements are present and explicit: *"The three renderings are covered only by task
    06's Playwright spec: tsc drops .svelte files, both vitest suites are .ts, and no lane in
    `just check` can see a component body"*, and *"The double-player claim is covered by reading the
    id ... plus one manual pass"*.
    Re-read on the second pass, against the redrafted message: the same two statements survive as
    *"the three renderings are covered only by task 06: nothing in `just check` renders a component,
    since tsc cannot parse .svelte and vitest runs in node"* and *"the double-player claim is covered
    by reading the id plus one manual pass in chromium"*, and the message now also declares the
    widened scope in its own paragraph, *"one pre-existing defect fixed opportunistically, and it is
    not part of the reader work"*, naming `BlogCard.svelte` and the measured zero-article wall. The
    changeset carries a second paragraph for the same fix. The widening is therefore declared, not
    silent.

- **O7 · Reviewable: exactly one video plays.**
  - *Claim:* on `/?actor=demo`, opening a video brick's card player and then opening the reader on
    that same brick from another card leaves exactly one video playing.
  - *Evidence to collect:* perform the sequence in a browser with audio on. Record that this claim
    has no automated lane: playing a demo video needs the external m3u8 at `test-streams.mux.dev`
    and the Playwright lane is otherwise network-free.
  - *Status:* **SATISFIED.** Exercised by this gate, twice, on `/?actor=demo` against the real
    static build. The external m3u8 at `test-streams.mux.dev` **was** reachable from this machine
    (`curl` 200), so the pass asserts live playback state (`paused`, an advancing `currentTime`) and
    not only element counts; the certificate's "no automated lane" note still stands for CI, where
    the Playwright lane is network-free.
    · Same brick: play `fixture-video-13`'s card player (1 video, outside the dialog,
    `paused:false`, `t:2.60`) → open the reader on that brick from its own watch link (still 1
    video, still the card's, `t:4.16`; the panel shows a poster and a play button, 0 `<video>`) →
    press play in the reader → **exactly one** `<video>` in the document, `inDialog:true`,
    `paused:false`, `t:3.32`, and `#wall article video` is 0.
    · Different card, which is the literal reading of "from another card": play
    `fixture-video-13`'s card player, open the reader on `fixture-video-19` from *its* card, press
    play → **exactly one** video, inside the dialog, playing; the first card's element is gone.
    Closing the reader leaves 0.

## Regression check

- `VideoCard.svelte:47` collapse effect: with the reader closed, pressing play on a card still
  claims under `brick.id` and mounts the inline player : **PRESERVED**. The effect and its
  `player.claim(brick.id)` call site are byte-identical in the diff; driven, the card's play button
  still mounts an inline `<video>` that plays. The only edits to that file are the local `runtime()`
  moving to `$lib/format`'s `runtimeLabel` and the badge gate changing from `{#if brick.durationMs}`
  to `{#if runtime}`; those agree for every value the wire can carry (`null` and `0` both hide the
  badge, `4_920_000` still reads "1h 22m", pinned by `format.test.ts`), and diverge only for a
  negative duration, where the old code rendered "-1s" and the new one renders nothing.
- `Sensitive.svelte` from task 02: a brick uncovered on the card is uncovered in the reader and the
  reverse : **PRESERVED**. All reader media is wrapped with `id={brick.id}` (`:188` post images,
  `:271` blog cover, `:318` video). Driven on a covered brick: the card shows "show anyway", the
  reader opens covered, revealing inside the reader keeps the dialog open, and after closing, the
  card is uncovered too.
- `web/tests/service-worker-smoke.test.ts`: run `just test-e2e` and expect it green :
  **PRESERVED**. 3 passed; 5 on the second pass, its own 3 still green beside the 2 new ones.
- Task 03's dialog shell (not named above, checked anyway): all four dismissals (Escape, close
  control, scrim click, back gesture) shut the reader, restore `documentElement.style.overflow` to
  its empty original and drop `inert` from the wrapper; focus returns to the opening card's anchor.
  The reload-then-forward sequence (open, back, reload, forward) leaves 0 dialogs, a live wall and
  24 bricks. A `MutationObserver` on `<html>`'s `style` attribute recorded **0** mutations across
  two arrow-key steps, so stepping never releases the scroll lock.
- Task 04's card wiring: **PRESERVED**. Post, blog, video and glaze cards all still open the reader
  on a plain left click.

## Residue

- The change spec's per-kind table also names "Post (from glaze)" as reusing the post rendering.
  That falls out of the kind switch rather than being a separate obligation; confirm it incidentally.
  **Confirmed:** on `/?actor=demo&mode=glaze` a glaze card opens the same dialog and renders the
  post body (`<figure>` stack with the alt text on the page), because the kind switch keys on
  `brick.kind === 'post'` and a glaze brick is a post brick.

## Notes outside the obligations

Two observations that were not DoD items. The first pass raised both; the orchestrator asked for the
first to be fixed now and left the second optional. Both shipped, and this is what shipped.

1. **The duplicate tag key, fixed at both of its two sites.** Tags reach the client undeduped
   (`standardsite.rs:88`, `tags: doc.tags`, straight off the record), and both tag loops keyed on the
   tag itself, which Svelte answers by throwing `each_key_duplicate` mid render. Both are now keyed
   by index, each with a comment saying why the key cannot be the tag:
   `BrickReader.svelte:306` (`{#each blog.tags as tag, i (i)}`) and `BlogCard.svelte:38`
   (`{#each brick.tags.slice(0, 4) as tag, i (i)}`). The `BlogCard` half is a pre-existing defect,
   not a regression this task introduced, and the commit message says so in its own paragraph.
   Measured by this gate on a rebuilt counterfactual (the same tree with only the two keys reverted,
   `pnpm build`, served alongside the real one), driving a crafted wall of three bricks with the
   service worker blocked and `/api/feed` fulfilled by the driver:
   · repeat *inside* the card's first four, `['a','b','a','c','d']`: **before**, `#wall article` is
   **0** and the page throws `each_key_duplicate`, the whole wall gone rather than one brick;
   **after**, **3** articles, 4 chips on the card (`#a #b #a #c`), the reader opens on it with all
   **5** (`#a #b #a #c #d`), and nothing is thrown.
   · repeat *beyond* the fourth, `['a','b','c','d','a']`, which is the reader's half alone since the
   card's slice then holds no repeat: **before**, the wall lays 3 articles but the click does nothing
   and no dialog ever appears, with `each_key_duplicate` thrown; **after**, the dialog opens with the
   5 chips. That second run is the "only the card fixed" counterfactual by construction, so each
   site's fix is shown to be necessary on its own.
   · `web/tests/repeated-tags.test.ts` pins both at the only tier that mounts a component, and its
   two cases pass inside `just test-e2e` (5 passed).
   · The index key trades a repeat-throw for node reuse across a data change, so that was measured
   too: stepping a 5-tag blog → 2-tag → 3-tag → back re-rendered exactly `[#a #b #a #c #d]`,
   `[#x #y]`, `[#p #q #p]`, `[#a #b #a #c #d]` with no stale chip, and the post body's
   index-keyed image loop behaved the same across 3 → 1 → 5 images, each figure carrying its own
   alt, caption and aspect ratio. `web/src` holds no third tag loop (`grep -rn "\.tags"` finds only
   these two), and no other keyed `{#each}` in the tree keys on a value that can repeat.
2. **Focus after a step, fixed.** `step()` now asks where focus ended up rather than which of two
   things happened to the control that stepped: `void tick().then(() => { if (panel &&
   !panel.contains(document.activeElement)) panel.focus(); })`. Measured on the demo wall:
   stepping off a blog's "read at" control (a control only blogs render) leaves
   `document.activeElement` on the panel (`DIV`, `role="dialog"`) where it was `<body>` before;
   stepping to the end so the next control goes `disabled` also lands on the panel; and a control
   that survives the step keeps focus (`BUTTON`, "next brick", still focused after a step). No case
   measured leaves focus on `<body>`.

## Second pass · what this gate re-ran

The retry delta is exactly six things (`jj diff` from the first pass's revision to `@`): the `tick`
import, `step()`'s focus block, the reader's tag key plus its comment, `BlogCard`'s tag key plus its
comment, a changeset paragraph, and the new `web/tests/repeated-tags.test.ts`. Nothing in the reader
bodies, the player claim, the stepping or `$lib/format` moved, so the first pass's reading of those
carries forward; everything below was collected fresh anyway.

- `just check`: **exit 0** (146 cargo tests, both `tsc` projects, 5 vitest files / 58 tests; the same
  four pre-existing oxlint warnings). `just test-e2e`: **5 passed**. `just guard-autoplay`: exit 0,
  and `grep -rn "\.play(" web/src` still returns the single `VideoPlayer.svelte:69`.
- Function resolution over the changed lines: `tick` resolves to the `svelte` import at `:17` (step
  4, imported), the file declaring no local or module-level `tick`; `panel` is the `$state` binding
  at `:30`, `bind:this` at `:443`, with no shadow; `document` is the global, no local declaration in
  either file; the two `i` bindings are each-block-scoped in different snippets and shadow nothing.
- Stepping, re-driven: 1 of 24 with `previous brick` `disabled` and `ArrowLeft` a no-op; 200
  consecutive `ArrowRight` land on 24 of 24 with `next brick` `disabled`, `#wall article` still
  **24**, the URL still `/?actor=demo`; controls step (23 of 24, 24 of 24); `ArrowDown`, `PageDown`
  and `Shift+ArrowLeft` do not step.
- The scroll lock across a step: a `MutationObserver` on `<html>`'s `style` attribute recorded **0**
  mutations across one step and **0** across three, with `style.overflow` still `hidden` after.
- The distinct player id, re-driven with `test-streams.mux.dev` reachable (`curl` 200): card playing
  (1 video, in the wall, `paused:false`) → reader opened over the same brick (still 1, still the
  card's, panel showing poster and play button) → play in the reader (**exactly 1**, `inDialog:true`,
  `paused:false`, `#wall article video` 0) → step off (0) → step back (0, poster and play button
  again) → play again (1, in the dialog) → Escape while playing (0). The different-card sequence
  ends the same way. Two videos never coexisted at any phase.
- Regression surface, re-driven: all four dismissals, plus a fifth case the retry could have
  disturbed (two arrow steps, then Escape), each leave 0 dialogs, `documentElement.style.overflow`
  back to its empty original, no `inert`, and focus on the opening card's anchor. Open, back, reload,
  forward leaves 0 dialogs over a live, clickable, non-inert 24-brick wall. Post, blog, video and
  glaze all still open the reader; the covered demo brick opens covered, reveals inside the dialog
  without closing it, and leaves its card uncovered afterwards.
- The bodies, spot-checked again since the blog body was touched: `fixture-post-9` renders 1 image on
  its card and 5 figures / 5 captions in the reader; the only two clamped nodes anywhere in the
  dialog are `AuthorChip`'s name and handle.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED on evidence this gate collected itself across two passes
(`just check` exit 0, `just guard-autoplay` exit 0, `just test-e2e` 5 passed, and the built site
driven in chromium for the per-kind field counts card-versus-reader, both ends of the stepping with
no pagination across 200 presses, zero `<html>` style mutations per step, and the double-player
sequence under real playback), the duplicate-tag defect is fixed at both of its sites with a
rebuilt-counterfactual before-and-after measured for each and the widened scope declared in the
commit message, and every regression line is PRESERVED.
