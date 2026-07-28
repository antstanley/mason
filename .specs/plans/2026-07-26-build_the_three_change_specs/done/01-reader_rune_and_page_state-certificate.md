# Done Certificate · Task 01: reader rune and page state

**Task:** [01-reader_rune_and_page_state.md](01-reader_rune_and_page_state.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 01. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. Do not mark
> an obligation SATISFIED without its evidence; do not record DONE with any non-SATISFIED obligation.

## Definition

DONE(Task 01) is every obligation O1 to O6 below holding, each backed by the evidence it names,
not by assertion.

## Premises

- **P1 · Goal.** `App.PageState` exists with `brick?: string`, and every open, close, step and
  modifier-key decision the reader makes lives in one `.svelte.ts` module vitest runs for real.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break `web/src/lib/state/feed.svelte.ts`'s freeze path:
  `reader.open` calls `feed.freeze()` and `freeze()` at `feed.svelte.ts:121` returns early on a
  settled wall, so calling it must stay a no-op there.

## Obligations

- **O1 · `App.PageState` declares exactly one member and `pushState` typechecks.**
  - *Claim:* `web/src/app.d.ts` declares `interface PageState { brick?: string }` and
    `pushState('', { brick: id })` typechecks under `exactOptionalPropertyTypes`.
  - *Evidence to collect:* read `web/src/app.d.ts`; confirm `PageState` is uncommented with one
    member and that `Error`, `Locals`, `PageData` and `Platform` stay commented. Run
    `cd web && pnpm check:ci`, expect clean.
  - *Collected:* `web/src/app.d.ts:11-16` declares `interface PageState` with exactly one member,
    `brick?: string` (`:15`), one member per line. `:5`, `:6`, `:7` and `:17` keep `Error`,
    `Locals`, `PageData` and `Platform` commented. `cd web && pnpm check:ci` (`svelte-kit sync &&
    tsc --noEmit -p tsconfig.json`) exits 0 with no diagnostics; `tsconfig.json` carries `strict`,
    `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes`. The typecheck is not vacuous:
    `tsc --listFiles` names `web/src/app.d.ts`, `web/src/lib/state/reader.svelte.ts` and
    `web/src/lib/state/reader.test.ts` in the program, and
    `node_modules/@sveltejs/kit/types/index.d.ts:3401` types the call as
    `pushState(url: string | URL, state: App.PageState): void`, so `reader.svelte.ts:100`'s
    `pushState("", { brick: brick.id })` is checked against this very interface.
  - *Status:* SATISFIED

- **O2 · `activate` rejects every modified click and accepts only a plain left click.**
  - *Claim:* `activate` returns false and calls neither `preventDefault` nor `open` when any of
    `metaKey`, `ctrlKey`, `shiftKey`, `altKey` is set or `button !== 0`; otherwise it returns true
    and calls `preventDefault` exactly once.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/reader.test.ts`; confirm
    there is one case per modifier and one for `button !== 0`, each asserting the return value AND
    that `preventDefault` was not called. Read `reader.svelte.ts`'s `activate` body.
  - *Checks:* resolve `open` inside `activate` to the class method, not a same-named import; resolve
    `pushState` to the `$app/navigation` import, not `history.pushState`.
  - *Collected:* `reader.svelte.ts:109-121`: the guard at `:115` is
    `event.metaKey || event.ctrlKey || event.shiftKey || event.altKey || event.button !== 0`, and it
    `return false` at `:116` before `preventDefault` (`:118`) or `open` (`:119`) can run. The
    accepting path calls `event.preventDefault()` exactly once at `:118`, then
    `this.open(brick, focusable(event.currentTarget))`, then returns true.
    `pnpm vitest run src/lib/state/reader.test.ts --reporter=verbose` lists five separate declining
    cases, one per modifier and one for the middle click: "declines cmd-click", "declines
    ctrl-click", "declines shift-click", "declines alt-click", "declines a middle click" (the
    `it.each` at `reader.test.ts:107-123` expands to five tests, not one). Each asserts
    `activate(...) === false`, `preventDefault` not called, a `vi.spyOn(opened, "open")` not called,
    `pushState` not called and `brick` still null. The accepting case at `:125-134` asserts `true`,
    `preventDefault` called exactly once, and `pushState` called exactly once with
    `("", { brick: "a" })`. 18 tests passed, 0 failed.
  - *Checks:* `open` at `:119` is written `this.open(...)`, so it resolves at function-resolution
    step 2 (enclosing class, `:89`); the file imports no symbol named `open`, and the only other
    `open` in the module is a local `const open = this.brick` at `:72` inside the `index` getter,
    a different function scope that the `:119` call cannot see. `pushState` at `:100` fails steps 1
    to 3 (no local, no class member, no module-level definition) and resolves at step 4 to
    `import { pushState } from "$app/navigation"` (`:1`). It is not `history.pushState`, which is
    reachable only through the `history` receiver and is never used here; the module's only bare
    `history` use is `history.back()` at `:133`. `replaceState` at `:139` and `:171` resolves the
    same way, to the `$app/navigation` import. No shadowing that changes behaviour.
  - *Status:* SATISFIED

- **O3 · Stepping is id-derived, clamped, and never paginates.**
  - *Claim:* `index` is derived by `feed.items.findIndex(b => b.id === brick.id)` rather than stored;
    `next()` and `prev()` clamp at 0 and `items.length - 1`; neither calls `feed.loadMore()` nor
    `fetchFeed`; a brick absent from `feed.items` yields `index === -1` with `canNext` and `canPrev`
    both false.
  - *Evidence to collect:* grep `reader.svelte.ts` for `loadMore` and `fetchFeed`, expect no hits.
    Run the vitest cases for clamping at both ends and for the absent-brick case.
  - *Checks:* resolve `feed` in `reader.svelte.ts` to the `feed` singleton exported from
    `$lib/state/feed.svelte`, not a local.
  - *Collected:* `index` is a getter at `:71-75`, computed on every read as
    `feed.items.findIndex((b) => b.id === open.id)` with `open` the currently held brick; there is
    no index field anywhere in the class (the fields are `brick`, `#opener`, `#pushed` only, at
    `:50`, `:53`, `:56`). `canPrev` is `this.index > 0` (`:78`) and `canNext` is
    `at >= 0 && at < feed.items.length - 1` (`:84`), so index -1 fails both. `#step` at `:158-172`
    reads the derived index, returns when it is negative (`:160`), reads `feed.items[at + delta]`
    and returns when that is `undefined` (`:166`), which under `noUncheckedIndexedAccess` is the
    same check at index -1 and at index `length`. Only then does it move the brick and
    `replaceState("", { brick: to.id })`, never `pushState`.
    `grep -n "loadMore\|fetchFeed" web/src/lib/state/reader.svelte.ts` returns no hits (exit 1).
    Test run: "stops at the last laid brick rather than paginating" and "stops at the first laid
    brick" both pass, each asserting the brick is unmoved, `replaceState` not called, a
    `vi.spyOn(feed, "loadMore")` not called and the mocked `fetchFeed` not called. "steps nowhere at
    all once its brick has left the wall" passes, asserting `index === -1`, `canPrev` false,
    `canNext` false, and that `next()` then `prev()` leave the brick unmoved with no `replaceState`,
    no `loadMore` and no `fetchFeed`. "locates the open brick by id and steps to its neighbours"
    passes, showing a step calls `replaceState` exactly once with `("", { brick: "c" })` while
    `pushState` stays at the single open.
  - *Checks:* `feed` at `:74`, `:84`, `:96` and `:161` fails steps 1 to 3 and resolves at step 4 to
    `import { feed } from "./feed.svelte"` (`:3`), which is the singleton
    `export const feed = new FeedState()` at `feed.svelte.ts:216`. No local or parameter named
    `feed` exists in the module.
  - *Status:* SATISFIED

- **O4 · The module is node-runnable and imports no component.**
  - *Claim:* the only SvelteKit imports are `$app/navigation` and `$app/state`, both mockable, and
    no `.svelte` component is imported. The **graph** reaches one more, so the test carries three
    mocks: those two plus `$lib/api`.
  - *Evidence to collect:* read the import block of `reader.svelte.ts`; confirm no path ends in
    `.svelte` other than `.svelte.ts` state modules. Read `reader.test.ts` and confirm all three
    `vi.mock` calls are present. Run `cd web && pnpm vitest run src/lib/state/reader.test.ts` and
    confirm it passes in the node environment.
  - *Checks:* trace the graph rather than the import block. `reader.svelte.ts` imports the `feed`
    singleton for `feed.freeze()` and `feed.items`, `feed.svelte.ts:1` imports `$lib/api`, and
    `api.ts:1` imports `$app/environment`, which has nothing to answer under
    `vitest.config.ts`'s `environment: "node"`. `feed.test.ts:10` mocks `$lib/api` for exactly this
    reason and says so in a comment; a two-mock test here fails on the third module, not on the
    module under test.
  - *Collected:* the whole import block is four lines, `reader.svelte.ts:1-4`:
    `$app/navigation` (`pushState`, `replaceState`), `$app/state` (`page`), `./feed.svelte` (the
    rune module, not a component) and a type-only `$lib/types`. No path ends in `.svelte` other
    than the `.svelte.ts` state module, and no component is imported.
    `reader.test.ts` carries exactly three `vi.mock` calls, at `:13` (`$app/navigation`), `:19`
    (`$app/state`, written as a getter so the factory's hoisting above module init is safe) and
    `:31` (`$lib/api`, with a comment naming the graph as the reason). The suite runs the real
    module: there is no `vi.mock("./reader.svelte")`, no `__mocks__` directory anywhere under
    `web/src`, and no vite alias for it, so `import { reader, ReaderState } from "./reader.svelte"`
    at `:10` is the file under review. `pnpm vitest run src/lib/state/reader.test.ts` reports
    1 file, 18 tests passed under `vitest.config.ts`'s `environment: "node"`.
  - *Checks:* the graph is as the certificate states. `feed.svelte.ts:1` is
    `import { fetchFeed, FeedError } from "$lib/api"`, and `api.ts:1` is
    `import { browser } from "$app/environment"`. The third mock is therefore load-bearing, and it
    also makes "the reader never fetches" assertable in O3.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* tests at the right tier, negative-space cases for every rejected activation, named
    constants for any new bound, gates green, no em dash.
  - *Evidence to collect:* run `just check` from the repo root, expect clean. This runs
    `guard-dashes`, `guard-autoplay`, `guard-toolchain`, `fmt-check`, `guard-wasm`, `lint` and
    `test` (cargo nextest plus `pnpm check:ci` plus vitest). No `just wasm` is needed: no Rust
    changed.
  - *Collected:* `just check` from `/Users/ant/code/mason-ws2` exits 0. The log shows every stage:
    `guard-dashes`, `guard-autoplay`, `guard-toolchain`, `oxfmt --check src` ("All matched files use
    the correct format", 19 files), `cargo fmt --all --check`,
    `cargo check ... --target wasm32-unknown-unknown --all-targets`, `oxlint src` (only the four
    pre-existing warnings in `FeedGrid.svelte` and `service-worker.ts`, none in the new files),
    `knip` (only the pre-existing `.css` configuration hint), `cargo clippy --workspace
    --all-targets -- -D warnings`, 97 Rust tests passed, `pnpm check:ci` clean, and vitest
    3 files / 39 tests passed. Tier: the behaviour lives in a rune module with a vitest suite, which
    is the only lane in this repo that both typechecks and executes it. Negative space: five
    declining activation cases (one per modifier plus the middle click), both step clamps, the brick
    that has left the wall, the not-our-entry close, and a click with no focusable target. New
    bounds: none introduced, the reader's limits are the ends of `feed.items` rather than a numeric
    bound, so there is no constant to name. Comments say why throughout (the freeze-before-push
    order at `:90-99`, the replace-not-push rule at `:168-171`, the two close branches at
    `:126-138`, and why the index is derived at `:66-70`). No em dash or en dash: a direct
    `grep` for both over the three changed files returns nothing, and `guard-dashes` passed inside
    `just check`. No changeset, correctly: the task ships no component and no wiring, so nothing
    here is visible to a visitor, and the repo rule is a changeset per user-visible change.
  - *Status:* SATISFIED

- **O6 · Reviewable: the module is the whole of the reader's decisions and no `.svelte` file moved.**
  - *Claim:* a reviewer runs the vitest file, reads the module, and finds every reader decision in
    it, with no component touched.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/reader.test.ts`, then list
    the diff's changed files and confirm none has a `.svelte` extension (`.svelte.ts` excepted).
  - *Collected:* the reviewable action was exercised. `cd web && pnpm vitest run
    src/lib/state/reader.test.ts` prints 1 file, 18 tests passed, and the verbose run names all 18.
    Reading `reader.svelte.ts` end to end: the modifier-key rule (`activate`, `:109`), the
    freeze-then-push open (`:89`), the two close paths (`:124`), stepping and its clamps
    (`#step`, `:158`), the derived position (`index`, `:71`) with `canPrev`/`canNext`, the open/shut
    signal (`isOpen`, `:62`) and the focus return (`returnFocus`, `:153`) are all in the one module.
    `jj st` lists exactly three files: `M web/src/app.d.ts`, `A web/src/lib/state/reader.svelte.ts`,
    `A web/src/lib/state/reader.test.ts`. None is a `.svelte` component, and nothing outside the
    task was touched.
  - *Status:* SATISFIED

## Regression check

- `web/src/lib/state/feed.svelte.ts:121` `freeze()` is now called by `reader.open` on a possibly
  settled wall. Trace: with `warming === false`, expect the guard at `:124` to return before the
  fetch : PRESERVED. `reader.svelte.ts:96` calls `void feed.freeze()` with no arguments, so
  `generation` defaults to `this.#generation` and the third clause of `:124`
  (`generation !== this.#generation`) is false; the first clause `!this.warming` is true on a
  settled wall, so the guard returns before `#generation` is bumped, before `loading` is set and
  before `fetchFeed` is reached. No state change and no request. On a warming wall it freezes, which
  is the intended new behaviour rather than a regression. `freeze` catches its own failures, so the
  fire-and-forget `void` leaves no unhandled rejection.
- No other existing caller is in scope: this task adds a module and uncomments a type. Confirmed:
  `grep -rn "pushState\|replaceState\|page\.state" web/src` outside the two new files returns
  nothing, so uncommenting `App.PageState` changes the type of no existing call site, and
  `pnpm check:ci` is clean over the whole program.

## Residue

- The close-on-reload behaviour (page state does not survive a reload, so a reload lands closed) is
  intended and stated in the task's steps, but it is not a DoD item and therefore not an obligation.
  Note it if the validator sees it broken.
  - *Validator's note:* not broken. Nothing in the module persists anything, `isOpen` reads only
    `page.state.brick`, and a reload starts the singleton with `brick = null` and `#pushed = false`,
    which the "the singleton" test pins.
- *Validator's note, for task 03 rather than this task.* `#pushed` is cleared only by `close()`.
  When the reader is dismissed by the browser's own back gesture, no code path clears it, so a bare
  `close()` afterwards, with no intervening `open()`, would call `history.back()` a second time and
  leave the wall. Nothing this task ships can reach that, the next `open()` resets the flag, and the
  DoD specifies `close()` exactly as written, so it is not a defect here. It is a constraint on the
  dialog: task 03 should drive its teardown from `reader.isOpen` and not call `close()` on a reader
  that is already shut.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED against evidence collected first-hand (`pnpm check:ci` clean
with both new files proven in the tsc program, 18 vitest cases passing on the real module behind
its three mocks, no `loadMore`/`fetchFeed` in the module, `just check` exit 0, and a three-file
`jj st` with no component), and the `feed.freeze()` regression is PRESERVED because the guard at
`feed.svelte.ts:124` returns on a settled wall before any fetch.
