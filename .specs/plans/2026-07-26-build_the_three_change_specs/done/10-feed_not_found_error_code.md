# Task 10 · feed_not_found error code

**Plan:** [plan.md](../plan.md) · **Certificate:** [10-feed_not_found_error_code-certificate.md](10-feed_not_found_error_code-certificate.md)

**Implements:** [`changes/merged/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/merged/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `06-wire-contract.md` → Errors, and the `MortarErrorCode` fragment in §Type changes; implementation notes 7 and part of 10. Targets [`06-wire-contract.md`](../../../06-wire-contract.md) §Errors.
**Depends on:** none
**Produces:** a fifth error code walks the whole forcing chain from `error.rs` to `types.ts` in one commit, leaving the repo green. **This is wire regeneration 1 of 3.**
**Pointers:** `error.rs:4` (the enum), `:37` (`status_and_code`), `:76` (`variants()`, an `[AppError; 4]`), `:90` and `:106` (both pinned expected arrays). `tests/contract.rs:227` (`ALL_CODES`, a `[&str; 4]`), `:236` (`errors()`), `:251` (`code_key`, an exhaustive match). `tests/fixtures/contract.json` `errors` object. `web/src/lib/types.ts:137` (`MortarErrorCode`). `contract-check.ts:78`'s `ErrorCodesMatch` needs no edit.

## Steps

- [ ] Add `AppError::FeedNotFound(String)` mapping to `(404, "feed_not_found")`.
- [ ] Grow `variants()` to `[AppError; 5]` and add the literal envelope strings to both `wasm_envelope_is_pinned_per_variant` and `server_envelope_is_pinned_per_variant`. **Neither is forced, so verify both by reading.** `variants()` (`:76`) is a hand-written array in the test module, and both `expected` arrays are consumed through `variants().iter().zip(expected)` (`:97`, `:113`); `zip` stops at the shorter side, so a five-entry `variants()` against a four-entry `expected` leaves both tests green with the new variant's wire strings unpinned. Consider folding all three into one `[(AppError, &str, &str); N]` table so the length forces them together.
- [ ] Grow `ALL_CODES` to `[&str; 5]`, add the `code_key` arm and an instance to `errors()`.
- [ ] Rebase on main, then regenerate: `UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract`.
- [ ] Add `"feed_not_found"` to `MortarErrorCode` in `web/src/lib/types.ts` until `pnpm check:ci` is green again.

## Definition of done

- [ ] `variants()` is `[AppError; 5]`, both pinned envelope tests carry the new literal strings **verified by reading rather than by a green run**, and the key-set assert in `contract()` passes. The chain that genuinely forces is `status_and_code`'s exhaustive match, then `contract.rs`'s `code_key` match, its constant index into `ALL_CODES`, and the key-set assert; `error.rs`'s own two arrays are outside it.
- [ ] The committed `contract.json` diff contains **only** `errors.feed_not_found.{server,wasm}`, reviewed line by line rather than trusted to `UPDATE_FIXTURE`.
- [ ] `contract-check.ts`'s existing `ErrorCodesMatch` passes with no edit to that file, which is what proves the fixture and the TS union agree in both directions.
- [ ] No production code constructs the variant yet, which is fine: `AppError` is public, so clippy at `-D warnings` does not flag it.
- [ ] Meets the repo definition of done (the wire changed, so `contract.json`, `types.ts` and this spec set all agree, and both `cargo test` and `tsc` pass in the same commit).
- [ ] Reviewable: `cd server && cargo nextest run` and `cd web && pnpm check:ci` are both green from a single checkout of this commit; neither passes without the other's half.
