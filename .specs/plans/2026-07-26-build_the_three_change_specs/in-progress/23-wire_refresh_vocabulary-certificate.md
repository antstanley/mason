# Done Certificate · Task 23: wire refresh vocabulary

**Task:** [23-wire_refresh_vocabulary.md](23-wire_refresh_vocabulary.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · The token is bound once and asserted in both directions.**
  - *Claim:* `contract.rs` binds `const REFRESH: &str = "1";` beside `GLAZE`, `PREVIEW` and `FREEZE`
    and uses it for both the parser assert and the fixture key; the parser is asserted with
    `refresh_from_query(Some(REFRESH))` true and `refresh_from_query(None)` false.
  - *Evidence to collect:* read `tests/contract.rs` around `:347` and `:361`. Confirm the const
    appears in both the assert and the map insertion.
  - *Checks:* resolve `refresh_from_query` from `tests/contract.rs`. It is an integration test, so
    the function must be `pub` and reachable as `mortar_core::feed::refresh_from_query`.
  - *Status:* unverified

- **O3 · The guard is proven live, not merely present.**
  - *Claim:* temporarily changing `FeedRefresh` to any other literal makes `pnpm check:ci` fail with
    TS2344.
  - *Evidence to collect:* make the change, run `cd web && pnpm check:ci`, record the error code,
    then revert. `keyof` over the numeric-looking JSON key `"1"` yields the string literal `"1"`, so
    the assertion is meaningful rather than vacuously true; this probe confirms it.
  - *Status:* unverified

- **O4 · The committed fixture passes without regeneration, and knip stays green.**
  - *Claim:* `cargo nextest run -p mortar-core --test contract` passes against the committed fixture
    with no `UPDATE_FIXTURE`, and `pnpm knip` is green with `FeedRefresh` exported from `types.ts`
    and consumed only by `contract-check.ts`.
  - *Evidence to collect:* run both commands. `web/knip.json` already lists `contract-check.ts` as an
    entry, which is why an export consumed only there is not dead code, exactly as `Blur` and
    `CaptionTrack` are today.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check`. Confirm the fixture and both web files landed in one
    commit; split across commits the repo is red in between.
  - *Status:* unverified

- **O6 · Reviewable: the rename probe goes red and comes back.**
  - *Claim:* a reviewer runs the deliberate-rename experiment and watches `pnpm check:ci` go red,
    then reverts it.
  - *Evidence to collect:* the two runs.
  - *Status:* unverified

## Regression check

- `contract-check.ts`'s five existing assertions (`BrickKindsMatch`, `ErrorCodesMatch`,
  `IntentVocabularyMatches`, `ModeVocabularyMatches`, `VideoSourcesMatch`) plus task 14's two.
  Trace: all seven still pass : (PRESERVED / REGRESSION)
- `contract.rs`'s existing `GLAZE`/`PREVIEW`/`FREEZE` parser asserts. Trace: still present and
  passing : (PRESERVED / REGRESSION)

## Residue

- `06-wire-contract.md` still describes three query keys until task 27 merges the refresh spec.
  Deliberate; record it rather than fixing it here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
