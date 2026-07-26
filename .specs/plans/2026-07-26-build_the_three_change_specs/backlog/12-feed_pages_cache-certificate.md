# Done Certificate · Task 12: feed_pages cache

**Task:** [12-feed_pages_cache.md](12-feed_pages_cache.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 12. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 12) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** One page of a feed generator, cached for sixty seconds and never persisted, so the
  preview-then-freeze pair is one network read and a back/forward is free.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break the nine persisted caches, the `persist::VERSION` contract, or
  `architecture-principles.md` rule 1 (core structs take types re-exported from `sources/mod.rs`,
  never a submodule directly).

## Obligations

- **O1 · The cache is deliberately unpersisted, with the reason recorded as a test.**
  - *Claim:* `persist::CACHE_NAMES` is still `[&str; 9]` and does not contain `feed_pages`, and a
    test asserts that absence with a comment saying why.
  - *Evidence to collect:* read `server/crates/mortar-core/src/persist.rs:40`. Run
    `grep -n feed_pages server/crates/mortar-core/src/persist.rs`, expect no hits. Find and run the
    named test.
  - *Checks:* resolve the assertion target: the test must read `CACHE_NAMES` itself, not a local
    copy of the list, or it would pass while the real list drifted.
  - *Status:* unverified

- **O2 · A repeat read issues no second upstream request.**
  - *Claim:* a second call for the same `(uri, cursor)` within the TTL issues no second `getFeed`.
  - *Evidence to collect:* find the wiremock test using `expect(1)` on the `getFeed` mock and run it.
  - *Checks:* trace the cache key: `format!("{uri}\u{1f}{}", cursor.unwrap_or_default())`. Confirm
    the separator is the unit separator U+001F, not a character that could appear in a uri or a
    cursor; a colon or a slash would let two distinct pairs collide.
  - *Status:* unverified

- **O3 · Upstream failures map to the right two errors.**
  - *Claim:* a 400 and a 404 each become `AppError::FeedNotFound` and a 500 becomes
    `AppError::Upstream`, each with its own negative-space test.
  - *Evidence to collect:* run the three named tests. Read the match arm and confirm the
    `400 | 404` pattern rather than a range.
  - *Status:* unverified

- **O4 · Both new bounds are named constants.**
  - *Claim:* the 60 second TTL and the 500 capacity are named constants with their units in the
    name, per the repo's limits discipline.
  - *Evidence to collect:* read `cache.rs` around the `Caches` struct and `Caches::new`; confirm the
    two values are named constants rather than literals, following `LIVE_TTL` at `cache.rs:190`.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wasm32 build still compiles the new tests and the gates are green.
  - *Evidence to collect:* run `just guard-wasm` and `just check`. Read the new test module's
    `cfg` attribute and confirm it is `#[cfg(all(test, not(target_arch = "wasm32")))]`, not a bare
    `#[cfg(test)]`; `wiremock` and `tokio` are `cfg(not(target_arch = "wasm32"))` dev-dependencies
    and `guard-wasm` compiles test targets.
  - *Status:* unverified

- **O6 · Reviewable: the cache is there and the persistence list is not.**
  - *Claim:* `cd server && cargo nextest run -p mortar-core fetch` is green and
    `grep -n feed_pages src/persist.rs` returns nothing.
  - *Evidence to collect:* both commands.
  - *Status:* unverified

## Regression check

- `sources/mod.rs`'s re-export list gains the cache value type. Trace: `cache.rs` names no
  `sources::bluesky` path directly; run
  `grep -n 'sources::bluesky' server/crates/mortar-core/src/cache.rs` and expect no hits :
  (PRESERVED / REGRESSION)
- `persist.rs`'s import/export round trip iterates `CACHE_NAMES`. Trace: the nine existing caches
  still import and export at `VERSION` 4 : (PRESERVED / REGRESSION)

## Residue

- Task 22 adds the `refresh` bypass to `feed_page_cached`. Leaving room for it (a bypass that skips
  the read and inserts as usual) is not an obligation here, but a structure that makes it a two-line
  change is worth noting either way.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
