# Done Certificate · Task 16: feed identity and the fourth error

**Task:** [16-feed_identity_and_fourth_error.md](16-feed_identity_and_fourth_error.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 16. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. The `.svelte`
> half of this task has no lane at all; O4 is where that is recorded rather than hidden.

## Definition

DONE(Task 16) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The generator's own face in the header, and "no such feed" instead of telling
  somebody with a bad feed link to fix their handle.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break `profile.svelte.ts`'s existing wall-owner avatar read, the
  three existing `FeedGrid` error panels, both handle-box recoveries, or `SwitchWall` on a graph
  wall.

## Obligations

- **O1 · One AppView base for the whole client.**
  - *Claim:* `web/src/lib/appview.ts` is the single client-side AppView base and
    `profile.svelte.ts`'s own constant is deleted rather than duplicated.
  - *Evidence to collect:* run `grep -rn 'public.api.bsky.app' web/src` and confirm exactly one
    source hit, in `lib/appview.ts`. Read `profile.svelte.ts` and confirm it imports rather than
    declares.
  - *Checks:* resolve the import in each of the three readers (`profile`, `feedinfo`, and later the
    picker) to `lib/appview.ts`, not to a re-declared local.
  - *Status:* unverified

- **O2 · The new error code is classified and typechecked.**
  - *Claim:* `#fail` maps `feed_not_found` to `"feed-not-found"` with a `satisfies MortarErrorCode`
    comparison, and `feed.test.ts:218`'s `it.each` table gains the row.
  - *Evidence to collect:* read `feed.svelte.ts`'s `#fail` and confirm the `satisfies` form. Run
    `cd web && pnpm test` and confirm the new table row passes.
  - *Checks:* a plain string comparison would compile and would not fail when mortar renames the
    code. Confirm the literal carries `satisfies MortarErrorCode`, matching the two existing uses at
    `:200` and `:204`.
  - *Status:* unverified

- **O3 · `feedinfo` never blocks and degrades to the rkey.**
  - *Claim:* `feedinfo.svelte.ts` reads `app.bsky.feed.getFeedGenerator` behind the `browser` guard,
    never blocks the wall, and on a miss leaves the header showing the feed's rkey; a vitest case
    covers the miss.
  - *Evidence to collect:* read the module and confirm the `browser` guard, following
    `profile.svelte.ts:26`. Run the named miss case in `feedinfo.test.ts`.
  - *Status:* unverified

- **O4 · The existing graph-wall surfaces are unchanged, and the coverage gap is stated.**
  - *Claim:* the three existing `FeedGrid` error panels and both handle-box recoveries are unchanged
    on a graph wall; and the PR states that `SwitchWall` and `FeedGrid`'s new panel ship with no
    automated coverage at all.
  - *Evidence to collect:* read the diff of `FeedGrid.svelte` and confirm the three existing panels'
    bodies are untouched. Read the PR body for the statement. Record why: no offline e2e case can
    reach a live `getFeedGenerator` or a `getFeed` 404, and neither vitest suite imports a component.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green and the e2e lane is green as a regression.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci`, `just test-e2e` and
    `just check`.
  - *Status:* unverified

- **O6 · Reviewable: the `.ts` half is covered and the rest is exercised by hand.**
  - *Claim:* `cd web && pnpm test` covers `appview`, `feedinfo` and `#fail`; the header and the new
    panel are read in the diff and exercised against a real feed uri.
  - *Evidence to collect:* the vitest run, plus a manual pass: open the built site at a real
    `?feed=` uri and confirm the generator's name and avatar appear in `SwitchWall`, then at a
    well-formed but nonexistent uri and confirm the "no such feed" panel.
  - *Status:* unverified

## Regression check

- `state/profile.svelte.ts` after the base is hoisted. Trace: `/?actor=demo` still shows the wall
  owner's avatar in `SwitchWall` : (PRESERVED / REGRESSION)
- `feed.svelte.ts:200` and `:204`: `login_required` and `actor_not_found` still classify to
  `login-required` and `handle-not-found` : (PRESERVED / REGRESSION)
- `FeedGrid.svelte`'s empty state on a graph wall: still reads as before : (PRESERVED / REGRESSION)

## Residue

- The change spec's open question about a feed wall wanting a header line naming what the reader is
  reading (beyond avatar and name) is not answered here and is not an obligation.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
