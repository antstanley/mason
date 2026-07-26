# Done Certificate · Task 00: service worker typecheck lane

**Task:** [00-service_worker_typecheck_lane.md](00-service_worker_typecheck_lane.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* ran `cd web && pnpm exec tsc -p tsconfig.worker.json --listFiles`. The program is
    282 files and holds BOTH named paths:
    `/Users/ant/code/mason-ws1/web/src/lib/mortar-wasm/pkg/mortar_wasm.d.ts` and
    `/Users/ant/code/mason-ws1/web/src/service-worker.ts`. The same command against
    `tsconfig.json` prints no `src/service-worker.ts` (it does print
    `tests/service-worker-smoke.test.ts`, see O6). The app project does list `mortar_wasm.d.ts`, but
    only because the generated `include` globs `../src/**/*.ts` over the pkg directory; no file in
    the app project imports it (`grep -rn mortar-wasm src/` outside the worker returns nothing), so
    before this change nothing anywhere was checked against that declaration.
  - *Check resolved:* `feed_page` at `service-worker.ts:260` resolves by step 4 (imported) to the
    named import at `:16` from `$lib/mortar-wasm/pkg/mortar_wasm`; `$lib` resolves through
    `tsconfig.worker.json` extends `./tsconfig.json` extends `./.svelte-kit/tsconfig.json`, whose
    `paths` maps `$lib/*` to `../src/lib/*`. Target is
    `mortar_wasm.d.ts:31`, `feed_page(actor: string, cursor?, mode?, intent?)`. No shadow: `grep -n
    feed_page src/` returns exactly two hits, the import at `:16` and the call at `:260`, so no
    local, class or module-level binding competes with the import.
  - *Status:* SATISFIED

- **O2 · The new project is green with no source edit.**
  - *Claim:* both `tsc --noEmit -p tsconfig.json` and `tsc --noEmit -p tsconfig.worker.json` pass,
    and the diff contains no change to any file under `web/src`.
  - *Evidence to collect:* run `cd web && pnpm check:ci`. List the diff's changed files and confirm
    the set is `web/tsconfig.worker.json`, `web/package.json`, `justfile` and
    `.specs/10-build-release-deploy.md`.
  - *Checks:* the plan verified this configuration green against the tree as it stands. A source fix
    in the diff means the shipped config differs from the verified one; resolve which option
    differs (`lib`, `types`, `include` or `exclude`) and whether the difference weakens the project.
  - *Collected:* `cd web && pnpm check:ci` exits 0 (it runs `svelte-kit sync`, then both projects).
    `jj diff --stat` in the workspace is exactly four files:
    `.specs/10-build-release-deploy.md` (+21), `justfile` (+31), `web/package.json` (+5),
    `web/tsconfig.worker.json` (+41, new). `jj st` shows nothing under `web/src` or `server/`.
  - *Check resolved:* the shipped `tsconfig.worker.json` is the configuration the plan named, option
    for option: `extends: ./tsconfig.json`, `lib: ["esnext", "webworker"]`, `include` of the two
    generated ambients plus `src/service-worker.ts`, `exclude: []`. Nothing was loosened, no `types`
    override, no `skipLibCheck` beyond the one the app project already sets. `.svelte-kit/env.d.ts`
    is the one generated file not carried over, and it is inert: it holds a single comment line, and
    the four `$env/*` module declarations live in `ambient.d.ts`, which IS included.
  - *Status:* SATISFIED

- **O3 · The lane bites on an arity change.**
  - *Claim:* passing an extra argument to `feed_page` at `service-worker.ts:260` fails
    `pnpm check:ci` with an error naming `feed_page`, and the break was reverted.
  - *Evidence to collect:* the PR body's record of the by-hand break, including the error text. Then
    confirm the committed `service-worker.ts` is unmodified.
  - *Checks:* an error that names a different symbol, or a `tsc` exit 0, means the call is being
    checked against something other than the generated `mortar_wasm.d.ts`.
  - *Collected:* the drafted commit message records the break and its error text, and
    `jj diff` shows `web/src/service-worker.ts` untouched. That record was not taken on trust: the
    break was reproduced here without editing the tracked tree, by copying `src/service-worker.ts`
    to an untracked probe file beside it and pointing a throwaway copy of the worker project at the
    copy. Three probes, all reverted and the probe files deleted:
    - fifth argument: `error TS2554: Expected 1-4 arguments, but got 5.` at `(260,63)`, exit 1.
    - `feed_page(42, ...)`: `error TS2345: Argument of type 'number' is not assignable to parameter
      of type 'string'.` at `(260,34)`, exit 1.
    - `mode` and `intent` swapped: exit 0, which is the limit O4 asks to be stated.
  - *Check resolved:* the error names the file, line and column, not the symbol, in tsc's plain
    output; `--pretty` (what a terminal shows) quotes the offending line, and `feed_page` appears
    there under the squiggle. It is the right symbol either way: the arity in the message, `1-4`,
    is exactly `mortar_wasm.d.ts:31`'s one required and three optional parameters, and TS2345 names
    `string`, that declaration's `actor` type. Nothing else in the program declares a `feed_page`.
  - *Status:* SATISFIED

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
  - *Collected, second half (the never-built tree): everything asked for, and it holds.*
    `web/src/lib/mortar-wasm/pkg/` was moved aside and the sequence run from that state:
    - `cd web && pnpm check:ci` fails, exit 1, with
      `src/service-worker.ts(18,8): error TS2307: Cannot find module
      '$lib/mortar-wasm/pkg/mortar_wasm' or its corresponding type declarations.` plus five
      implicit-any follow-ons. The message names the missing module, which is the correct failure.
    - `cd web && pnpm knip` also fails there, on `Unresolved imports (2)`,
      `src/service-worker.ts:18:8` and `:19:21`. This confirms the plan's own correction: `lint`
      needs the pkg on its own terms, so `test: wasm` alone would not have made `just check` green.
      The shipped diff declares `wasm` on BOTH `lint` and `test`, which is what closes it.
    - `just test` from that state: exit 0 in 7.3s (wasm-pack, 97 nextest, both tsc projects,
      21 vitest).
    - pkg deleted again, `just check`: exit 0 in 11.5s. The run log shows `wasm-pack build` exactly
      once, ordered after `guard-wasm` and before `oxlint`/`knip`, so `just` does dedupe the
      doubly-declared dependency.
    - `.github/workflows/ci.yml:48` still runs `just wasm` as its own step before
      `just check` at `:57`. The now-redundant step was left in place.
    - `justfile:42` reads `test: wasm`, `:63` reads `lint: wasm`, and `check` at `:171` still
      reaches both.
  - *Re-run in the second pass, not carried on the first pass's word.* The whole never-built-tree
    sequence was executed again: `pkg/` moved aside, `cd web && pnpm check:ci` exits 1 with the same
    `src/service-worker.ts(18,8): error TS2307: Cannot find module
    '$lib/mortar-wasm/pkg/mortar_wasm' or its corresponding type declarations.` plus the same five
    implicit-any follow-ons, and `just check` from that state exits 0 with exactly one
    `wasm-pack build` in the log, ordered before `pnpm oxlint` and `pnpm knip`. `justfile:42`,
    `:63` and `:171`, and `ci.yml:48` before `:57`, all re-read and unchanged. `pkg/` restored.
  - *Collected, first half (the stated limit): stated in full, after the second-pass fix.* The
    commit message states the arity-and-type limit better than asked, having measured the
    transposition rather than asserting it (`mode` and `intent` swapped at `:260`, tsc exits 0), and
    it now NAMES the two lanes. The sentence reads "The Playwright cases carried by tasks 13 and 22
    are what cover the order: each drives one demo request carrying `cursor`, `mode` and `intent` at
    once and reads three independent effects out of the answer", closed by "Task 00 buys the arity,
    13 and 22 buy the order". The pair appears in the opening paragraph as well ("Tasks 13 and 22
    are about to change how many arguments that call takes"), so the referent is set before the
    limit is stated and resolved again after it.
  - *Referent checked, not taken on trust:* `plan.md:405` reads "Task 00 buys the arity, the
    Playwright cases in 13 and 22 buy the order", and the following clause is the source of the
    message's own phrasing about one demo request carrying all three. Both task files exist,
    `13-feed_target_and_feed_wall.md` and `22-refresh_entry_point_and_fronts.md`, and the plan names
    13 as where `feed_page` is fixed at `(actor, feed, cursor, mode, intent)` and 22 as where
    `refresh` is appended. The numbers point at the tasks that actually widen the call. A reader of
    the commit alone can now resolve which lanes owe the order coverage. The rest of the message is
    unchanged from the message the first pass read, and it holds no U+2014, no U+2013 and no
    non-ASCII byte.
  - *Check resolved:* `guard-wasm` was not treated as a substitute anywhere: `justfile:130` is still
    `cargo check --target wasm32-unknown-unknown`, it emits no `pkg/`, and both new comments say so.
    Cost: the `check` comment was corrected rather than left stale (`~8s` to `~9s`, with a `wasm ~2s`
    entry added to the cheapest-first list). Re-measured here, three warm runs: `just wasm` 2.28s,
    2.39s, 2.39s, which matches the comment's `~2s` and its claim that wasm-opt dominates. The
    absolute gate figure ran higher on this host (12.6s warm), but it ran equally high before the
    change, so the comment's DELTA is right and the absolute was already an underestimate. Corrected
    rather than stale: satisfied.
  - *Status:* SATISFIED (both halves; the stated-limit half by a second-pass fix to the commit
    message alone, the never-built-tree half re-run first-hand in that same pass)

- **O5 · Nothing else in the gate noticed.**
  - *Claim:* `pnpm knip`, `pnpm test` and `just check` are all still green, and the `just test` row
    at `.specs/10-build-release-deploy.md:54` now says two tsc projects and a wasm build.
  - *Evidence to collect:* run `cd web && pnpm knip`, `cd web && pnpm test`, then `just check` from
    the repo root. Read the spec row.
  - *Checks:* knip reads `tsconfig.json`, not every tsconfig in the directory, and oxfmt formats only
    `src`. Both are expectations; the runs are the evidence.
  - *Collected:* `cd web && pnpm knip` exit 0 (one pre-existing `.css` configuration hint, not an
    error). `cd web && pnpm test` exit 0, 2 files, 21 tests. `just check` exit 0 from the repo root.
    `just guard-dashes`, `just guard-autoplay` and `just fmt-check` all exit 0, and a byte-level
    `grep -c` for U+2014 over all four changed files returns 0 for each.
    `.specs/10-build-release-deploy.md`'s `just test` row now reads "Build the wasm first, then
    `cargo nextest run`, `pnpm check:ci` (tsc over **two** projects, the app and the service
    worker), `pnpm test` (vitest)", and the `just lint` row was updated too, correctly, since that
    recipe also changed. The spec additionally gained a paragraph on why the dependency has to be
    declared twice, the new config file in the implementation layout, and an extension to the
    existing "build the wasm before the web checks" decision.
  - *Check resolved:* knip did not notice the new project because `knip.json` points at
    `tsconfig.json`; oxfmt did not notice the new file because `fmt-check` runs `oxfmt --check src`
    and the file sits at `web/`. Both were run, not assumed. The repo definition of done is met:
    `just check` green, no em dash, and `web/tsconfig.worker.json` opens with a why comment that
    explains the generated `exclude`, the `exclude: []` mechanism, the `lib` narrowing, and what the
    lane cannot see. No changeset, correctly: nothing user-visible changed.
  - *Status:* SATISFIED

- **O6 · Reviewable: one command, one file, one absence.**
  - *Claim:* `pnpm exec tsc -p tsconfig.worker.json --listFiles | grep service-worker` prints the
    file and the same command against `tsconfig.json` prints nothing.
  - *Evidence to collect:* both command outputs.
  - *Collected:* exercised both. Against `tsconfig.worker.json` the pipeline prints
    `/Users/ant/code/mason-ws1/web/src/service-worker.ts`. Against `tsconfig.json` the literal
    command in this claim, `| grep service-worker`, does NOT print nothing: it prints
    `/Users/ant/code/mason-ws1/web/tests/service-worker-smoke.test.ts`, the Playwright smoke, which
    is in the app project and always was. With the precise pattern `grep 'src/service-worker.ts'`
    the app project prints nothing and the worker project prints the file, which is the absence this
    obligation is actually about. The implementer found the same thing and recorded both forms in
    the commit message, so the imprecision is disclosed rather than papered over. Counted as
    satisfied on substance: the certificate's claim was written a shade too loosely, the underlying
    fact holds.
  - *Status:* SATISFIED

## Regression check

- `web/src/service-worker.ts` behaviour. Trace: nothing in it changed, so `just test-e2e` and
  `web/tests/service-worker-smoke.test.ts` are still green : PRESERVED. `jj diff` touches no file
  under `web/src`, and `just test-e2e` was run: the build succeeded and
  `tests/service-worker-smoke.test.ts:7 the demo wall round-trips /api/feed through the wasm
  service worker` passed in 504ms, 1 passed.
- The app tsc project. Trace: `tsc --noEmit -p tsconfig.json` still passes and still excludes the
  worker, so the app project's meaning did not change : PRESERVED. It runs first inside `check:ci`
  and exits 0, and `--listFiles` still has no `src/service-worker.ts` in it. The new project is a
  second invocation, not an edit to the first: `web/tsconfig.json` is unchanged in the diff.
- CI's `check` job. Trace: `just wasm` then `just check` still passes with the extra tsc invocation,
  and the now-redundant `just wasm` step is left in place rather than removed : PRESERVED.
  `.github/workflows/ci.yml` is unchanged in the diff; its `just wasm` step at `:48` still precedes
  `just check` at `:57`, and `just check` was run to green locally from both a warm and a
  never-built tree.
- `just check`'s stated cost. Trace: the recipe's own comment claims a warm gate of about eight
  seconds; re-measure with `test: wasm` in place and confirm the comment either still holds or was
  corrected in this diff : PRESERVED, by correction. The comment was updated to `~9s` with a
  `wasm ~2s` entry and a note that it was re-measured over three runs. Independently re-measured
  here: `just wasm` warm is 2.28s / 2.39s / 2.39s, matching. The gate's absolute warm figure ran
  higher on this host (12.6s, and 14.5s again in the second pass with other agents building), but
  that is host and load, not the change: the added dependency's measured cost is the ~2.3s the
  comment claims, and the pre-change `~8s` figure was optimistic on this host by the same margin.
  Recorded as a non-blocking imprecision in an absolute number, not a wrong delta.

## Residue

- The second project covers all of `service-worker.ts`, not only the `feed_page` call, so a future
  edit anywhere in that file can fail the gate for an unrelated reason. Recorded as this task's open
  question, not an obligation.
- `svelte-kit sync` regenerates `.svelte-kit/tsconfig.json` and its `exclude` array on every check.
  The worker project survives that only because its own `"exclude": []` overrides the inherited one.
  If a future SvelteKit changes how `exclude` merges through `extends`, this lane goes quietly
  green-and-empty; O1's `--listFiles` check is what would catch it.

## Validator's notes

- **Certificate to DoD drift, noted not silently absorbed.** The task's `Definition of done` has
  eight items; this certificate has six obligations. O4 folds DoD items 4 and 5 (the stated limit,
  and the never-built tree) into one, and O5 folds DoD items 6 and 7 (knip plus `just check`, and
  the repo definition of done) into one. Validated against the DoD, which is the contract; every
  one of the eight items is discharged. The folding is why the first pass could hold O4
  half-evidenced and still UNSATISFIED as a whole, and it is why the second pass had to re-run the
  already-satisfied half rather than only read the fixed sentence.
- **The one gap was prose, not code, and it is closed.** Nothing in the shipped configuration was
  ever wrong. The first pass held O4 UNSATISFIED because the commit message did not name tasks 13
  and 22 as the lanes that cover argument order. The second pass confirmed the fix landed as a
  sentence-level edit to the message and nothing else: the four content files carry mtimes of
  22:04 to 22:11, all of them earlier than both the first pass's certificate (22:24) and the retry
  brief (22:28), and `jj diff` over them is line-for-line the diff the first pass recorded
  (`.specs/10-build-release-deploy.md` +21, `justfile` +31, `web/package.json` +5,
  `web/tsconfig.worker.json` +41 new). The correctness verdict therefore transfers rather than
  being re-derived, which is what the second-pass brief scoped.
- **What the second pass re-ran rather than inherited.** `just check` in the workspace (exit 0),
  the whole never-built-tree sequence behind O4, and O6's `--listFiles` pair. All three reproduced.
  Only O4's first half changed value.
- **Two things the implementer got right against its own plan, worth recording.** The plan's
  `test: wasm` prescription was insufficient, and the implementer measured that rather than
  shipping it: `lint` reads the pkg too, through knip, and `just` does not hoist a shared
  dependency to the front of `check`. Declaring `wasm` on both recipes is the correct fix and was
  verified here to still run wasm-pack exactly once. Separately, the Reviewable command in this
  certificate was written a shade too loosely; the implementer found that, disclosed it, and gave
  the precise pattern rather than quietly passing the loose one.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: All six obligations are SATISFIED on first-hand evidence and every regression line is
PRESERVED. O1, O2, O3, O5 and O6 were discharged in the first pass (the worker project holds both
`src/service-worker.ts` and `mortar_wasm.d.ts` in a 282-file program, an independently reproduced
arity break gives TS2554 and a type break TS2345 while a transposition passes, no source file is
touched, and `just check`, `just test`, `knip`, `vitest` and `test-e2e` are all green). O4 was the
one open item, and the second pass closes it: the commit message now names tasks 13 and 22 as the
lanes whose Playwright cases cover argument order, three times over, and the referent was checked
against `plan.md:405` and against both task files rather than taken on trust. The fix touched the
message only, which was verified by mtime and by re-reading the diff, so the first pass's CORRECT
carries. The never-built-tree half of O4 was re-run rather than inherited: with
`web/src/lib/mortar-wasm/pkg/` absent, `cd web && pnpm check:ci` fails with the TS2307 that names
the missing module, and `just check` from that same state exits 0 with `wasm-pack` firing exactly
once ahead of `oxlint` and `knip`.
