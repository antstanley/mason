# Done Certificate · Task 11: shared mapper and get_feed

**Task:** [11-shared_mapper_and_get_feed.md](11-shared_mapper_and_get_feed.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 11. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 11) is every obligation O1 to O5 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** One mapping path, shared, so a feed wall inherits moderation, `!warn` blur, video
  unwrapping and repost dropping rather than growing a second copy of them.
- **P2 · Obligations.** Done iff O1 to O5 all hold; O5 is the Reviewable item.
- **P3 · Invariants.** Must not change what `author_feed` and `get_image_feed` return for any input.
  This is a refactor plus an addition; the seven existing author-feed tests are the invariant.

## Obligations

- **O1 · The seven existing author-feed tests pass unedited.**
  - *Claim:* the tests at `bluesky.rs:464`, `:509`, `:535`, `:564`, `:594`, `:622` and `:654` pass
    with no change to any of their bodies.
  - *Evidence to collect:* run `cd server && cargo nextest run -p mortar-core bluesky`, expect green.
    Diff `bluesky.rs`'s test module and confirm no line inside those seven tests changed. Count them:
    seven, not six. `author_feed_drops_hidden_posts` at `:654` is the one an earlier draft of the
    task omitted, and it is the one that pins the hidden-label drop.
  - *Checks:* trace one of them (an opted-out author's post being dropped) through the new
    `author_feed` and into `map_feed_page`; confirm the filter order is unchanged, since
    `hidden_from_logged_out` runs before the `!warn` blur and a reordering would change which bricks
    carry a blur. Then trace `author_feed_drops_hidden_posts` specifically: the hidden tier is the
    behaviour the shared mapper exists to give a feed wall, so a green run that skipped it proves the
    wrong thing.
  - *Status:* unverified

- **O2 · The filters exist in exactly one place.**
  - *Claim:* `map_feed_page` is the only place the three filters and `post_to_brick` appear.
  - *Evidence to collect:* run
    `grep -n 'post_to_brick\|hidden_from_logged_out\|warned_for_logged_out\|reason.is_none' server/crates/mortar-core/src/sources/bluesky.rs`
    and confirm every hit is inside `map_feed_page` (or is its definition).
  - *Checks:* resolve `map_feed_page`'s callers: `author_feed` and `get_feed` only. A third caller
    would mean a third read path with its own URL construction.
  - *Status:* unverified

- **O3 · `get_feed` returns the upstream cursor and handles its absence.**
  - *Claim:* `get_feed(http, base, feed_uri, cursor, limit)` returns
    `(AuthorYield, Option<String>)`, `AuthorFeed` gained `#[serde(default)] cursor: Option<String>`,
    and a page with no upstream cursor yields `None`.
  - *Evidence to collect:* read the signature and the `AuthorFeed` struct. Run the wiremock cases:
    upstream order preserved, a repost dropped, an opted-out author's post dropped, a `!warn` post
    blurred, the cursor returned, and a page without a cursor yielding `None`.
  - *Checks:* `#[serde(default)]` is what keeps both author-feed reads deserializing pages that omit
    the field. Confirm it is present; without it every author-feed read would fail to parse.
  - *Status:* unverified

- **O4 · Meets the repo definition of done.**
  - *Claim:* wiremock cases cover every filter, the wasm32 build still compiles the new tests, and
    the gates are green.
  - *Evidence to collect:* run `just guard-wasm` (the only gate that sees a test module gated with a
    bare `#[cfg(test)]` when it uses `wiremock`) and `just check`.
  - *Status:* unverified

- **O5 · Reviewable: the diff touches no existing test body.**
  - *Claim:* `cargo nextest run -p mortar-core bluesky` is green with a diff whose only test-module
    changes are additions.
  - *Evidence to collect:* the command output plus the diff read.
  - *Status:* unverified

## Regression check

- `sources/fetch.rs:143 author_feed_cached` calls `bluesky::get_author_feed`. Trace: one author's
  feed with a repost and a `!warn` post still yields the same bricks with the same blur :
  (PRESERVED / REGRESSION)
- `sources/fetch.rs:177 image_feed_cached` calls `bluesky::get_image_feed`, which calls
  `author_feed(.., "posts_with_media", 100)`. Trace: still returns the same shape :
  (PRESERVED / REGRESSION)

## Residue

- The `&`-in-uri assertion is an obligation of this task's DoD via O3's wiremock set; if the
  validator finds it asserted only by substring rather than by `query_param`, note it: a substring
  match would pass on a request that split the value across two parameters.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
