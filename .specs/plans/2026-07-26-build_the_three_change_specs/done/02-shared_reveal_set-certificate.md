# Done Certificate · Task 02: shared reveal set

**Task:** [02-shared_reveal_set.md](02-shared_reveal_set.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 02. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 02) is every obligation O1 to O5 below holding, O2b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** A brick uncovered on the wall stays uncovered wherever it is rendered next, and the
  demo wall carries a covered brick so the behaviour can be observed at all.
- **P2 · Obligations.** Done iff O1, O2, O2b and O3 to O5 all hold; O5 is the Reviewable item.
- **P3 · Invariants.** Must not break `GlazeCard.svelte:69`'s unrelated `revealed` state (the touch
  pill toggle), the existing `Sensitive` cover markup, or `web/tests/service-worker-smoke.test.ts`,
  which drives the same demo wall whose fixtures change here.

## Obligations

- **O1 · The set is session-only and never persisted.**
  - *Claim:* `web/src/lib/state/sensitive.svelte.ts` exports a `SvelteSet<string>` and contains no
    storage of any kind.
  - *Evidence to collect:* read the module; confirm the import is from `svelte/reactivity`. Run
    `grep -nE 'localStorage|sessionStorage|indexedDB' web/src/lib/state/sensitive.svelte.ts`, expect
    no hits.
  - *Checks:* resolve `SvelteSet` to the `svelte/reactivity` export, not a local alias for `Set`; a
    plain `Set` would compile and would not be reactive.
  - *Collected:* `web/src/lib/state/sensitive.svelte.ts` is 11 lines: a doc comment and
    `export const revealed = new SvelteSet<string>();`, imported at `:1` from `"svelte/reactivity"`.
    `grep -nE 'localStorage|sessionStorage|indexedDB'` over it returns no hits (the single
    `storage` match is the words "never storage" in the comment). `SvelteSet` resolves by step 4,
    imported, to `svelte/reactivity`'s export (`web/node_modules/svelte/src/reactivity/set.js`), not
    to a local alias; no `Set` is declared anywhere in the module. `sensitive.test.ts` pins it with
    `expect(revealed).toBeInstanceOf(SvelteSet)`, and that case passes.
    Measured too, rather than only read: in chromium against the built site, revealing the covered
    brick and then reloading `/?actor=demo` brings the reveal control back (1 control after the
    reload), so nothing survives the page.
  - *Status:* SATISFIED

- **O2 · `Sensitive` takes an id and all four call sites forward it.**
  - *Claim:* no local reveal `$state` remains in `Sensitive.svelte`; it covers when
    `blur && !revealed.has(id)`; `PostCard.svelte`, `VideoCard.svelte` and both `GlazeCard.svelte`
    branches pass `id={brick.id}`; `GlazeCard.svelte:69` is untouched.
  - *Evidence to collect:* read `Sensitive.svelte` and confirm the `let revealed = $state(false)`
    that was at `:17` is gone. Run
    `grep -rn '<Sensitive' web/src/lib/components/cards/` and confirm four hits, each carrying `id=`.
    Read `GlazeCard.svelte:69` and confirm its local `revealed` is unchanged and unshadowed.
  - *Checks:* resolve `revealed` inside `GlazeCard.svelte`'s template to the local at `:69`, not to
    the imported set; the two share a name and the wrong one is easy to delete.
  - *Collected:* `Sensitive.svelte` now has no `$state` at all: the props line at `:22` is
    `let { id, blur, children }: { id: string; blur?: Blur | undefined; children: Snippet }`, the
    set arrives by import at `:13`, and the cover test at `:26` is `{#if blur && !revealed.has(id)}`.
    `grep -rn '<Sensitive' web/src/lib/components/cards/` returns exactly four hits, all carrying
    `id={brick.id}`: `GlazeCard.svelte:138`, `GlazeCard.svelte:201`, `VideoCard.svelte:52`,
    `PostCard.svelte:19`. No other file in `web/src` or `web/tests` mounts the component.
  - *Checks:* `GlazeCard.svelte:69`'s `let revealed = $state(false)` is unchanged (the diff touches
    only `:138` and `:201` in that file), and `GlazeCard` never imports the shared set, so every
    `revealed` in its template (`:242`, `:252`, `:253`, `:254`, `:257`, `:262`) resolves by step 1/3
    to that local rune. No shadowing. Inside `Sensitive.svelte`, `revealed` resolves by step 4 to the
    imported set, there being no local of that name left.
  - *Status:* SATISFIED

- **O2b · The reveal button stops propagating.**
  - *Claim:* `Sensitive.svelte`'s show-anyway button calls `event.stopPropagation()` alongside adding
    the id to the set, with a comment naming why.
  - *Evidence to collect:* read the handler. Read `PostCard.svelte:17`-`:19` and
    `GlazeCard.svelte:195`-`:201` and confirm the containment the comment claims: on both, the button
    is a **descendant** of the anchor task 04 gives an `onclick`. Read `GlazeCard.svelte:138` and
    confirm the carousel branch is the exception, with `Sensitive` wrapping the anchors instead.
  - *Checks:* without the stop, a click on "show anyway" bubbles to the anchor, `reader.activate`
    returns true, `preventDefault` fires and the reader opens on a reveal. Nothing in `just check`
    can see it (tsc drops both files) and task 06's "still revealed when the reader opens on it"
    assertion is satisfied by the wrong behaviour too, so the check here is a read and task 06's
    dedicated case is the lane.
  - *Collected:* the handler at `Sensitive.svelte:40`-`:54` calls `event.preventDefault()`, then
    `event.stopPropagation()`, then `revealed.add(id)`, under a ten-line comment naming what each
    stop answers. The containment reads as claimed: `PostCard.svelte:17` passes `href` to
    `BrickShell`, whose `:32` anchor wraps the whole card, so `Sensitive` at `:19` is inside it;
    `GlazeCard.svelte:195` opens the single/grid anchor and `Sensitive` at `:201` sits inside it;
    the carousel branch is the exception, `Sensitive` at `:138` wrapping the per-image anchors.
  - *Checks:* better than a read was available, so it was taken. Driven in chromium against the
    `just build` output, with a stand-in delegated `click` installed on the card anchor under
    Svelte's own `Symbol('events')` (exactly the shape task 04 will add): the reveal click reaches
    it **0** times, while a plain click on the brick reaches it **1** time, on both descendant call
    sites (`PostCard` on the bento wall, `GlazeCard`'s single branch on the glaze wall). The reveal
    also opens no tab: `context.pages()` stays at 1 and no `page` event fires in 5s, while the plain
    click opens `https://bsky.app/profile/fixture/post/0` in a second tab, so the zero is a measured
    zero. Two counterfactuals on the same build, patching the button's delegated handler at runtime:
    a no-op handler (the pre-diff shape's effect on the event) opens the tab, and a
    stopPropagation-only handler opens it too, so the navigation is pre-existing and
    `preventDefault()` is the half that removes it. `[role=dialog]` stays absent throughout.
  - *Status:* SATISFIED

- **O3 · The demo wall carries exactly one covered brick and the wire did not change.**
  - *Claim:* the fixture post at `i == 0` has a `blur`, no other brick does, `cargo nextest run` is
    green, and `contract.json` is untouched.
  - *Evidence to collect:* read `server/crates/mortar-core/src/fixtures.rs:186` and confirm the post
    arm's `blur` is a condition on `i` yielding `Some` only at `i == 0`. Run
    `cd server && cargo nextest run`, expect green. Confirm `git`/`jj` shows no change to
    `server/crates/mortar-core/tests/fixtures/contract.json`.
  - *Checks:* resolve how many bricks the edited expression builds. `:152`'s `_ =>` arm is shared by
    all 84 posts in the 120-brick pool, so a `Blur` written without a condition covers two thirds of
    the wall and the task's own "one covered brick" claim is false. Then resolve that the chosen
    index renders a reveal control at all: `PostCard.svelte:18` mounts `<Sensitive>` only inside
    `{#if img}`, and `:153` gives images only to `i.is_multiple_of(3)`, so a blurred post at an
    index without an image is invisible to task 06. Then resolve where `contract.json` comes from:
    `tests/contract.rs` builds its own canonical instances and imports nothing from `fixtures.rs`,
    so the fixture change must not appear in it.
  - *Collected:* `fixtures.rs:193` reads `blur: (i == 0).then(|| Blur { label: "!warn".into() })`,
    inside the shared `_ =>` post arm that opens at `:152`. `bool::then` (step 5, builtin) yields
    `Some` only at `i == 0`, so 1 of the 84 posts carries it and the video arm at `:148` and the
    blog arm stay `None`. `cd server && cargo nextest run`: 99 tests run, 99 passed, including the
    two new ones, `fixtures::tests::exactly_one_fixture_brick_arrives_covered` (the whole covered
    list equals `[("fixture-post-0", "!warn")]`) and `fixtures::tests::the_covered_brick_carries_an_image`.
    `jj diff --stat` lists 7 files and `tests/fixtures/contract.json` is not among them;
    `mortar-core::contract wire_contract_matches_the_committed_fixture` passes unchanged.
  - *Checks:* on the live wall the count is 1, not 84: `/?actor=demo` lays 24 articles and carries
    exactly one "Show sensitive media" control, at article index 0, aria-label "post by Brick
    Layer", wrapping an `img` (`picsum.photos/seed/img0-0/800/500`) inside one `.blur-2xl` wrapper.
    `i == 0` satisfies `i.is_multiple_of(3)` at `:153`, so the post has an image and
    `PostCard.svelte:18`'s `{#if img}` mounts the control; it is first on the glaze wall too
    (measured: index 0 there as well).
  - *Status:* SATISFIED

- **O4 · Meets the repo definition of done.**
  - *Claim:* vitest covers the set, the gates are green, and the wasm was rebuilt because a Rust file
    changed.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/sensitive.test.ts`, then
    `just wasm`, then `just check` from the repo root. Expect all clean. `just lint` includes knip,
    which must see the new module as reachable through `Sensitive.svelte`.
  - *Collected:* `cd web && pnpm vitest run src/lib/state/sensitive.test.ts`: 1 file, 4 tests,
    all passed. `just build` (which runs `just wasm` first, and the log opens on
    `wasm-pack build crates/mortar-wasm`) rebuilt the engine, and the covered brick then appearing
    in the browser is proof the wasm carries the Rust change. `just check` from the repo root: exit
    0 end to end, guards then `fmt-check`, `guard-wasm`
    (`cargo check --target wasm32-unknown-unknown`), oxlint (4 pre-existing warnings in files this
    task does not touch), knip clean with the new module reachable, clippy `-D warnings` clean,
    99/99 nextest, `tsc --noEmit` clean, 25/25 vitest across 3 files. No changeset is expected here:
    the plan assigns this change spec's changeset to task 07.
  - *Status:* SATISFIED

- **O5 · Reviewable: a reveal survives a re-place.**
  - *Claim:* on the built demo wall one brick is covered, and revealing it once leaves it revealed
    after a layout switch re-places it.
  - *Evidence to collect:* run `just build`, serve `web/build`, open `/?actor=demo`, find the covered
    brick, press "show anyway", then switch layout with the picker and observe the same brick still
    uncovered.
  - *Collected:* exercised, not reasoned about. `just build`, then `web/build` served on :4181 and
    `/?actor=demo` driven in chromium (playwright-core, no network beyond localhost). One covered
    brick on the first screen. Switched Bento to Masonry with the picker: the first article is a
    fresh element (a `data-gate-tag` written before the switch is gone), and the brick is still
    covered, 1 control. Pressed "show anyway": the media uncovers, the control disappears and no
    `.blur-2xl` wrapper is left. Switched back to Bento: the first article is a fresh element again,
    and the wall carries **0** reveal controls and 0 blurred wrappers across its 24 articles. The
    reveal survived the re-place. A reload then brings the cover back, which is the same evidence
    from the other side: it is the set, not the element, that remembers.
  - *Status:* SATISFIED

## Regression check

- `web/tests/service-worker-smoke.test.ts` drives `/?actor=demo` and counts laid bricks. Trace:
  after the fixture gains a `blur`, expect the same article count and no new console error :
  **PRESERVED**. `just test-e2e` ran green (1 passed, chromium, 672ms): the round trip through the
  wasm service worker still answers 200 with a non-empty `items`, and `#wall article` is visible.
  The gate's own drive of the same build counts 24 articles on the first screen with no console
  error.
- `server/crates/mortar-core/src/fixtures.rs` `pool()` is consumed by `feed.rs:190 demo_page`.
  Trace: `demo_page(0, Mode::Wall)` still returns `PAGE_SIZE` items : **PRESERVED**. `pool()` still
  builds 120 bricks (only the `blur` field of brick 0 changed), `demo_page` takes `PAGE_SIZE` = 24
  from it, and the wall shows 24. The glaze arm (`filter(Brick::is_image_post)`) is untouched by a
  `blur` and still puts brick 0 first, measured. `GlazeCard.svelte:69`'s touch-pill `revealed` is
  unchanged and unshadowed, and the existing `Sensitive` cover markup is byte-for-byte the same
  apart from the handler and the `{#if}` test.

## Residue

- `Sensitive.svelte`'s own body has no automated lane in this task: tsc drops it and both vitest
  suites are `.ts`. Task 06 is what observes it. Not an obligation here.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1, O2, O2b, O3, O4 and O5 are all SATISFIED with collected evidence rather than reading
alone (99/99 nextest, 4/4 and 25/25 vitest, `just check` and `just test-e2e` green, and the built
demo wall driven in chromium: one covered brick, a reveal that opens no tab and reaches no
delegated handler on the anchor it sits inside, and a cover that stays off across a layout
re-place), and both named regression traces are PRESERVED.

> Note for the reader of this certificate: the residue below is narrower than it was. The
> `Sensitive.svelte` body still has no *automated* lane, and task 06 is still the task that pins
> this behaviour in CI. But the body was exercised here by hand through a browser, including the
> propagation and default-navigation halves of the show-anyway button, so the gap is coverage that
> does not yet run on every push, not behaviour nobody has seen work.
