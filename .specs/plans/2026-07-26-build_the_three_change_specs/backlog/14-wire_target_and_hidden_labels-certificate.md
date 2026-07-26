# Done Certificate · Task 14: wire target and hidden labels

**Task:** [14-wire_target_and_hidden_labels.md](14-wire_target_and_hidden_labels.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O2 · The hidden-label list is compared in both directions.**
  - *Claim:* `types.ts` exports `HiddenLabel` and `contract-check.ts` compares it to
    `keyof typeof contract.vocab.hiddenLabels` bidirectionally.
  - *Evidence to collect:* read the assertion. Confirm the fixture's map is generated from
    `HIDDEN_LABELS` in `contract.rs` rather than retyped, by reading `tests/contract.rs`.
  - *Checks:* resolve `HIDDEN_LABELS` at its use in `contract.rs`. It must be the `pub` const
    re-exported from `sources/mod.rs`; `tests/contract.rs` is an integration test and can only see
    `pub` items, so a private const would not compile there and a retyped array would compile and
    lie.
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O4 · A one-sided rename fails the gate.**
  - *Claim:* dropping a member from the TS union (for example `porn` from `HiddenLabel`) makes
    `pnpm check:ci` fail.
  - *Evidence to collect:* make the change, run `cd web && pnpm check:ci`, record the failure, then
    revert. This is the probe that distinguishes a live guard from a present one.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check` and `cd server && cargo nextest run`. Note the spec-set
    half lands in task 19.
  - *Status:* unverified

- **O6 · Reviewable: one list, two languages, a machine between them.**
  - *Claim:* the hidden-label list appears exactly once in Rust and once in TypeScript, with the
    fixture and `contract-check.ts` comparing them.
  - *Evidence to collect:* run
    `grep -rn 'graphic-media' server/crates/mortar-core/src web/src` and confirm exactly two source
    hits (the Rust const and the TS union), plus the generated fixture.
  - *Status:* unverified

## Regression check

- `contract-check.ts:78`-`:83`'s four existing vocabulary assertions. Trace: `ErrorCodesMatch`,
  `IntentVocabularyMatches`, `ModeVocabularyMatches` and `VideoSourcesMatch` all still pass :
  (PRESERVED / REGRESSION)
- `sources/bluesky.rs`'s `hidden_from_logged_out` reads `HIDDEN_LABELS`. Trace: making the const
  `pub` changes no behaviour; the six author-feed tests still pass : (PRESERVED / REGRESSION)

## Residue

- Task 23 regenerates this fixture a third time. If it is run on a tree that does not carry this
  task's keys, they vanish silently and still pass `cargo test` on that branch. Not this task's
  obligation, but the reason its diff must be read rather than trusted.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
