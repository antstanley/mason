# Done Certificate · Task 06: reader e2e spec

**Task:** [06-reader_e2e_spec.md](06-reader_e2e_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 06. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task is
> the coverage for tasks 03, 04 and 05, so an UNSATISFIED obligation here leaves those three
> unverified as well.

## Definition

DONE(Task 06) is every obligation O1 to O5 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The only automated lane that renders `BrickReader` at all, and the only place the
  shared reveal becomes observable.
- **P2 · Obligations.** Done iff O1 to O5 all hold; O5 is the Reviewable item.
- **P3 · Invariants.** Must not break `web/tests/service-worker-smoke.test.ts`, which shares the
  single chromium project in `playwright.config.ts` and the same demo wall.

## Obligations

- **O1 · Every named assertion is present and green offline.**
  - *Claim:* `web/tests/reader.test.ts` asserts: a plain left click on the first post card opens a
    `[role=dialog]` carrying that brick's full text; `page.url()` is unchanged; a `window` marker set
    before the click survives it; Escape closes and `document.activeElement` is the clicked anchor;
    `page.goBack()` closes and leaves the same article count with no skeletons; with the reader open
    the layout picker radio is not focusable and the close control is;
    `document.documentElement.style.overflow` is `hidden` while open and restored after every close
    route; the panel's **computed** animation matches the `09-design-system.md` motion row under
    `no-preference` and the panel does not move under `reduce`; the right-arrow key changes
    the brick and does not step past the last laid one; revealing the covered fixture brick on the
    card leaves it revealed when the reader opens on it; and clicking "show anyway" reveals the media
    while leaving `[role=dialog]` absent.
  - *Evidence to collect:* read the spec and check off each of the eleven assertions by name. Run
    `just test-e2e` and record the per-assertion result.
  - *Checks:* the `window` marker is what distinguishes "no navigation" from "navigated and came
    back". Confirm the marker is set before the click and read after, not merely that the URL string
    matched. The last two assertions must be **separate cases**: on `PostCard` and on `GlazeCard`'s
    single/grid branch the reveal button is a descendant of the anchor task 04 intercepts, so a
    missing `stopPropagation` opens the reader on the reveal, and the "still revealed when the reader
    opens on it" assertion passes under that broken behaviour too. Only the `[role=dialog]` absence
    tells them apart. The scroll-lock and motion assertions are the other two items the plan's
    "Where the gate is blind" table hands from task 03 to here, so confirm they are real reads:
    `getComputedStyle` on the panel and on `document.documentElement`, not a class-list or
    inline-attribute check, and the motion one run under both media states rather than only the
    default.
  - *Status:* SATISFIED
    - collect: all eleven assertions present, one case each (13 cases in 563 lines):
      `:206` plain click (dialog + `aria-modal` + `aria-label` + `getByText(text, {exact:true})` +
      `page.url()` + marker); `:235` Escape + `toBeFocused()` on the clicked anchor; `:252` goBack +
      `toHaveCount(laid)` + `#wall .animate-pulse` count 0; `:282` inert from the outside; `:311`
      scroll lock through all four routes; `:348` and `:373` motion under both media states; `:400`
      arrow stepping and the end of the laid wall; `:434` reveal follows the brick; `:461` reveal and
      nothing else. Three beyond the contract: `:487` glaze image anchor, `:515` the reader's own
      player id, `:543` the modified click.
    - collect: `CI=1 just test-e2e` run twice by the validator, both green, 20 passed in 7.5s /
      7.9s. The list reporter confirms all 13 reader cases pass by name.
    - check (marker): `reader.test.ts:211-214` sets `window.masonNeverLeft` by `page.evaluate`
      **before** the click at `:215`; `:227` reads it back after. The same pattern at `:257` /
      `:271` in the goBack case. Not a URL-only check.
    - check (the two reveal cases are separate in substance, not only in shape): the validator
      deleted `event.stopPropagation()` from `Sensitive.svelte:52`, rebuilt and re-ran. `:461`
      "show anyway reveals the media and nothing else" FAILED, as did `:434` and `:487`. Restored,
      hash-verified, green again. The absence assertion bites.
    - check (scroll lock is a real read, and route by route): `rootOverflow()` at `:164-166` reads
      `document.documentElement.style.overflow` through `page.evaluate`; no class list anywhere. The
      validator moved the restore out of the `$effect` teardown in `BrickReader.svelte` and into the
      three explicit-close handlers, leaving the back gesture alone. The case failed with
      `Error: unlocked again after the back gesture / Expected: "" Received: "hidden"`, having
      passed escape, the close control and the scrim first. The lock is checked per route, and the
      route is named in the failure.
    - check (motion is computed, and under both states): `motion()` at `:172-197` reads
      `getComputedStyle(element).animationName` / `.animationDuration` plus the running animation's
      own resolved keyframes via `getAnimations()` then `KeyframeEffect.getKeyframes()`, and carries
      `frames` out so "no frame transforms" cannot pass on an empty list. Each case sets
      `page.emulateMedia({ reducedMotion })` itself. The validator changed `app.css:124-126`'s
      reduced-motion override from `brick-fade 0.15s` to `reader-in 0.24s`; `:373` FAILED. Restored.
    - collect (offline): the validator drove the demo wall in a chromium launched with
      `proxy: { server: "http://127.0.0.1:1", bypass: "localhost,127.0.0.1" }`, so nothing was
      reachable but loopback, against its own preview server on port 4199. The wall laid, the click
      opened the dialog with the brick's text and an unchanged URL and a surviving marker, the lock
      read `hidden`, the arrow stepped `brick 2 of 24` to `brick 3 of 24`, Escape restored the lock,
      and "show anyway" revealed with no dialog. The only origins the browser tried and could not
      reach were `fonts.googleapis.com` (the layout's stylesheet link) and `picsum.photos` (fixture
      image `src`s). No assertion depends on either.

- **O2 · The file says what it is.**
  - *Claim:* the file header states plainly that this is the only lane that can see a component, so
    a later reader does not mistake a green `just check` for coverage of tasks 03, 04 and 05.
  - *Evidence to collect:* read the first comment block of `web/tests/reader.test.ts`.
  - *Status:* SATISFIED
    - collect: `reader.test.ts:3-22`. "THIS IS THE ONLY LANE IN THE REPO THAT RENDERS `BrickReader`
      AT ALL, and a green `just check` says nothing whatsoever about it", with the reason (tsc
      cannot parse `.svelte`, both vitest suites are `.ts` in node with no DOM), the list of claims
      that exist only while a case here is green, and the instruction: "do not read a passing
      `just check` as coverage of the reader, and do not delete a case here because 'the types cover
      it'".

- **O3 · The spec typechecks and the Rust side is still green.**
  - *Claim:* `pnpm check:ci` typechecks the new spec, and `cargo nextest run` passes after task 02's
    `fixtures.rs` change.
  - *Evidence to collect:* run `cd web && pnpm check:ci` (the `web/tests/**/*.ts` include in
    `web/.svelte-kit/tsconfig.json` is what puts the file in the program) and
    `cd server && cargo nextest run`. Expect both clean.
  - *Status:* SATISFIED
    - collect: `just test` ran green: `cargo nextest run` 154 tests, 154 passed, 0 skipped;
      `pnpm check:ci` (both `tsconfig.json` and `tsconfig.worker.json`) clean; vitest 5 files / 60
      tests passed.
    - check: the include is live, not assumed. `web/.svelte-kit/tsconfig.json:44` carries
      `"../tests/**/*.ts"`, and `pnpm exec tsc --noEmit -p tsconfig.json --listFiles` lists
      `tests/reader.test.ts` exactly once. The new spec really is in the program.

- **O4 · Meets the repo definition of done.**
  - *Claim:* the gates are green and the wasm was rebuilt.
  - *Evidence to collect:* run `just check`, then `just test-e2e` (which runs `just build` first,
    which runs `just wasm`). Expect both clean.
  - *Status:* SATISFIED for everything this diff governs, with one pre-existing failure recorded
    below rather than charged to this task.
    - collect: `CI=1 just test-e2e` green, and its `build` dependency emitted a fresh
      `mortar_wasm_bg.CCebITai.wasm` (845.95 kB) before the run, so the wasm was rebuilt.
    - collect: `just check` FAILS at its first step. `guard-dashes` reports one offender,
      `.specs/plans/2026-07-26-build_the_three_change_specs/done/22-refresh_entry_point_and_fronts-certificate.md`.
      That file is **not in this diff**: `jj diff --stat` is one added file, and
      `jj file show -r @-` on that path returns the offending lines (108, 154, 160), so it arrived
      with the parent commit `pxtrqyzn f4543b30`. It is a `-certificate.md` under `.specs/plans/`,
      which this task is forbidden both to read and to touch.
    - collect: re-running the same grep with `.specs/plans` excluded reports nothing at all, and
      the new spec itself contains zero U+2014.
    - collect: every remaining step of `check`, run individually in check's own order, is green:
      `guard-autoplay`, `guard-toolchain`, `fmt-check`, `guard-wasm`, `lint` (oxlint 4 pre-existing
      warnings and no errors, knip clean, clippy clean), `test`.

- **O5 · Reviewable: the spec passes and discharges what tasks 03 to 05 could not.**
  - *Claim:* `just test-e2e` is green with `web/tests/reader.test.ts` passing, and every claim tasks
    03, 04 and 05 could not verify is either discharged here or named in their PRs as still
    unverified.
  - *Evidence to collect:* run `just test-e2e`. Then walk the plan's "Where the gate is blind" table
    rows for tasks 03, 04 and 05 and confirm each is either covered by an assertion above or listed
    as still uncovered.
  - *Status:* SATISFIED
    - collect: the Reviewable action exercised. `CI=1 just test-e2e`, list reporter, all 13
      `tests/reader.test.ts` cases pass by name alongside the 7 pre-existing ones.
    - walk, row 03 (focus in and out, `inert` inheritance, scroll lock, Escape, click-away, the
      motion row): all six covered. Focus in at `:293-298`, focus out at `:245`, `inert` at `:288`
      and `:300` with a positive control so the refusal cannot be an empty selector, the lock at
      `:311`, Escape at `:243`, click-away as the scrim route at `:337`, the motion row at `:348`
      and `:373`.
    - walk, row 04 (`reader.activate` wired into five anchors across four cards, and the
      show-anyway button being a descendant of two of them): `BrickShell`'s card-wide anchor covered
      on a post at `:206` and on a blog by `repeated-tags.test.ts:82` in the same lane; `GlazeCard`'s
      single/grid anchor at `:499`; `VideoCard`'s watch link at `:527`. Both in-anchor reveal sites
      covered, at `:461` and `:493`. **One activation site is asserted nowhere**: `GlazeCard`'s
      carousel-branch anchor, the four-or-more-image strip, which the demo wall does reach (a post
      with `(i / 3) % 4 == 3` carries five images). It is neither covered nor named in the residuals
      list. Recorded as a gap in the naming, not in the contract: it is not one of the task's eight
      named assertions.
    - walk, row 05 (all three kind renderings, and the distinct player id): post body at `:224`,
      blog body by `repeated-tags.test.ts:82`, video body at `:532`. The row calls the double-player
      claim one with "**no lane at all**"; that is now stale in the task's favour. The validator
      pointed `BrickReader.svelte:68`'s `playerId` at `brick.id`, rebuilt, and `:515` FAILED with
      `#wall video` still at 1 behind the scrim. The lane exists and bites, entirely offline,
      because it watches the element mount rather than playback.
    - walk, row 02 (`Sensitive.svelte`'s body and its `stopPropagation`): covered at `:461`, and
      mutation-confirmed under O1.
    - the residual uncovered claims are named rather than papered over: the two visible step
      controls' click paths and the tick-then-refocus rescue; `feed.freeze()` on open (a demo
      preview answers settled, so there is no warming arrangement for a click to commit); the reader
      outliving its rune across reload-then-forward; `index === -1` (unreachable, nothing prunes
      laid bricks); and the blog body's layout beyond the tags. They are named in the implementer's
      report but **not** in the commit message that stands in for the PR body, which is where a
      later reader would look.

## Regression check

- `web/tests/service-worker-smoke.test.ts` runs in the same chromium project. Trace: run
  `just test-e2e` and expect both specs green with no cross-spec state leakage (each navigates
  fresh) : **PRESERVED**. `CI=1 just test-e2e` ran 20 tests across three specs in one chromium
  project on three workers: `service-worker-smoke.test.ts` 5 cases, `repeated-tags.test.ts` 2,
  `reader.test.ts` 13. All green, no retries recorded, no flaky marker. Each case calls its own
  `page.goto`, and Playwright gives each test its own browser context, so no state crosses. No
  existing unit was modified at all: the diff is one added file, and the four component and
  stylesheet files the validator mutated for the checks above were restored and verified byte for
  byte by SHA-256, with `jj st` and `jj diff --stat` back to `A web/tests/reader.test.ts`, 563
  insertions.

## Residue

- The e2e job's runtime roughly doubles here and again at tasks 18 and 26. Not an obligation, but
  worth recording if the CI `e2e` job approaches its timeout. Measured here: 7.5s wall clock for 20
  cases on three workers. Nowhere near a ceiling yet.
- `guard-dashes` is red on the branch for a reason outside this task (O4). The offending file is a
  done certificate written by an earlier gate agent. Until it is fixed, `just push` cannot run for
  this branch, whoever lands next.
- `fmt-check` runs `oxfmt --check src` and `guard-autoplay` greps `web/src`, so neither sees
  `web/tests/`. The new spec is therefore unchecked by those two. Pre-existing scope of both
  recipes, not a defect of this diff.
- `panel()` selects `[role="dialog"]` document-wide, and `GlazeCard` renders a second one for its
  ALT overlay. No case opens it today, so the locator is unambiguous; a future case that opens ALT
  would meet a strict-mode violation.
- The comment at `reader.test.ts:200-205` says a SvelteKit client-side navigation would take the
  `window` marker with it. It would not: only a full page load rebuilds `window`. The pair of
  assertions is still complete (the URL check is what catches the soft navigation), so this is a
  comment overstating one half of its own justification, not a hole.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O5 all SATISFIED on evidence the validator collected itself, including five
deliberate mutations (Sensitive's `stopPropagation`, the scroll-lock restore made route-specific,
the reader's `playerId`, the reduced-motion override, and mounting `BrickReader` inside the wrapper
it makes inert) that each turned the naming case red and were then restored and hash-verified; the
two sibling Playwright specs are PRESERVED in the same 20-case run; the one red step of `just check`
is a U+2014 in a done certificate carried in by the parent commit, outside this diff and outside
this task's permission to touch.