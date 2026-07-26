# Done Certificate · Task 19: merge the feed spec

**Task:** [19-merge_the_feed_spec.md](19-merge_the_feed_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 19. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 19) is every obligation O1 to O6 below holding, O3b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** The canonical pages and the type schema describe the feed wall that shipped,
  including the one place the implementation still diverges from the proposal.
- **P2 · Obligations.** Done iff O1 to O3, O3b and O4 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break task 07's goal 8 or any of the reader prose it merged; must
  not break `canonical-types.schema.json`'s existing `$defs` or any intra-spec link.

## Obligations

- **O1 · Every block is applied, every page redated, the goals still number, and the two enumerated lists match the tree.**
  - *Claim:* each `Proposed changes` block appears in its canonical page; ten pages carry the merge
    date; `00-overview.md`'s goals list is contiguous with this task's two additions numbered
    after task 07's goal 8; `08-wall-and-bricks.md` §Implementation layout names `FeedPicker.svelte`
    and `FeedCard.svelte`, both directly under `components/`; and `07-web-client.md` §Shape names
    `appview.ts` in the `lib/` tree and no longer describes `+page.svelte` as
    `actor ? wall : landing form`.
  - *Evidence to collect:* walk the change spec's `Proposed changes` section block by block and
    locate each in its target page. Read the ten `**Date:**` headers. Read the goals list end to end.
    Diff the `08` component list against `ls web/src/lib/components/*.svelte` and the `07` `lib/`
    list against `ls web/src/lib/*.ts`; neither page may omit a member.
  - *Checks:* the last two were blockless when this plan was written and are not any more: the
    2026-07-26 correction gave the feed spec an Implementation layout block and made its Shape block
    edit the tree rather than only append a paragraph under it. So they are applications, and the
    failure to look for is a block applied by halves (the Shape paragraph landed, the two tree lines
    did not) rather than a merge that forgot to author them. A page that enumerates by name is wrong
    the moment one name is absent, and the reader spec's `BrickReader.svelte` and the refresh spec's
    `RefreshWall.svelte` join the same `08` list, so a missing pair stands out only if it is looked
    for.
  - *Status:* unverified

- **O2 · The schema fragment is folded in.**
  - *Claim:* `canonical-types.schema.json` gains `FeedRef` and `HiddenLabel` and its `CursorPayload`
    and `MortarErrorCode` are the replaced versions.
  - *Evidence to collect:* read the four `$defs`. Confirm `MortarErrorCode`'s enum has five members
    matching the fixture, and `CursorPayload` is the two-shape `oneOf` with the feed shape first.
  - *Checks:* resolve the schema against the code, not the proposal: `CursorPayload`'s shapes must
    match the `Cursor` enum task 09 shipped, and `MortarErrorCode`'s enum must match `ALL_CODES`.
  - *Status:* unverified

- **O3 · The one divergence is described as the code behaves, and the four absorbed ones landed with their blocks.**
  - *Claim:* `07-web-client.md`'s Reactive state table carries a row of its own for
    `state/feeds.svelte.ts` holding `mason:feeds`, with the `lastHandle` row keeping `mason:handle`
    alone. And the four corrections the change spec absorbed on 2026-07-26 are present because their
    blocks carry them: `FeedRef::parse` returns two cases rather than an `Option<String>`; the glaze
    branch truncates on the `Mode::Wall` arm alone; `+layout.svelte` gates the header on
    `actor || feed`; and the `bad_request` wording names both parameters in two places, one of them
    the service worker's own copy.
  - *Evidence to collect:* read the Reactive state table. Then, for each of the four, read the
    canonical sentence and the shipped code (`sources/feedref.rs`, `feed.rs`'s glaze branch,
    `+layout.svelte:111`, `routes/feed.rs:42` and `service-worker.ts:254`) and confirm they agree.
  - *Checks:* the divergence is the one a verbatim paste gets wrong. The change spec's block extends
    the `lastHandle` row's storage note to `mason:handle, mason:feeds` and adds no row for the new
    module, but task 17 shipped the recents list in `feeds.svelte.ts`. A table that names
    `mason:feeds` on `handle.svelte.ts` is a spec-versus-code divergence created by this very merge,
    and O6's `spec-reviewer` pass is the second net under it. For the other four the failure mode is
    the opposite one: they are block text now, so a page missing one means the block was applied
    incompletely, and re-authoring a fifth "correction" the spec already states is churn.
  - *Status:* unverified

- **O3b · The Reactive state sentence counts the table beneath it.**
  - *Claim:* `07-web-client.md:95` no longer reads "Five singletons, each a class holding `$state`
    fields". It names ten rows, and it does not claim every one is a class: `revealed` is a
    `SvelteSet` from `svelte/reactivity`.
  - *Evidence to collect:* read the sentence, then count the table's rows. Six before this plan, plus
    `reader` and `revealed` from task 07, plus `feedinfo` and `feeds` from this task, is ten.
  - *Checks:* the sentence was already wrong by one before this plan touched it (five claimed, six
    rows), which is what a prose count does when nothing checks it. No block covers this line, and
    this task is the last one that grows the table, so the reconciliation lands here or nowhere. Read
    `web/src/lib/state/sensitive.svelte.ts` to confirm the `SvelteSet` claim rather than taking the
    table's word for it.
  - *Status:* unverified

- **O4 · The spec is merged and relocated.**
  - *Claim:* `**Status:** Merged` with a `**Merged:**` date, the file lives at
    `.specs/changes/merged/2026-07-26-lay_a_bluesky_feed.md`, and `.specs/README.md`'s pending table
    no longer lists it.
  - *Evidence to collect:* run
    `ls .specs/changes/merged/2026-07-26-lay_a_bluesky_feed.md .specs/changes/2026-07-26-lay_a_bluesky_feed.md`
    and expect the first to exist and the second not to. Run
    `grep -n lay_a_bluesky_feed .specs/README.md`.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green, the e2e lane is green, and a minor changeset exists in mason's
    voice.
  - *Evidence to collect:* run `just guard-dashes`, `just check` and `just test-e2e`. Run
    `ls .changeset/*.md` and read the new file.
  - *Status:* unverified

- **O6 · Reviewable: a spec-versus-code pass finds nothing.**
  - *Claim:* the `spec-reviewer` skill in mode 2 (canonical versus code) over `.specs/00` to
    `.specs/08` reports no divergence on anything this plan built.
  - *Evidence to collect:* run the skill and read its findings, discounting only divergences that
    predate this plan.
  - *Status:* unverified

## Regression check

- Task 07's reader prose in `07-web-client.md` and `08-wall-and-bricks.md`. Trace: the reactive-state
  rows for `reader` and `revealed`, the brick-reader section and the accessibility bullet are all
  still present after this task's edits to the same tables : (PRESERVED / REGRESSION)
- `.specs/README.md`'s merged table. Trace: it now lists four merged specs (three from 2026-07-25
  plus the reader) and gains this one : (PRESERVED / REGRESSION)
- `canonical-types.schema.json`'s untouched `$defs`. Trace: `FeedMode`, `FeedIntent` and the brick
  shapes are unchanged : (PRESERVED / REGRESSION)

## Residue

- Task 27 corrects `.specs/README.md`'s "Three change specs are pending" sentence; leave it.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
