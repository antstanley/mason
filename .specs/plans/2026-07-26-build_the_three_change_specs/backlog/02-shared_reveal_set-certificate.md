# Done Certificate · Task 02: shared reveal set

**Task:** [02-shared_reveal_set.md](02-shared_reveal_set.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O4 · Meets the repo definition of done.**
  - *Claim:* vitest covers the set, the gates are green, and the wasm was rebuilt because a Rust file
    changed.
  - *Evidence to collect:* run `cd web && pnpm vitest run src/lib/state/sensitive.test.ts`, then
    `just wasm`, then `just check` from the repo root. Expect all clean. `just lint` includes knip,
    which must see the new module as reachable through `Sensitive.svelte`.
  - *Status:* unverified

- **O5 · Reviewable: a reveal survives a re-place.**
  - *Claim:* on the built demo wall one brick is covered, and revealing it once leaves it revealed
    after a layout switch re-places it.
  - *Evidence to collect:* run `just build`, serve `web/build`, open `/?actor=demo`, find the covered
    brick, press "show anyway", then switch layout with the picker and observe the same brick still
    uncovered.
  - *Status:* unverified

## Regression check

- `web/tests/service-worker-smoke.test.ts` drives `/?actor=demo` and counts laid bricks. Trace:
  after the fixture gains a `blur`, expect the same article count and no new console error :
  (PRESERVED / REGRESSION)
- `server/crates/mortar-core/src/fixtures.rs` `pool()` is consumed by `feed.rs:190 demo_page`.
  Trace: `demo_page(0, Mode::Wall)` still returns `PAGE_SIZE` items : (PRESERVED / REGRESSION)

## Residue

- `Sensitive.svelte`'s own body has no automated lane in this task: tsc drops it and both vitest
  suites are `.ts`. Task 06 is what observes it. Not an obligation here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
