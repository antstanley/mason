# Task 12 · feed_pages cache

**Plan:** [plan.md](../plan.md) · **Certificate:** [12-feed_pages_cache-certificate.md](12-feed_pages_cache-certificate.md)

**Implements:** [`changes/merged/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/merged/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `05-caching-and-persistence.md` → The caches; implementation notes 4 and 5. Targets [`05-caching-and-persistence.md`](../../../05-caching-and-persistence.md) §The caches and [`01-domain-model.md`](../../../01-domain-model.md) §Required query patterns.
**Depends on:** 10, 11
**Produces:** one page of a feed generator, cached for sixty seconds and never persisted, so the preview-then-freeze pair is one network read and a back/forward is free.
**Pointers:** `sources/fetch.rs:177` (`image_feed_cached`, the neighbour the new function sits beside). `cache.rs:194` (the `Caches` struct), `:236` (`Caches::new`). `persist.rs:40` (`CACHE_NAMES`, a `[&str; 9]` that must stay 9). `sources/mod.rs:15` (the re-export line). `architecture-principles.md` rule 1: `cache.rs` takes the yield types re-exported from `sources/mod.rs`, never a submodule directly.

## Steps

- [ ] Define a small value type in `sources/` carrying `Arc<AuthorYield>` plus the next upstream cursor, and re-export it from `sources/mod.rs` so `cache.rs` never names `sources::bluesky`.
- [ ] Add `Caches.feed_pages` with a named 60 second TTL constant and a named 500 capacity constant, created in `Caches::new`.
- [ ] Add `feed_page_cached(state, uri, cursor, limit)` to `sources/fetch.rs`, keyed on `format!("{uri}\u{1f}{limit}\u{1f}{}", cursor.unwrap_or_default())`. **The limit is part of the key**: `Mode::Wall` asks `getFeed` for `PAGE_SIZE` and `Mode::Glaze` asks for 100, so a key of `(uri, cursor)` alone serves a glaze request the 24-item page a mixed request cached a moment earlier, and the image wall silently runs a quarter as deep.
- [ ] Map a 400 or 404 from `getFeed` to `AppError::FeedNotFound` and every other failure to `AppError::Upstream`.
- [ ] Leave `feed_pages` out of `persist::CACHE_NAMES` and add a test naming the reason.

## Definition of done

- [ ] `CACHE_NAMES` is still `[&str; 9]`, and a test asserts `feed_pages` is absent from it with a comment saying why: a persisted ranking would be laid hours later as though it were fresh.
- [ ] A second call for the same `(uri, cursor, limit)` issues no second upstream request, asserted with a wiremock `expect(1)`.
- [ ] Two calls for the same `(uri, cursor)` at **different** limits do not collide: the second issues its own upstream request and gets its own page, asserted with a wiremock expectation of two. Without this the glaze wall is served the mixed wall's 24 items and nothing anywhere says so.
- [ ] A 400 and a 404 each become `FeedNotFound` and a 500 becomes `Upstream`, each its own negative-space test.
- [ ] Both new bounds are named constants with their units in the name, per the repo's limits discipline.
- [ ] Meets the repo definition of done (`just guard-wasm` green, `just check` green, new wiremock tests in a module gated `#[cfg(all(test, not(target_arch = "wasm32")))]`).
- [ ] Reviewable: `cd server && cargo nextest run -p mortar-core fetch` is green, and `grep -n feed_pages src/persist.rs` returns nothing.
