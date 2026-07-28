# Done Certificate · Task 14: wire target and hidden labels

**Task:** [14-wire_target_and_hidden_labels.md](14-wire_target_and_hidden_labels.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 14. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This is wire
> regeneration 2 of 3.

## Definition

DONE(Task 14) is every obligation O1 to O6 below holding, O1b and O3b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** The target vocabulary and mortar's hidden-label list are pinned on both sides of
  the wire by the same mechanism that already keeps the error codes and video sources in step.
- **P2 · Obligations.** Done iff O1, O1b, O2, O3, O3b and O4 to O6 all hold; O6 is the Reviewable
  item.
- **P3 · Invariants.** Must not drop task 10's `errors.feed_not_found` from the fixture, must not
  break the four existing vocabulary assertions in `contract-check.ts`, and must leave `just lint`
  green, which means exporting no type nothing consumes.

## Obligations

- **O1 · The target assertion compares against a kind union, and nothing unused is exported.**
  - *Claim:* `api.ts` exports `FeedTargetKind = "actor" | "feed"` and `contract-check.ts` compares
    `keyof typeof contract.query.target` to it. `FeedTarget` itself is **not** exported yet; it
    arrives at task 15 with its first consumer.
  - *Evidence to collect:* read both files. Confirm the `Equal<>` right-hand side is
    `FeedTargetKind`. Run `cd web && pnpm exec knip` and expect no "Unused exported types" section.
  - *Checks:* `keyof` over a union type is `never`, so
    `Equal<keyof typeof contract.query.target, keyof FeedTarget>` would be false and the opposite
    spelling would pass vacuously. Probe: change `FeedTargetKind` to `"actor"` alone and confirm
    `pnpm check:ci` fails; restore. Second probe, for the export half: knip reports unused exported
    types in this repo (a throwaway `export type` in `api.ts` is reported), so an early `FeedTarget`
    makes `just lint` and therefore `just check` red at this boundary.
  - *Status:* SATISFIED. `web/src/lib/api.ts:45` exports `FeedTargetKind = "actor" | "feed"`;
    `web/src/lib/contract-check.ts:85`-`:87` is
    `Assert<Equal<keyof typeof contract.query.target, FeedTargetKind>>`, with `Equal<>` at `:73` the
    bidirectional `[A] extends [B] ? ([B] extends [A] ? true : false) : false`. `grep -rn FeedTarget
    web/src` finds no exported `FeedTarget`, only `FeedTargetKind` and two comments. `pnpm knip`
    exits 0 with no "Unused exported types" section. Probe (in a scratch copy of the tree, the
    workspace left untouched): `FeedTargetKind` reduced to `"actor"` fails tsc with
    `src/lib/contract-check.ts(86,3): error TS2344: Type 'false' does not satisfy the constraint
    'true'`. Second probe: appending `export type ProbeUnusedType = "probe";` to `api.ts` makes knip
    exit 1 with `Unused exported types (1) ProbeUnusedType`, so an early `FeedTarget` export would
    indeed have reddened `just lint`.

- **O1b · The two target tokens are bound once and asserted against the engine.**
  - *Claim:* `tests/contract.rs` binds `actor` and `feed` as consts used for the fixture keys **and**
    for an assert against `FeedTarget::kind()` on a real `FeedTarget::from_query` result.
  - *Evidence to collect:* read the block; it should be the shape of the `GLAZE`/`PREVIEW`/`FREEZE`
    block at `:347`. Confirm neither token appears a second time as a literal.
  - *Checks:* rename the Rust token to `author` and confirm `cargo nextest run -p mortar-core --test contract`
    fails on the assert rather than only on the fixture comparison; restore. Without the assert the
    keys are retyped literals and a one-sided rename stays green on the Rust side, which is the whole
    failure this mechanism exists to prevent. Task 13's `FeedTarget::from_query` is what makes the
    assert reachable: `tests/contract.rs` is a mortar-core integration test and can see neither front
    crate.
  - *Status:* SATISFIED. `tests/contract.rs:360`-`:361` binds `const ACTOR: &str = "actor";` and
    `const FEED: &str = "feed";` inside the `GLAZE`/`PREVIEW`/`FREEZE` block; `:370`-`:384` asserts
    both against `FeedTarget::from_query(...).expect(...).kind()`, and `:391`-`:394` builds
    `target_map` from the same two consts. `grep -n '"actor"\|"feed"' tests/contract.rs` returns
    exactly those two const lines, so neither token is a second literal. Probe: renaming
    `FeedTarget::kind`'s `Self::Actor(_) => "actor"` arm to `"author"` fails
    `cargo test -p mortar-core --test contract` at `contract.rs:370`, the assert, with the diff
    `< author / > actor`, not at the `:452` fixture comparison. Restored and re-run green.

- **O2 · The hidden-label list is compared in both directions.**
  - *Claim:* `types.ts` exports `HiddenLabel` and `contract-check.ts` compares it to
    `keyof typeof contract.vocab.hiddenLabels` bidirectionally.
  - *Evidence to collect:* read the assertion. Confirm the fixture's map is generated from
    `HIDDEN_LABELS` in `contract.rs` rather than retyped, by reading `tests/contract.rs`.
  - *Checks:* resolve `HIDDEN_LABELS` at its use in `contract.rs`. It must be the `pub` const
    re-exported from `sources/mod.rs`; `tests/contract.rs` is an integration test and can only see
    `pub` items, so a private const would not compile there and a retyped array would compile and
    lie.
  - *Status:* SATISFIED. `web/src/lib/types.ts:144` exports the five-member `HiddenLabel`;
    `contract-check.ts:94`-`:96` is
    `Assert<Equal<keyof typeof contract.vocab.hiddenLabels, HiddenLabel>>`, and `Equal<>` at `:73`
    is bidirectional, so both a dropped and an added member fail. `contract.rs:417`-`:420` builds
    `hidden_label_map` by iterating `HIDDEN_LABELS`, not a retyped array. Resolution: `HIDDEN_LABELS`
    at `contract.rs:418` resolves through the file's `use mortar_core::sources::HIDDEN_LABELS`
    (`contract.rs:37`, step 4 imported) to the re-export at `sources/mod.rs:16` and thence to
    `pub const HIDDEN_LABELS: [&str; 5]` at `sources/bluesky.rs:100`. No shadow: that name exists
    nowhere else in the crate. Probe: renaming `"porn"` to `"pornography"` in the Rust const fails
    `cargo test -p mortar-core --test contract` at the `:452` fixture comparison; restored and green.

- **O3 · The fixture diff is exactly the three things and carries task 10's keys.**
  - *Claim:* the committed `contract.json` diff adds `query.target` and `vocab.hiddenLabels` and
    rewords `errors.bad_request.message` to the literal task 13 shipped, nothing else, and the file
    still carries `errors.feed_not_found`.
  - *Evidence to collect:* read the diff line by line. Run
    `python3 -c "import json;d=json.load(open('server/crates/mortar-core/tests/fixtures/contract.json'));print(list(d['errors']), list(d['query']), list(d['vocab']))"`
    and confirm five error codes, three query keys and two vocab keys.
  - *Checks:* the reworded message must equal what the engine emits. Read
    `tests/contract.rs`'s `errors()` instance against `feed.rs`'s `FeedTarget::from_query` error arm
    and `error.rs`'s `variants()`; all three name both parameters, and the one-task window where the
    fixture pinned the older wording is now closed.
  - *Status:* SATISFIED. `jj diff --git` on the fixture is 15 lines in two hunks: the `server` and
    `wasm` `bad_request` messages reworded to `"bad request: actor or feed"`, the new
    `query.target` object (`actor`, `feed`), and the new `vocab.hiddenLabels` object (five labels).
    Nothing else. The key inventory prints `errors: ['actor_not_found', 'bad_request',
    'feed_not_found', 'login_required', 'upstream']`, `query: ['intent', 'mode', 'target']`,
    `vocab: ['hiddenLabels', 'videoSource']`: five, three and two. Task 10's
    `errors.feed_not_found` survives with both envelopes byte-identical (message
    `"feed not found: at://did:plc:nobody/app.bsky.feed.generator/gone"`, wasm status 404). The
    three sources of the wording agree: `contract.rs:245` `AppError::BadRequest("actor or feed")`,
    `feed.rs:38` `NO_TARGET = "actor or feed"` raised at `feed.rs:79`, and `error.rs:108`-`:110`.
    The window task 13 left open is closed.

- **O3b · The `bad_request` message is honest for both callers, and no variant was added.**
  - *Claim:* `AppError::BadRequest`'s `#[error(...)]` Display at `error.rs:5` has been reworded so
    that neither-parameter-present and an unparseable `?feed=` both read truthfully, the payload is
    still a `&'static str`, and a mortar-core test asserts both messages.
  - *Evidence to collect:* read the attribute and both messages. Run the mortar-core test that
    exercises the two `BadRequest` callers (`FeedTarget::from_query`'s Err arm and the `FeedRef`
    rejection) and read the strings it asserts. Read `error.rs:90` and `:106` and confirm both pinned
    envelope arrays carry the new wording, and `contract.json`'s `errors.bad_request.message` with
    them.
  - *Checks:* resolve the shape of the fix. The old Display, `missing required parameter: {0}`,
    is a lie for a parameter that was present and malformed, and `&'static str` cannot carry the
    offending value, so the honest wording has to come from the Display rather than from the payload.
    Confirm the fix is **not** a second variant: `AppError::variants()` must still hold exactly one
    `BadRequest`, `ALL_CODES` must be unchanged, and `code_key`'s match must have gained no arm. A
    new variant would walk task 10's whole forcing chain again and would make this the fourth touch
    of a fixture the plan regenerates exactly three times. Then confirm the ordering held: task 13
    changed only the payload literal and left `contract.rs`'s `errors()` alone, so the fixture pinned
    the older wording for exactly one task and this regeneration closes that window.
  - *Status:* SATISFIED. `error.rs:13` reads `#[error("bad request: {0}")]` over the unchanged
    `BadRequest(&'static str)`. The new test `feed::feed_wall_tests::
    a_bad_request_reads_honestly_for_both_of_its_callers` (`feed.rs:1457`-`:1489`) raises both
    errors through their real call sites, `FeedTarget::from_query(None, None)` and
    `handle_feed(.., FeedTarget::Feed("nonsense"), ..)` against a wiremock with nothing mounted, and
    asserts `"bad request: actor or feed"`, `"bad request: feed"` and that neither contains
    `"missing"`; `cargo nextest run -E 'test(honestly)'` PASSes. No variant was added: the pinned
    table `pinned_wire()` at `error.rs:101` is still `[(AppError, &str, &str); 5]` with one
    `BadRequest`, `variants()` at `:138` derives from it, `ALL_CODES` at `contract.rs:230` is still
    the same five codes, and `code_key`'s match at `:260`-`:266` still has five arms.
    `error.rs:109`-`:110` carry the new wording in both pinned envelopes, and the fixture with them.
    `web/src/service-worker.ts:259`'s hand copy reads `"bad request: actor or feed"`, which is what
    `from_query` now emits for the neither-present case: read, since nothing in the repo compares
    them, and `grep -rn "missing required parameter"` finds no live string left, only the two
    comments that explain the reword and the plan/spec prose.

- **O4 · A one-sided rename fails the gate.**
  - *Claim:* dropping a member from the TS union (for example `porn` from `HiddenLabel`) makes
    `pnpm check:ci` fail.
  - *Evidence to collect:* make the change, run `cd web && pnpm check:ci`, record the failure, then
    revert. This is the probe that distinguishes a live guard from a present one.
  - *Status:* SATISFIED. Run by this gate in a scratch copy of the tree: dropping `"porn"` from
    `types.ts`'s `HiddenLabel` fails the typecheck with
    `src/lib/contract-check.ts(95,3): error TS2344: Type 'false' does not satisfy the constraint
    'true'`; restored, green again. The other three directions bite too: renaming the fixture's
    `"porn"` key fails at `contract-check.ts(95,3)`, renaming the fixture's `"feed"` target key to
    `"generator"` fails at `contract-check.ts(86,3)`, and `FeedTargetKind` reduced to `"actor"`
    fails at the same line. All four probes are recorded in the implementer's commit message.

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check` and `cd server && cargo nextest run`. Note the spec-set
    half lands in task 19.
  - *Status:* SATISFIED. `just check` exits 0 from the workspace: guard-dashes, guard-autoplay,
    guard-toolchain, fmt-check, guard-wasm, lint (oxlint reports only the four pre-existing warnings
    in `FeedGrid.svelte` and `service-worker.ts:277`, knip clean, clippy clean), 147 rust tests, tsc
    on both tsconfigs, 45 vitest. `cd server && cargo nextest run` is 147 passed, 0 skipped, and
    `cd web && pnpm check:ci` exits 0 from the same checkout. The spec-set half is task 19's, as
    this obligation notes and as task 10 already established: `.specs/06-wire-contract.md:170` still
    lists neither `feed_not_found` nor the new vocabulary rows.

- **O6 · Reviewable: one list, two languages, a machine between them.**
  - *Claim:* the hidden-label list appears exactly once in Rust and once in TypeScript, with the
    fixture and `contract-check.ts` comparing them.
  - *Evidence to collect:* run
    `grep -rn 'graphic-media' server/crates/mortar-core/src web/src` and confirm exactly two source
    hits (the Rust const and the TS union), plus the generated fixture.
  - *Status:* SATISFIED. That grep returns exactly two hits, `sources/bluesky.rs:105` inside
    `HIDDEN_LABELS` and `types.ts:144` inside `HiddenLabel`, plus `contract.json:406` in the
    generated fixture. `grep -rn HIDDEN_LABELS` over both trees finds one array, its one consumer
    (`bluesky.rs:134`, `hidden_from_logged_out`), the re-export, and the fixture generator. The
    machine between the two languages is `contract.rs:417`-`:420` writing the keys and
    `contract-check.ts:94`-`:96` reading them, both probed live above.

## Regression check

- `contract-check.ts:78`-`:83`'s four existing vocabulary assertions. Trace: `ErrorCodesMatch`,
  `IntentVocabularyMatches`, `ModeVocabularyMatches` and `VideoSourcesMatch` all still pass :
  PRESERVED. All four survive verbatim at `:79`-`:90` (the diff only inserts between them), and
  `pnpm check:ci` exits 0 with `Assert<T extends true>` still gating each. `BrickKindsMatch` and the
  fourteen field-set assertions are untouched and green with them.
- `sources/bluesky.rs`'s `hidden_from_logged_out` reads `HIDDEN_LABELS`. Trace: making the const
  `pub` changes no behaviour; the six author-feed tests still pass : PRESERVED. `bluesky.rs:134`
  still reads the same array through the same `.contains`, visibility is not a value, and the whole
  147-test suite passes, the author-feed and feed-wall cases included. `just guard-wasm` is green,
  so the `pub` re-export did not disturb the wasm32 build either.

## Residue

- Task 23 regenerates this fixture a third time. If it is run on a tree that does not carry this
  task's keys, they vanish silently and still pass `cargo test` on that branch. Not this task's
  obligation, but the reason its diff must be read rather than trusted.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1, O1b, O2, O3, O3b, O4, O5 and O6 are all SATISFIED with evidence collected by this gate,
including five live break-and-restore probes run in a scratch copy (two on each side of the wire
plus the knip export probe), and both named regression traces are PRESERVED.
