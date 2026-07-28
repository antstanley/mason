# Done Certificate · Task 27: merge the refresh spec and reconcile

**Task:** [27-merge_the_refresh_spec.md](27-merge_the_refresh_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27

> Verification protocol for Task 27. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This is the
> last task in the plan and the only one that reconciles across all three specs.

## Definition

DONE(Task 27) is every obligation O1 to O7 below holding, O2b and O2c included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** All three change specs merged, the goals numbered once, and `.specs/README.md` with
  an empty pending list.
- **P2 · Obligations.** Done iff O1, O2, O2b, O2c and O3 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break tasks 07 and 19's merged prose, the goals they added, or any
  of the four merged specs already in `.specs/changes/merged/`. **Three** of the refresh spec's
  blocks land on text task 19 rewrote, and a verbatim paste of any of the three **is** a break of
  this invariant, not an application of the spec. A fourth, `05`'s cache bullet, pastes cleanly and
  still has to be read: it describes a table that has grown a row since it was drafted.

## Obligations

- **O1 · Every block is applied, redated, and honest about what shipped.**
  - *Claim:* each `Proposed changes` block appears in its canonical page with the merge date; the
    `refresh` row sits with `Snapshot`'s own immutable identity fields rather than in the `Inner`
    state table at `01:161`-`:174`; `FeedRefresh` is in `canonical-types.schema.json` beside
    `FeedMode` and `FeedIntent` with a description pointing at `refresh_from_query`; and the `07`
    state-machine block, the `08` Refreshing paragraph and `02`'s Refresh section describe what
    tasks 25, 26 and 21 actually did. The `08` paragraph is the block's own prose (the control does
    not scroll at all, and why; the single live region and no new announcement) plus **one sentence
    the block does not carry**, saying what a **reduced-motion** refresh is: a cursorless preview
    carrying the flag plus a freeze deferred by task 25's in-flight marker until that preview's
    cursor is adopted, so one cursorless request, one refreshed fan-out and one reflow when the
    preview lands. The reader close is described where it shipped, at the control in `RefreshWall`,
    not in `FeedState`.
  - *Evidence to collect:* walk the change spec's `Proposed changes` blocks and locate each. Read
    `01-domain-model.md`'s Snapshot section and confirm the row is not inside the `Inner` table. Read
    the four what-shipped passages and compare each against `feed.svelte.ts`, `RefreshWall.svelte`,
    `FeedGrid.svelte:182`-`:187` and `algo/snapshot.rs`.
  - *Checks:* resolve the `02` claim about snapshot creation. `ensure_snapshot` runs its `make`
    closure only on a genuine insert, so the prose must say the flag rides on creation and that a
    colliding request inherits the existing snapshot's flag, rather than claiming a refresh always
    lands on a brand new snapshot id.
  - *Status:* SATISFIED. All fourteen `Proposed changes` blocks are on their pages: 01's identity
    prose (`01-domain-model.md:158`), 02's Entry point parameter (`:39`) and the new Refresh section
    (`:76`), 04's two feed-read paragraphs (`:103`, `:124`) and the failure-semantics row (`:320`),
    05's cache bullet (`:93`), 06's parameter row (`:36`), fallback paragraph (`:47`) and fixture row
    (`:186`), 07's Responsibilities clause (`:19`), the `refresh()` diagram block (`:252`) and three
    Decisions (`:507`), and 08's wall-state row (`:315`), Refreshing section (`:376`), accessibility
    sentence (`:516`) and layout line (`:560`). `FeedRefresh` is in `canonical-types.schema.json`
    directly after `FeedIntent`, `"const": "1"`, its description naming `refresh_from_query` in
    `server/crates/mortar-core/src/feed.rs`, and the file still parses as JSON. Every edited page
    carries `**Date:** 2026-07-27`, the merge date on all three specs; `10-build-release-deploy.md`
    was 2026-07-25 and is bumped, the other eight already carried it from tasks 07 and 19. The
    `refresh` prose sits in the identity paragraph above the state table, and the table's lead-in
    says why, which is what the code shows: `pub refresh: bool` at `algo/snapshot.rs:103` sits beside
    `id`, `seed`, `viewer` and `mode` and outside the `Mutex<Inner>` at `:108`, and the table's twelve
    rows are exactly the fields of `Inner` at `:244` onward. The four what-shipped passages were read
    against the code rather than either spec: 07's state machine matches `feed.svelte.ts`, where the
    flag is spent on adoption (`:168`-`:172`), in `#warm`'s catch (`:195` and `:211`), in `freeze`'s
    `finally` and in `reset` (`:93`-`:94`), and `freeze` returns with no side effect at all while the
    in-flight marker is set (`:250`), a held commit being dropped rather than queued; 08's
    reduced-motion sentence matches `FeedGrid.svelte:199`-`:205`, which freezes the instant `warming`
    flips true with no listener attached, so it is one cursorless request, one refreshed fan-out and
    one reflow when the preview lands, the same account `RefreshWall.svelte:83`-`:95` carries; 02's
    Refresh section matches `feed.rs` and `fill.rs` (`fill::fill` passes `snapshot.refresh` at `:67`
    and `:104`, `extend` passes literal `false`s at `:174` and `:178`, the rule stated at `:167`, and `refresh_tests` hold both); and the
    reader close is described at the control in `RefreshWall`, in 07 (`:318`) and 08 (`:408`), never
    in `FeedState`, which is where `RefreshWall.svelte:31`-`:59` actually puts it. Check: resolved
    `ensure_snapshot` (step 4, imported, `algo::snapshot`, defined at `snapshot.rs:362`); it builds
    through `get_or_insert_with`, so only the winning insert reads `refresh`, and 02 `:116` onward
    says the flag rides on creation and that a request meeting an existing snapshot inherits that
    snapshot's flag. One imprecision inherited verbatim from the block, recorded rather than counted
    against the obligation: the retained sentence "a refresh always lands on a brand new snapshot id"
    is only practically true, since `fresh_seed` (`snapshot.rs:343`) hashes the current millisecond;
    the paragraph below it states the collision case correctly, so the page is not misleading.

- **O2 · The three blocks that collide with task 19 were merged, not pasted.**
  - *Claim:* the refresh spec was drafted before the feed spec landed, so three of its blocks
    overwrite text task 19 already wrote. All three now read as the merge:
    (a) `02-feed-engine.md` §Entry point prints
    `handle_feed(state, target: FeedTarget<'_>, cursor, mode, intent, refresh: bool)`, **not** the
    refresh spec's `actor: &str` version;
    (b) `06-wire-contract.md` §What the fixture covers carries
    `query.mode` / `query.intent` / `query.target` / `query.refresh` and still carries the
    `vocab.hiddenLabels` row, **not** the refresh spec's single `mode` / `intent` / `refresh` row;
    (c) the `07-web-client.md` state-machine block says "drop this (target, mode) from the session
    cache", **not** "(actor, mode)".
  - *Evidence to collect:* read all three passages in the canonical pages. Then read the same three
    blocks in `.specs/changes/merged/2026-07-26-refresh_the_wall.md` and confirm each is annotated
    with the merge that was actually made. Then read the code: `feed.rs`'s `handle_feed` signature,
    `tests/fixtures/contract.json`'s `query` object keys, and `feed.svelte.ts`'s `#key`.
  - *Checks:* the failure mode here is a green-looking paste. Applying the refresh spec's Entry point
    block verbatim reverts `FeedTarget` out of `02` and leaves the page describing an engine that has
    not existed since task 13; applying its fixture row verbatim deletes `query.target` and, if the
    row is replaced rather than extended, `vocab.hiddenLabels` with it. Compare against the code and
    the fixture, never against either change spec, because both specs are right about their own delta
    and neither is right about the sum.
  - *Status:* SATISFIED. (a) `02-feed-engine.md:33`-`:40` prints
    `handle_feed(state, target: FeedTarget<'_>, cursor, mode, intent, refresh: bool)`, which matches
    `server/crates/mortar-core/src/feed.rs:137`-`:144` parameter for parameter; both request-flow
    diagrams (`:133`, `:168`) carry `refresh` too. (b) `06-wire-contract.md:186` reads
    `query.mode` / `query.intent` / `query.refresh` / `query.target`, and the `vocab.hiddenLabels`
    row at `:188` is untouched; `tests/fixtures/contract.json` parses to `query` keys
    `['intent', 'mode', 'refresh', 'target']` and `vocab` keys `['hiddenLabels', 'videoSource']`, so
    the page and the fixture agree. (c) `07-web-client.md:254` says "drop this (target, mode) from
    the session cache", which matches `feed.svelte.ts:69` `#key(target: FeedTarget, mode?: FeedMode)`
    and its use in `refresh()` at `:133`. All three are annotated in place in
    `.specs/changes/merged/2026-07-26-refresh_the_wall.md` as three-way merges rather than
    applications (`:85`, `:169` and `:180`), so the history records what was drafted
    beside what landed. Nothing was compared against either change spec to reach this.

- **O2b · The cache bullet is true of the table it landed on, and still states no count.**
  - *Claim:* `05-caching-and-persistence.md` §The caches says a refresh bypasses `author_feed` and
    `image_feed` on a graph wall and `feed_pages` on a feed wall, and that every other cache stays
    warm. The `02` feed-wall clause says the same thing in the same words. Neither states a number.
  - *Evidence to collect:* read the applied sentence and the `02` clause and compare the two
    wordings. Confirm `feed_pages` is a row in the table above the sentence, put there by task 12.
  - *Checks:* the block is countless on purpose, and that is what makes it applicable rather than
    mergeable: an earlier draft said "exactly two of these ... the other nine are not", which was
    true only of the eleven-row table it was written against, and task 12's twelfth row falsified
    both halves. So the failure to look for here is a **reintroduced** number, in either page, and a
    sentence that names `feed_pages` on a feed wall while the table above it does not carry that row.
    Nothing mechanical catches either.
  - *Status:* SATISFIED. A script comparing the two passages reports them byte identical after
    whitespace folding: `05-caching-and-persistence.md:93` and `02-feed-engine.md:104` both read
    "**A refresh bypasses two of these on a graph wall, and one on a feed wall.** `author_feed` and
    `image_feed` are re-read on a `refresh=1` request; on a feed wall it is `feed_pages` instead.
    Every other cache stays warm." Neither carries a remainder count: the only numbers in either are
    the block's own "two" and "one" bypass counts and the literal `refresh=1`. Both are true of the
    table above them, which now holds twelve rows including `feed_pages` at `05:79` with the key
    `<feed uri>\u{1f}<limit>\u{1f}<upstream cursor>`, and true of the code: `fetch.rs:282` skips the
    `feed_pages` entry on a refresh and `fetch.rs:182`/`:217` skip `author_feed`/`image_feed`, with
    no other cache consulting the flag. One honest clause was added beyond the block at `05:101`:
    `feed_pages` is not persisted, which `persist::CACHE_NAMES` confirms (nine names, no
    `feed_pages`), so the block's "the next persist cycle captures the fresher data" would have been
    two thirds true without it.

- **O2c · Both e2e lane descriptions name what the lane became.**
  - *Claim:* `07-web-client.md` §Testing's Playwright row and `10-build-release-deploy.md:55`'s
    `just test-e2e` row describe four specs rather than one service-worker smoke, and the `07` row
    keeps its point that Playwright is the only lane that can render a component.
  - *Evidence to collect:* read both rows. Run `ls web/tests/*.test.ts` and confirm the count and the
    names match what the rows say: the smoke, `reader.test.ts` (task 06), `feed-picker.test.ts`
    (task 18) and `refresh.test.ts` (task 26).
  - *Checks:* no change spec carries a block for either row, and no earlier task can correct them
    truthfully because each lands with only some of the four specs in the tree. This task is the only
    one that runs after all four exist, which is why the obligation sits here rather than at task 06,
    18 or 26.
  - *Status:* SATISFIED, with the drift noted. `07-web-client.md:437` and
    `10-build-release-deploy.md:55` both name the lane's specs rather than one smoke, and `07`'s row
    keeps, in bold, that it is the only lane in the repo that renders a component at all;
    `10-build-release-deploy.md:91`'s testing-tier row and its `web/playwright.config.ts` layout line
    at `:388` were stale in the same way and are corrected too. The drift: this obligation and the
    DoD item behind it both say **four** specs, and the check they name says otherwise.
    `ls web/tests/*.test.ts` returns five (`feed-picker`, `reader`, `refresh`, `repeated-tags`,
    `service-worker-smoke`), and `playwright test --list` reports "35 tests in 5 files".
    `repeated-tags.test.ts` landed with task 05 and the certificate's enumeration missed it. Both
    rows say five and name all five, which is what the named check demands, so the obligation is
    satisfied against the evidence rather than against the count that predicted it.

- **O3 · The pending list is empty and the count sentence is true.**
  - *Claim:* `.specs/README.md` lists no pending change specs, its merged table carries all three of
    the 2026-07-26 batch, and the sentence at `:47` no longer claims three are pending.
  - *Evidence to collect:* read `.specs/README.md` end to end. Run
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` and confirm
    each name appears only in the merged table.
  - *Status:* SATISFIED. `.specs/README.md` was read end to end: the pending table and the "Three
    change specs are pending" sentence are gone, replaced at `:47`-`:49` by "No change spec is
    pending. The three proposed on 2026-07-26 from one batch of feature requests all merged...", and
    the merged table below it carries all three of the batch, each dated 2026-07-27. The Plans
    table's "All three pending change specs" wording is corrected to "All three of the 2026-07-26
    change specs" at `:70`.
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` returns three
    hits, lines 56, 57 and 58, all inside the merged table.

- **O4 · The goals list is contiguous with each addition present once.**
  - *Claim:* `00-overview.md`'s goals are numbered contiguously after tasks 07 and 19 both appended,
    and each of the three additions appears exactly once.
  - *Evidence to collect:* read the goals list end to end and count. Confirm the reader goal, the
    source-and-view goal and the feed-picker goal are each present and unduplicated.
  - *Status:* SATISFIED. `00-overview.md`'s goals were read end to end and counted: 1 to 10, no gap
    and no repeat. Goal 8 is the in-place reader (task 07's addition), goal 9 the source-and-view
    split and goal 10 the feed picker (task 19's two). Each appears once; nothing in this diff
    touches the file, so the obligation is a confirmation rather than an edit, which is what it
    asked for.

- **O5 · All three specs are relocated and none remains pending.**
  - *Claim:* all three 2026-07-26 files exist under `.specs/changes/merged/` and none remains in
    `.specs/changes/`; each carries `**Status:** Merged` and a `**Merged:**` date; and each spec's
    "what it owes the other two" note is discharged or annotated as discharged.
  - *Evidence to collect:* run `ls .specs/changes/*.md` and expect nothing. Run
    `ls .specs/changes/merged/` and expect six files. Read each 2026-07-26 spec's interaction section
    and confirm the obligations (the `App.PageState` sharing, the overlay exclusion rule, the
    `feed_pages` bypass, closing an open reader on refresh) are each annotated with where they landed.
  - *Checks:* trace each of the four obligations to its task: the interface to 01 and 17, the
    exclusion rule to 17, the bypass to 22, and the reader close to **26**, at the control, not to
    25. An obligation annotated as discharged but not implemented is the failure mode this check
    exists for; so is one annotated against the wrong task, because the next reader of the merged
    spec will go looking in `feed.svelte.ts` and find nothing.
  - *Status:* SATISFIED. `ls .specs/changes/*.md` returns no matches and `ls .specs/changes/`
    returns only `merged`; `.specs/changes/merged/` holds six files, the three 2026-07-25 specs and
    all three of the 2026-07-26 batch. Each of the three carries
    `**Status:** Merged · **Date:** 2026-07-26 · **Merged:** 2026-07-27`. The obligations trace to
    the tasks the check names, and each annotation names where the work landed rather than only that
    it did: `App.PageState` gained both members rather than being replaced and the two-overlay rule
    is the shape of the push itself (`web/src/app.d.ts:8`-`:29`, tasks 01 and 17), annotated in
    `lay_a_bluesky_feed.md:867`; the `feed_pages` bypass shipped in `fetch::feed_page_cached`
    (`sources/fetch.rs:261`-`:284`, task 22), annotated in both that spec and
    `refresh_the_wall.md:435`; and the reader close shipped at the control in
    `RefreshWall.svelte:59`, not in `FeedState` (task 26), which is what
    the same annotation at `refresh_the_wall.md:435` says. `read_a_brick_in_place.md` carries no such interaction
    note, so it owed nothing and none was invented for it. No obligation is annotated as discharged
    against a place the code does not hold it.

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates and the e2e lane are green and a minor changeset exists.
  - *Evidence to collect:* run `just guard-dashes`, `just check` and `just test-e2e`. Run
    `ls .changeset/*.md` and confirm three changesets exist across the plan (tasks 07, 19 and this
    one), each minor.
  - *Status:* SATISFIED. Run in this workspace, not taken from the report: `just guard-dashes`
    exits 0 (run again after this certificate was written, since the plan folder is tracked);
    `just check` exits 0 end to end, which is guard-dashes, guard-autoplay, guard-toolchain,
    fmt-check, guard-wasm, lint and test, with `cargo nextest` reporting 154 tests run, 154 passed
    (including `feed::tests::a_feed_cursor_on_the_graph_wall_lays_a_fresh_wall`, which the
    implementer reported failing: it passes here in 1.076s, so that failure was the machine's
    outbound network at the time and not this diff, which changes no code, no test and no config);
    `pnpm check:ci` clean over both tsc projects; vitest 141 passed in 8 files. `CI=1 just test-e2e`
    exits 0 with its own `vite preview --port 4173 --strictPort`, 35 tests passed in 10.5s, so it was
    this workspace's build and not another's. A minor changeset for this task exists at
    `.changeset/the-drawings-match-the-wall.md`. Drift on the check's arithmetic, recorded rather
    than counted against the obligation: it expects three changesets across the plan, and there are
    eight, because tasks beyond 07, 19 and 27 added their own. Every one of the eight is
    `"mason": minor`.

- **O7 · Reviewable: nothing is pending and every name is in the merged table.**
  - *Claim:* `ls .specs/changes/*.md` returns nothing, and
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` shows each
    name only in the merged table.
  - *Evidence to collect:* both commands.
  - *Status:* SATISFIED. Both commands run in the workspace: `ls .specs/changes/*.md` returns no
    matches (zsh reports "no matches found", exit 1), and
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` returns
    exactly three lines, 56, 57 and 58, each a row of the merged table. No name appears in a pending
    list, because there is no pending list left.

## Regression check

- Task 07's reader prose and task 19's feed prose in `07-web-client.md` and `08-wall-and-bricks.md`.
  Trace: the reactive-state rows, the brick-reader section, the picker section and the accessibility
  bullets from both are all still present after this task's edits to the same tables :
  PRESERVED. `07`'s reactive-state rows for `state/reader.svelte.ts` and `state/sensitive.svelte.ts`
  are at `:122`-`:123`, its feed-picker screen prose at `:196` and its `reset(target, mode)`
  paragraph at `:209`; `08` still carries "The brick reader" (`:176`), "Dialog behaviour" (`:206`)
  and "The feed picker" (`:424`) with its States table (`:474`), and the accessibility section
  (`:497`) gained the `RefreshWall` sentence without losing the `LayoutPicker` and `SwitchWall`
  ones beside it. The refresh additions are inserts, never replacements, in every table this diff
  touches.
- `.specs/changes/merged/`'s three 2026-07-25 specs. Trace: untouched and still linked from the
  README : PRESERVED. `jj diff --summary` lists only the two 2026-07-26 specs under `changes/`, and
  the README's merged table still carries all three 2026-07-25 rows (`:53`-`:55`), each link
  resolving.
- `canonical-types.schema.json`'s `FeedRef`, `HiddenLabel`, `CursorPayload` and `MortarErrorCode`
  from task 19. Trace: unchanged by this task's `FeedRefresh` addition : PRESERVED. The diff is six
  added lines and nothing else; `$defs` parses to 27 keys in the old order with `FeedRefresh`
  inserted between `FeedIntent` and `FeedRef`.
- `02-feed-engine.md` §Entry point, written by task 19. Trace: the signature still names
  `target: FeedTarget<'_>` and has gained `refresh: bool`, rather than having reverted to
  `actor: &str` : PRESERVED. `:33`-`:40` is six parameters, `actor: &str` appears nowhere on the
  page, and the diff for this hunk is one added line.
- `06-wire-contract.md` §What the fixture covers, written by task 19. Trace: the table still lists
  `query.target` and still carries the `vocab.hiddenLabels` row, and has gained `query.refresh` :
  PRESERVED. `:186` carries all four query keys and `:188` is the untouched `vocab.hiddenLabels`
  row; the vocabulary-assert list at `:202` gained `FeedRefresh` beside `HiddenLabel`, matching
  `web/src/lib/contract-check.ts:88` and `:103`.
- `07-web-client.md`'s feed state machine, written by tasks 15 and 19. Trace: the session-cache key
  in the new `refresh()` block reads (target, mode), not (actor, mode) : PRESERVED. `:254` reads
  "(target, mode)"; "(actor, mode)" appears nowhere on the page, and `feed.svelte.ts:69` keys on a
  `FeedTarget`.

## Residue

- The plan's three open questions (whether `warmFeed` means anything for a feed target, whether a
  snapshot-id collision is worth code, whether the glaze page size needs its own bound) are recorded
  in `plan.md`, not in the canonical pages. If any was answered during
  the build, this is the last chance to move the answer into the right page's `Assumptions and open
  questions` block. Not an obligation.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7, O2b and O2c included, are all SATISFIED on evidence collected in the workspace,
and all six regression traces are PRESERVED: every block landed with the merge date, the three
collisions read as merges checked against `feed.rs`, `contract.json` and `feed.svelte.ts` rather than
against either change spec, the cache sentence is byte identical on `02` and `05` and carries no
remainder count, `.specs/changes/` is empty with all three specs on the merged shelf, and
`just guard-dashes`, `just check` and `CI=1 just test-e2e` were all green when run here. Two places
where the authored protocol drifted from the tree are recorded in the statuses rather than silently
resolved: the lane holds five Playwright specs, not the four O2c predicted, and the plan carries
eight changesets, not the three O6 predicted, each minor.
