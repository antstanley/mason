# Done Certificate · Task 20: refreshable seam reads

**Task:** [20-refreshable_seam_reads.md](20-refreshable_seam_reads.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 20. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task
> changes no observable behaviour; the review is about the seam alone.

## Definition

DONE(Task 20) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The two fast content reads become bypassable with a cached-yield fallback, changing
  no observable behaviour yet because every caller passes a literal `false`.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not change the non-refreshed path: same cache hit behaviour, same
  `None`-on-transient-failure, same refusal-caches-as-empty behaviour, and exactly one cache lookup
  on the happy path.

## Obligations

- **O1 · A refreshed read bypasses a warm cache and overwrites it.**
  - *Claim:* with `refresh: true` and a fresh entry already cached, the AppView is still called and
    the new answer replaces the old one.
  - *Evidence to collect:* run the named wiremock test. Confirm it seeds the cache, calls with
    `refresh: true`, asserts the mock was hit, and then reads the cache to confirm the new value.
  - *Checks:* resolve which `get` is skipped. `author_feed_cached` has one at `:147` and
    `image_feed_cached` one at `:181`; the fallback lookup added inside the transient arm is a
    *different* call and must not be the one guarded by the flag.
  - *Status:* SATISFIED. Ran `sources::fetch::refresh_tests::a_refreshed_read_reaches_past_a_fresh_cache_entry`
    (PASS, 0.23s): it seeds the cache from a live mock, swaps the mock's answer, proves an ordinary
    read is still served the OLD post, proves the refreshed read returns the NEW post, and reads
    `state.caches.author_feed.get()` back to prove the new post is what stayed behind.
    `a_refreshed_image_read_bypasses_and_falls_back_to_its_own_cache` (PASS, 1.73s) proves the same
    for the glaze read against its own cache. Check: the `get` the flag guards is the
    top-of-function one, now `fetch.rs:182` (author) and `:217` (image), each written
    `if !refresh && let Some(cached) = ...`. The fallback `get` is a different call at `:162`,
    inside `refresh_fallback`, reached only from the transient arm and only when `refresh` is set.

- **O2 · A refreshed read that fails transiently falls back to the cached yield.**
  - *Claim:* three 5xx responses under `refresh: true` with an entry cached returns that entry rather
    than `None`, so the author is still counted as answered and the refreshed wall is never thinner.
  - *Evidence to collect:* run the named test. Confirm the mock returns 5xx enough times to exhaust
    `http.rs`'s retries, and that the returned value is the seeded one.
  - *Checks:* resolve `transient()` at `fetch.rs:132`. It covers `Transport`, `RetriesExhausted`,
    429 and 5xx. Confirm the fallback sits in that arm, not in the non-transient arm, which must
    still cache an empty yield.
  - *Status:* SATISFIED. Ran `a_refreshed_read_that_fails_falls_back_to_the_cached_yield` (PASS,
    1.53s): it seeds the cache, resets the mock to an always-503 responder so `http.rs`'s
    three-attempt loop exhausts and returns `HttpError::Status(503)`, then asserts the refreshed
    read returns the seeded post rather than `None`, and that the older entry survives in the
    cache. The 1.5s runtime is the real backoff sleeps, which is evidence the retries were spent.
    Check: `transient()` resolves to the module-level fn at `fetch.rs:133` (`Transport`,
    `RetriesExhausted`, 429, `>= 500`), and the fallback call sits inside that arm only
    (`:192` author, `:225` image). The non-transient arm is untouched and still builds and inserts
    an empty `AuthorYield`. Because the fallback returns `Some`, `fan_out_authors` counts the
    author as answered.

- **O3 · A refreshed read with nothing cached behaves exactly like a cold one.**
  - *Claim:* `refresh: true` with an empty cache returns `None` on a transient failure.
  - *Evidence to collect:* run the named negative-space test.
  - *Status:* SATISFIED. Ran `a_refreshed_read_with_nothing_cached_fails_like_a_cold_one` (PASS,
    1.73s): a refreshed read against an always-503 AppView with an empty cache returns `None`, and
    the cache is still empty afterwards, so the blip is not remembered as "this author posts
    nothing". The code path is `refresh_fallback` returning whatever `TtlCache::get` returns, and
    `cache.rs:61` returns `None` for a missing entry and prunes-then-returns `None` for an expired
    one.

- **O4 · A refreshed read marks the cache dirty.**
  - *Claim:* after a refreshed read the cache reports dirty, which is what makes the persistence
    claim in `05-caching-and-persistence.md` true.
  - *Evidence to collect:* run the named test asserting `caches.image_feed.is_dirty()` (or the
    equivalent for `author_feed`).
  - *Status:* SATISFIED. Ran `a_refreshed_read_leaves_the_cache_dirty` (PASS, 0.22s): a cold read
    dirties the cache, `take_dirty()` stands in for a persist cycle and clears it, and the
    refreshed read that follows leaves `state.caches.author_feed.is_dirty()` true. Traced the
    write path: a refreshed read takes the same `insert` at `fetch.rs:199` as any other, and
    `TtlCache::insert` delegates to `insert_with_ttl` (`cache.rs:169`), which sets the dirty flag.

- **O5 · Meets the repo definition of done.**
  - *Claim:* negative-space test present for the nothing-cached case, and the gates are green with
    `guard-wasm` run as a first-class check.
  - *Evidence to collect:* run `cd server && cargo nextest run -p mortar-core`, then `just guard-wasm`
    on its own, then `just check`. Read the new test module's `cfg` attribute and confirm it is
    `#[cfg(all(test, not(target_arch = "wasm32")))]`; `fetch.rs:373` is a bare `#[cfg(test)]` module
    and adding wiremock tests there would break the wasm32 build without breaking any test.
  - *Status:* SATISFIED. `cd server && cargo nextest run -p mortar-core`: 121 tests run, 121
    passed, 0 skipped, the five new `refresh_tests` cases among them. `just guard-wasm` on its
    own: green, and green again after touching `fetch.rs` to force a real recompile of
    `mortar-core` and `mortar-wasm` for `wasm32-unknown-unknown --all-targets`, so the pass is not
    a stale fingerprint. `just check`: exit 0 (guard-dashes, guard-autoplay, guard-toolchain,
    fmt-check, guard-wasm, wasm, lint, test, which is cargo nextest 121 passed plus
    `pnpm check:ci` plus vitest 43 passed). The new module carries
    `#[cfg(all(test, not(target_arch = "wasm32")))]` at `fetch.rs:476`, with a comment above it
    saying why it is not folded into the bare `#[cfg(test)]` module at `:409`. The negative-space
    case is present (O3). No new bound or constant was introduced, the wire is unchanged (the
    `wire_contract_matches_the_committed_fixture` test passes), and `just check` runs `wasm` and
    then typechecks the web app against it.

- **O6 · Reviewable: every caller passes `false`, so the seam is reviewed alone.**
  - *Claim:* the diff shows `fan_out_authors` passing a literal `false` at its single call site pair,
    so no behaviour changes and the review is about the seam.
  - *Evidence to collect:* read `algo/fill.rs` around `:232` and confirm both branches pass `false`.
    Run `grep -rn 'author_feed_cached\|image_feed_cached' server/crates/mortar-core/src` and confirm
    every non-test call passes `false`.
  - *Status:* SATISFIED. Exercised both halves. `algo/fill.rs:234`-`:238`: both branches of the
    `deep_media` choice pass a literal `false`
    (`fetch::image_feed_cached(&state, &author.did, false)` and
    `fetch::author_feed_cached(&state, &author.did, false)`), with a comment saying what the
    argument is and why it is false. `grep -rn 'author_feed_cached\|image_feed_cached'` over
    `server/`, `web/` and `.specs/` returns exactly two non-test call sites, both in `fill.rs`,
    both passing `false`; the rest are the two definitions, a doc-comment mention, and the new
    tests. No default argument was introduced (Rust has none), so no caller can omit the flag.
    The two commands this item names are green: `cargo nextest run -p mortar-core` (121 passed)
    and `just guard-wasm`.

## Regression check

- `algo/fill.rs:219 fan_out_authors` is the only production caller. Trace: a cold graph wall still
  fans out and lays the same bricks; run `cd server && cargo nextest run` : PRESERVED.
  `feed::tests::a_feed_cursor_on_the_graph_wall_lays_a_fresh_wall` drives `handle_feed` for
  `did:plc:viewer` through `fill` into `fan_out_authors`, which calls
  `author_feed_cached(&state, "did:plc:friend", false)` against a wiremock AppView and lays the
  friend's post; it passes, as does the whole workspace run (`cargo nextest run`: 121 passed).
  `fan_out_authors` itself is private to `algo/fill.rs`, its own signature is unchanged, and its
  two callers (`fill` and `extend`) are untouched.
- Happy-path lookup count: read the non-refreshed path and confirm it still performs exactly one
  cache `get`, not two : PRESERVED. With `refresh == false` the only `get` on the success path is
  the top-of-function one (`fetch.rs:182`, `:217`); the fallback lookup is inside
  `refresh_fallback`, which returns `None` at `:157` before touching the cache when `refresh` is
  false, so the ordinary path pays one lookup on a hit or a miss and zero extra on a failure.

## Residue

- The follow graph, PDS endpoints, blog documents, archived streams, the owner's opt-out and the
  per-viewer activity list are all deliberately left warm. That split is spec prose merged at task
  27, not an obligation here.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED with evidence collected in the workspace (121 mortar-core tests
green including the five new refresh cases, `just guard-wasm` green on a forced recompile, `just
check` exit 0, and the only two production call sites passing a literal `false`), and both
regression lines are PRESERVED: the graph-wall fill still lays its bricks and the non-refreshed
happy path still pays exactly one cache lookup.
