# Done Certificate · Task 01: reader rune and page state

**Task:** [01-reader_rune_and_page_state.md](01-reader_rune_and_page_state.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · `activate` rejects every modified click and accepts only a plain left click.**
  - *Claim:* `activate` returns false and calls neither `preventDefault` nor `open` when any of
    `metaKey`, `ctrlKey`, `shiftKey`, `altKey` is set or `button !== 0`; otherwise it returns true
    and calls `preventDefault` exactly once.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/reader.test.ts`; confirm
    there is one case per modifier and one for `button !== 0`, each asserting the return value AND
    that `preventDefault` was not called. Read `reader.svelte.ts`'s `activate` body.
  - *Checks:* resolve `open` inside `activate` to the class method, not a same-named import; resolve
    `pushState` to the `$app/navigation` import, not `history.pushState`.
  - *Status:* unverified

- **O3 · Stepping is id-derived, clamped, and never paginates.**
  - *Claim:* `index` is derived by `feed.items.findIndex(b => b.id === brick.id)` rather than stored;
    `next()` and `prev()` clamp at 0 and `items.length - 1`; neither calls `feed.loadMore()` nor
    `fetchFeed`; a brick absent from `feed.items` yields `index === -1` with `canNext` and `canPrev`
    both false.
  - *Evidence to collect:* grep `reader.svelte.ts` for `loadMore` and `fetchFeed`, expect no hits.
    Run the vitest cases for clamping at both ends and for the absent-brick case.
  - *Checks:* resolve `feed` in `reader.svelte.ts` to the `feed` singleton exported from
    `$lib/state/feed.svelte`, not a local.
  - *Status:* unverified

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
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* tests at the right tier, negative-space cases for every rejected activation, named
    constants for any new bound, gates green, no em dash.
  - *Evidence to collect:* run `just check` from the repo root, expect clean. This runs
    `guard-dashes`, `guard-autoplay`, `guard-toolchain`, `fmt-check`, `guard-wasm`, `lint` and
    `test` (cargo nextest plus `pnpm check:ci` plus vitest). No `just wasm` is needed: no Rust
    changed.
  - *Status:* unverified

- **O6 · Reviewable: the module is the whole of the reader's decisions and no `.svelte` file moved.**
  - *Claim:* a reviewer runs the vitest file, reads the module, and finds every reader decision in
    it, with no component touched.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/reader.test.ts`, then list
    the diff's changed files and confirm none has a `.svelte` extension (`.svelte.ts` excepted).
  - *Status:* unverified

## Regression check

- `web/src/lib/state/feed.svelte.ts:121` `freeze()` is now called by `reader.open` on a possibly
  settled wall. Trace: with `warming === false`, expect the guard at `:124` to return before the
  fetch : (PRESERVED / REGRESSION)
- No other existing caller is in scope: this task adds a module and uncomments a type.

## Residue

- The close-on-reload behaviour (page state does not survive a reload, so a reload lands closed) is
  intended and stated in the task's steps, but it is not a DoD item and therefore not an obligation.
  Note it if the validator sees it broken.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
