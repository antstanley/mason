# Done Certificate · Task 07: merge the reader spec

**Task:** [07-merge_the_reader_spec.md](07-merge_the_reader_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · The four corrections are in the canonical prose, carried by the blocks rather than re-authored.**
  - *Claim:* the pages say the reader holds the brick and locates it by id rather than storing an
    index; that activation is from `BrickShell`'s anchor on post and blog cards, the watch link on a
    video card and an image anchor on a glaze card; that the reader mounts as the wrapper's sibling,
    outside the inert subtree; and that the reader claims the video player under `reader:<brick.id>`
    so the card yields. The Dialog behaviour and Accessibility blocks' statements about the pump and
    the freeze listeners are present and true.
  - *Evidence to collect:* read `08-wall-and-bricks.md`'s new brick-reader section and
    `07-web-client.md`'s reactive-state prose. For each of the four, confirm the canonical text
    matches the code: `reader.svelte.ts`'s `findIndex`, the five intercepted anchors,
    `+layout.svelte`'s mount position, and the reader's `VideoPlayer` id.
  - *Checks:* the change spec once got all four wrong and was corrected in commit `b55ef455` on
    2026-07-26, so each is now block text: a correction missing from the canonical page means the
    block was applied incompletely, not that the merge failed to invent it. Resolve each canonical
    claim against the shipped file all the same, never against the block.
  - *Status:* unverified

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
  - *Status:* unverified

- **O4 · A changeset exists.**
  - *Claim:* a minor changeset describes the reader as a new surface.
  - *Evidence to collect:* run `ls .changeset/*.md` and read the new file; confirm the bump level is
    minor and the text is in mason's voice.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green over the moved and edited prose.
  - *Evidence to collect:* run `just guard-dashes` and `just check`. `guard-dashes` scans the whole
    tree by denylist, so a moved spec file is still in scope.
  - *Status:* unverified

- **O6 · Reviewable: the page describes the component that exists.**
  - *Claim:* a reviewer reads `08-wall-and-bricks.md`'s new brick-reader section alongside
    `BrickReader.svelte` and finds no statement the component does not satisfy.
  - *Evidence to collect:* the side-by-side read. Optionally run the `spec-reviewer` skill in
    mode 2 (canonical versus code) scoped to `08-wall-and-bricks.md`.
  - *Status:* unverified

## Regression check

- `.specs/README.md`'s pending table loses one row. Trace: the other two pending rows
  (`lay_a_bluesky_feed`, `refresh_the_wall`) are still present and their links still resolve :
  (PRESERVED / REGRESSION)
- `00-overview.md`'s goals list gains item 8. Trace: items 1 to 7 are unchanged and the list is
  contiguous : (PRESERVED / REGRESSION)

## Residue

- Task 19 adds two more goals to the same list and task 27 reconciles the numbering across all
  three. Do not renumber here beyond appending.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
