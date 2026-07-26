# Task 04 · card click activation

**Plan:** [plan.md](../plan.md) · **Certificate:** [04-card_click_activation-certificate.md](04-card_click_activation-certificate.md)

**Implements:** [`changes/2026-07-26-read_a_brick_in_place.md`](../../../changes/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `08-wall-and-bricks.md` → Cards (the Outbound links bullet); implementation note 6. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md) §Cards.
**Depends on:** 01, 03
**Produces:** a plain unmodified left click opens the reader on all four card kinds, and every modified click still reaches the source in a new tab.
**Pointers:** `BrickShell.svelte:6` (`href?: string`), `:31` (`{#if href}`), `:32` (the `<a>`). `PostCard.svelte:17` and `BlogCard.svelte:13` pass an href. **`VideoCard.svelte:51` and `GlazeCard.svelte:132` pass BrickShell no href at all**, so those two cards need their own activation points: `VideoCard.svelte:147` (the watch-at-source anchor), `GlazeCard.svelte:147` and `:195` (the two image anchors, one per branch). `GlazeCard.svelte:170`, `:178`, `:250` and `:273` are siblings of the anchors, not children. **`Sensitive`'s show-anyway button is not a sibling, it is a descendant**, on two of the four cards: `PostCard.svelte:17`'s `<BrickShell href=...>` wraps `<Sensitive>` at `:19`, and the single/grid anchor at `GlazeCard.svelte:195` wraps `<Sensitive>` at `:201`, so `Sensitive.svelte:32`'s button is inside the anchor this task intercepts. Only the carousel branch has it outside (`Sensitive` at `:138` wraps the anchors). Task 02 gives that button a `stopPropagation`; this task's job is to not undo it and to check the containment while reading.

## Steps

- [ ] Give `BrickShell` an optional `brick?: Brick` prop and add an `onclick` to its existing `<a>` that calls `reader.activate(event, brick)`.
- [ ] Pass `brick` from `PostCard.svelte:17` and `BlogCard.svelte:13`.
- [ ] Add the same single `reader.activate(event, brick)` call to the watch-at-source anchor in `VideoCard.svelte:147`.
- [ ] Add it to both image anchors in `GlazeCard.svelte` (`:147` and `:195`).
- [ ] Verify by reading that no anchor lost its `href`, `target="_blank"` or `rel="noopener noreferrer"`, that the four `clientUrl` call sites, one per card type except the blog (`PostCard.svelte:17`, `GlazeCard.svelte:148` and `:196`, `VideoCard.svelte:148`), still wrap their urls, and that no card gained a wrapping button. `BlogCard.svelte:13` passes the raw `brick.url` and always has: `clientUrl` rewrites `bsky.app` hostnames only, so a blog anchor has no rewrite to lose.
- [ ] Verify by reading that `Sensitive`'s show-anyway button still calls `event.stopPropagation()` (task 02 added it). It is a descendant of the intercepted anchor on `PostCard` and on `GlazeCard`'s single/grid branch, so without the stop this task turns a reveal into a reader open. Nothing in `just check` can see it: tsc drops both files.

## Definition of done

- [ ] Every interception is a single call to `reader.activate(event, brick)`, so the modifier-key rule exists in exactly one place and stays vitest-covered by task 01.
- [ ] An unmodified left click on a post card and on a blog card opens the reader and performs no navigation.
- [ ] A video brick opens in the reader from its watch link, and its play button still mounts the inline player without opening the reader; a glaze brick opens from an image, and the filmstrip arrows, the ALT panel and the touch reveal pill still do their own jobs. Those are all **siblings** of the anchors. The one **descendant** is `Sensitive`'s show-anyway button, on `PostCard` and on `GlazeCard`'s single/grid branch, and it stays a reveal only because task 02 stopped its propagation; task 06 asserts that a click on it leaves `[role=dialog]` absent.
- [ ] cmd-click, ctrl-click, shift-click, alt-click and middle-click still reach the source in a new tab from every intercepted anchor.
- [ ] Meets the repo definition of done (`just check` green; the wiring itself is visible only to Playwright, and the PR says so rather than citing a green typecheck).
- [ ] Reviewable: on `/?actor=demo`, left-click one card of each kind and confirm the reader opens with no navigation, then cmd-click each and confirm a new tab reaches the source. Task 06 automates the post-card path.
