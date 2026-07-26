# Done Certificate · Task 18: feed picker screen

**Task:** [18-feed_picker_screen.md](18-feed_picker_screen.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 18. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. Playwright is
> the only lane that can see either new component.

## Definition

DONE(Task 18) is every obligation O1 to O6 below holding, O1b and O4b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** The second front door: a screen that stands beside the handle box as a peer,
  reachable from the landing page and from a laid wall.
- **P2 · Obligations.** Done iff O1, O1b, O2 to O4, O4b, O5 and O6 all hold; O6 is the Reviewable
  item.
- **P3 · Invariants.** Must not break the handle box (`HandleForm`), `SwitchWall`'s existing
  switch-walls affordance, or task 03's reader, which shares `App.PageState`, the dialog language
  **and the wrapper's `inert` condition** this task widens.

## Obligations

- **O1 · All five states render and a bad paste navigates nowhere.**
  - *Claim:* loading, a search with no results, a handle with no feeds, the AppView unreachable, and
    an unparseable pasted value each render their row of the picker's states table; the paste error
    appears in place with no navigation.
  - *Evidence to collect:* drive each of the five states in the browser (using the browse-unavailable
    path or a stubbed list where the network is not available) and read the rendered copy against the
    change spec's table. For the paste row, watch the address bar.
  - *Checks:* resolve which module decides the paste is unparseable. It should be the same
    `FeedRef` shape mortar enforces, reached by navigating to `/?feed=<value>`, or a client-side
    pre-check that agrees with it. A client check that is stricter than mortar's would reject values
    mason can lay.
  - *Status:* unverified

- **O1b · The picker mounts outside the subtree it makes inert, and widens the reader's condition.**
  - *Claim:* `FeedPicker` mounts after the `+layout.svelte` wrapper's closing tag at `:134`, beside
    `BrickReader`, and that wrapper's `inert` is a single widened expression,
    `page.state.brick || page.state.picker`, rather than a second attribute or a replacement of task
    03's condition.
  - *Evidence to collect:* read `+layout.svelte` around `:110` and `:134`. Run
    `grep -n inert web/src/routes/+layout.svelte` and expect one hit naming both keys. Run
    `just test-e2e` and confirm task 06's reader case is still green.
  - *Checks:* both halves fail silently. `inert` is inherited, so a picker mounted inside the wrapper
    is inert itself: unfocusable, unclickable, and invisible to every lane except a human at the
    browser. And a condition **replaced** rather than widened leaves the reader open with a live wall
    behind it, which task 06 only catches if it asserts the wall's inertness rather than the dialog's
    presence. Neither tsc nor vitest reads this file at all.
  - *Status:* unverified

- **O2 · Touch targets and motion follow the house rules.**
  - *Claim:* every control is at least 44px and the card's hover lift is behind `motion-safe:`.
  - *Evidence to collect:* read `FeedPicker.svelte` and `FeedCard.svelte` for `min-h-11` (or
    equivalent) on every interactive element and `motion-safe:` on the lift. Measure one control in
    devtools at 375px.
  - *Status:* unverified

- **O3 · The Playwright spec is green offline and says what it is.**
  - *Claim:* `web/tests/feed-picker.test.ts` covers opening from the landing page, Escape closing,
    the back gesture closing, and a bad paste showing the inline error without navigating; it runs
    with no network; and its header states that Playwright is the only lane that can see either new
    component.
  - *Evidence to collect:* run `just test-e2e` and confirm the four assertions pass. Read the file
    header. Confirm the AppView list is stubbed or the browse-unavailable state is used.
  - *Status:* unverified

- **O4 · Neither new component reads as dead code.**
  - *Claim:* `pnpm knip` is green, which means both components are reachable from an entry.
  - *Evidence to collect:* run `cd web && pnpm knip`. `web/knip.json` declares
    `src/routes/**/+*.{svelte,ts}` as the entries, so the reachability runs through `HandleForm` and
    `SwitchWall` into the picker.
  - *Checks:* confirm `FeedCard.svelte` is in `components/`, not `components/cards/`, which is the
    brick renderers' directory; a feed is not a brick and the misplacement would read as one.
  - *Status:* unverified

- **O4b · The history writes stayed in `.ts`.**
  - *Claim:* both entry points open the picker by calling task 17's `openPicker()` and close it with
    `closePicker()`; neither new component calls `pushState` itself.
  - *Evidence to collect:* run
    `grep -n pushState web/src/lib/components/FeedPicker.svelte web/src/lib/components/FeedCard.svelte`
    and expect nothing. Read both call sites in `HandleForm.svelte` and `SwitchWall.svelte`.
  - *Checks:* the mutual-exclusion rule with the reader is task 17's and is vitest-covered on both
    halves only while both halves are `.ts`. A `pushState` here would move one half into the file
    class nothing in `just check` can read, which is the exact failure this plan names task by task.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates and the e2e lane are green.
  - *Evidence to collect:* run `just guard-autoplay`, `just guard-dashes`, `just check` and
    `just test-e2e`.
  - *Status:* unverified

- **O6 · Reviewable: open, paste garbage, press back.**
  - *Claim:* on the built site a reviewer opens the picker from the landing page, pastes a garbage
    value, sees the inline error with no navigation, and presses back to close the picker and land
    where they started.
  - *Evidence to collect:* run `just test-e2e`, then perform the sequence in a browser.
  - *Status:* unverified

## Regression check

- `HandleForm.svelte`: the handle box still submits and still warms. Trace: type a handle, submit,
  expect a wall : (PRESERVED / REGRESSION)
- `SwitchWall.svelte`: the existing switch-walls affordance still opens and closes on a graph wall :
  (PRESERVED / REGRESSION)
- Task 03's reader: `web/tests/reader.test.ts` still green after `App.PageState` gained a second
  key : (PRESERVED / REGRESSION)

## Residue

- The picker's resting state is the network's own popular ranking, which is the most mainstream view
  of the atmosphere and an odd first impression for mason. That is the change spec's open question,
  not an obligation here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
