# Done Certificate · Task 12: feed_pages cache

**Task:** [12-feed_pages_cache.md](12-feed_pages_cache.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* `persist.rs:40` reads `pub const CACHE_NAMES: [&str; 9] = [...]`, the same nine names
    (did, follows, author_feed, image_feed, std_docs, pds, streams, profiles, activity); persist.rs is
    not in the diff at all. `grep -n feed_pages server/crates/mortar-core/src/persist.rs` exits 1 with
    no output. The test is `cache::tests::a_feed_page_is_never_persisted` (cache.rs:312-353); ran it,
    PASS, and it also passes inside the full 133-test run. Its doc comment gives the reason in the
    required terms ("imported after a service worker was reaped it would be laid hours later as
    though it were the feed's current head, which is exactly the lie the persistence layer exists to
    avoid") and explains why it sits in cache.rs rather than persist.rs.
  - *Check resolved:* the assertion reads the real list. cache.rs:326 does
    `use crate::persist::{CACHE_NAMES, dirty_cache_names, export_cache};` inside the test body
    (function-resolution step 4, imported) and asserts `!CACHE_NAMES.contains(&"feed_pages")`; there
    is no local shadow named `CACHE_NAMES` in cache.rs (grep shows the only two occurrences in the
    crate are persist.rs:40 and this import). Two further assertions close the same hole from the
    other side: `dirty_cache_names(&caches)` is empty after a `feed_pages` insert (persist.rs:81
    reads only the nine fixed flags), and `export_cache(&caches, "feed_pages")` is `None`
    (persist.rs's match falls through to `_ => None`).
  - *Status:* SATISFIED

- **O2 · A repeat read issues no second upstream request, and two limits never collide.**
  - *Claim:* a second call for the same `(uri, cursor, limit)` within the TTL issues no second
    `getFeed`; and two calls for the same `(uri, cursor)` at **different** limits each issue their
    own, because the limit is part of the key.
  - *Evidence to collect:* find the wiremock test using `expect(1)` on the `getFeed` mock and run it.
    Then find and run the differing-limits test, which expects two upstream requests.
  - *Checks:* trace the cache key:
    `format!("{uri}\u{1f}{limit}\u{1f}{}", cursor.unwrap_or_default())`. Confirm the separator is the
    unit separator U+001F, not a character that could appear in a uri or a cursor; a colon or a slash
    would let two distinct pairs collide. Then confirm the limit is genuinely in the key and not just
    in the request: `Mode::Wall` asks `getFeed` for `PAGE_SIZE` and `Mode::Glaze` asks for 100, so a
    `(uri, cursor)` key serves a glaze request the 24-item page a mixed request cached a moment
    earlier and the image wall runs a quarter as deep, silently, with `expect(1)` still green.
  - *Collected:* both tests live in `sources::fetch::feed_page_tests` (fetch.rs:780+) and both pass.
    `the_second_read_of_a_page_never_reaches_the_appview` mounts one `getFeed` mock through the
    `answers` helper, which ends `.expect(1).mount(server)` (fetch.rs:857), calls
    `feed_page_cached(&state, FEED, None, 24)` twice, and asserts both the payload
    (`post_uri("1")`), the surviving cursor (`next == Some("page2")`) and
    `server.received_requests().len() == 1`. `two_limits_of_one_page_do_not_collide` mounts two
    mocks, `query_param("limit", "24")` answering rkey `mixed` and `query_param("limit", "100")`
    answering rkey `glaze`, each `.expect(1)`, and asserts each call got ITS OWN page plus
    `received_requests().len() == 2`.
  - *Check resolved:* the key at fetch.rs:266 is
    `format!("{feed_uri}\u{1f}{limit}\u{1f}{}", cursor.unwrap_or_default())` - the literal Rust
    escape for U+001F, twice, not a colon or a slash; the limit is genuinely in the key, and the
    comment above it says why. `state.caches.feed_pages.get(&key)` resolves to `TtlCache::get`
    (cache.rs:61, step 4 imported via the `Caches` field type), which is expiry-checked, not
    `HashMap::get`.
  - *Mutation-checked, in a scratch copy of the tree, never in the workspace:* (a) dropping `{limit}`
    from the key makes `two_limits_of_one_page_do_not_collide` FAIL on the payload assertion
    (`left: .../mixed, right: .../glaze`), so the test distinguishes a stale page rather than only
    counting requests; (b) deleting the cache lookup makes
    `the_second_read_of_a_page_never_reaches_the_appview` FAIL (`left: 2, right: 1`); (c) with that
    same mutation AND the explicit counter deleted, wiremock's `expect(1)` alone still fails the test
    on drop ("Number of matched incoming requests: 2"), so the `expect(1)` the DoD names is
    load-bearing and not decoration over a hand-rolled counter.
  - *Status:* SATISFIED

- **O3 · Upstream failures map to the right two errors.**
  - *Claim:* a 400 and a 404 each become `AppError::FeedNotFound` and a 500 becomes
    `AppError::Upstream`, each with its own negative-space test.
  - *Evidence to collect:* run the three named tests. Read the match arm and confirm the
    `400 | 404` pattern rather than a range.
  - *Collected:* three separate tests, all PASS: `a_400_is_a_feed_that_is_not_there`,
    `a_404_is_a_feed_that_is_not_there` (both `match` on `AppError::FeedNotFound(uri)` and assert the
    payload is the feed uri the reader asked for) and `a_500_is_an_upstream_failure` (asserts
    `matches!(failure, AppError::Upstream(_))`). The match arm at fetch.rs:286 is
    `HttpError::Status(400 | 404) => AppError::FeedNotFound(feed_uri.to_string())` - an explicit
    or-pattern, not a range - with `other => AppError::Upstream(other.to_string())` beneath it, so
    `Transport` and `RetriesExhausted` land in `Upstream` too. `AppError` resolves to
    `crate::error::AppError` (fetch.rs:20, step 4 imported); it is the crate's only such type, and
    `error.rs:47` maps `FeedNotFound` to `(404, "feed_not_found")`, the pair the wire fixture pins.
    The 500 test's mock sets `retry-after: 0`, so the http retry loop's three attempts cost no real
    backoff and the final attempt returns `HttpError::Status(500)` (http.rs:168) as intended.
  - *Mutation-checked:* narrowing the arm to `HttpError::Status(404)` in a scratch copy makes
    `a_400_is_a_feed_that_is_not_there` FAIL with `got Upstream("upstream returned 400")`.
  - *Status:* SATISFIED

- **O4 · Both new bounds are named constants.**
  - *Claim:* the 60 second TTL and the 500 capacity are named constants with their units in the
    name, per the repo's limits discipline.
  - *Evidence to collect:* read `cache.rs` around the `Caches` struct and `Caches::new`; confirm the
    two values are named constants rather than literals, following `LIVE_TTL` at `cache.rs:190`.
  - *Collected:* `cache.rs:197` `pub const FEED_PAGE_TTL: Duration = Duration::from_secs(60);`, sited
    directly beneath `LIVE_TTL` and carrying its unit in the `Duration` type exactly as `LIVE_TTL`,
    `STD_DOCS_POSITIVE_TTL` and `STREAMS_NEGATIVE_TTL` do; `cache.rs:202`
    `const FEED_PAGES_MAX_ENTRIES: usize = 500;`, which spells its unit out the way a scalar bound
    must (`MAX_FEED_REF_LEN_BYTES`, `MAX_FUTURE_SKEW_SECS`). Both are used exactly once, at
    `cache.rs:268` `feed_pages: TtlCache::new(FEED_PAGE_TTL, FEED_PAGES_MAX_ENTRIES)`; neither `60`
    nor `500` appears as a literal on the new lines, and each constant's doc comment says the value
    in words and why it is that value.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wasm32 build still compiles the new tests and the gates are green.
  - *Evidence to collect:* run `just guard-wasm` and `just check`. Read the new test module's
    `cfg` attribute and confirm it is `#[cfg(all(test, not(target_arch = "wasm32")))]`, not a bare
    `#[cfg(test)]`; `wiremock` and `tokio` are `cfg(not(target_arch = "wasm32"))` dev-dependencies
    and `guard-wasm` compiles test targets.
  - *Collected:* `just guard-wasm` green, and green again after
    `cargo clean -p mortar-core --target wasm32-unknown-unknown` forced a real recompile (it printed
    `Checking mortar-core` / `Checking mortar-wasm` and finished, exit 0), so the pass is the current
    source and not a stale artifact. `just check` exit 0 end to end: guard-dashes, guard-autoplay,
    guard-toolchain, fmt-check, guard-wasm, lint (knip clean, `cargo clippy --workspace --all-targets
    -- -D warnings` clean, oxlint's only four warnings are the pre-existing ones in FeedGrid.svelte
    and service-worker.ts, both untouched by this diff) and test (133 Rust tests passed, `pnpm
    check:ci` typecheck, 43 vitest). `just test` and `just lint` both depend on `wasm`, so the pkg
    under `web/src/lib/mortar-wasm/` was rebuilt from this Rust and the web typechecked against it,
    which is the guidelines' `just wasm` bullet. The new module's attribute at fetch.rs:779 is
    `#[cfg(all(test, not(target_arch = "wasm32")))]`, with a comment naming wiremock and tokio as the
    reason; the never-persisted test went into cache.rs's `mod tests`, already carrying the identical
    gate at cache.rs:273. No bare `#[cfg(test)]` was added anywhere in the diff.
  - *Status:* SATISFIED

- **O6 · Reviewable: the cache is there and the persistence list is not.**
  - *Claim:* `cd server && cargo nextest run -p mortar-core fetch` is green and
    `grep -n feed_pages src/persist.rs` returns nothing.
  - *Evidence to collect:* both commands.
  - *Collected:* exercised both. `cd server && cargo nextest run -p mortar-core fetch` →
    `15 tests run: 15 passed, 118 skipped`, the six new `feed_page_tests` among them, and re-run
    green after `cargo clean -p mortar-core` forced a full rebuild from the workspace source.
    `grep -n feed_pages src/persist.rs` (from `server/crates/mortar-core`) → no output, exit 1.
  - *Status:* SATISFIED

## Regression check

- `sources/mod.rs`'s re-export list gains the cache value type. Trace: `cache.rs` names no
  `sources::bluesky` path directly; run
  `grep -n 'sources::bluesky' server/crates/mortar-core/src/cache.rs` and expect no hits :
  **PRESERVED**. The grep exits 1 with no output. `sources/mod.rs:16` now reads
  `pub use bluesky::{AuthorYield, FeedPage, Follow};` (additive, nothing removed or renamed) and
  cache.rs:19 takes `FeedPage` from `crate::sources`, beside the yield types it already took there.
  Architecture rule 1 holds.
- `persist.rs`'s import/export round trip iterates `CACHE_NAMES`. Trace: the nine existing caches
  still import and export at `VERSION` 4 : **PRESERVED**. persist.rs is not in the diff; `VERSION`
  is still 4 and `CACHE_NAMES` still `[&str; 9]`. Its round-trip tests
  (`round_trip_restores_live_entries` and the rest of `persist::tests`) pass inside the 133. Nothing
  is stranded by a cache outside the list: `dirty_cache_names` reads a fixed nine-flag array, so a
  `feed_pages` insert never enters the persist cycle, `export_cache`/`import_cache` fall through to
  `None`/no-op for the name, and the service worker only ever writes keys for the names
  `cache_names()` returns (mortar-wasm/src/lib.rs:38, straight from `CACHE_NAMES`). I checked the
  `idbSweepStale` claim rather than repeating it: web/src/service-worker.ts:94 deletes every
  `mortar-cache:*` key whose suffix is not in that same set, so even a hypothetical orphan is reaped;
  in fact none can exist, because no `feed_pages` key is ever written.
- Additional trace, not on the certificate but on the task brief's regression surface: `Caches::new`
  (cache.rs:255-270) gains exactly one line and every pre-existing field keeps its own TTL and
  capacity, unmoved and unrenamed (verified line by line against the diff). `refresh_fallback` and
  the two `refresh`-taking readers from task 20 are untouched, and all five `refresh_tests` pass :
  **PRESERVED**.

## Residue

- Task 22 adds the `refresh` bypass to `feed_page_cached`. Leaving room for it (a bypass that skips
  the read and inserts as usual) is not an obligation here, but a structure that makes it a two-line
  change is worth noting either way.
  - Observed: the room is there. The lookup is a standalone
    `if let Some(cached) = state.caches.feed_pages.get(&key).await { return Ok(cached); }` at the top
    of the function, the exact shape `author_feed_cached` grew `!refresh &&` in front of, and the
    insert below it is unconditional. One argument and one `!refresh &&`.
- Two deviations, neither an obligation and neither a defect, recorded so the merge is deliberate:
  (1) the function returns `FeedPage` rather than the change spec's implementation-note-4 tuple
  `(Arc<AuthorYield>, Option<String>)`. It is the same pair, and the task's own Step 1 is what asks
  for the named value type; the DoD names no signature. (2) No canonical spec page is edited and no
  changeset is added, matching the plan (the change spec's blocks land at task 19, and nothing
  user-visible ships until `feed_page_cached` has a caller at task 13).
- Edge cases noted, none of them defects against this task's contract: a genuinely simultaneous
  preview and freeze can both miss and both fetch, since the reader is a plain `get` rather than
  `get_or_insert_with` (the same property `author_feed_cached` has, and the freeze arrives hundreds
  of milliseconds behind the preview in practice); failures are not negatively cached, so a bad feed
  reference costs one AppView call per poll; and `Some("")` shares a key with `None`, which is what
  the spec's own `cursor.unwrap_or_default()` prescribes and is upstream-equivalent anyway.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED on evidence collected here (the six new tests and the whole
133-test suite run green from a forced rebuild, `just check` exits 0, `just guard-wasm` recompiles
wasm32 `--all-targets` clean, and the Reviewable pair was exercised), the two named regression traces
are PRESERVED with `persist.rs` untouched at `VERSION` 4 and `CACHE_NAMES` still `[&str; 9]`, and
three scratch-copy mutations confirm the limit-in-key, cache-hit and 400-mapping obligations are
genuinely held by their tests rather than passing by luck.
