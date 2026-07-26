# Done Certificate · Task 27: merge the refresh spec and reconcile

**Task:** [27-merge_the_refresh_spec.md](27-merge_the_refresh_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
    tasks 25, 26 and 21 actually did. The `08` paragraph is the block's own prose (the order,
    `refresh()` first and the scroll second; the single live region and no new announcement) plus
    **one sentence the block does not carry**, saying what a **reduced-motion** refresh is: a
    cursorless preview carrying the flag plus a cursorless freeze that does not, held to one flagged
    request by task 25's in-flight guard, so one refreshed fan-out and no reflow. The reader close is
    described where it shipped, at the control in `RefreshWall`, not in `FeedState`.
  - *Evidence to collect:* walk the change spec's `Proposed changes` blocks and locate each. Read
    `01-domain-model.md`'s Snapshot section and confirm the row is not inside the `Inner` table. Read
    the four what-shipped passages and compare each against `feed.svelte.ts`, `RefreshWall.svelte`,
    `FeedGrid.svelte:182`-`:187` and `algo/snapshot.rs`.
  - *Checks:* resolve the `02` claim about snapshot creation. `ensure_snapshot` runs its `make`
    closure only on a genuine insert, so the prose must say the flag rides on creation and that a
    colliding request inherits the existing snapshot's flag, rather than claiming a refresh always
    lands on a brand new snapshot id.
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O3 · The pending list is empty and the count sentence is true.**
  - *Claim:* `.specs/README.md` lists no pending change specs, its merged table carries all three of
    the 2026-07-26 batch, and the sentence at `:47` no longer claims three are pending.
  - *Evidence to collect:* read `.specs/README.md` end to end. Run
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` and confirm
    each name appears only in the merged table.
  - *Status:* unverified

- **O4 · The goals list is contiguous with each addition present once.**
  - *Claim:* `00-overview.md`'s goals are numbered contiguously after tasks 07 and 19 both appended,
    and each of the three additions appears exactly once.
  - *Evidence to collect:* read the goals list end to end and count. Confirm the reader goal, the
    source-and-view goal and the feed-picker goal are each present and unduplicated.
  - *Status:* unverified

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
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates and the e2e lane are green and a minor changeset exists.
  - *Evidence to collect:* run `just guard-dashes`, `just check` and `just test-e2e`. Run
    `ls .changeset/*.md` and confirm three changesets exist across the plan (tasks 07, 19 and this
    one), each minor.
  - *Status:* unverified

- **O7 · Reviewable: nothing is pending and every name is in the merged table.**
  - *Claim:* `ls .specs/changes/*.md` returns nothing, and
    `rg -n 'refresh_the_wall|lay_a_bluesky_feed|read_a_brick_in_place' .specs/README.md` shows each
    name only in the merged table.
  - *Evidence to collect:* both commands.
  - *Status:* unverified

## Regression check

- Task 07's reader prose and task 19's feed prose in `07-web-client.md` and `08-wall-and-bricks.md`.
  Trace: the reactive-state rows, the brick-reader section, the picker section and the accessibility
  bullets from both are all still present after this task's edits to the same tables :
  (PRESERVED / REGRESSION)
- `.specs/changes/merged/`'s three 2026-07-25 specs. Trace: untouched and still linked from the
  README : (PRESERVED / REGRESSION)
- `canonical-types.schema.json`'s `FeedRef`, `HiddenLabel`, `CursorPayload` and `MortarErrorCode`
  from task 19. Trace: unchanged by this task's `FeedRefresh` addition : (PRESERVED / REGRESSION)
- `02-feed-engine.md` §Entry point, written by task 19. Trace: the signature still names
  `target: FeedTarget<'_>` and has gained `refresh: bool`, rather than having reverted to
  `actor: &str` : (PRESERVED / REGRESSION)
- `06-wire-contract.md` §What the fixture covers, written by task 19. Trace: the table still lists
  `query.target` and still carries the `vocab.hiddenLabels` row, and has gained `query.refresh` :
  (PRESERVED / REGRESSION)
- `07-web-client.md`'s feed state machine, written by tasks 15 and 19. Trace: the session-cache key
  in the new `refresh()` block reads (target, mode), not (actor, mode) : (PRESERVED / REGRESSION)

## Residue

- The plan's three open questions (whether `warmFeed` means anything for a feed target, whether a
  snapshot-id collision is worth code, whether the glaze page size needs its own bound) are recorded
  in `plan.md`, not in the canonical pages. If any was answered during
  the build, this is the last chance to move the answer into the right page's `Assumptions and open
  questions` block. Not an obligation.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
