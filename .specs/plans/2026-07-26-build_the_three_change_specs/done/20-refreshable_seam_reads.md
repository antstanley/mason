# Task 20 · refreshable seam reads

**Plan:** [plan.md](../plan.md) · **Certificate:** [20-refreshable_seam_reads-certificate.md](20-refreshable_seam_reads-certificate.md)

**Implements:** [`changes/merged/2026-07-26-refresh_the_wall.md`](../../../changes/merged/2026-07-26-refresh_the_wall.md) §Proposed changes → `04-sources-and-moderation.md` → Per source → Bluesky and → Failure semantics, and → `05-caching-and-persistence.md` → The caches; implementation note 1. Targets [`04-sources-and-moderation.md`](../../../04-sources-and-moderation.md) §Failure semantics.
**Depends on:** none
**Produces:** the two fast content reads become bypassable with a cached-yield fallback, changing no observable behaviour yet because every caller passes a literal `false`.
**Pointers:** `sources/fetch.rs:143` (`author_feed_cached` signature), `:147` (its cache read), `:155` (its transient arm), `:177` (`image_feed_cached` signature), `:181` (its cache read), `:187` (its transient arm, which the change spec's note 1 names; `:188` is the `tracing::debug!` inside it). `algo/fill.rs:232`-`:236` (the single call site). **`fetch.rs:373` is a bare `#[cfg(test)]` module**, and `wiremock` and `tokio` are `cfg(not(target_arch = "wasm32"))` dev-dependencies (`Cargo.toml:41`), while `just guard-wasm` compiles test targets with `--all-targets`.

## Steps

- [ ] Give both readers a `refresh: bool` third argument. When set, do not consult the cache at the top, and let the AppView answer overwrite whatever was there.
- [ ] In the transient-failure arm, return the previously cached entry when one exists and `None` when none does. Perform that cache lookup **lazily inside the arm**, so the non-refreshed happy path still pays exactly one lookup.
- [ ] Pass a literal `false` from `fan_out_authors`, so this task changes nothing observable.
- [ ] Add the new tests in a **second** module gated `#[cfg(all(test, not(target_arch = "wasm32")))]`, matching `feed.rs:214`, `snapshot.rs:618` and `cohort.rs:123`. A bare `#[cfg(test)]` here breaks the wasm32 build without breaking any test.

## Definition of done

- [ ] A refreshed read reaches the AppView even when a fresh entry is cached, and the new answer replaces the old one.
- [ ] A refreshed read whose fetch fails transiently (three 5xx) returns the cached yield rather than `None`, so the author is still counted as answered and the refreshed wall is never thinner than the one it replaced.
- [ ] A refreshed read with nothing cached behind it behaves exactly like a cold one: `None` on a transient failure.
- [ ] A refreshed read leaves the cache dirty, which is what makes the claim in `05-caching-and-persistence.md` about the next persist cycle true.
- [ ] Meets the repo definition of done (negative-space test for the nothing-cached case, `just guard-wasm` green as a first-class check rather than folded into `just check`, `just check` green).
- [ ] Reviewable: `cd server && cargo nextest run -p mortar-core` and `just guard-wasm` are both green, and the diff shows every caller passing `false`, so the review is about the seam alone.
