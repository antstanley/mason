# Done Certificate · Task 23: wire refresh vocabulary

**Task:** [23-wire_refresh_vocabulary.md](23-wire_refresh_vocabulary.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 23. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This is wire
> regeneration 3 of 3, the last one; the fixture must carry all three tasks' keys afterwards.

## Definition

DONE(Task 23) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The refresh token joins the query vocabulary, pinned on both sides by the same
  mechanism as `mode` and `intent`.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not drop task 10's `errors.feed_not_found` or task 14's `query.target`
  and `vocab.hiddenLabels` from the fixture, and must not break the five existing assertions in
  `contract-check.ts`.

## Obligations

- **O1 · The fixture carries all three regenerations and only the new key changed.**
  - *Claim:* `contract.json` carries `query.refresh` alongside `query.mode`, `query.intent` and
    `query.target`, five error codes including `feed_not_found`, and `vocab.hiddenLabels`; the
    committed diff contains only the new key.
  - *Evidence to collect:* run
    `python3 -c "import json;d=json.load(open('server/crates/mortar-core/tests/fixtures/contract.json'));print(sorted(d['errors']),sorted(d['query']),sorted(d['vocab']))"`
    and confirm five error codes, four query keys and two vocab keys. Read the diff line by line.
  - *Checks:* `UPDATE_FIXTURE=1` rewrites the file wholesale. If the tree was not rebased on tasks 10
    and 14 first, their keys vanish and `cargo test` still passes on this branch. The key count above
    is the check that catches it.
  - *Collected:* the dump gives errors `['actor_not_found','bad_request','feed_not_found',
    'login_required','upstream']` (5), query `['intent','mode','refresh','target']` (4), vocab
    `['hiddenLabels','videoSource']` (2), with `hiddenLabels` still carrying all five labels and
    `errors.bad_request.server.message` still task 14's reworded "bad request: actor or feed".
    `jj diff` on the fixture is three added lines, `"refresh": { "1": true }` between `mode` and
    `target`, and nothing else. `jj diff --stat` over the working copy is four files, 42 insertions,
    10 deletions, nothing outside contract.rs, contract.json, contract-check.ts and types.ts.
  - *Status:* SATISFIED

- **O2 · The token is bound once and asserted in both directions.**
  - *Claim:* `contract.rs` binds `const REFRESH: &str = "1";` beside `GLAZE`, `PREVIEW` and `FREEZE`
    and uses it for both the parser assert and the fixture key; the parser is asserted with
    `refresh_from_query(Some(REFRESH))` true and `refresh_from_query(None)` false.
  - *Evidence to collect:* read `tests/contract.rs` around `:347` and `:361`. Confirm the const
    appears in both the assert and the map insertion.
  - *Checks:* resolve `refresh_from_query` from `tests/contract.rs`. It is an integration test, so
    the function must be `pub` and reachable as `mortar_core::feed::refresh_from_query`.
  - *Collected:* the const is bound once at `contract.rs:363` and read twice, at the assert pair
    `:372`/`:373` and at the fixture key `:406` (`refresh_map.insert(REFRESH.to_string(), ...)`).
    Both directions are asserted: `assert!(refresh_from_query(Some(REFRESH)))` and
    `assert!(!refresh_from_query(None))`. Function resolution: `refresh_from_query` at `:372` has no
    local, no enclosing item and no module-level definition in `contract.rs`, so it resolves at step
    4 to the import at `contract.rs:31`, which is
    `pub fn refresh_from_query(raw: Option<&str>) -> bool` at
    `server/crates/mortar-core/src/feed.rs:133`. No shadowing. That function is untouched by this
    diff; it arrived with the parent commit. A rename probe (`REFRESH` set to `"9"`) fails at
    `contract.rs:372:5`, `assertion failed: refresh_from_query(Some(REFRESH))`, which is the proof
    that one binding drives both the parser assert and the key.
  - *Status:* SATISFIED

- **O3 · The guard is proven live, not merely present.**
  - *Claim:* temporarily changing `FeedRefresh` to any other literal makes `pnpm check:ci` fail with
    TS2344.
  - *Evidence to collect:* make the change, run `cd web && pnpm check:ci`, record the error code,
    then revert. `keyof` over the numeric-looking JSON key `"1"` yields the string literal `"1"`, so
    the assertion is meaningful rather than vacuously true; this probe confirms it.
  - *Collected:* four probes, run by the validator, each restored afterwards.
    `FeedRefresh = "2"` gives `src/lib/contract-check.ts(88,3): error TS2344: Type 'false' does not
    satisfy the constraint 'true'.` `FeedRefresh = string` gives the same error, which rules out the
    widening that would have made the assertion vacuous. The mirror direction was run too: the
    fixture key edited from `"1"` to `"9"` fails `pnpm check:ci` with the same TS2344 and fails
    `cargo nextest run -p mortar-core --test contract`, so neither side can drift alone. After each
    probe the file was restored and verified byte-identical by md5 (types.ts
    `62433cbb1fb4e1d7fde8937c889f10c2`, contract.json `569e5c7641184d12eb648c3bcb2fe42e`), and
    `pnpm check:ci` is green again.
  - *Status:* SATISFIED

- **O4 · The committed fixture passes without regeneration, and knip stays green.**
  - *Claim:* `cargo nextest run -p mortar-core --test contract` passes against the committed fixture
    with no `UPDATE_FIXTURE`, and `pnpm knip` is green with `FeedRefresh` exported from `types.ts`
    and consumed only by `contract-check.ts`.
  - *Evidence to collect:* run both commands. `web/knip.json` already lists `contract-check.ts` as an
    entry, which is why an export consumed only there is not dead code, exactly as `Blur` and
    `CaptionTrack` are today.
  - *Collected:* `cargo nextest run -p mortar-core --test contract` with no `UPDATE_FIXTURE` in the
    environment: `1 test run: 1 passed`. `pnpm knip`: only the pre-existing `.css` configuration
    hint, no unused exports and no unused files. A grep confirms `FeedRefresh` has exactly one
    consumer, the type import at `contract-check.ts:36`, and the fixture has exactly one importer,
    `contract-check.ts:26`, so nothing new reaches the runtime import graph.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check`. Confirm the fixture and both web files landed in one
    commit; split across commits the repo is red in between.
  - *Collected:* the three artefacts agree. `types.ts:138` is `export type FeedRefresh = "1"`, the
    fixture key is `"1"`, and the change spec's `FeedRefresh` fragment is `"const": "1"` with the
    same cursorless-only rule the doc comment states, which `feed.rs:167`
    (`let refresh = refresh && decoded.is_none();`) actually enforces. `06-wire-contract.md` and
    `canonical-types.schema.json` are untouched, which is the Residue below and task 27's job.
    `just test` is green (nextest 154/154 including `mortar-core::contract`, `pnpm check:ci`, vitest
    60/60), as are `guard-autoplay`, `guard-toolchain`, `fmt-check`, `guard-wasm` and `lint` (oxlint
    with four pre-existing warnings, knip, clippy `-D warnings`). All four files sit in one working
    copy revision, so they land as one commit. `just check` as a whole is red on one recipe only,
    `guard-dashes`, and on one file only,
    `.specs/plans/.../done/22-refresh_entry_point_and_fronts-certificate.md`, which holds five em
    dashes and arrived in the parent commit (`jj diff -r @-` lists it as an addition). It is outside
    this task's four files and outside its reach; no file this task touched contains an em dash.
  - *Status:* SATISFIED

- **O6 · Reviewable: the rename probe goes red and comes back.**
  - *Claim:* a reviewer runs the deliberate-rename experiment and watches `pnpm check:ci` go red,
    then reverts it.
  - *Evidence to collect:* the two runs.
  - *Collected:* exercised by the validator. Red run: `FeedRefresh = "2"` then `pnpm check:ci` gives
    `src/lib/contract-check.ts(88,3): error TS2344: Type 'false' does not satisfy the constraint
    'true'.` and exit 1. Green run: restored to `"1"`, `pnpm check:ci` exits 0 with no diagnostics,
    and the file's md5 matches its pre-probe value.
  - *Status:* SATISFIED

## Regression check

- `contract-check.ts`'s five existing assertions (`BrickKindsMatch`, `ErrorCodesMatch`,
  `IntentVocabularyMatches`, `ModeVocabularyMatches`, `VideoSourcesMatch`) plus task 14's two.
  Trace: all seven still pass : PRESERVED. All seven are still in the file and typecheck in a green
  `pnpm check:ci`; the mechanism was spot-checked live rather than assumed, by renaming `FeedMode` to
  `"glazed"`, which turned `ModeVocabularyMatches` red at
  `src/lib/contract-check.ts(82,44): error TS2344` before being restored.
- `contract.rs`'s existing `GLAZE`/`PREVIEW`/`FREEZE` parser asserts. Trace: still present and
  passing : PRESERVED. They stand unchanged at `:364` to `:368`, with the `ACTOR`/`FEED` pair at
  `:377` to `:391`, inside a contract test that passes against the committed fixture.
- No runtime code was touched: the diff is a Rust integration test, its fixture, a type-only export
  and a tsc-only assertion file, so no caller of engine code changed. `clippy --workspace
  --all-targets` and the wasm32 `cargo check --all-targets` both stay clean : PRESERVED.

## Residue

- `06-wire-contract.md` still describes three query keys until task 27 merges the refresh spec.
  Deliberate; record it rather than fixing it here.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED against evidence the validator collected itself, including four
rename probes in both directions that prove the numeric-looking key `"1"` pins as a string literal
rather than vacuously, and both regression traces are PRESERVED.
NOTE: `just check` is red for the orchestrator rather than for this task. `guard-dashes` finds five
em dashes in `.specs/plans/.../done/22-refresh_entry_point_and_fronts-certificate.md`, a gate-written
file inherited from the parent commit that this task neither touches nor may edit. Every other
recipe in `just check` is green.