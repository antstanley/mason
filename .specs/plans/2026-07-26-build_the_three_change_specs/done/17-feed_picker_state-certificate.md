# Done Certificate · Task 17: feed picker state

**Task:** [17-feed_picker_state.md](17-feed_picker_state.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* `web/src/app.d.ts` was extended, not replaced. The diff over that file is additive
    only: `brick?: string` and its doc comment survive untouched and `picker?: "feeds"` is added
    beside it inside the same `interface PageState`, with a block comment naming why a push replaces
    rather than merges. `grep -rn "page.state.picker\|page.state.brick" web/src` returns exactly two
    code sites: `reader.svelte.ts:73` (`const id = page.state.brick`) and `feeds.svelte.ts:306`
    (`return page.state.picker === "feeds"`). One module per overlay, and no call site of either
    clears the other's key by hand. `grep -rn pushState web/src/lib/components/` returns nothing at
    all, and `FeedPicker.svelte` does not exist yet; the only `pushState` / `replaceState` sites in
    `web/src` outside tests are `feeds.svelte.ts:322,341` and `reader.svelte.ts:132,171,203`.
    `pnpm vitest run src/lib/state/feeds.test.ts` case "opens on its own key alone, which is what
    shuts the reader" sets `pageState.brick = "a"` first, then asserts the pushed object
    `toEqual({ picker: "feeds" })` AND `not.toHaveProperty("brick")` AND
    `toHaveBeenCalledExactlyOnceWith("", { picker: "feeds" })`: PASS. The mirror case in
    `reader.test.ts`, "opens on its own key alone, which is what shuts the feed picker", sets
    `pageState.picker` and asserts the reader's pushed object `not.toHaveProperty("picker")`: PASS.
    Both halves are therefore in `.ts` and both are pinned.
  - *Checks resolved:* `pushState` and `replaceState` in `feeds.svelte.ts` resolve by step 4
    (imported) to `$app/navigation`, not to `history.pushState`; `page` resolves by step 4 to
    `$app/state`. Neither state module imports the other: `feeds.svelte.ts` imports
    `$app/environment`, `$app/navigation`, `$app/state`, `$lib/appview`, `./feedinfo.svelte`,
    `./handle.svelte` and a type from `$lib/types`, and `reader.svelte.ts` imports `./feed.svelte`
    and a type. No cycle, and the exclusion is structural on both sides rather than an import.
  - *Status:* SATISFIED

- **O2 · Recents are capped and deduped, with the cap named.**
  - *Claim:* `mason:feeds` holds the recents most recent first, deduped, capped at a named constant
    of 12, with a vitest case for the cap and one for the dedupe.
  - *Evidence to collect:* read the module for the named constant. Run the two cases.
  - *Collected:* `feeds.svelte.ts:25` is `export const MAX_RECENT_FEEDS = 12`, and it is the only
    spelling of the bound: `grep -nE "\b(12|30)\b" src/lib/state/feeds.svelte.ts` returns the
    constant, its doc comment, `RESULTS_PER_PAGE = 30` at `:36` and that one's doc comment, and
    nothing else. Both rules live in one function, `ordered()` at `:170`, which keeps the first
    occurrence of each uri and breaks at `MAX_RECENT_FEEDS`; `remember()` at `:389` puts the newest
    at the head before calling it, and `readRecent()` at `:219` applies the same function on the way
    in from storage. Cases run, all PASS: "caps the list at MAX_RECENT_FEEDS" (remembers
    `MAX_RECENT_FEEDS + 3`, asserts the length, the head and that `at://0` fell off), "moves a feed
    opened again to the front rather than listing it twice" (dedupe, and the newer record's name
    wins), "caps and dedupes what it reads, not just what it writes", and "writes the list back".
    The cases bite: with `if (kept.length === MAX_RECENT_FEEDS) break;` deleted the suite fails 2,
    with the `seen.has` guard deleted it fails 1; the file was restored byte-for-byte after each
    probe (md5 `2b779a265e3ab80c6a7e0de2433defa7`).
  - *Status:* SATISFIED

- **O3 · The hidden-tier filter is driven from a runtime list the type checks, not a retyped one.**
  - *Claim:* a feed whose own view carries any hidden label, or whose creator does, is never listed;
    there is one vitest case per label in `HiddenLabel`, generated from a runtime value whose
    completeness `HiddenLabel` enforces.
  - *Evidence to collect:* read the test file and confirm the cases are enumerated from a
    `Record<HiddenLabel, ...>` table, or from an array declared
    `as const satisfies readonly HiddenLabel[]` **with** a companion exhaustiveness check. Run them
    and count the cases against the union's members.
  - *Checks:* a type cannot drive anything at runtime, so "driven from the type" has to cash out as
    one of the two spellings above. Probe the guard rather than reading it: add a sixth member to
    `HiddenLabel` locally and confirm `pnpm check:ci` fails **in this test file**, then revert. A bare
    `satisfies readonly HiddenLabel[]` passes that probe only if the exhaustiveness check is present,
    because a short array satisfies the constraint on its own. Then resolve `HiddenLabel` to the
    export task 14 added to `types.ts`, which `contract-check.ts` pins against mortar's
    `HIDDEN_LABELS`; a locally re-declared union would compile and would drift.
  - *Collected:* the second spelling the DoD allows. `feeds.svelte.ts:64` is
    `export const HIDDEN_LABELS: Record<HiddenLabel, true>` with all five members, and it is the
    value the filter itself consults: `hidden()` at `:135` is
    `labels?.some((l) => l.val !== undefined && Object.hasOwn(HIDDEN_LABELS, l.val)) ?? false`,
    `Object.hasOwn` rather than `in` so a feed labelled "constructor" is not hidden by the prototype
    chain. `listing()` at `:148` calls it twice, `hidden(view.labels) || hidden(view.creator?.labels)`,
    so the creator's tier counts as well as the feed's own. `feeds.test.ts` takes
    `const labels = Object.keys(HIDDEN_LABELS)` and runs two `it.each(labels)` blocks plus a guard
    case asserting the generated list has 5 entries, so an empty iteration cannot pass silently.
    `--reporter=verbose` names 11 cases in that describe block: one per label in the feed's own
    position and one per label in the creator's, all five labels present by name, plus the count
    guard. Negative space beside them: a `!warn` feed is still listed, and feeds with absent, empty
    or malformed label arrays are still listed.
  - *Checks resolved:* the probes were run, not read. Deleting `sexual: true` from the record made
    `tsc --noEmit -p tsconfig.json` fail with
    `src/lib/state/feeds.svelte.ts(64,14): error TS2741: Property 'sexual' is missing ... but
    required in type 'Record<HiddenLabel, true>'`. Adding a sixth member `"fake-tier"` to
    `HiddenLabel` in `types.ts` failed in two places at once:
    `src/lib/contract-check.ts(103,3): error TS2344` (the fixture pin) and the same TS2741 in
    `feeds.svelte.ts(64,14)`. Both files were restored byte-for-byte (md5
    `2b779a265e3ab80c6a7e0de2433defa7` and `62433cbb1fb4e1d7fde8937c889f10c2`) and `jj diff --stat`
    is unchanged. The chain holds end to end: `mortar-core/src/sources/bluesky.rs:100`
    `HIDDEN_LABELS` is the source, `tests/contract.rs:431` generates `vocab.hiddenLabels` from that
    array into the fixture and asserts the committed fixture matches
    (`wire_contract_matches_the_committed_fixture` PASS in the nextest run),
    `contract-check.ts:102` asserts `Equal<keyof typeof contract.vocab.hiddenLabels, HiddenLabel>`,
    and this Record is the last link. `HiddenLabel` in `feeds.svelte.ts:7` resolves by step 4 to
    `$lib/types`, task 14's export at `types.ts:152`; nothing re-declares it locally. The filter
    bites: replacing `hidden()`'s body with `false` fails 10 cases, which is exactly the two
    `it.each` blocks.
  - *Status:* SATISFIED

- **O4 · An AppView failure degrades rather than empties.**
  - *Claim:* an AppView failure sets a browse-unavailable flag and leaves recents and paste working,
    rather than throwing or emptying the picker; a vitest case covers it.
  - *Evidence to collect:* run the named case. Read the catch path and confirm recents are still
    served from `localStorage` while the flag is set.
  - *Collected:* `feeds.svelte.ts:282` declares `browseUnavailable = $state(false)`, and there is one
    failure path for every way browsing can fail: `:423` is `await fetch(...).catch(() => null)`,
    `:428` takes the body only when `res.ok` and only through `res.json().catch(() => null)`, and
    `:435` treats a null body as the flag being set, the cursor dropped and nothing thrown.
    `loading` is set false at `:433` before that branch, so a failure never leaves skeletons up. An
    `it.each` over the three failure modes (unreachable, 500, a body that is not JSON) asserts
    `browse()` resolves rather than rejects, the flag is set, `loading` is false, and that the two
    load-bearing paths still work: recents planted in `mason:feeds` still read back through
    `readRecent()` (which never touches the network: `:219` reads `localStorage` only) and
    `remember()` still puts a new feed at the head. All three PASS, as do "does not page on from a
    failure" (the cursor goes with the failure, so `more()` does not re-ask) and "clears the flag as
    soon as browsing works again" (`:407` resets it at the head of every question). Paste never
    reaches this module at all: it is mortar's `?feed=` parse, and `web/src` holds no fetch of these
    endpoints outside `#ask`.
  - *Status:* SATISFIED

- **O5 · One AppView base, and every fetch is browser-guarded.**
  - *Claim:* the module reads its base from `lib/appview.ts` and guards every fetch with SvelteKit's
    `browser` flag; no fourth hardcoded AppView constant exists in the tree.
  - *Evidence to collect:* run `grep -rn 'public.api.bsky.app' web/src` and expect exactly one hit,
    in `lib/appview.ts`. Read each fetch in `feeds.svelte.ts` for the `browser` guard.
  - *Collected:* `grep -rn "public.api.bsky.app" web/src web/tests` returns exactly one line,
    `web/src/lib/appview.ts:14`. `grep -rn APPVIEW web/src` shows the base has four readers and no
    copies: `feedinfo.svelte.ts:2`, `profile.svelte.ts:2`, `feedinfo.test.ts:7` and
    `feeds.svelte.ts:4`, which builds both of its endpoint constants from it (`POPULAR` at `:45`,
    `ACTOR_FEEDS` at `:51`). The module has exactly one `fetch`, at `:423` inside `#ask`, and `#ask`
    returns at `:401` on `!browser` before reaching it, so all four public methods are guarded by one
    guard. Storage has its own second gate, `storageAvailable()` at `:214`, which is `browser` plus a
    `typeof localStorage`, because node has no such global to reach for. The case "never asks the
    AppView from a build with no browser" flips the mocked flag and asserts `fetch` was never called
    across `browse()`, `search()` and `byCreator()`: PASS. `byCreator` sends the handle straight
    through `cleanHandle` to `?actor=`, with no resolution hop: the case "lists one person's feeds
    from a bare handle" asserts one call, to `${ACTOR_FEEDS}?actor=alice.test&limit=30`.
  - *Status:* SATISFIED

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, including knip on a new module reachable only from a later
    component.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci`, `pnpm knip` and `just check`.
    Note that until task 18 mounts the picker, knip may see `feeds.svelte.ts` as unreachable; if so,
    this task must land together with a consumer or the gate is red.
  - *Collected:* run first-hand in the workspace. `cd web && pnpm test`: 7 files, 117 tests, all
    passing (46 of them new in `feeds.test.ts`). `pnpm check:ci`: exit 0 for both the app and the
    worker project. `pnpm knip`: exit 0, only the pre-existing `.css` configuration hint; the new
    module is reachable because knip's vitest plugin treats `feeds.test.ts` as an entry, so the
    feared red gate did not happen and no consumer had to be invented. `just check`: exit 0, which
    covers guard-dashes, guard-autoplay, guard-toolchain, fmt-check, guard-wasm, the wasm build, lint
    (oxlint plus knip plus clippy) and test (154 Rust via nextest, check:ci, 117 vitest). oxlint
    reports only the 4 pre-existing warnings in `FeedGrid.svelte` and `service-worker.ts`; nothing in
    the new files. `CI=1 just test-e2e`: exit 0, 20 passed, run with `CI=1` so Playwright started its
    own strict-port server rather than attaching to another workspace's. On the repo's own list: the
    behaviour is tested at the right tier (a `.ts` module in vitest, the only tier that can see it),
    negative space is covered (unparseable and non-array `mason:feeds`, entries with no uri or wrong
    field types, a view with no uri, an emptied search box, an at-only handle, a superseded answer,
    three AppView failure modes, a build with no browser), both new bounds are named constants with
    their reasoning in doc comments, and the why-comments are dense throughout. No changeset, which
    is consistent with that list's wording ("a changeset exists for any user-visible change"): no
    route or component imports this module yet, so nothing a visitor can reach changed; the commit
    message states that reasoning and puts the changeset with task 18's screen.
  - *Status:* SATISFIED

- **O7 · Reviewable: every state of the picker's table is reachable from the module.**
  - *Claim:* `cd web && pnpm test` runs the real module in node, and each of the five rows of the
    picker's states table (loading, search with no results, handle with no feeds, AppView
    unreachable, unparseable paste) corresponds to a reachable state.
  - *Evidence to collect:* the vitest run, plus a walk of the five rows against the module's state
    fields.
  - *Collected:* the suite runs the real module, not a double: `feeds.test.ts` mocks only
    `$app/environment`, `$app/navigation`, `$app/state` and the `localStorage` / `fetch` / `history`
    globals, and imports `FeedsState`, `feeds`, `HIDDEN_LABELS` and `MAX_RECENT_FEEDS` from
    `./feeds.svelte` itself. The walk, row by row. Loading: `loading` is set true at `:406` and false
    at `:433`, and the case "is loading while an answer is in flight, and not after it" pins both
    edges with a hand-landed promise. A search with no results: `question === "search"` with
    `results` empty and `term` kept, which "searches the same endpoint with the term, escaped"
    asserts together, and the sibling case shows an emptied box falls back to `browse()` instead, so
    that sentence is never shown to somebody who typed nothing. A handle with no feeds:
    `question === "creator"` with `results` empty, asserted by "lists one person's feeds from a bare
    handle". The AppView unreachable: `browseUnavailable` true with recents intact, asserted by the
    three-mode `it.each` under O4. The fifth row, a pasted value that will not parse, is not a state
    of this module and cannot be: the task ships no input, and both the picker's one input and its
    inline paste error are task 18's first and fifth steps, with mortar's `?feed=` parse behind them.
    The four rows this module owns are each pinned by a named case; the fifth is recorded here as
    belonging to task 18's Playwright lane rather than as coverage this task withheld.
  - *Status:* SATISFIED

## Regression check

- Task 01's `reader` still opens and closes with `App.PageState.brick`. Trace: run
  `cd web && pnpm vitest run src/lib/state/reader.test.ts` and `just test-e2e`, expect
  `web/tests/reader.test.ts` green : PRESERVED. `reader.svelte.ts:132`'s
  `pushState("", { brick: brick.id })` still typechecks against the widened interface
  (`pnpm check:ci` exit 0 under `exactOptionalPropertyTypes`, which an added optional member cannot
  loosen), `pnpm vitest run src/lib/state/reader.test.ts src/lib/state/feedinfo.test.ts` is 31 green
  including the new mirror case, and `CI=1 just test-e2e` is 20 green.
- `state/handle.svelte.ts`'s `mason:handle` read/write. Trace: the landing form still remembers the
  last handle : PRESERVED. The file is untouched by this diff; `feeds.svelte.ts` reads a different
  key (`mason:feeds`, `:30`) and imports only the pure `cleanHandle` helper, which it calls with a
  string and does not mutate. The e2e lane, which drives the landing form in a real chromium, is
  green.
- One caller outside the two named above was modified: `feedinfo.svelte.ts`'s `feedRkey` gained an
  `export` keyword and nothing else (the diff shows only the keyword and a doc paragraph) :
  PRESERVED, with `feedinfo.test.ts` green as the trace.

## Residue

- Browse and search both ride `app.bsky.unspecced.getPopularFeedGenerators`, which carries no
  stability promise. O4's browse-unavailable branch is the whole mitigation; a cached mirror is out
  of scope, per the plan's "what this plan does not do".
- Two shadowings were found and are benign, recorded so task 18 does not trip over them: the
  parameter `feeds` in `ordered()` (`:170`) and the local `const feeds` in `readRecent()` (`:236`)
  both shadow the module-level `export const feeds` at `:455`. Neither body references the outer
  binding, and a reference would have thrown at import time, since `readRecent()` runs inside the
  singleton's own field initializer while that const is still in its temporal dead zone. The 46
  green cases, which import the singleton, are the proof it does not.
- `closePicker()` with no entry of its own calls `replaceState("", {})`, which would also clear a
  `brick` key if the picker were closed while the reader was open. That pairing is unreachable
  through task 18's wiring (each overlay closes itself, and opening either shuts the other), and
  `reader.close()` has carried the identical shape since task 01.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED on evidence collected first-hand, the load-bearing one by probe
rather than by reading (deleting a label from `HIDDEN_LABELS` and adding a sixth to `HiddenLabel`
both fail `tsc` in `feeds.svelte.ts`, and neutering the filter fails 10 cases), the gates are green
in this workspace (`just check` exit 0, 117 vitest, 154 Rust, `CI=1 just test-e2e` 20), and the
reader, `mason:handle` and `feedRkey` regressions are all PRESERVED; the fifth row of the states
table is task 18's input rather than a gap in this module.
