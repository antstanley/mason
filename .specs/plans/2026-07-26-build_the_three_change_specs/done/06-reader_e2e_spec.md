# Task 06 · reader e2e spec

**Plan:** [plan.md](../plan.md) · **Certificate:** [06-reader_e2e_spec-certificate.md](06-reader_e2e_spec-certificate.md)

**Implements:** [`changes/merged/2026-07-26-read_a_brick_in_place.md`](../../../changes/merged/2026-07-26-read_a_brick_in_place.md) implementation note 9, and the Dialog behaviour and Accessibility behaviours blocks it verifies. Verifies [`07-web-client.md`](../../../07-web-client.md) §Testing rather than editing it: the Playwright row there, and the `just test-e2e` row at [`10-build-release-deploy.md`](../../../10-build-release-deploy.md)`:55`, both describe the lane as one service-worker smoke, and they go stale as each of this plan's four e2e specs lands. **Task 27 owns that reconciliation**, because it is the only task that runs after all four exist. Do not edit either row here; a half-corrected row is worse than a wholly stale one.
**Depends on:** 04, 05
**Produces:** the only automated lane that renders `BrickReader` at all, and the only place the shared reveal becomes observable.
**Pointers:** `web/tests/` currently holds one spec, `service-worker-smoke.test.ts`. The directory is already in the tsc program via `web/.svelte-kit/tsconfig.json`'s `../tests/**/*.ts` include, so `just test` typechecks it and `just test-e2e` runs it. `playwright.config.ts` has a single chromium project. `just test-e2e` runs `just build` first, which rebuilds the wasm.

## Steps

- [ ] Add `web/tests/reader.test.ts` driving `/?actor=demo` against the static build.
- [ ] Assert that a plain left click on the first post card opens a `[role=dialog]` carrying that brick's full text, that `page.url()` is unchanged, and that a marker set on `window` before the click survives it.
- [ ] Assert Escape closes the dialog and `document.activeElement` is the anchor that was clicked.
- [ ] Assert `page.goBack()` also closes it and leaves the wall laid: the same article count, no skeletons.
- [ ] Assert the inert shape from the outside: with the reader open, the layout picker radio is not focusable and the reader's close control is.
- [ ] Assert the scroll lock, which is task 03's and nothing else's: `document.documentElement.style.overflow` reads `hidden` while the reader is open and is back to its former value after every one of the four close routes, the back gesture included.
- [ ] Assert the motion row from `09-design-system.md`, under **both** media states: with `prefers-reduced-motion: no-preference` the panel's computed `animation` names the reader's own keyframes and a non-zero duration; with `reduce` (a second Playwright project or `page.emulateMedia({ reducedMotion: 'reduce' })`) the panel does not move and only the scrim fades. Read the computed style rather than the class list: the override lives in `app.css:99`'s media block, so a class present proves nothing about what it resolves to.
- [ ] Assert the right-arrow key changes the brick shown and that the dialog does not step past the last laid brick, and that revealing the covered fixture brick on the card leaves it revealed when the reader opens on it.
- [ ] Assert the reveal is a reveal and nothing else: clicking "show anyway" on the covered fixture brick uncovers the media **and** leaves `[role=dialog]` absent. Two of the four cards put that button inside the anchor task 04 intercepts, so without task 02's `stopPropagation` the click opens the reader; the assertion above ("still revealed when the reader opens on it") is satisfied by that wrong behaviour too, which is why this is a separate case.

## Definition of done

- [ ] Every assertion above is present and green, offline, against the real static build.
- [ ] The file header states plainly that this is the only lane that can see a component, so a later reader does not mistake a green `just check` for coverage of tasks 03, 04 and 05.
- [ ] `just test` typechecks the new spec under `pnpm check:ci`, and `cd server && cargo nextest run` is green (task 02 changed `fixtures.rs`).
- [ ] Meets the repo definition of done (`just check` green, `just test-e2e` green, the wasm rebuilt by `just build`).
- [ ] Reviewable: run `just test-e2e` and watch `web/tests/reader.test.ts` pass; every claim tasks 03 to 05 could not verify is discharged here or named as still unverified.
