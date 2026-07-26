# Done Certificate · Task 00: service worker typecheck lane

**Task:** [00-service_worker_typecheck_lane.md](00-service_worker_typecheck_lane.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 00. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task adds
> a gate rather than a feature, so most of its evidence is a program's file list, not a behaviour.

## Definition

DONE(Task 00) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** `web/src/service-worker.ts` enters a tsc program for the first time, so the
  positional `feed_page` call has a compiler counting its arguments before tasks 13 and 22 change
  how many there are.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break `pnpm check:ci` for the app project, `pnpm knip`, `pnpm test`,
  `just check`, or CI's `check` job. Must not require a source edit to pass.

## Obligations

- **O1 · The service worker is in a program, and so is the module it calls into.**
  - *Claim:* `cd web && pnpm exec tsc -p tsconfig.worker.json --listFiles` lists both
    `src/service-worker.ts` and `src/lib/mortar-wasm/pkg/mortar_wasm.d.ts`.
  - *Evidence to collect:* run the command and read both paths out of the output. Run the same
    command against `tsconfig.json` and confirm `src/service-worker.ts` is absent there, which is
    the baseline this task exists to change.
  - *Checks:* listing `service-worker.ts` without `mortar_wasm.d.ts` means the import resolved to
    nothing and every argument is `any`, so the project would be green and worthless. Resolve the
    `$lib` alias: it comes from the generated `.svelte-kit/tsconfig.json` through the `extends`
    chain, so an `extends` that skips `./tsconfig.json` would break it.
  - *Status:* unverified

- **O2 · The new project is green with no source edit.**
  - *Claim:* both `tsc --noEmit -p tsconfig.json` and `tsc --noEmit -p tsconfig.worker.json` pass,
    and the diff contains no change to any file under `web/src`.
  - *Evidence to collect:* run `cd web && pnpm check:ci`. List the diff's changed files and confirm
    the set is `web/tsconfig.worker.json`, `web/package.json`, `justfile` and
    `.specs/10-build-release-deploy.md`.
  - *Checks:* the plan verified this configuration green against the tree as it stands. A source fix
    in the diff means the shipped config differs from the verified one; resolve which option
    differs (`lib`, `types`, `include` or `exclude`) and whether the difference weakens the project.
  - *Status:* unverified

- **O3 · The lane bites on an arity change.**
  - *Claim:* passing an extra argument to `feed_page` at `service-worker.ts:260` fails
    `pnpm check:ci` with an error naming `feed_page`, and the break was reverted.
  - *Evidence to collect:* the PR body's record of the by-hand break, including the error text. Then
    confirm the committed `service-worker.ts` is unmodified.
  - *Checks:* an error that names a different symbol, or a `tsc` exit 0, means the call is being
    checked against something other than the generated `mortar_wasm.d.ts`.
  - *Status:* unverified

- **O4 · The limits are stated, and the gate survives a never-built tree.**
  - *Claim:* the PR says this lane catches arity and type, not order, because after task 22 the
    call is six `Option<String>` slots and a transposition typechecks, with tasks 13 and 22 named as
    the lanes that cover order. And `just test` now depends on `wasm`, so `just check` on a tree with
    no `web/src/lib/mortar-wasm/pkg/` is green rather than failing on an unresolved module; a bare
    `cd web && pnpm check:ci` still fails there, which the PR names along with `just wasm` as the
    remedy.
  - *Evidence to collect:* read the PR body for both statements. Read `justfile`'s `test` recipe and
    confirm it depends on `wasm`, and `check` at `:152` and confirm it reaches `test`. Delete
    `web/src/lib/mortar-wasm/pkg/`, run `just check`, and expect green; run `cd web && pnpm check:ci`
    from the same state and expect the unresolved-module failure. Read
    `.github/workflows/ci.yml` and confirm the `just wasm` step still precedes `just check`.
  - *Checks:* `guard-wasm` is not a substitute and must not be treated as one: it is a `cargo check`
    for `wasm32` and emits no `pkg/`, so a gate that ran it and not `wasm` would still fail the new
    tsc project. Then resolve the cost: the added dependency puts an incremental wasm-pack build in
    front of every `just check`. Confirm the timing comment above the `check` recipe still matches a
    measured warm run, or was corrected.
  - *Status:* unverified

- **O5 · Nothing else in the gate noticed.**
  - *Claim:* `pnpm knip`, `pnpm test` and `just check` are all still green, and the `just test` row
    at `.specs/10-build-release-deploy.md:54` now says two tsc projects and a wasm build.
  - *Evidence to collect:* run `cd web && pnpm knip`, `cd web && pnpm test`, then `just check` from
    the repo root. Read the spec row.
  - *Checks:* knip reads `tsconfig.json`, not every tsconfig in the directory, and oxfmt formats only
    `src`. Both are expectations; the runs are the evidence.
  - *Status:* unverified

- **O6 · Reviewable: one command, one file, one absence.**
  - *Claim:* `pnpm exec tsc -p tsconfig.worker.json --listFiles | grep service-worker` prints the
    file and the same command against `tsconfig.json` prints nothing.
  - *Evidence to collect:* both command outputs.
  - *Status:* unverified

## Regression check

- `web/src/service-worker.ts` behaviour. Trace: nothing in it changed, so `just test-e2e` and
  `web/tests/service-worker-smoke.test.ts` are still green : (PRESERVED / REGRESSION)
- The app tsc project. Trace: `tsc --noEmit -p tsconfig.json` still passes and still excludes the
  worker, so the app project's meaning did not change : (PRESERVED / REGRESSION)
- CI's `check` job. Trace: `just wasm` then `just check` still passes with the extra tsc invocation,
  and the now-redundant `just wasm` step is left in place rather than removed :
  (PRESERVED / REGRESSION)
- `just check`'s stated cost. Trace: the recipe's own comment claims a warm gate of about eight
  seconds; re-measure with `test: wasm` in place and confirm the comment either still holds or was
  corrected in this diff : (PRESERVED / REGRESSION)

## Residue

- The second project covers all of `service-worker.ts`, not only the `feed_page` call, so a future
  edit anywhere in that file can fail the gate for an unrelated reason. Recorded as this task's open
  question, not an obligation.
- `svelte-kit sync` regenerates `.svelte-kit/tsconfig.json` and its `exclude` array on every check.
  The worker project survives that only because its own `"exclude": []` overrides the inherited one.
  If a future SvelteKit changes how `exclude` merges through `extends`, this lane goes quietly
  green-and-empty; O1's `--listFiles` check is what would catch it.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
