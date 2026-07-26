# Done Certificate · Task 10: feed_not_found error code

**Task:** [10-feed_not_found_error_code.md](10-feed_not_found_error_code.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 10. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This is wire
> regeneration 1 of 3; the fixture diff is the load-bearing artifact.

## Definition

DONE(Task 10) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** A fifth error code walks the whole forcing chain from `error.rs` to `types.ts` in
  one commit, leaving the repo green.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break the four existing error codes' pinned wire strings, the
  service worker's `satisfies MortarErrorCode` uses, or `feed.svelte.ts:197 #fail`'s classification.

## Obligations

- **O1 · The Rust half of the forcing chain is complete.**
  - *Claim:* `AppError::FeedNotFound(String)` maps to `(404, "feed_not_found")`; `variants()` is
    `[AppError; 5]`; both `wasm_envelope_is_pinned_per_variant` and
    `server_envelope_is_pinned_per_variant` carry the new literal strings; `ALL_CODES` is
    `[&str; 5]`; `code_key` has a matching arm; `errors()` carries an instance; and the key-set
    assert in `contract()` passes.
  - *Evidence to collect:* read `error.rs` and `tests/contract.rs` at the named sites. Run
    `cd server && cargo nextest run -p mortar-core`, expect green.
  - *Checks:* `code_key` asserts `code == error.status_and_code().1`. Confirm the new arm indexes
    `ALL_CODES[4]` and that the assertion still fires for it, so the fixture key cannot diverge from
    the wire code.
  - *Status:* unverified

- **O2 · The fixture diff contains only the new error entry.**
  - *Claim:* the committed `contract.json` diff adds `errors.feed_not_found.server` and
    `errors.feed_not_found.wasm` and nothing else.
  - *Evidence to collect:* read the diff of
    `server/crates/mortar-core/tests/fixtures/contract.json` line by line. `UPDATE_FIXTURE=1`
    rewrites the file wholesale, so a regeneration on a stale tree would silently drop keys; confirm
    the pre-existing `bricks`, `pages`, `query` and `vocab` objects are byte-identical.
  - *Status:* unverified

- **O3 · The TypeScript half follows and the existing guard proves it.**
  - *Claim:* `MortarErrorCode` in `web/src/lib/types.ts` includes `"feed_not_found"`, and
    `contract-check.ts`'s existing `ErrorCodesMatch` passes with **no edit** to that file.
  - *Evidence to collect:* read `types.ts` and confirm the union has five members. Confirm
    `contract-check.ts` is unchanged in the diff. Run `cd web && pnpm check:ci`, expect clean.
  - *Checks:* `ErrorCodesMatch` is `Equal<keyof typeof contract.errors, MortarErrorCode>`, which is
    bidirectional. Confirm removing the new member makes it fail, so the pass is not vacuous.
  - *Status:* unverified

- **O4 · The unconstructed variant is not a lint failure.**
  - *Claim:* no production code constructs `FeedNotFound` yet, and clippy at `-D warnings` does not
    flag it, because `AppError` is public.
  - *Evidence to collect:* run `cd server && cargo clippy --workspace --all-targets -- -D warnings`,
    expect clean.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check`. Note the spec-set half lands in task 19; record here
    that `06-wire-contract.md` still describes four codes and that the divergence is scheduled.
  - *Status:* unverified

- **O6 · Reviewable: neither half passes without the other.**
  - *Claim:* `cd server && cargo nextest run` and `cd web && pnpm check:ci` are both green from a
    single checkout of this commit.
  - *Evidence to collect:* run both from one checkout. Then, as a one-off probe, revert the
    `types.ts` line and confirm `pnpm check:ci` goes red; restore it.
  - *Status:* unverified

## Regression check

- `web/src/service-worker.ts:253` uses `"bad_request" satisfies MortarErrorCode`. Trace: still
  typechecks after the union grew : (PRESERVED / REGRESSION)
- `web/src/lib/state/feed.svelte.ts:200` and `:204` use `satisfies MortarErrorCode`. Trace: both
  still typecheck, and `#fail`'s else branch still catches the new code as `feed-unavailable` until
  task 16 gives it its own arm : (PRESERVED / REGRESSION)

## Residue

- `06-wire-contract.md` now describes four error codes while the fixture carries five. That
  divergence is deliberate and closes in task 19. Record it rather than fixing it here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
