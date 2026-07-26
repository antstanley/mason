# Done Certificate · Task 20: refreshable seam reads

**Task:** [20-refreshable_seam_reads.md](20-refreshable_seam_reads.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · A refreshed read that fails transiently falls back to the cached yield.**
  - *Claim:* three 5xx responses under `refresh: true` with an entry cached returns that entry rather
    than `None`, so the author is still counted as answered and the refreshed wall is never thinner.
  - *Evidence to collect:* run the named test. Confirm the mock returns 5xx enough times to exhaust
    `http.rs`'s retries, and that the returned value is the seeded one.
  - *Checks:* resolve `transient()` at `fetch.rs:132`. It covers `Transport`, `RetriesExhausted`,
    429 and 5xx. Confirm the fallback sits in that arm, not in the non-transient arm, which must
    still cache an empty yield.
  - *Status:* unverified

- **O3 · A refreshed read with nothing cached behaves exactly like a cold one.**
  - *Claim:* `refresh: true` with an empty cache returns `None` on a transient failure.
  - *Evidence to collect:* run the named negative-space test.
  - *Status:* unverified

- **O4 · A refreshed read marks the cache dirty.**
  - *Claim:* after a refreshed read the cache reports dirty, which is what makes the persistence
    claim in `05-caching-and-persistence.md` true.
  - *Evidence to collect:* run the named test asserting `caches.image_feed.is_dirty()` (or the
    equivalent for `author_feed`).
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* negative-space test present for the nothing-cached case, and the gates are green with
    `guard-wasm` run as a first-class check.
  - *Evidence to collect:* run `cd server && cargo nextest run -p mortar-core`, then `just guard-wasm`
    on its own, then `just check`. Read the new test module's `cfg` attribute and confirm it is
    `#[cfg(all(test, not(target_arch = "wasm32")))]`; `fetch.rs:373` is a bare `#[cfg(test)]` module
    and adding wiremock tests there would break the wasm32 build without breaking any test.
  - *Status:* unverified

- **O6 · Reviewable: every caller passes `false`, so the seam is reviewed alone.**
  - *Claim:* the diff shows `fan_out_authors` passing a literal `false` at its single call site pair,
    so no behaviour changes and the review is about the seam.
  - *Evidence to collect:* read `algo/fill.rs` around `:232` and confirm both branches pass `false`.
    Run `grep -rn 'author_feed_cached\|image_feed_cached' server/crates/mortar-core/src` and confirm
    every non-test call passes `false`.
  - *Status:* unverified

## Regression check

- `algo/fill.rs:219 fan_out_authors` is the only production caller. Trace: a cold graph wall still
  fans out and lays the same bricks; run `cd server && cargo nextest run` : (PRESERVED / REGRESSION)
- Happy-path lookup count: read the non-refreshed path and confirm it still performs exactly one
  cache `get`, not two : (PRESERVED / REGRESSION)

## Residue

- The follow graph, PDS endpoints, blog documents, archived streams, the owner's opt-out and the
  per-viewer activity list are all deliberately left warm. That split is spec prose merged at task
  27, not an obligation here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
