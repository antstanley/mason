# Done Certificate · Task 07: merge the reader spec

**Task:** [07-merge_the_reader_spec.md](07-merge_the_reader_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27

> Verification protocol for Task 07. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 07) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** Canonical pages 00, 07, 08 and 09 describe the reader that shipped, with the four
  corrections the change spec absorbed on 2026-07-26 confirmed present rather than re-authored.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break `.specs/README.md`'s tables or any intra-spec link. Canonical
  pages describe the current branch, so nothing may be written that the code does not do.

## Obligations

- **O1 · Every proposed block is applied and every touched page is redated.**
  - *Claim:* each `Proposed changes` block in
    `.specs/changes/merged/2026-07-26-read_a_brick_in_place.md` appears in its canonical page, and
    `00-overview.md`, `07-web-client.md`, `08-wall-and-bricks.md` and `09-design-system.md` each
    carry the merge date in their `**Date:**` header.
  - *Evidence to collect:* walk the change spec's `Proposed changes` section block by block and
    locate each in its target page. Read the four `**Date:**` headers.
  - *Collected:* `grep -n '^### `\.specs' .specs/changes/merged/2026-07-26-read_a_brick_in_place.md`
    returns thirteen blocks, and all thirteen landed. `00-overview.md`: Goals item 8 at `:54`,
    the replaced No-blog-content non-goal at `:68`. `07-web-client.md`: Responsibilities item 5 at
    `:21`, the two Reactive state rows at `:109` and `:110`, the `### The reader is history, not a
    URL` subsection at `:129` to `:173`, the four Decisions bullets at `:370`, `:374`, `:377` and
    `:381`. `08-wall-and-bricks.md`: Responsibilities item 6 at `:21`, the replaced Outbound-links
    bullet at `:125` to `:144`, the whole `## The brick reader` section at `:174` to `:236` with its
    `### Dialog behaviour` subsection at `:204`, the Sensitive media prose at `:285`, the
    Accessibility dialog bullet at `:385`, and `BrickReader.svelte` in the implementation layout at
    `:415`. `09-design-system.md`: the Brick-reader-open motion row at `:176`. The four
    `**Status:** Draft · **Date:**` headers at `:3` of 00, 07, 08 and 09 all read 2026-07-27, the
    merge date. Three merge-time departures from block text are annotated in the merged change spec
    rather than made silently, which is what `.specs/README.md:44` asks of a merged spec: the
    Reactive state block's "single source of truth" clause (annotated at `:122` to `:130`), the
    brick-reader block's open predicate and stepping bullet (`:242` to `:249`), and the Sensitive
    block's missing double stop (`:263` to `:270`).
  - *Status:* SATISFIED

- **O2 · The four corrections are in the canonical prose, carried by the blocks rather than re-authored.**
  - *Claim:* the pages say the reader holds the brick and locates it by id rather than storing an
    index; that activation is from `BrickShell`'s anchor on post and blog cards, the watch link on a
    video card and an image anchor on a glaze card; that the reader mounts as the wrapper's sibling,
    outside the inert subtree; and that the reader claims the video player under `reader:<brick.id>`
    so the card yields. The Dialog behaviour and Accessibility blocks' statements about the pump and
    about the arrow keys are present and true, the second of them stated as **disjoint key sets**
    (`FeedGrid.svelte:174`'s vertical `NAV_KEYS` against the reader's `ArrowLeft` / `ArrowRight`)
    rather than as listeners the reader's freeze has torn down, which the freeze does not do: it is
    async and leaves `warming` set until its fetch resolves.
  - *Evidence to collect:* read `08-wall-and-bricks.md`'s new brick-reader section and
    `07-web-client.md`'s reactive-state prose. For each of the four, confirm the canonical text
    matches the code: `reader.svelte.ts`'s `findIndex`, the five intercepted anchors,
    `+layout.svelte`'s mount position, and the reader's `VideoPlayer` id.
  - *Checks:* the change spec once got all four wrong and was corrected in commit `b55ef455` on
    2026-07-26, so each is now block text: a correction missing from the canonical page means the
    block was applied incompletely, not that the merge failed to invent it. Resolve each canonical
    claim against the shipped file all the same, never against the block.
  - *Collected:* all four are on the canonical pages, and all four resolve against the shipped
    files, not against the block.
    (1) `07-web-client.md:160` reads "**The reader holds the brick, and locates it by id.**" with the
    `feed.items.findIndex(b => b.id === id)` rationale. `reader.svelte.ts:94` to `:98` is a `get
    index()` accessor computing `feed.items.findIndex((b) => b.id === open.id)` on every read; there
    is no stored index field on `ReaderState` (`:51` `brick`, `:54` `#opener`, `:57` `#pushed`, and
    nothing else), and `#step` at `:190` re-reads it.
    (2) `08-wall-and-bricks.md:138` reads "**Which anchor stands for the brick differs by card, and
    there are three of them.**", naming `BrickShell`'s anchor on post and blog, the video card's
    watch link and the glaze card's per-image anchors. `grep -rn 'reader.activate' web/src` returns
    exactly four call sites for those three roles: `BrickShell.svelte:52`, `VideoCard.svelte:154`,
    `GlazeCard.svelte:159` (carousel branch) and `GlazeCard.svelte:214` (single and grid branch).
    Only `PostCard.svelte:17` and `BlogCard.svelte:13` pass `BrickShell` an `href`;
    `VideoCard.svelte:47` and `GlazeCard.svelte:136` pass none, so intercepting `BrickShell` alone
    would leave the glaze wall and every video brick with no way in, exactly as the page says.
    (3) `08-wall-and-bricks.md:214` reads "The reader is mounted as that wrapper's **sibling**, not
    inside it". `+layout.svelte:135` opens the wrapper `div` carrying `inert={overlayOpen}` at
    `:136`, `:165` renders `{@render children()}`, `:166` closes it, and `<BrickReader />` sits at
    `:171`, after the close.
    (4) `08-wall-and-bricks.md:229` reads "the reader must claim the player under **its own id**
    (`reader:<brick.id>`)". `BrickReader.svelte:74` derives `playerId` as `` `reader:${brick.id}` ``,
    `:367` claims it with `player.claim(playerId)`, and `:344` hands the same id to `VideoPlayer`.
    `VideoCard.svelte` claims the bare `brick.id`, so the two never collide.
    The three limit statements are present and true too. `08:219` says `document.documentElement`
    gets `overflow: hidden` and that this does not stop the pump, which matches
    `BrickReader.svelte:150` to `:157` (it sets `root.style.overflow` and touches nothing else) and
    `FeedGrid.svelte:146` to `:156`, where the sentinel's `IntersectionObserver` is disconnected only
    by its own effect teardown. `08:222` says an append is harmless because it never moves a laid
    brick and the reader locates its own brick by id, which is the same `findIndex` as (1).
    `08:385` to `:395` states the arrow-key case as disjoint key sets and says in as many words
    "The disjointness is what carries this, not the freeze", with the async-freeze reason.
  - *Checked:* `NAV_KEYS` resolves to `FeedGrid.svelte:192`, a module-scope `const` inside the
    component script, `new Set(['ArrowDown', 'ArrowUp', 'PageDown', 'PageUp', 'Home', 'End', ' '])`,
    read only by `freezeOnKey` at `:193`. It has moved from `:174` since the certificate was
    authored; the canonical page cites no line number, so nothing on it went stale. Its members are
    disjoint from the reader's `ArrowLeft` and `ArrowRight` at `BrickReader.svelte:131`. `freeze`
    resolves to `FeedState.freeze` at `feed.svelte.ts:140`, an `async` method whose `warming` clears
    in the `finally` at `:163` to `:165`, after its `await fetchFeed(...)` at `:154`, so the page's
    reason for refusing to credit the freeze is the shipped one. The certificate's evidence line
    says "the five intercepted anchors"; the code has four `reader.activate` call sites across three
    anchor roles, so I discharged the claim against the four and the drift is noted in the summary.
    One block claim did NOT survive the build and is not on the canonical page: `page.state.brick`
    as the single source of truth for whether the reader is open. `grep -rn 'single source of truth'
    .specs/*.md` returns nothing. What shipped is `ReaderState.showing` at `reader.svelte.ts:72`
    (page state naming a brick AND the rune holding that same brick) with `isOpen` at `:85`, and
    `+layout.svelte:38` derives `overlayOpen` from `reader.isOpen`. `07-web-client.md:144` carries
    that shipped predicate under "**Open is page state and the held brick agreeing.**", and
    `08-wall-and-bricks.md:178` says the reader renders nothing "unless the reader is showing a
    brick" rather than unless page state is set. The superseded clause is annotated in place in the
    merged change spec, per `.specs/README.md:44`.
  - *Status:* SATISFIED

- **O3 · The spec is merged and relocated.**
  - *Claim:* the change spec's `**Status:**` is `Merged`, it carries a `**Merged:**` date, it lives
    at `.specs/changes/merged/2026-07-26-read_a_brick_in_place.md`, it no longer exists at
    `.specs/changes/`, and `.specs/README.md`'s merged table lists it while the pending table does
    not.
  - *Evidence to collect:* run
    `ls .specs/changes/merged/2026-07-26-read_a_brick_in_place.md .specs/changes/2026-07-26-read_a_brick_in_place.md`
    and expect the first to exist and the second not to. Run
    `grep -n read_a_brick_in_place .specs/README.md` and confirm it appears only in the merged table.
    Confirm the "Three change specs are pending" sentence is still present, because task 27 owns it.
  - *Collected:* `ls` reports `.specs/changes/merged/2026-07-26-read_a_brick_in_place.md` present and
    `.specs/changes/2026-07-26-read_a_brick_in_place.md` absent; `.specs/changes/` now holds only
    `2026-07-26-lay_a_bluesky_feed.md`, `2026-07-26-refresh_the_wall.md` and `merged/`. `jj st`
    records the move as a rename (`R .specs/changes/{ => merged}/...`), so the history is kept rather
    than a delete plus an add. The moved file's header at `:3` reads
    `**Status:** Merged · **Date:** 2026-07-26 · **Merged:** 2026-07-27`, matching the three merged
    specs already in that folder. `grep -n read_a_brick_in_place .specs/README.md` returns one hit,
    `:61`, in the merged table, dated 2026-07-27; the pending table at `:51` to `:54` now carries only
    the other two rows. `.specs/README.md:47` still opens "Three change specs are pending", untouched
    by this diff, which is what task 27 owns.
  - *Checks (regression surface named in the task):* the five `../` links inside the moved file were
    rebased to `../../` (`:44` to `:47` and `:488`) and all five resolve from
    `.specs/changes/merged/`. The two remaining relative links at `:207` and `:255` sit inside quoted
    `Proposed changes` blocks, which the file's own note at `:53` to `:55` declares page-relative
    rather than directory-relative, and both resolve from the canonical page each block landed on.
  - *Status:* SATISFIED

- **O4 · A changeset exists.**
  - *Claim:* a minor changeset describes the reader as a new surface.
  - *Evidence to collect:* run `ls .changeset/*.md` and read the new file; confirm the bump level is
    minor and the text is in mason's voice.
  - *Collected:* `.changeset/a-brick-opens-where-it-lies.md` is the one file this diff adds. Its
    front matter is `"mason": minor`, which is the level a new surface takes. The body is lowercase,
    brick-punning and brief: a plain click lifts the brick into a reader over the wall, the back
    gesture, Escape, the close button and the scrim all shut it, a modified click still goes straight
    out because every card is still a real link, and the sensitive reveal now follows the brick. It
    is distinct from the pre-existing `warm-bricks-read-themselves.md`, which describes what the
    reader renders rather than that the surface exists.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green over the moved and edited prose.
  - *Evidence to collect:* run `just guard-dashes` and `just check`. `guard-dashes` scans the whole
    tree by denylist, so a moved spec file is still in scope.
  - *Collected:* I ran both in `/Users/ant/code/mason-ws1` myself rather than reading the report.
    `just guard-dashes` alone: exit 0, no U+2014 anywhere the denylist does not exclude, and
    `justfile:109` shows it greps the filesystem from `.`, so both the moved spec and this
    certificate are in scope. `just check` (`justfile:171`, which is guard-dashes, guard-autoplay,
    guard-toolchain, fmt-check, guard-wasm, lint and test): exit 0. Inside it, oxfmt reported all 26
    files correctly formatted, `cargo fmt --all --check` was silent, the wasm32 `cargo check` and the
    wasm-pack build finished clean, oxlint emitted only the four pre-existing warnings (three
    `no-await-in-loop` in `FeedGrid.svelte`, one `preserve-caught-error` in `service-worker.ts`),
    knip was clean, `cargo clippy --workspace --all-targets -- -D warnings` was clean, nextest ran
    154 tests with 154 passed, `pnpm check:ci` typechecked both projects, and vitest ran 70 tests in
    6 files with 70 passed. I also ran `CI=1 just test-e2e`, which the task's definition of done does
    not require but which is the only lane that can see the component this prose describes: 20 tests
    passed in chromium against a build served on a Playwright-owned port 4173 with `--strictPort`, so
    it cannot have attached to another workspace's server.
  - *Status:* SATISFIED

- **O6 · Reviewable: the page describes the component that exists.**
  - *Claim:* a reviewer reads `08-wall-and-bricks.md`'s new brick-reader section alongside
    `BrickReader.svelte` and finds no statement the component does not satisfy.
  - *Evidence to collect:* the side-by-side read. Optionally run the `spec-reviewer` skill in
    mode 2 (canonical versus code) scoped to `08-wall-and-bricks.md`.
  - *Collected:* I read `08-wall-and-bricks.md:174` to `:236` against all 510 lines of
    `BrickReader.svelte`, claim by claim, and every claim resolves.
    Mounted once in `+layout.svelte` (`:171`) and renders nothing unless the reader is showing a
    brick (`BrickReader.svelte:39` `const brick = $derived(reader.showing)`, gating the whole markup
    at `:425` `{#if brick}`). Reading width is `max-w-2xl` at `:449`. Nothing is fetched: the three
    body snippets read only the `Brick` they are handed.
    The per-kind table. Post (`:196` to `:270`): every image in an `{#each post.images}` with a
    per-image `aspect-ratio` and the alt text rendered as a `figcaption`, not only as an attribute;
    the text `whitespace-pre-wrap` and unclamped; the external embed with its whole description; a
    `<time>` stamp. Post from glaze: the same `postBody`, since a glaze brick is `kind === 'post'`.
    Video (`:329` to `:422`): poster plus play button, `VideoPlayer` mounted only after the click,
    title, viewer count or activity, runtime, and the author through the shared `AuthorChip` header.
    Blog (`:273` to `:327`): full-width cover, publication chip, whole description, `{#each
    blog.tags}` with no slice against `BlogCard.svelte:38`'s `slice(0, 4)`, published date, and
    "read at {blog.publication.name}" as the one prominent control.
    Dialog behaviour. `role="dialog"` and `aria-modal="true"` at `:445` to `:446`, `aria-label` from
    `label` at `:53` to `:58` (blog title, else the author line). Focus in on open via the `panel`
    binding effect at `:87` to `:89`, and back to the opening card through `reader.returnFocus()` in
    the teardown at `:164`. Escape at `:126`, the close control at `:454`, the scrim at `:432`, and
    the back gesture through `reader.close()` at `reader.svelte.ts:165`. Scroll lock and the pump at
    `:150` to `:157`. Arrows and two controls at `:131` to `:142`, `:439` and `:485`, both `disabled`
    on `reader.canPrev` and `reader.canNext`, which stop at the ends of `feed.items`
    (`reader.svelte.ts:109` to `:117`) and never paginate. The stepping bullet's added sentence is
    real: `step()` at `:96` to `:114` restores focus to the panel after a `tick` when the swapped
    body dropped it, and `:498` to `:501` is an `sr-only` `aria-live="polite"` position line. The
    player claim is the reader's own id, checked under O2.
    I did not run `spec-reviewer` mode 2; instead I ran the lane that observes the component for
    real. `CI=1 just test-e2e` passed 20 of 20, and `web/tests/reader.test.ts` carries 13 of those,
    one per behaviour above: a plain click reads in place, Escape shuts it and hands focus back, the
    back gesture shuts it and leaves the wall laid, the wall behind refuses focus, the scroll lock is
    taken and released by every way out, the panel rises under full motion and only fades under
    reduced motion, the arrows step and stop at the last laid brick, a reveal on the wall survives
    into the reader, "show anyway" reveals and opens no dialog, a glaze image anchor reads in place,
    the reader claims the player under its own id so the card behind lets go, and a modified click
    still goes to the source.
  - *Status:* SATISFIED

## Regression check

- `.specs/README.md`'s pending table loses one row. Trace: the other two pending rows
  (`lay_a_bluesky_feed`, `refresh_the_wall`) are still present and their links still resolve :
  PRESERVED. The table at `:51` to `:54` keeps both rows, and both targets exist on disk at
  `.specs/changes/2026-07-26-lay_a_bluesky_feed.md` and
  `.specs/changes/2026-07-26-refresh_the_wall.md`. The diff's only change to this file is the one
  deleted pending row and the one added merged row.
- `00-overview.md`'s goals list gains item 8. Trace: items 1 to 7 are unchanged and the list is
  contiguous : PRESERVED. `:32` to `:56` runs 1 to 8 with no gap and no repeat, the diff touches
  none of items 1 to 7, and nothing was renumbered, which leaves tasks 19 and 27 the append-only
  list the Residue below asks for.
- Not in the certificate but worth recording, since the move is the only structural edit here: seven
  files in this plan folder still link to `changes/2026-07-26-read_a_brick_in_place.md`, which the
  move makes dangle (`plan.md:3` and the `Implements:` lines of tasks 01 to 07). The two pending
  change specs also name it by bare filename (`lay_a_bluesky_feed.md:827`,
  `refresh_the_wall.md:425`), and those two self-heal when tasks 19 and 27 move their own files into
  the same `merged/` folder. The plan-folder links do not self-heal. No canonical `.specs/*.md` page
  links to the change spec at all, so nothing a reader of the spec set follows is broken. Outside the
  task's definition of done, and left for whoever owns plan hygiene rather than silently fixed here.

## Residue

- Task 19 adds two more goals to the same list and task 27 reconciles the numbering across all
  three. Do not renumber here beyond appending.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED with collected evidence rather than inferred: thirteen of
thirteen blocks landed with all four pages redated 2026-07-27, the four corrections and the three
limit statements resolve against `reader.svelte.ts`, `BrickShell`/`VideoCard`/`GlazeCard`,
`+layout.svelte:171` and `BrickReader.svelte:74` rather than against the block, the one block claim
the build disproved (page state as the open predicate) is off the canonical pages and annotated in
the merged spec, the spec is relocated with the README's two tables agreeing with the filesystem and
the "Three change specs are pending" sentence left for task 27, a minor changeset exists, and
`just guard-dashes`, `just check` and `CI=1 just test-e2e` (20 passed, 13 of them the reader's) are
green; both regression traces are PRESERVED.
