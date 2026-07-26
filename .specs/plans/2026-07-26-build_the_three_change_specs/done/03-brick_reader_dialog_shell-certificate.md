# Done Certificate · Task 03: brick reader dialog shell

**Task:** [03-brick_reader_dialog_shell.md](03-brick_reader_dialog_shell.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 03. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. Note that
> `just check` cannot see this task's deliverable at all; several obligations are read-and-drive
> rather than run-a-test, deliberately.

> **Second pass.** The first pass ruled DONE and recorded one concern in the residue: the layout's
> `inert` and scroll lock hung on `page.state.brick !== undefined`, which is wider than the
> condition the component renders on. That concern was fixed rather than deferred, and every
> obligation below was re-discharged from scratch against the revised bytes by a second validator
> (not the implementer): `just check`, `just test-e2e`, the reader unit suite, and 77 fresh
> assertions driven in chromium. Line references and evidence are the second pass's.

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
  - *Collected:* `BrickReader.svelte:107-113` carries `role="dialog"`, `aria-modal="true"`,
    `aria-label={label}` and `tabindex="-1"`. `label` (`:41-45`) returns a blog's `title` and every
    other kind's `author.displayName ?? "@" + handle`. The two-condition guard now lives in the rune
    and is read once: `const brick = $derived(reader.showing)` (`:26`), where `showing`
    (`reader.svelte.ts:72-76`) is `id !== undefined && held?.id === id ? held : null`; all markup
    sits inside `{#if brick}` at `:89`. Driven in chromium against `pnpm dev` on `/?actor=demo`:
    the dialog resolved with `aria-label "Brick Layer"` and `ariaSnapshot()` rendered
    `- dialog "Brick Layer":`. Every kind on the demo wall was opened in turn: a post named
    `"Brick Layer"` (author line), a blog named `"Building a feed algorithm nobody hates"` (its
    `title`, not its author), a video named `"Brick Layer"` (author line again). Under a restored
    history entry with no brick held, no dialog rendered and no page error fired.
  - *Check resolved:* `page` is no longer read in the component at all. The one read is
    `reader.svelte.ts:73`, and `page` there is `import { page } from "$app/state"` (`:2`, step 4,
    imported), not `$app/stores`, not a local, not a class member; `grep -rn "page.state" web/src`
    returns exactly that one executable hit (everything else is prose). `reader` at
    `BrickReader.svelte:11` and `+layout.svelte:11` resolves to the singleton exported at
    `reader.svelte.ts:198`; no shadow in any changed file.
  - *Status:* SATISFIED

- **O2 · Four close paths work and the URL never changes.**
  - *Claim:* Escape, the close control, a scrim click and the browser back gesture all close it, and
    the address bar reads `?actor=<handle>` throughout.
  - *Evidence to collect:* with `just dev` and `/?actor=demo`, open the reader and close it once by
    each of the four routes, reading the address bar before and after each.
  - *Collected:* all four re-driven in chromium on `/?actor=demo`, each from a fresh open, each
    followed by a read of `location.href`. Escape (document keydown listener, `:57-59`, attached at
    `:78`): dialog gone, `href` still `/?actor=demo`. The close control (`:117-124`): gone, `href`
    unchanged. The scrim (`:93-99`), clicked both at viewport 8,8 and in the gutter beside the panel
    (the `pointer-events-none` overlay passes the click through): gone, `href` unchanged.
    `page.goBack()`: gone, `href` unchanged, `#wall` still standing after each. Focus returned to the
    recorded opener on all four, `document.documentElement.style.overflow` was back to `""` and the
    wrapper carried no `inert` attribute after all four.
  - *Status:* SATISFIED

- **O3 · The reader is not a descendant of the inert wrapper.**
  - *Claim:* the wrapper div that opens at `+layout.svelte:110` carries `inert` while the reader is
    open, and `<BrickReader />` is mounted after that wrapper's closing tag, not after
    `{@render children()}`.
  - *Evidence to collect:* read `web/src/routes/+layout.svelte`; confirm the mount line sits after
    `</div>`. With the reader open in a browser, confirm the close control is focusable by Tab and
    the layout picker radio is not.
  - *Checks:* `inert` is inherited, so a mount inside the wrapper would make the reader itself inert.
    Confirm by DOM position, not by the visual result, which looks the same until focus is attempted.
  - *Collected:* markup read. `{@render children()}` is at `+layout.svelte:148`, the wrapper closes
    at `:149`, and `<BrickReader />` is at `:154`, after that closing tag, under an explanatory
    comment. The wrapper carries `inert={overlayOpen}` at `:123`. Driven: with the reader open,
    `wrapper.hasAttribute("inert")` is true, exactly one element in the document is inert, and
    `wrapper.contains(dialog)` is **false**. A header anchor inside the wrapper refused `focus()`
    (activeElement stayed on the dialog); the dialog's own close control took focus on `focus()`;
    six consecutive `Tab` presses never put activeElement inside the wrapper. At rest the wrapper
    carries no `inert` attribute at all, and a header control both focuses and receives a real mouse
    click.
  - *Status:* SATISFIED

- **O4 · Motion matches the design row in both directions.**
  - *Claim:* scrim 200ms `linear` fade; panel `0.24s cubic-bezier(0.16, 1, 0.3, 1)` from
    `translateY(8px) scale(0.99)`; under `prefers-reduced-motion: reduce` the scrim fades in
    `0.15s linear` and the panel does not move.
  - *Evidence to collect:* read `web/src/app.css` around `:53`, `:55` and `:99`; confirm
    `--animate-reader-in`, its keyframes and its reduced-motion override sit beside the
    `--animate-brick-in` ones. Toggle the OS or devtools reduced-motion setting and open the reader
    under each.
  - *Collected:* `app.css:69` declares `--animate-reader-in: reader-in 0.24s cubic-bezier(0.16, 1,
    0.3, 1) both` and `:72` `--animate-scrim-in: brick-fade 200ms linear both`, both inside `@theme`
    beside `--animate-brick-in` at `:53`; the `reader-in` keyframes are at `:74-83`; the
    reduced-motion overrides are at `:124-129`, beside the `.animate-brick-in` one at `:119`.
    Re-measured in chromium: the panel computes `reader-in / 0.24s / cubic-bezier(0.16, 1, 0.3, 1) /
    both`, the scrim `brick-fade / 0.2s / linear / both`, and the `reader-in` keyframes read out of
    the CSSOM as `0% { opacity: 0; transform: translateY(8px) scale(0.99); }`. In a context launched
    with `reducedMotion: "reduce"` (media query confirmed matching) the panel computes
    `brick-fade / 0.15s / linear` with `transform: none`, sampled both at rest and 60ms into the
    animation, so the override replaces the transform rather than only the duration; the scrim
    computes `brick-fade / 0.15s / linear`. Both survive the production build:
    `web/build/_app/immutable/assets/0.DP1kBM6N.css` carries
    `@keyframes reader-in{0%{opacity:0;transform:translateY(8px)scale(.99)}...}` and, inside
    `@media (prefers-reduced-motion:reduce)`,
    `.animate-reader-in,.animate-scrim-in{animation:.15s linear both brick-fade}`.
  - *Status:* SATISFIED

- **O5 · The autoplay guard still passes over the new file.**
  - *Claim:* `BrickReader.svelte` contains no `.play(` and not the word autoplay, not even in a
    comment.
  - *Evidence to collect:* run `just guard-autoplay`, expect clean. The grep is case-insensitive
    over all of `web/src`, so a comment saying the reader does not autoplay would fail it.
  - *Collected:* `just guard-autoplay` re-run standalone in the workspace against the revised bytes,
    exit 0, and again inside `just check`. `just guard-dashes` also re-run standalone, exit 0, which
    matters because this revision added prose to three files. `BrickReader.svelte` mounts no player
    and touches no media element.
  - *Status:* SATISFIED

- **O6 · Meets the repo definition of done.**
  - *Claim:* the gates are green, which here proves compilation, knip reachability and the two greps
    and nothing more.
  - *Evidence to collect:* run `just check`. `just lint` runs knip, which must see
    `BrickReader.svelte` as reachable from the `+layout.svelte` entry declared in `web/knip.json`;
    a component landed without its mount would fail here.
  - *Collected:* `just check` re-run in the workspace on the revised bytes, exit 0: guard-dashes,
    guard-autoplay, guard-toolchain, `fmt-check`, guard-wasm, `lint` (oxlint reports only the four
    pre-existing warnings in `FeedGrid.svelte` and `service-worker.ts`, none in any changed file;
    knip clean, so `BrickReader.svelte` is reachable from the `+layout.svelte` entry; clippy clean)
    and `test` (114 nextest passed, both tsc projects clean, 41 vitest passed, up from 39 as the
    reader suite grew from 18 cases to 20). No changeset was added; the change is not user-visible on
    its own (nothing opens the reader until task 04 and the panel carries no body until task 05), and
    the change spec's implementation step 10 (`.specs/changes/2026-07-26-read_a_brick_in_place.md:369-370`)
    puts one `pnpm changeset` at the end of the whole change.
  - *Status:* SATISFIED

- **O7 · Reviewable: the dialog is driven by hand for focus, Escape, scrim, back and scroll lock.**
  - *Claim:* a reviewer runs `just dev`, opens `/?actor=demo`, and observes focus moving in on open
    and back to the opener on close, `document.documentElement.style.overflow === 'hidden'` while
    open and reverted on close, and all four close routes working.
  - *Evidence to collect:* the five observations above, made in a browser. Record that they are
    manual: task 06 is what makes them automated, and this task ships them unautomated on purpose.
  - *Collected:* driven by this validator in chromium (playwright-core, scratchpad harnesses written
    from scratch for this pass, never added to `web/tests`) against `pnpm dev` on `/?actor=demo`:
    77 assertions across seven probes. Focus lands on the panel on open
    (`activeElement === [role=dialog]`) and returns to the recorded opener after each of the four
    dismissals. `document.documentElement.style.overflow` is `"hidden"` while open and `""` after
    every close, including the back gesture, a client-side navigation out from under an open reader,
    and the state where no brick is held. With the reader open and no opener recorded (so no
    focus-driven scroll confounds it), a 900px wheel left `scrollY` at 600; once shut the same wheel
    moved it to 1300. All four close routes leave the address bar at `/?actor=demo`. The trap task
    01's certificate left was exercised three ways: open then back gesture then Escape stays on the
    wall; two Escapes with no wait between them stay on the wall; two `reader.close()` calls in one
    tick stay on the wall. Two probe assertions did not pass on first run and both were defects in
    the probe, not the code: a wrapper selector that matched SvelteKit's `display: contents` shell
    instead of the layout wrapper, and a count of `[inert]` elements on `/` that caught
    `LandingWall.svelte:38`'s own static `inert` on the decorative wall (pre-existing, `aria-hidden`,
    untouched by this diff). Both re-checked with corrected probes and passing. These observations
    are manual by design; task 06 is what makes them permanent, and `just check` can see none of them
    because tsc parses no `.svelte` file.
  - *Status:* SATISFIED

## Regression check

- `web/src/routes/+layout.svelte`'s header renders for every wall. Trace: with no reader open,
  Tab from the top of `/?actor=demo` and expect the skip link, then the layout picker, to receive
  focus as before : PRESERVED (with no reader up the wrapper carries no `inert` attribute,
  `a[href="#wall"]` "skip to the wall" takes focus, and a header control takes focus and receives a
  real mouse click; the wrapper keeps its `pb-24 md:pb-0` class)
- `web/tests/service-worker-smoke.test.ts` drives the same layout. Trace: run `just test-e2e` and
  expect it green : PRESERVED (`just test-e2e` re-run in the workspace on the revised bytes, exit 0,
  1 passed; it rebuilds the static site first, so the prerender path through the new `showing` getter
  is proven too)
- `app.css` is additive only. Trace: `--animate-brick-in`, `.animate-brick-in` and `.animate-pulse`
  are untouched in source and still present in the built CSS, so the brick entrance `BrickShell` and
  `FeedGrid` rely on is unchanged : PRESERVED
- `web/src/lib/state/reader.svelte.ts` (task 01's) **is** now in the diff, which the first pass could
  record as untouched and this pass cannot: the fix moves the render predicate into the rune. Traced
  as a dependency edit rather than waved through : PRESERVED. The change is one new getter
  (`showing`, `:72-76`) plus a narrowing of `isOpen` (`:85-87`) from `page.state.brick !== undefined`
  to `this.showing !== null`; `open`, `close`, `activate`, `next`, `prev`, `#step`, `index`,
  `canNext`, `canPrev` and `returnFocus` are byte-identical. Task 01's DoD pins none of them to
  `isOpen`, and its own suite still passes with all 18 original cases intact (20 now, and
  `pnpm vitest run src/lib/state/reader.test.ts` is green). The only consumers of `isOpen` in
  `web/src` are `BrickReader.svelte:36` and `+layout.svelte:25`, both of which want the narrow
  answer; there is no third caller that wanted the wide one. The intermediate states inside `open()`
  (push before assign) and `#step()` (assign before replace) are both synchronous within one tick, so
  no reader ever observes the halves disagreeing: measured, a step inside an open reader keeps the
  panel up, keeps the lock on, and produces zero `<html style>` mutations.

## Residue

- The scroll-lock revert on unmount (as opposed to on close) is in the task's steps but is not its
  own DoD item. If the validator finds a route where the overflow style survives, note it.
  - *Resolved:* no such route found, re-checked on the revised bytes. The lock lives in one `$effect`
    keyed on the reader's own open state (`BrickReader.svelte:67-86`) whose cleanup restores the
    captured previous value, and a Svelte effect cleanup runs on unmount as well as on re-run. Every
    close route observed left `document.documentElement.style.overflow === ""`, including the back
    gesture, a client-side navigation performed while the reader was open, and the state where the
    entry names a brick nobody holds.

- **The first pass's concern, and what shipped for it.** The first pass recorded that
  `+layout.svelte`'s `overlayOpen = page.state.brick !== undefined` was wider than the condition the
  component renders on, so open-back-reload-forward left the wall `inert` and scroll locked under a
  reader that renders nothing.
  - *Resolved, root cause rather than symptom.* There is now **one** predicate, owned by the rune:
    `ReaderState.showing` (`reader.svelte.ts:72-76`) is page state and the held brick agreeing, and
    `isOpen` (`:85-87`) is defined off `showing` rather than beside it. The component renders
    `$derived(reader.showing)` (`:26`), keys its teardown on `$derived(reader.isOpen)` (`:36`), and
    the layout reads `const overlayOpen = $derived(reader.isOpen)` (`+layout.svelte:25`).
    `page.state.brick` has exactly one executable reader in `web/src`. Fixing only the layout line
    would have left the same two-predicate shape one edit away from drifting again; narrowing the
    shared definition is what removes it.
  - *Measured, not reasoned, and the measurement discriminates.* The exact sequence was driven in
    chromium: open, `goBack()`, `reload()`, `goForward()`. The scenario reproduces (the restored
    entry carries `fixture-post-0`; the reloaded rune holds `null`), and in that state nothing is
    inert, `overflow` is `""`, no dialog renders, no page error fires, an element takes focus, a real
    mouse click reaches it, and the wall scrolls 900px. A separate probe read **both** predicates out
    of the live page in that same state, from the same `$app/state` module instance the app imports:
    `page.state.brick !== undefined` is `true` while `reader.isOpen` is `false`. The old condition
    would therefore have been inert exactly where the probe asserts the wall is live, so the probe
    measures the fix rather than agreeing with it, and it establishes that without editing a byte of
    source.
  - *The invariant, checked at every stop.* Walking the stale entry out one step at a time (stale
    entry, reopen the same brick, Escape, Escape) the wrapper is inert and the page scroll locked
    **exactly** when a panel is over the wall, at all four stops.

- **Still true for task 18: the condition widens rather than gets rewritten.** `+layout.svelte:25` is
  one `$derived` on one line, and the picker is a one-token append (`reader.isOpen || picker.isOpen`).
  The comment above it says so, and says the new part: each disjunct asks its own surface whether it
  is up rather than re-deriving page state, which is the lesson of this defect.
  - *For whoever runs task 18:* that task's DoD names the widened expression literally as
    `page.state.brick || page.state.picker`, and asks that `grep -n inert web/src/routes/+layout.svelte`
    return "a single hit naming both keys". That literal text no longer matches what shipped. The
    intent survives intact (one expression, widened, not replaced) but the wording needs updating to
    the rune-owned predicates, and the picker's own disjunct should be its state object's predicate,
    not a second raw read of `page.state`.

- **A corner of a corner, recorded rather than fixed.** From the stale entry (page state names brick
  X, nothing held), opening that same brick X again pushes a second entry naming X, so one Escape
  pops back onto the stale entry, which also names X while the rune now holds X, and the reader
  legitimately stays up; a second Escape takes `close()`'s `replaceState` branch and shuts it. Two
  Escapes instead of one, reachable only after a reload between a back and a forward. It does not
  violate the invariant above (the wall is never frozen under nothing, and is live and focusable at
  the end), it is strictly better than the behaviour the first pass found, and nothing in this task's
  DoD speaks to it. Worth knowing when task 05 adds stepping and task 06 automates the reader.

- **A note for task 05, from the implementer and confirmed here.** The teardown effect reads its
  boolean through a `$derived` rather than straight off the rune deliberately: a derived wakes its
  readers only when the value changes, and a step replaces the history entry without changing whether
  the reader is up. Measured on the shipped bytes: a step moves `fixture-post-0` to
  `fixture-post-1`, the panel stays up, the lock stays on, and a `MutationObserver` on
  `<html style>` records zero mutations across the step.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED against evidence this second validator collected first-hand on
the revised bytes (`just check`, `just test-e2e` and the reader unit suite all green in the
workspace, plus 77 chromium assertions covering the dialog's role, name across all three kinds, focus
in and out, the inert wall, the scroll lock actually stopping the wall, all four close routes, the
motion row in both directions, task 01's double-pop trap under three doubled dismissals, and the
open-back-reload-forward sequence with a predicate read that proves the probe discriminates); all
four regression traces are PRESERVED, including the newly-in-scope edit to task 01's rune, and the
first pass's one concern is resolved at its root by a single shared predicate rather than by two
booleans that agree today.
