# Done Certificate · Task 16: feed identity and the fourth error

**Task:** [16-feed_identity_and_fourth_error.md](16-feed_identity_and_fourth_error.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27

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
  - *Status:* **SATISFIED.** `grep -rn "public.api.bsky.app" web/src` returns exactly one line,
    `web/src/lib/appview.ts:14`. A wider `grep -rn "bsky\.app" web/src` finds no second host
    constant: the other hits are the client-rewrite table in `state/client.svelte.ts`, the icon map,
    a doc comment and two test fixtures. `profile.svelte.ts:2` now reads
    `import { APPVIEW } from "$lib/appview";` and declares nothing; the old line 8 is gone from the
    file. Resolution for the two readers that exist today: `APPVIEW` in `profile.svelte.ts:35` and
    in `feedinfo.svelte.ts:63` is step 4 (imported) in both, traced to `lib/appview.ts:14`, with no
    local, class-level or module-level redeclaration in either file to shadow it; `feedinfo.test.ts`
    imports the same symbol rather than respelling the host, so the grep proof survives the test
    file too. The third reader is task 17's picker and does not exist yet. The engine's own
    `appview_base` in `server/crates/mortar-core/src/config.rs` is untouched and `appview.ts`'s
    header comment says why the two stay apart. Confirmed live, not only by grep: the built site's
    header fetched `https://public.api.bsky.app/xrpc/app.bsky.feed.getFeedGenerator` and rendered
    Discover's real avatar, so the hoisted constant is the working host.

- **O2 · The new error code is classified and typechecked.**
  - *Claim:* `#fail` maps `feed_not_found` to `"feed-not-found"` with a `satisfies MortarErrorCode`
    comparison, and `feed.test.ts:218`'s `it.each` table gains the row.
  - *Evidence to collect:* read `feed.svelte.ts`'s `#fail` and confirm the `satisfies` form. Run
    `cd web && pnpm test` and confirm the new table row passes.
  - *Checks:* a plain string comparison would compile and would not fail when mortar renames the
    code. Confirm the literal carries `satisfies MortarErrorCode`, matching the two existing uses at
    `:200` and `:204`.
  - *Status:* **SATISFIED.** `feed.svelte.ts:234` reads
    `e.code === ("feed_not_found" satisfies MortarErrorCode)`, the same shape as the two branches
    above it, inserted after `actor_not_found` and before the `else`, so the two older codes keep
    their order and their branches. `cd web && pnpm test` is 6 files, 70 tests, all pass, and the
    verbose run names the new row: `maps a FeedError feed_not_found wall to the feed-not-found
    token`. The `satisfies` is not decorative, and this was mutation-tested rather than read: a
    scratch edit of that one literal to `"feed_gone"` made `pnpm exec tsc --noEmit -p tsconfig.json`
    fail with `src/lib/state/feed.svelte.ts(234,66): error TS1360: Type '"feed_gone"' does not
    satisfy the expected type 'MortarErrorCode'.`, and the file was restored byte for byte (md5
    `20de2ba7f4b59f04a24e569ac2f45b8a` before and after, `jj st` unchanged). `MortarErrorCode` in
    `lib/types.ts:151` carries `feed_not_found`, and `contract-check.ts:79` pins that union to the
    committed fixture, so a rename in `mortar-core/src/error.rs:55` lands here as a type error.

- **O3 · `feedinfo` never blocks and degrades to the rkey.**
  - *Claim:* `feedinfo.svelte.ts` reads `app.bsky.feed.getFeedGenerator` behind the `browser` guard,
    never blocks the wall, and on a miss leaves the header showing the feed's rkey; a vitest case
    covers the miss.
  - *Evidence to collect:* read the module and confirm the `browser` guard, following
    `profile.svelte.ts:26`. Run the named miss case in `feedinfo.test.ts`.
  - *Status:* **SATISFIED.** `feedinfo.svelte.ts:61` is `if (!browser || !feed) return;`, the same
    guard in the same position as `profile.svelte.ts:26`, with `browser` imported from
    `$app/environment` (step 4) in both. The name is set at `:60`, one line before the guard, so
    the rkey is in place in the no-browser build and on the first frame of every feed wall. Nothing
    is awaited: the request is a bare `void fetch(...)` with two `.then`s and a `.catch`, so the
    wall behind it is never gated on it, and `#fail`, `warming` and pagination never see it.
    `pnpm vitest run src/lib/state/feedinfo.test.ts` is 9 cases, all pass, including the two miss
    cases named in the module's contract: `leaves the rkey and no face when the feed is unknown to
    the AppView` (res.ok false) and `leaves the rkey when the AppView is unreachable` (rejected
    fetch), plus `drops an answer that lands after the reader moved to another feed`. Confirmed in
    chromium against the built site and the live AppView: on
    `?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/definitely-no-such-feed-xyzzy`
    the AppView misses and the header still names the feed by its rkey
    (`definitely-no-such-feed-xyzzy`, initial `D`, no avatar), with no page errors; and on
    `?feed=nonsense`, a reference with no slash at all, `feedRkey` keeps the whole string rather
    than naming the feed nothing.

- **O4 · The existing graph-wall surfaces are unchanged, and the coverage gap is stated.**
  - *Claim:* the three existing `FeedGrid` error panels and both handle-box recoveries are unchanged
    on a graph wall; and the PR states that `SwitchWall` and `FeedGrid`'s new panel ship with no
    automated coverage at all.
  - *Evidence to collect:* read the diff of `FeedGrid.svelte` and confirm the three existing panels'
    bodies are untouched. Read the PR body for the statement. Record why: no offline e2e case can
    reach a live `getFeedGenerator` or a `getFeed` 404, and neither vitest suite imports a component.
  - *Status:* **SATISFIED.** The diff adds branches and rewraps; it rewrites nothing. The
    `handle-not-found`, `login-required` and generic headings and copy are the same strings, the
    emoji expression only gains a `noFeed ? '🧱🔎' :` arm inside the existing `sealed ?` ternary, the
    handle form's guard `{#if notFound || sealed}` is byte-identical, and the try-again guard moves
    from `{:else}` to `{:else if !noFeed}`, which is the same branch for every error a graph wall can
    reach (`noFeed` is false there). The `retryValue` effect at `FeedGrid.svelte:51-64`, which is
    both handle-box recoveries (typo keeps its text, sealed clears it), is not in the diff at all.
    Driven in chromium on the built site: `/?actor=demo` lays 24 bricks with the switcher reading
    `@demo`, its picsum avatar and the old aria-label verbatim; a bad handle
    (`/?actor=no-such-waller-xyzzy.bsky.social`) still renders `no wall for that handle` with the
    unchanged copy, exactly 1 `#retry-handle` box, its retry button and the demo link. The sealed
    panel needs a `!no-unauthenticated` account and was not driven; its branch conditions and copy
    are unchanged, so it is preserved by inspection rather than by a drive. The coverage statement
    is in the drafted commit message, under its own paragraph and not softened: "the `.svelte` half
    of this change (`SwitchWall`, `FeedGrid`) ships with NO automated coverage at all. no offline
    e2e case can reach a live `getFeedGenerator` or a `getFeed` 404, and neither vitest suite
    imports a component, so nothing in CI renders either of them", closing with "that is a hand
    drive and it is not coverage". No fake coverage was added to buy it: `web/tests/` is not in the
    diff, so no stubbed e2e case was written and presented as the header's lane.

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green and the e2e lane is green as a regression.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci`, `just test-e2e` and
    `just check`.
  - *Status:* **SATISFIED**, with one pre-existing red recorded rather than hidden.
    `cd web && pnpm test` is 70/70 (9 new `feedinfo` cases plus the new mapping row).
    `cd web && pnpm check:ci` is clean over both tsc projects. `CI=1 just test-e2e` is 7/7 on
    Playwright's own server: `CI=1` is load-bearing here, because several workspaces of this repo
    are being built on this machine and a bare run attaches to whichever one holds 4173. `just check`
    fails, at its first recipe, `guard-dashes`, and not on this diff: the em dash is in
    `.specs/plans/2026-07-26-build_the_three_change_specs/done/22-refresh_entry_point_and_fronts-certificate.md`,
    which `jj file show -r @-` proves carries five of them in the parent commit `f4543b30`, and
    which is absent from `jj diff --name-only` here. Every other recipe in the gate was run directly
    over this tree, `just guard-autoplay guard-toolchain fmt-check guard-wasm lint test`, and exits
    0: oxfmt and cargo fmt clean, oxlint clean, knip clean, clippy clean, 154 rust tests, 70 vitest
    tests. A changeset is present for the user-visible change. The dash is a repo-level defect that
    somebody who may touch `.specs/plans/` must fix; it is not this task's to answer.

- **O6 · Reviewable: the `.ts` half is covered and the rest is exercised by hand.**
  - *Claim:* `cd web && pnpm test` covers `appview`, `feedinfo` and `#fail`; the header and the new
    panel are read in the diff and exercised against a real feed uri.
  - *Evidence to collect:* the vitest run, plus a manual pass: open the built site at a real
    `?feed=` uri and confirm the generator's name and avatar appear in `SwitchWall`, then at a
    well-formed but nonexistent uri and confirm the "no such feed" panel.
  - *Status:* **SATISFIED.** The vitest half: 9 `feedinfo` cases (the rkey fallback in both
    reference spellings, the happy read, the percent-encoding of a reference full of colons and
    slashes into the query, a generator with no display name, the once-per-reference guard, the
    miss, the unreachable AppView, the late answer), the `feed_not_found` row in `feed.test.ts`, and
    `appview` exercised through `feedinfo.test.ts`, which imports `APPVIEW` and asserts the whole
    request URL built from it. The hand pass, in chromium against `just build`'s output on port 4322
    (my own port, never 4173) and the live AppView:
    `/?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot` renders the
    button text `Discover`, one `img` whose src is the generator's real `cdn.bsky.app` avatar, and
    the aria-label `Switch wall, currently viewing the Discover feed by @bsky.app`, over 20 bricks
    with no handle box and no page errors; the bogus rkey renders `no such feed` with the new copy,
    zero `#retry-handle` boxes, zero try-again buttons and the demo link. Task 15's open handover is
    closed by the same pass: `/?feed=nonsense` now reads
    `Switch wall, currently viewing the nonsense feed` instead of `Switch wall, currently viewing @`,
    and `/?actor=demo&feed=nonsense` names the feed rather than `@demo`, which is the wall that is
    actually laid.

## Regression check

- `state/profile.svelte.ts` after the base is hoisted. Trace: `/?actor=demo` still shows the wall
  owner's avatar in `SwitchWall` : **PRESERVED.** Driven: the button carries
  `https://picsum.photos/seed/masondemo/96/96`, reads `@demo`, keeps the old aria-label string and
  the wall lays 24 bricks. `profile.load` is still called on a graph wall (`SwitchWall.svelte:34`,
  with `actor ?? ''` for the null the widened prop now allows), and the only behaviour change in
  `profile.svelte.ts` is where the host string is defined.
- `feed.svelte.ts:200` and `:204`: `login_required` and `actor_not_found` still classify to
  `login-required` and `handle-not-found` : **PRESERVED.** Both rows of the `it.each` table pass
  unchanged, and the live bad-handle drive still lands on `no wall for that handle` with its box.
- `FeedGrid.svelte`'s empty state on a graph wall: still reads as before : **PRESERVED** by
  derivation and by reading, not by a drive. `emptyWall` is
  `currentFeed ? 'this feed has no bricks yet' : 'this wall has no bricks yet'`, and `currentFeed`
  is the URL's `feed` parameter (not the `feed` state singleton this file also imports), so a graph
  wall gets the identical old literal in both the heading and the live region; the copy paragraph
  keeps the old sentence verbatim in its `{:else}`. No lane in this repo can produce a graph wall
  with zero bricks and `done`, so no drive reached it.

## Residue

- The change spec's open question about a feed wall wanting a header line naming what the reader is
  reading (beyond avatar and name) is not answered here and is not an obligation.
- The change spec's error row asks for "no such feed", **with a way into the feed picker**. There is
  no picker until tasks 17 and 18, so this panel ships with the switcher above it and the demo link
  below as its way out, and a comment in `FeedGrid.svelte` saying so. Task 18 should close that
  clause.
- Pre-existing and untouched: on any error panel with zero bricks the sr-only live region still
  reads "laying bricks", because the `wallStatus` chain has no arm for `error && n === 0`. Observed
  on the untouched `handle-not-found` panel as well as on the new one, so the new panel inherits it
  rather than causing it.
- Cosmetic: `profile.svelte.ts`'s doc comment at `:4-8` used to sit above the constant that moved,
  and now floats above `NO_UNAUTHENTICATED`'s own doc comment, attached to nothing. Harmless, and
  outside every obligation.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED on evidence this gate collected itself, including a mutation
test that proves the `satisfies MortarErrorCode` bites, the nine `feedinfo` cases, `CI=1 just
test-e2e` at 7/7 on its own server, and a chromium drive of six URLs that shows the real generator's
name, face and creator in the header, the "no such feed" panel with no handle box, task 15's
`@`-only switcher closed on both its handover URLs, and the graph wall unchanged; the three named
regression traces are PRESERVED, and the one red in `just check` is `guard-dashes` firing on a
certificate from task 22 that this diff neither contains nor may touch.