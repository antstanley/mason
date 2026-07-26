# Task 02 · shared reveal set

**Plan:** [plan.md](../plan.md) · **Certificate:** [02-shared_reveal_set-certificate.md](02-shared_reveal_set-certificate.md)

**Implements:** [`changes/2026-07-26-read_a_brick_in_place.md`](../../../changes/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `08-wall-and-bricks.md` → Sensitive media; implementation notes 2 and 3. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md) §Sensitive media and [`07-web-client.md`](../../../07-web-client.md) §Reactive state.
**Depends on:** none
**Produces:** a brick uncovered on the wall stays uncovered wherever it is rendered next, and the demo wall carries a covered brick so the behaviour can be observed at all.
**Pointers:** `web/src/lib/components/Sensitive.svelte:15` (props), `:17` (`let revealed = $state(false)`, at 17 not 19), `:34` (the show-anyway button). Four call sites, not three: `cards/PostCard.svelte:19`, `cards/VideoCard.svelte:52`, `cards/GlazeCard.svelte:138` (carousel branch) and `cards/GlazeCard.svelte:201` (single/grid branch). `GlazeCard.svelte:69` has an unrelated `let revealed = $state(false)` for the touch pill; it is not the one being removed. `server/crates/mortar-core/src/fixtures.rs:148` and `:186` are both `blur: None` today.

## Steps

- [ ] Add `web/src/lib/state/sensitive.svelte.ts`: `export const revealed = new SvelteSet<string>()` from `svelte/reactivity`, and nothing else.
- [ ] Give `Sensitive.svelte` a required `id: string` prop, delete its local `revealed` state, and cover when `blur && !revealed.has(id)`. The show-anyway button adds the id to the set.
- [ ] Have that same button call `event.stopPropagation()`, with a comment saying why. On two of the four call sites the button is a **descendant of the anchor task 04 intercepts**: `PostCard.svelte:17`'s `<BrickShell href=...>` wraps `<Sensitive>` at `:19`, and `GlazeCard.svelte`'s single/grid anchor opens at `:195` with `<Sensitive>` at `:201`. Only the carousel branch differs, where `Sensitive` at `:138` wraps the anchors instead. Without the stop, a click on "show anyway" bubbles to the anchor, `reader.activate` returns true, `preventDefault` fires and the reader opens on a reveal. Doing it here rather than in task 04 keeps the two tasks independent: this file is already open, and the reveal is correct on its own terms whether or not the reader exists.
- [ ] Forward `id={brick.id}` at all four call sites. Leave `GlazeCard.svelte:69` alone and do not import the set into `GlazeCard`.
- [ ] Give one demo fixture post a `blur: Some(Blur { label: "!warn".into() })` in `server/crates/mortar-core/src/fixtures.rs`, so the reveal path exists on the offline wall.
- [ ] Add `web/src/lib/state/sensitive.test.ts` covering the empty start, idempotent add, and per-id membership.

## Definition of done

- [ ] `revealed` is a `SvelteSet<string>` and the module contains no `localStorage`, no `sessionStorage` and no persistence of any kind, proven by grep.
- [ ] No local reveal `$state` remains in `Sensitive.svelte`, and all four call sites pass `id={brick.id}`; `GlazeCard.svelte:69` is untouched and unshadowed.
- [ ] The show-anyway button stops propagation, so it stays a reveal rather than becoming a way into the reader once task 04 intercepts the anchors it sits inside. Task 06 asserts it: a click on "show anyway" reveals the media and leaves `[role=dialog]` absent.
- [ ] The demo wall renders one covered brick with a working show-anyway control, and `cargo nextest run` is still green after the fixture change. `contract.json` is **not** affected: it is built from its own canonical instances in `tests/contract.rs`, which imports nothing from `fixtures.rs`.
- [ ] Meets the repo definition of done (vitest covers the set, `just check` green including knip seeing the new module as reachable through `Sensitive.svelte`, `just wasm` run because a Rust file changed).
- [ ] Reviewable: run `just build` and open `/?actor=demo`; one brick is covered, revealing it once leaves it revealed after a layout switch re-places it.

## Open questions

- The `Sensitive.svelte` body itself has no lane: tsc drops it and both vitest suites are `.ts`. The fixture blur is what makes it observable, and task 06 is what observes it.
