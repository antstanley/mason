# Done Certificate · Task 06: reader e2e spec

**Task:** [06-reader_e2e_spec.md](06-reader_e2e_spec.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
    the layout picker radio is not focusable and the close control is; the right-arrow key changes
    the brick and does not step past the last laid one; revealing the covered fixture brick on the
    card leaves it revealed when the reader opens on it; and clicking "show anyway" reveals the media
    while leaving `[role=dialog]` absent.
  - *Evidence to collect:* read the spec and check off each of the nine assertions by name. Run
    `just test-e2e` and record the per-assertion result.
  - *Checks:* the `window` marker is what distinguishes "no navigation" from "navigated and came
    back". Confirm the marker is set before the click and read after, not merely that the URL string
    matched. The last two assertions must be **separate cases**: on `PostCard` and on `GlazeCard`'s
    single/grid branch the reveal button is a descendant of the anchor task 04 intercepts, so a
    missing `stopPropagation` opens the reader on the reveal, and the "still revealed when the reader
    opens on it" assertion passes under that broken behaviour too. Only the `[role=dialog]` absence
    tells them apart.
  - *Status:* unverified

- **O2 · The file says what it is.**
  - *Claim:* the file header states plainly that this is the only lane that can see a component, so
    a later reader does not mistake a green `just check` for coverage of tasks 03, 04 and 05.
  - *Evidence to collect:* read the first comment block of `web/tests/reader.test.ts`.
  - *Status:* unverified

- **O3 · The spec typechecks and the Rust side is still green.**
  - *Claim:* `pnpm check:ci` typechecks the new spec, and `cargo nextest run` passes after task 02's
    `fixtures.rs` change.
  - *Evidence to collect:* run `cd web && pnpm check:ci` (the `web/tests/**/*.ts` include in
    `web/.svelte-kit/tsconfig.json` is what puts the file in the program) and
    `cd server && cargo nextest run`. Expect both clean.
  - *Status:* unverified

- **O4 · Meets the repo definition of done.**
  - *Claim:* the gates are green and the wasm was rebuilt.
  - *Evidence to collect:* run `just check`, then `just test-e2e` (which runs `just build` first,
    which runs `just wasm`). Expect both clean.
  - *Status:* unverified

- **O5 · Reviewable: the spec passes and discharges what tasks 03 to 05 could not.**
  - *Claim:* `just test-e2e` is green with `web/tests/reader.test.ts` passing, and every claim tasks
    03, 04 and 05 could not verify is either discharged here or named in their PRs as still
    unverified.
  - *Evidence to collect:* run `just test-e2e`. Then walk the plan's "Where the gate is blind" table
    rows for tasks 03, 04 and 05 and confirm each is either covered by an assertion above or listed
    as still uncovered.
  - *Status:* unverified

## Regression check

- `web/tests/service-worker-smoke.test.ts` runs in the same chromium project. Trace: run
  `just test-e2e` and expect both specs green with no cross-spec state leakage (each navigates
  fresh) : (PRESERVED / REGRESSION)

## Residue

- The e2e job's runtime roughly doubles here and again at tasks 18 and 26. Not an obligation, but
  worth recording if the CI `e2e` job approaches its timeout.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
