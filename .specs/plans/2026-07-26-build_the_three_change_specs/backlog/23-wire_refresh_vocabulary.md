# Task 23 · wire refresh vocabulary

**Plan:** [plan.md](../plan.md) · **Certificate:** [23-wire_refresh_vocabulary-certificate.md](23-wire_refresh_vocabulary-certificate.md)

**Implements:** [`changes/2026-07-26-refresh_the_wall.md`](../../../changes/2026-07-26-refresh_the_wall.md) §Proposed changes → `06-wire-contract.md` → What the fixture covers, and the `FeedRefresh` fragment in §Type changes; implementation note 6. Targets [`06-wire-contract.md`](../../../06-wire-contract.md) §What the fixture covers.
**Depends on:** 14, 22
**Produces:** the refresh token joins the query vocabulary, pinned on both sides by the same mechanism as `mode` and `intent`. **This is wire regeneration 3 of 3.**
**Pointers:** `tests/contract.rs:347` (the `GLAZE`/`PREVIEW`/`FREEZE` const block: each token bound **once** and used for both the parser assert and the fixture key), `:361` (the `query` object, which task 14 has already grown a `target` key on). `web/src/lib/types.ts:130` (`FeedMode`, the doc-comment style to copy). `web/src/lib/contract-check.ts:80` (`ModeVocabularyMatches`, the neighbour). `web/knip.json` already lists `contract-check.ts` as an entry, which is why an export consumed only there is not dead code.

## Steps

- [ ] Bind the token once in `contract.rs` as `const REFRESH: &str = "1";` beside the existing three, and assert `refresh_from_query` in both directions: `Some(REFRESH)` is true, `None` is false.
- [ ] Add a `refresh` map to the `query` object keyed by the token, so the fixture carries `query.refresh = {"1": true}` and `keyof` on the web side sees the literal.
- [ ] Rebase on main so the tree already carries tasks 10 and 14's fixture keys, then regenerate: `UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract`.
- [ ] Add `export type FeedRefresh = "1"` to `types.ts` with a doc comment naming `refresh_from_query` in `server/crates/mortar-core/src/feed.rs` as the thing it mirrors.
- [ ] Add `RefreshVocabularyMatches` to `contract-check.ts` beside `ModeVocabularyMatches`.
- [ ] Commit the regenerated fixture and both web files in the **same** commit.

## Definition of done

- [ ] `contract.json` carries `query.refresh` and still carries task 10's `errors.feed_not_found` and task 14's `query.target` and `vocab.hiddenLabels`; the committed diff contains only the new key, reviewed line by line.
- [ ] The guard is proven live, not just present: temporarily changing `FeedRefresh` to any other literal makes `pnpm check:ci` fail with TS2344, and the change is reverted. (`keyof` over the numeric-looking JSON key `"1"` does yield the string literal `"1"`, so the assertion is meaningful rather than vacuously true.)
- [ ] `cd server && cargo nextest run -p mortar-core --test contract` passes **without** `UPDATE_FIXTURE`, against the committed fixture.
- [ ] `pnpm knip` stays green: `FeedRefresh` is exported from `types.ts` and consumed only by `contract-check.ts`, exactly as `Blur` and `CaptionTrack` are today.
- [ ] Meets the repo definition of done (the wire changed, so `contract.json`, `types.ts` and this spec set agree and both `cargo test` and `tsc` pass).
- [ ] Reviewable: run the deliberate-rename experiment above and watch `pnpm check:ci` go red, then revert it.
