# Task 03 · brick reader dialog shell

**Plan:** [plan.md](../plan.md) · **Certificate:** [03-brick_reader_dialog_shell-certificate.md](03-brick_reader_dialog_shell-certificate.md)

**Implements:** [`changes/2026-07-26-read_a_brick_in_place.md`](../../../changes/2026-07-26-read_a_brick_in_place.md) §Proposed changes → `08-wall-and-bricks.md` → The brick reader → Dialog behaviour, and → `09-design-system.md` → Motion; implementation notes 7 and 8. Targets [`08-wall-and-bricks.md`](../../../08-wall-and-bricks.md) §Accessibility behaviours and [`09-design-system.md`](../../../09-design-system.md) §Motion.
**Depends on:** 01
**Produces:** a modal dialog that opens over the demo wall, holds focus, locks the page scroll and closes four ways, with the wall behind it inert.
**Pointers:** `web/src/routes/+layout.svelte:110` (the wrapper div opens), `:133` (`{@render children()}`, **inside** the wrapper), `:134` (the wrapper closes; the mount goes after this). `web/src/app.css:53` (`--animate-brick-in`), `:55` (its keyframes), `:99` (the `prefers-reduced-motion` override). `SwitchWall.svelte` is the existing dialog pattern to mirror. `web/knip.json` treats `src/routes/**/+*.{svelte,ts}` as entries, so the component and its mount must land together.

## Steps

- [ ] Add `web/src/lib/components/BrickReader.svelte`: scrim, panel, close control, accessible name, focus in on open and back to the recorded opener on close, Escape, click-away. Render only the author line and the close control for now; the kind switch is task 05.
- [ ] Render nothing unless `page.state.brick` is set **and** `reader.brick.id` equals it, so a forward navigation that restores the state without the rune renders nothing rather than throwing.
- [ ] Mark the wrapper div at `+layout.svelte:110` `inert` while the reader is open, and mount `<BrickReader />` **after** the wrapper's closing tag at `:134`, not after `{@render children()}` at `:133`.
- [ ] Add `--animate-reader-in` beside `--animate-brick-in` in `app.css`, its keyframes, and its `prefers-reduced-motion: reduce` override beside the existing `.animate-brick-in` one.
- [ ] Set `document.documentElement.style.overflow = 'hidden'` on open and revert it on close, including on unmount.

## Definition of done

- [ ] The panel carries `role="dialog"`, `aria-modal="true"` and an accessible name taken from the brick (a blog's title, otherwise the author line).
- [ ] Escape, the close control, a click on the scrim and the browser back gesture all close it, and the address bar still reads `?actor=<handle>` throughout.
- [ ] The reader is **not** a descendant of the inert wrapper, verifiable by reading the markup: the mount sits after `</div>`.
- [ ] Motion matches the design row: scrim 200ms `linear` fade; panel `0.24s cubic-bezier(0.16, 1, 0.3, 1)` from `translateY(8px) scale(0.99)`; under `prefers-reduced-motion: reduce` the scrim fades in `0.15s linear` and the panel does not move.
- [ ] `just guard-autoplay` passes: the new file contains no `.play(` and not the word autoplay, not even in a comment (the grep is case-insensitive over all of `web/src`).
- [ ] Meets the repo definition of done (`just check` green, which here proves compilation, knip reachability from the `+layout.svelte` entry, and the two greps, and **nothing about the DOM behaviour above**).
- [ ] Reviewable: run `just dev`, open `/?actor=demo`, and drive the dialog by hand for focus, Escape, scrim, back and scroll lock. Task 06 is what makes those assertions automated; this task ships them unautomated, deliberately.
