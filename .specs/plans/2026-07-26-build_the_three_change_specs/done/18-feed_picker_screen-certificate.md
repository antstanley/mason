# Done Certificate · Task 18: feed picker screen

**Task:** [18-feed_picker_screen.md](18-feed_picker_screen.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* all five rows are driven by `web/tests/feed-picker.test.ts`, and every case passed
    in my own `CI=1 just test-e2e` run (32 passed, no retries): skeletons at `:381` ("a page in
    flight is six skeletons at the picker's own column count", six `.animate-pulse` boxes plus the
    `aria-live` line "looking for feeds"), the empty search and the empty creator at `:333` ("no
    feeds by that name" and "that person has not made any feeds" plus the `/?actor=alice.test`
    link), the AppView unreachable at `:358` ("browsing is quiet right now", with the recents list
    and the paste box still answering), and the unparseable paste at `:266`, which drives three
    shapes of bad value and asserts the URL never moves. The rendered copy matches the change spec's
    States table row for row (`.specs/changes/2026-07-26-lay_a_bluesky_feed.md`, the feed picker
    section). I also drove the paste row myself against the real static build on a private port: the
    alert reads "that is not a feed mason can lay. paste a bsky.app feed link, or an at:// uri
    ending in a feed generator.", the URL was unchanged, the window object was never rebuilt and the
    value stayed in the box.
  - *Checks resolved:* the decision resolves to `askedFor()` in the new module
    `web/src/lib/feedref.ts`, imported at `FeedPicker.svelte:124` (step 4 of the resolution
    sequence). No shadowing: nothing local, nothing at module level and no global shares the name.
    It is the allowed client-side pre-check, and it is not stricter than mortar's: `AT_URI_PREFIX`,
    `FEED_GENERATOR_COLLECTION`, `BSKY_FEED_URL_PREFIX`, `BSKY_FEED_SEGMENT`, `DID_METHOD_PREFIXES`,
    the exactly-three-segments rule and the rkey, DID-id and handle character sets are the same sets
    `server/crates/mortar-core/src/sources/feedref.rs` uses, and the length cap is 1024 characters
    against mortar's 1024 bytes, which is the permissive direction. Every value mortar accepts
    begins `at://` or `https://bsky.app/profile/`, so every value mortar accepts is
    `referenceShaped()` and reaches the parser rather than falling through to search. 11 vitest
    cases in `web/src/lib/feedref.test.ts` pass and pair each accepted spelling with the rejections
    it implies.
  - *Status:* SATISFIED

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
  - *Collected:* `web/src/routes/+layout.svelte:44` reads
    `const overlayOpen = $derived(reader.isOpen || feeds.isOpen);`: one expression, the reader's
    disjunct untouched and the picker's added beside it. `grep -n inert
    web/src/routes/+layout.svelte` returns exactly one attribute hit, `:142` `inert={overlayOpen}`;
    the four other hits are the comments at `:32`, `:38`, `:43` and `:175`. `<FeedPicker />` is at
    `:179`, after the wrapper's closing `</div>` at `:172` and directly below `<BrickReader />` at
    `:178`, so it is a sibling of the inert subtree and not a descendant. The literal spelling the
    task file asks for, `page.state.brick || page.state.picker`, is stale: task 03 deliberately
    replaced a page-state test with `reader.isOpen`, because page state is wider than the condition
    the reader renders on and left the wall frozen under nothing. Each disjunct here asks its own
    surface for its own predicate, which is that decision carried forward, and `feeds.isOpen` is
    `page.state.picker === "feeds"` (`state/feeds.svelte.ts:305`). Behaviourally, both ways round, in
    my own `CI=1 just test-e2e` run: `feed-picker.test.ts:310` asserts `#handle` refuses focus with
    the picker up and the dialog's input takes it, `feed-picker.test.ts:402` asserts the wall's
    layout radio refuses focus with the picker up from the switcher, and task 06's own case,
    `reader.test.ts:282` "the wall behind an open reader refuses focus, and the reader takes it",
    passed unchanged. I confirmed it by hand as well, on the built site: with the picker open,
    focusing `#handle` left `document.activeElement` elsewhere, and after the back gesture the same
    call succeeded and the wrapper carried no `inert` attribute.
  - *Status:* SATISFIED

- **O2 · Touch targets and motion follow the house rules.**
  - *Claim:* every control is at least 44px and the card's hover lift is behind `motion-safe:`.
  - *Evidence to collect:* read `FeedPicker.svelte` and `FeedCard.svelte` for `min-h-11` (or
    equivalent) on every interactive element and `motion-safe:` on the lift. Measure one control in
    devtools at 375px.
  - *Collected:* `min-h-11` is on the close control (`FeedPicker.svelte:338`), the input (`:363`),
    the submit (`:367`), "more feeds" (`:418`), the empty-creator wall link (`:436`) and the card
    anchor (`FeedCard.svelte:60`), and on both entry-point triggers (`HandleForm.svelte:85`,
    `SwitchWall.svelte:513`). `FeedCard.svelte:60` carries `motion-safe:hover:-translate-y-1`, with
    only colour and shadow left in the unguarded transition, the same shape as `BrickShell.svelte:38`.
    Measured rather than read: `feed-picker.test.ts:424` measures the rendered bounding box of every
    `a, button, input` inside the dialog under reduced motion and requires 44 or more, with a floor
    on the control count so an empty selector cannot pass; it passed. I repeated the measurement at
    a 375px viewport against the built site: close 44.0px, input 48.0px, submit 44.0px, card
    111.3px, "more feeds" 44.0px, and the landing trigger 44.0px. In the same run the card moved
    0.00px on hover under `prefers-reduced-motion: reduce` and lifted under full motion.
  - *Status:* SATISFIED

- **O3 · The Playwright spec is green offline and says what it is.**
  - *Claim:* `web/tests/feed-picker.test.ts` covers opening from the landing page, Escape closing,
    the back gesture closing, and a bad paste showing the inline error without navigating; it runs
    with no network; and its header states that Playwright is the only lane that can see either new
    component.
  - *Evidence to collect:* run `just test-e2e` and confirm the four assertions pass. Read the file
    header. Confirm the AppView list is stubbed or the browse-unavailable state is used.
  - *Collected:* my own `CI=1 just test-e2e` run reported `32 passed (10.7s)`, exit 0, with no flaky
    or retried case. All four named assertions are among them and passed: `:187` opening from the
    landing page, `:227` Escape closing with focus back on the trigger, `:241` the back gesture
    closing with the URL unmoved, and `:266` the bad paste said in place. The header states, in
    capitals, "THIS IS THE ONLY LANE IN THE REPO THAT RENDERS `FeedPicker` OR `FeedCard` AT ALL",
    and gives the reason: tsc cannot parse `.svelte`, so no component file enters the typecheck
    program, and the vitest suites are `.ts` in node with no DOM. A grep over the vitest suites finds
    no component import, which corroborates it. Offline: `appView()` at `:300` routes
    `/public\.api\.bsky\.app/`, which is the whole of `web/src/lib/appview.ts`'s `APPVIEW`, and
    fulfils or aborts it; the two cases that need no listing use `addInitScript` on `mason:feeds` and
    the `down: true` abort; and the one case that lays a wall uses `/?actor=demo`, which the wasm
    service worker answers from fixtures. The only external host left is the Google Fonts stylesheet
    in `svelte:head`, which every existing spec in this repo already inherits and which no assertion
    depends on.
  - *Status:* SATISFIED

- **O4 · Neither new component reads as dead code.**
  - *Claim:* `pnpm knip` is green, which means both components are reachable from an entry.
  - *Evidence to collect:* run `cd web && pnpm knip`. `web/knip.json` declares
    `src/routes/**/+*.{svelte,ts}` as the entries, so the reachability runs through `HandleForm` and
    `SwitchWall` into the picker.
  - *Checks:* confirm `FeedCard.svelte` is in `components/`, not `components/cards/`, which is the
    brick renderers' directory; a feed is not a brick and the misplacement would read as one.
  - *Collected:* `cd web && pnpm knip` exits 0, reporting only the pre-existing `.css` configuration
    hint; it also ran inside `just check`'s `lint` step, which exited 0. `FeedPicker` is imported by
    `+layout.svelte:9`, `FeedCard` by `FeedPicker.svelte:126`, and `lib/feedref.ts` by
    `FeedPicker.svelte:124` and by its own unit suite.
  - *Checks resolved:* the new file is `web/src/lib/components/FeedCard.svelte`, one directory above
    `components/cards/`, and its opening comment says why. Confirmed against `jj diff --stat`.
  - *Status:* SATISFIED

- **O4b · The history writes stayed in `.ts`.**
  - *Claim:* both entry points open the picker by calling task 17's `openPicker()` and close it with
    `closePicker()`; neither new component calls `pushState` itself.
  - *Evidence to collect:* run
    `grep -n pushState web/src/lib/components/FeedPicker.svelte web/src/lib/components/FeedCard.svelte`
    and expect nothing. Read both call sites in `HandleForm.svelte` and `SwitchWall.svelte`.
  - *Checks:* the mutual-exclusion rule with the reader is task 17's and is vitest-covered on both
    halves only while both halves are `.ts`. A `pushState` here would move one half into the file
    class nothing in `just check` can read, which is the exact failure this plan names task by task.
  - *Collected:* the grep produced no output and exited 1. A wider
    `grep -rn "pushState|replaceState" web/src/lib/components/` is also empty, so no component in
    the tree writes history. `HandleForm.svelte:76`-`:88` focuses its own trigger and then calls
    `feeds.openPicker()`; `SwitchWall.svelte:507`-`:516` calls `closePanel()` first, which hands
    focus to the switcher's own button because the control that opened the picker unmounts with the
    panel, and then `feeds.openPicker()`. `FeedPicker.svelte:171`-`:173` closes with
    `feeds.closePicker()` and nothing else.
  - *Checks resolved:* both names resolve to `state/feeds.svelte.ts` (step 4, imported), and that
    module is untouched by this task: `jj diff --stat` lists ten files and it is not one of them.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates and the e2e lane are green.
  - *Evidence to collect:* run `just guard-autoplay`, `just guard-dashes`, `just check` and
    `just test-e2e`.
  - *Collected:* run by me in the task's workspace. `just guard-dashes` exit 0. `just guard-autoplay`
    exit 0. `just check` exit 0, which is guard-dashes, guard-autoplay, guard-toolchain, fmt-check,
    guard-wasm, lint (oxlint clean bar the four pre-existing `FeedGrid` and `service-worker`
    warnings, knip clean, clippy clean) and test (154 Rust tests, tsc clean on both projects, 130
    vitest tests in 8 files). `CI=1 just test-e2e` exit 0, 32 passed. The e2e lane was run as `CI=1`
    throughout, never bare, so it built and served its own tree on a strict port rather than
    attaching to another workspace's. A changeset is present at `.changeset/a-second-front-door.md`,
    minor, which is right for a new surface.
  - *Status:* SATISFIED

- **O6 · Reviewable: open, paste garbage, press back.**
  - *Claim:* on the built site a reviewer opens the picker from the landing page, pastes a garbage
    value, sees the inline error with no navigation, and presses back to close the picker and land
    where they started.
  - *Evidence to collect:* run `just test-e2e`, then perform the sequence in a browser.
  - *Collected:* `CI=1 just test-e2e` green, 32 passed. I then performed the sequence myself against
    `web/build`, served on a private port, in chromium, with the real network left in place.
    Clicking "or pick a feed to lay" opened the dialog with focus in `#feed-query` and the URL
    unchanged; the popular list arrived from the live AppView and rendered as cards with avatars,
    creator handles, clamped descriptions and like counts. Pasting
    `https://bsky.app/profile/alice.test/post/3k2abc` and submitting produced the inline alert "that
    is not a feed mason can lay...", `aria-invalid="true"` on the input, an unchanged URL, an
    unchanged window object, and the value still in the box. The back gesture removed the dialog,
    left the URL at the landing page, put focus back on the "or pick a feed to lay" trigger and
    returned `#handle` to focusable. No page errors and no console errors in the whole run.
  - *Status:* SATISFIED

## Regression check

- `HandleForm.svelte`: the handle box still submits and still warms. Trace: type a handle, submit,
  expect a wall : PRESERVED. The only change to the file is one import and one button appended to
  the copy block under the form; `submit()`, the focus effect and the `warmFeed` effect are
  untouched, and the landing page rendered and hydrated in every run I drove.
- `SwitchWall.svelte`: the existing switch-walls affordance still opens and closes on a graph wall :
  PRESERVED. The only change is one import and one button appended inside the open panel;
  `feed-picker.test.ts:402` opens the switcher on the demo wall and uses it, and
  `service-worker-smoke.test.ts:72` ("a feed wall renders the chrome a graph wall gets") passed.
- Task 03's reader: `web/tests/reader.test.ts` still green after `App.PageState` gained a second
  key : PRESERVED. All 14 reader cases passed, the inert case and the two shallow-routing cases
  included. `openPicker()` pushes `{ picker: "feeds" }`, which replaces page state and so clears
  `brick`, and `reader.isOpen` requires `page.state.brick`, so the two overlays cannot both be up.

## Residue

- The picker's resting state is the network's own popular ranking, which is the most mainstream view
  of the atmosphere and an odd first impression for mason. That is the change spec's open question,
  not an obligation here.
- Noted, outside the DoD and not charged against it: `atEnd` in `FeedPicker.svelte` is reset only on
  submit, so paging a search to its end and then reopening the picker leaves the freshly fetched
  popular list without its "more feeds" control until the next submit. Cosmetic, and paging is not a
  row of the states table.
- This task also added one case to `web/tests/reader.test.ts` (a glaze card's filmstrip anchor), a
  file it does not own. It is purely additive and green.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1, O1b, O2, O3, O4, O4b, O5 and O6 are all SATISFIED on evidence collected in this
workspace by the validator (`just check` exit 0, `CI=1 just test-e2e` 32 passed with every named
case green, `pnpm knip` exit 0, both greps exactly as specified, the 44px boxes measured at 375px,
and the Reviewable sequence driven by hand on the built site), and all three named regression
callers are PRESERVED.
