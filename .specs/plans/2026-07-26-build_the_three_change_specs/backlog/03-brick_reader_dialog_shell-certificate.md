# Done Certificate · Task 03: brick reader dialog shell

**Task:** [03-brick_reader_dialog_shell.md](03-brick_reader_dialog_shell.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 03. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. Note that
> `just check` cannot see this task's deliverable at all; several obligations are read-and-drive
> rather than run-a-test, deliberately.

## Definition

DONE(Task 03) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** A modal dialog opens over the demo wall, holds focus, locks the page scroll and
  closes four ways, with the wall behind it inert.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break `+layout.svelte`'s existing header, its skip link, the
  deploy-reload effect at `:50`, or `FeedGrid`'s freeze listeners; and must not make the header
  unreachable when no reader is open.

## Obligations

- **O1 · The panel is a named modal dialog.**
  - *Claim:* the panel carries `role="dialog"`, `aria-modal="true"` and an accessible name taken
    from the brick (a blog's title, otherwise the author line), and the component renders nothing
    unless `page.state.brick` is set and `reader.brick.id` equals it.
  - *Evidence to collect:* read `web/src/lib/components/BrickReader.svelte`; confirm the three
    attributes and the two-condition guard. Drive `just dev`, open `/?actor=demo`, open a reader and
    inspect the element in devtools for a resolved accessible name.
  - *Checks:* resolve `page` to the `$app/state` import, not a prop or a local; a stale
    `$app/stores` import would compile and read a different value.
  - *Status:* unverified

- **O2 · Four close paths work and the URL never changes.**
  - *Claim:* Escape, the close control, a scrim click and the browser back gesture all close it, and
    the address bar reads `?actor=<handle>` throughout.
  - *Evidence to collect:* with `just dev` and `/?actor=demo`, open the reader and close it once by
    each of the four routes, reading the address bar before and after each.
  - *Status:* unverified

- **O3 · The reader is not a descendant of the inert wrapper.**
  - *Claim:* the wrapper div that opens at `+layout.svelte:110` carries `inert` while the reader is
    open, and `<BrickReader />` is mounted after that wrapper's closing tag, not after
    `{@render children()}`.
  - *Evidence to collect:* read `web/src/routes/+layout.svelte`; confirm the mount line sits after
    `</div>`. With the reader open in a browser, confirm the close control is focusable by Tab and
    the layout picker radio is not.
  - *Checks:* `inert` is inherited, so a mount inside the wrapper would make the reader itself inert.
    Confirm by DOM position, not by the visual result, which looks the same until focus is attempted.
  - *Status:* unverified

- **O4 · Motion matches the design row in both directions.**
  - *Claim:* scrim 200ms `linear` fade; panel `0.24s cubic-bezier(0.16, 1, 0.3, 1)` from
    `translateY(8px) scale(0.99)`; under `prefers-reduced-motion: reduce` the scrim fades in
    `0.15s linear` and the panel does not move.
  - *Evidence to collect:* read `web/src/app.css` around `:53`, `:55` and `:99`; confirm
    `--animate-reader-in`, its keyframes and its reduced-motion override sit beside the
    `--animate-brick-in` ones. Toggle the OS or devtools reduced-motion setting and open the reader
    under each.
  - *Status:* unverified

- **O5 · The autoplay guard still passes over the new file.**
  - *Claim:* `BrickReader.svelte` contains no `.play(` and not the word autoplay, not even in a
    comment.
  - *Evidence to collect:* run `just guard-autoplay`, expect clean. The grep is case-insensitive
    over all of `web/src`, so a comment saying the reader does not autoplay would fail it.
  - *Status:* unverified

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, which here proves compilation, knip reachability and the two greps
    and nothing more.
  - *Evidence to collect:* run `just check`. `just lint` runs knip, which must see
    `BrickReader.svelte` as reachable from the `+layout.svelte` entry declared in `web/knip.json`;
    a component landed without its mount would fail here.
  - *Status:* unverified

- **O7 · Reviewable: the dialog is driven by hand for focus, Escape, scrim, back and scroll lock.**
  - *Claim:* a reviewer runs `just dev`, opens `/?actor=demo`, and observes focus moving in on open
    and back to the opener on close, `document.documentElement.style.overflow === 'hidden'` while
    open and reverted on close, and all four close routes working.
  - *Evidence to collect:* the five observations above, made in a browser. Record that they are
    manual: task 06 is what makes them automated, and this task ships them unautomated on purpose.
  - *Status:* unverified

## Regression check

- `web/src/routes/+layout.svelte`'s header renders for every wall. Trace: with no reader open,
  Tab from the top of `/?actor=demo` and expect the skip link, then the layout picker, to receive
  focus as before : (PRESERVED / REGRESSION)
- `web/tests/service-worker-smoke.test.ts` drives the same layout. Trace: run `just test-e2e` and
  expect it green : (PRESERVED / REGRESSION)

## Residue

- The scroll-lock revert on unmount (as opposed to on close) is in the task's steps but is not its
  own DoD item. If the validator finds a route where the overflow style survives, note it.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
