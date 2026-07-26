# Done Certificate · Task 17: feed picker state

**Task:** [17-feed_picker_state.md](17-feed_picker_state.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 17. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task is
> entirely `.ts`, so vitest genuinely covers it.

## Definition

DONE(Task 17) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** Everything the picker knows, in the one lane a test can see: recents, three
  queries, their loading and error states, and the hidden-tier filter.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break task 01's `App.PageState.brick` or the reader that reads it;
  must not break `state/handle.svelte.ts`'s `mason:handle` storage beside which `mason:feeds` sits.

## Obligations

- **O1 · The interface carries both keys, and both halves of the exclusion rule are in `.ts`.**
  - *Claim:* `App.PageState` declares both `brick?: string` and `picker?: 'feeds'`, the interface was
    extended rather than replaced, and opening either overlay pushes a page state carrying only its
    own key. The reader's half is `reader.svelte.ts` (task 01) and the picker's half is
    `feeds.svelte.ts`'s `openPicker()` / `closePicker()`, both vitest-covered.
  - *Evidence to collect:* read `web/src/app.d.ts`. Run
    `grep -rn "page.state.picker\|page.state.brick" web/src` and confirm the clearing happens in one
    module per overlay, not at each call site. Run the vitest case asserting `openPicker()` pushes a
    state whose `brick` key is absent. Run
    `grep -rn pushState web/src/lib/components/` and expect no hits.
  - *Checks:* resolve which module owns the clear. If both `reader.svelte.ts` and `feeds.svelte.ts`
    clear each other, they import each other and the cycle is a runtime hazard in a rune module; each
    pushing only its own key achieves the exclusion with no import between them. Then check the lane,
    which is the reason for the placement: a `pushState` written inside `FeedPicker.svelte` (task 18)
    would put half of a rule this task claims to own in a file neither tsc nor vitest can see, so the
    DoD would be half unverifiable.
  - *Status:* unverified

- **O2 · Recents are capped and deduped, with the cap named.**
  - *Claim:* `mason:feeds` holds the recents most recent first, deduped, capped at a named constant
    of 12, with a vitest case for the cap and one for the dedupe.
  - *Evidence to collect:* read the module for the named constant. Run the two cases.
  - *Status:* unverified

- **O3 · The hidden-tier filter is driven from the type, not a retyped list.**
  - *Claim:* a feed whose own view carries any hidden label, or whose creator does, is never listed;
    there is one vitest case per label in `HiddenLabel`, driven from the type.
  - *Evidence to collect:* read the test file and confirm the cases are generated from
    `HiddenLabel` (for example a `satisfies Record<HiddenLabel, ...>` table) rather than a hand
    array. Run them.
  - *Checks:* resolve `HiddenLabel` to the export task 14 added to `types.ts`, which
    `contract-check.ts` pins against mortar's `HIDDEN_LABELS`. A locally re-declared union would
    compile and would drift.
  - *Status:* unverified

- **O4 · An AppView failure degrades rather than empties.**
  - *Claim:* an AppView failure sets a browse-unavailable flag and leaves recents and paste working,
    rather than throwing or emptying the picker; a vitest case covers it.
  - *Evidence to collect:* run the named case. Read the catch path and confirm recents are still
    served from `localStorage` while the flag is set.
  - *Status:* unverified

- **O5 · One AppView base, and every fetch is browser-guarded.**
  - *Claim:* the module reads its base from `lib/appview.ts` and guards every fetch with SvelteKit's
    `browser` flag; no fourth hardcoded AppView constant exists in the tree.
  - *Evidence to collect:* run `grep -rn 'public.api.bsky.app' web/src` and expect exactly one hit,
    in `lib/appview.ts`. Read each fetch in `feeds.svelte.ts` for the `browser` guard.
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, including knip on a new module reachable only from a later
    component.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci`, `pnpm knip` and `just check`.
    Note that until task 18 mounts the picker, knip may see `feeds.svelte.ts` as unreachable; if so,
    this task must land together with a consumer or the gate is red.
  - *Status:* unverified

- **O7 · Reviewable: every state of the picker's table is reachable from the module.**
  - *Claim:* `cd web && pnpm test` runs the real module in node, and each of the five rows of the
    picker's states table (loading, search with no results, handle with no feeds, AppView
    unreachable, unparseable paste) corresponds to a reachable state.
  - *Evidence to collect:* the vitest run, plus a walk of the five rows against the module's state
    fields.
  - *Status:* unverified

## Regression check

- Task 01's `reader` still opens and closes with `App.PageState.brick`. Trace: run
  `cd web && pnpm vitest run src/lib/state/reader.test.ts` and `just test-e2e`, expect
  `web/tests/reader.test.ts` green : (PRESERVED / REGRESSION)
- `state/handle.svelte.ts`'s `mason:handle` read/write. Trace: the landing form still remembers the
  last handle : (PRESERVED / REGRESSION)

## Residue

- Browse and search both ride `app.bsky.unspecced.getPopularFeedGenerators`, which carries no
  stability promise. O4's browse-unavailable branch is the whole mitigation; a cached mirror is out
  of scope, per the plan's "what this plan does not do".

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
