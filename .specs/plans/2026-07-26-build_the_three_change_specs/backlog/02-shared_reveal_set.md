# Task 02 · shared reveal set

**Plan:** [plan.md](../plan.md) · **Certificate:** [02-shared_reveal_set-certificate.md](02-shared_reveal_set-certificate.md)

**Implements:** [`changes/2026-07-26-read_a_brick_in_place.md`](../../../changes/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `08-wall-and-bricks.md` → Sensitive media; implementation notes 2, 3 and 4. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md) §Sensitive media and [`07-web-client.md`](../../../07-web-client.md) §Reactive state.
**Depends on:** none
**Produces:** a brick uncovered on the wall stays uncovered wherever it is rendered next, and the demo wall carries a covered brick so the behaviour can be observed at all.
**Pointers:** `web/src/lib/components/Sensitive.svelte:15` (props), `:17` (`let revealed = $state(false)`, at 17 not 19), `:32` (the show-anyway `<button>` opens there; `:34` is its `onclick`). Four call sites, not three: `cards/PostCard.svelte:19`, `cards/VideoCard.svelte:52`, `cards/GlazeCard.svelte:138` (carousel branch) and `cards/GlazeCard.svelte:201` (single/grid branch). `GlazeCard.svelte:69` has an unrelated `let revealed = $state(false)` for the touch pill; it is not the one being removed. `server/crates/mortar-core/src/fixtures.rs:148` is the **video** arm's `blur: None` and `:186` the **post** arm's; the post one is the one this task changes.

## Steps

- [ ] Add `web/src/lib/state/sensitive.svelte.ts`: `export const revealed = new SvelteSet<string>()` from `svelte/reactivity`, and nothing else.
- [ ] Give `Sensitive.svelte` a required `id: string` prop, delete its local `revealed` state, and cover when `blur && !revealed.has(id)`. The show-anyway button adds the id to the set.
- [ ] Have that same button call `event.stopPropagation()`, with a comment saying why. On two of the four call sites the button is a **descendant of the anchor task 04 intercepts**: `PostCard.svelte:17`'s `<BrickShell href=...>` wraps `<Sensitive>` at `:19`, and `GlazeCard.svelte`'s single/grid anchor opens at `:195` with `<Sensitive>` at `:201`. Only the carousel branch differs, where `Sensitive` at `:138` wraps the anchors instead. Without the stop, a click on "show anyway" bubbles to the anchor, `reader.activate` returns true, `preventDefault` fires and the reader opens on a reveal. Doing it here rather than in task 04 keeps the two tasks independent: this file is already open, and the reveal is correct on its own terms whether or not the reader exists.
- [ ] Forward `id={brick.id}` at all four call sites. Leave `GlazeCard.svelte:69` alone and do not import the set into `GlazeCard`.
- [ ] Give the demo fixture post at **`i == 0`** a `blur: Some(Blur { label: "!warn".into() })` at `server/crates/mortar-core/src/fixtures.rs:186`, so the reveal path exists on the offline wall. It is a condition on `i`, not a new literal: `:152`'s `_ =>` arm builds all 84 posts from one expression, so an unconditional `Blur` would cover two thirds of the wall. `i == 0` is the index that works on every axis: `0 % 20` matches neither the blog arm at `:96` nor the video arm at `:118`, so it is a post; `:153`'s `i.is_multiple_of(3)` gives it an image, which matters because `PostCard.svelte:18` renders `<Sensitive>` only inside `{#if img}`, so a post with no image carries no reveal control at all; and it is the first brick on both the full wall and the glaze wall, so task 06 reaches it without scrolling.
- [ ] Add `web/src/lib/state/sensitive.test.ts` covering the empty start, idempotent add, and per-id membership.

## Definition of done

- [ ] `revealed` is a `SvelteSet<string>` and the module contains no `localStorage`, no `sessionStorage` and no persistence of any kind, proven by grep.
- [ ] No local reveal `$state` remains in `Sensitive.svelte`, and all four call sites pass `id={brick.id}`; `GlazeCard.svelte:69` is untouched and unshadowed.
- [ ] The show-anyway button stops propagation, so it stays a reveal rather than becoming a way into the reader once task 04 intercepts the anchors it sits inside. Task 06 asserts it: a click on "show anyway" reveals the media and leaves `[role=dialog]` absent.
- [ ] The demo wall renders **exactly one** covered brick, the post at `i == 0`, with a working show-anyway control, and `cargo nextest run` is still green after the fixture change. The blur is behind a condition on `i`, so a second brick carrying one means the condition was dropped from the shared post arm. `contract.json` is **not** affected: it is built from its own canonical instances in `tests/contract.rs`, which imports nothing from `fixtures.rs`.
- [ ] Meets the repo definition of done (vitest covers the set, `just check` green including knip seeing the new module as reachable through `Sensitive.svelte`, `just wasm` run because a Rust file changed).
- [ ] Reviewable: run `just build` and open `/?actor=demo`; one brick is covered, revealing it once leaves it revealed after a layout switch re-places it.

## Open questions

- The `Sensitive.svelte` body itself has no lane: tsc drops it and both vitest suites are `.ts`. The fixture blur is what makes it observable, and task 06 is what observes it.
