# Done Certificate · Task 11: shared mapper and get_feed

**Task:** [11-shared_mapper_and_get_feed.md](11-shared_mapper_and_get_feed.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* `cd server && cargo nextest run -p mortar-core bluesky` run by the gate: 16 tests
    run, 16 passed, 103 skipped. All seven are named in that output and all seven pass:
    `author_feed_parses_video_and_drops_reposts`, `image_feed_reads_media_deep`,
    `author_feed_drops_a_no_unauthenticated_author`, `author_feed_drops_adult_posts`,
    `author_feed_shows_nudity`, `author_feed_blurs_warned_posts`, `author_feed_drops_hidden_posts`.
    Seven, counted, not six. They now sit at `bluesky.rs:517`, `:562`, `:588`, `:617`, `:647`, `:675`
    and `:707`, each exactly 53 lines below its authored line number, the uniform shift the 53 new
    production lines above the test module produce. The test module was diffed mechanically against
    the parent revision (`jj file show -r @-` piped through `diff` from the `mod tests` line down):
    **zero** removed or changed lines, additions only. `jj diff --git | grep '^-'` returns eight
    removed lines in total, all in the production half (the three-line `author_feed` doc comment, one
    blank line, `let bricks = page`, `.feed`, `.collect();`, `Ok(AuthorYield { bricks })`).
  - *Checks:* the extraction was read line by line against the parent revision. The three filters are
    byte-identical and in the original order: `.filter(|item| item.reason.is_none())` first
    (`bluesky.rs:272`), then the hidden-tier `.filter` on author labels and post labels (`:276-279`),
    then the `.filter_map` that computes `warned` and calls `post_to_brick` before `set_blur`
    (`:280-292`). No condition is inverted, nothing is reordered.
    Trace, `author_feed_drops_a_no_unauthenticated_author`: `get_author_feed` →
    `author_feed(.., "posts_no_replies", 30)` → one `FeedItem` with `reason: None` and
    `post.author.labels = [!no-unauthenticated]` → `map_feed_page` → repost filter keeps it →
    `hidden_from_logged_out([!no-unauthenticated])` is true because `NO_UNAUTHENTICATED` is in
    `HIDDEN_LABELS` → predicate false → dropped before the blur tier is ever consulted → `bricks`
    empty. Trace, `author_feed_drops_hidden_posts`: same path with `post.labels = [!hide]` →
    `hidden_from_logged_out` true on the post arm of the `&&` → dropped → `bricks` empty, and the
    hard-hidden post never reaches `warned_for_logged_out`, which is the ordering the blur tier
    depends on.
  - *Status:* SATISFIED

- **O2 · The filters exist in exactly one place.**
  - *Claim:* `map_feed_page` is the only place the three filters and `post_to_brick` appear.
  - *Evidence to collect:* run
    `grep -n 'post_to_brick\|hidden_from_logged_out\|warned_for_logged_out\|reason.is_none' server/crates/mortar-core/src/sources/bluesky.rs`
    and confirm every hit is inside `map_feed_page` (or is its definition).
  - *Collected:* the grep was run by the gate over `bluesky.rs`, and again repo-wide over
    `server/crates/` with `set_blur` added to the pattern. Every feed-item hit is inside
    `map_feed_page`: `:272` (`reason.is_none`), `:277-278` (`hidden_from_logged_out`), `:283-284`
    (`warned_for_logged_out`), `:285` (`post_to_brick`), `:287` (`set_blur`). The remaining hits are
    the definitions themselves (`:99`, `:107`, `:416`, and `model.rs:32`) and the pre-existing
    cohort-level `Follow::hidden()` at `:39`, which is an author-level check on the follow graph that
    predates this task and is not one of the three feed-item filters. Nothing outside `bluesky.rs`
    names any of the four.
  - *Checks:* `map_feed_page` is called at `:219` (inside `author_feed`) and `:255` (inside
    `get_feed`), and nowhere else in the workspace. Two callers, both named. Resolution of the calls
    in the changed lines: `urlencode` is step 4, the `use crate::sources::util::urlencode` import at
    `:11` resolving to `util.rs:8`, with no module-level or local definition of that name in
    `bluesky.rs` to shadow it; `map_feed_page` is step 3, module level at `:269`;
    `hidden_from_logged_out` and `warned_for_logged_out` are step 3 at `:99` and `:107`;
    `post_to_brick` is step 3 at `:416`; `set_blur` is an inherent method on `Brick`
    (`model.rs:32`); `page.cursor.take()` is step 5, `Option::take` on the `Option<String>` field,
    with no `take` on `AuthorFeed` to shadow it. No shadowing found anywhere in the changed lines.
  - *Status:* SATISFIED

- **O3 · `get_feed` returns the upstream cursor and handles its absence.**
  - *Claim:* `get_feed(http, base, feed_uri, cursor, limit)` returns
    `(AuthorYield, Option<String>)`, `AuthorFeed` gained `#[serde(default)] cursor: Option<String>`,
    and a page with no upstream cursor yields `None`.
  - *Evidence to collect:* read the signature and the `AuthorFeed` struct. Run the wiremock cases:
    upstream order preserved, a repost dropped, an opted-out author's post dropped, a `!warn` post
    blurred, the cursor returned, and a page without a cursor yielding `None`.
  - *Collected:* the signature at `bluesky.rs:236-242` is
    `pub async fn get_feed(http: &Http, base: &str, feed_uri: &str, cursor: Option<&str>, limit: u32)
    -> Result<(AuthorYield, Option<String>), HttpError>`, exactly the shape the task names.
    `AuthorFeed` at `:296-305` now carries `#[serde(default)] cursor: Option<String>` beside `feed`.
    All five new wiremock cases were run by the gate and pass:
    `get_feed_keeps_upstream_order_and_returns_the_cursor` (three posts come back in the upstream
    order `/3, /1, /2`, and `next` is `Some("page3")`), `get_feed_reports_no_cursor_at_the_end` (a
    body with no `cursor` key yields one brick and `next.is_none()`),
    `get_feed_drops_reposts_and_hidden_authors` (a `reason` object and a `!no-unauthenticated`
    author both dropped, one brick left), `get_feed_blurs_warned_posts` (a `!warn` post kept with
    `blur.label == "!warn"`), and `get_feed_percent_encodes_the_feed_reference`.
    Trace, the primary path this task's `Produces` promises:
    `get_feed(http, base, "at://did:plc:gen/app.bsky.feed.generator/whats-hot", Some("page2"), 24)`
    → url `.../xrpc/app.bsky.feed.getFeed?feed=at%3A%2F%2Fdid%3Aplc%3Agen%2F...&limit=24&cursor=page2`
    → body `{feed: [/3, /1, /2], cursor: "page3"}` → `page.cursor.take()` leaves `next = Some("page3")`
    and the page holding `None` → `map_feed_page` applies the same three filters the author feed uses
    → `(AuthorYield { bricks: [/3, /1, /2] }, Some("page3"))`.
  - *Checks:* `#[serde(default)]` is present at `:303`. Worth recording that serde's derive already
    treats a missing `Option` field as `None` (the pre-existing `FollowsPage.cursor` at `:166` relies
    on exactly that), so the attribute is belt and braces rather than load-bearing. It is harmless,
    the task asked for it explicitly, and the outcome the check cares about is proved dynamically:
    the seven author-feed mocks all send bodies with no `cursor` key and all seven deserialise and
    pass, so no author-feed read regressed on the new field.
  - *Status:* SATISFIED

- **O4 · Meets the repo definition of done.**
  - *Claim:* wiremock cases cover every filter, the wasm32 build still compiles the new tests, and
    the gates are green.
  - *Evidence to collect:* run `just guard-wasm` (the only gate that sees a test module gated with a
    bare `#[cfg(test)]` when it uses `wiremock`) and `just check`.
  - *Collected:* the new cases sit in the existing module at `bluesky.rs:496`, gated
    `#[cfg(all(test, not(target_arch = "wasm32")))]`, which is the gating the `wiremock` and `tokio`
    dev-dependencies at `Cargo.toml:41` require. `just guard-wasm` was run by the gate twice: once
    cold from cache, then again after forcing a real recompile, and the second run genuinely
    recompiled both crates (`Checking mortar-core`, `Checking mortar-wasm`) and finished with no
    errors. `just check` was run by the gate to completion and exited **0**: the four guards, oxfmt
    and `cargo fmt --check`, guard-wasm, oxlint (four warnings, all pre-existing, in
    `FeedGrid.svelte` and `service-worker.ts`, files this diff does not touch), knip,
    `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run` (119 tests run, 119
    passed, 0 skipped, the committed wire-contract fixture test among them), both `tsc` projects, and
    39 vitest tests. Wiremock coverage of every filter: repost, hidden tier and `!warn` blur each have
    a `get_feed` case of their own, listed under O3.
  - *Status:* SATISFIED

- **O5 · Reviewable: the diff touches no existing test body.**
  - *Claim:* `cargo nextest run -p mortar-core bluesky` is green with a diff whose only test-module
    changes are additions.
  - *Evidence to collect:* the command output plus the diff read.
  - *Collected:* the gate ran the Reviewable action itself, not the implementer's transcript of it:
    `cd server && cargo nextest run -p mortar-core bluesky` → `Summary [0.487s] 16 tests run: 16
    passed, 103 skipped`. The diff was read in full (`jj diff --git`, one file, 221 insertions and 8
    deletions) and the test module was diffed mechanically against the parent revision: additions
    only, zero removed or changed lines below `mod tests`. The single test-module hunk is
    `@@ -673,6 +726,166 @@`, appended after `author_feed_drops_hidden_posts`.
  - *Status:* SATISFIED

## Regression check

- `sources/fetch.rs:143 author_feed_cached` calls `bluesky::get_author_feed` at `fetch.rs:151`.
  Trace: `get_author_feed` at `bluesky.rs:191` still has the signature
  `(&Http, &str, &str) -> Result<AuthorYield, HttpError>` and still delegates to
  `author_feed(.., "posts_no_replies", 30)`; `author_feed` builds the same URL it always did and now
  returns `AuthorYield { bricks: map_feed_page(page) }`, where `map_feed_page` holds the same three
  filters in the same order. A feed with a repost and a `!warn` post therefore yields the same bricks
  with the same blur, which
  `author_feed_parses_video_and_drops_reposts` and `author_feed_blurs_warned_posts` both assert and
  both pass. `AuthorYield` itself is untouched, so the caller's `Arc<AuthorYield>`, its TTL cache and
  its persistence round trip are unchanged : **PRESERVED**
- `sources/fetch.rs:177 image_feed_cached` calls `bluesky::get_image_feed`, which calls
  `author_feed(.., "posts_with_media", 100)`. Trace: `get_image_feed` at `bluesky.rs:201` is
  unchanged, the `filter` and `limit` it passes are unchanged, and `image_feed_reads_media_deep`
  still asserts `query_param("filter", "posts_with_media")` and `query_param("limit", "100")` reach
  the mock and that the result is an image post : **PRESERVED**
- Third caller, checked though the certificate did not name it: `algo/fill.rs:235` calls
  `fetch::author_feed_cached`, whose signature and `Option<Arc<AuthorYield>>` return are untouched by
  this diff : **PRESERVED**

## Residue

- The `&`-in-uri assertion is an obligation of this task's DoD via O3's wiremock set; if the
  validator finds it asserted only by substring rather than by `query_param`, note it: a substring
  match would pass on a request that split the value across two parameters.
  - *Resolved:* it is asserted by `query_param`, not by substring.
    `get_feed_percent_encodes_the_feed_reference` at `bluesky.rs:869` sends
    `at://did:plc:gen/app.bsky.feed.generator/x&limit=1` and matches on
    `query_param("feed", sneaky)` and `query_param("limit", "24")`. wiremock 0.6.5's
    `QueryParamExactMatcher` (`src/matchers.rs:885-890`) matches on `request.url.query_pairs()`,
    which percent-decodes, so the whole reference has to have arrived as one encoded value for the
    matcher to see it; had `&` gone through raw, `feed` would decode to the truncated
    `at://did:plc:gen/app.bsky.feed.generator/x`, the mock would not match, and the test would fail
    on the 404. `urlencode` (`sources/util.rs:8`) escapes everything outside the RFC 3986 unreserved
    set, `&` included. The case was run by the gate and passes.
  - *One caveat recorded, not a defect:* the matcher is an `any()` over the query pairs, so the
    `limit` assertion alone would still pass if a duplicate `limit` were smuggled in beside it. The
    `feed` assertion is what carries the proof, and it does carry it.

## Notes outside the contract

None of these are defects in the delivered unit, and none change the verdict. They are recorded so
tasks 12 and 13 inherit them rather than rediscover them.

- `limit` reaches the query unvalidated. `getFeed` caps at 100 upstream, so a caller asking for more
  earns a 400. The task defined `limit` as the caller's parameter deliberately, and the callers do
  not exist yet.
- An upstream `cursor: ""` would be handed back as `Some("")`, which a naive pager would read as
  "there is more". The AppView does not do this, and no pager exists yet.
- `FeedItem`/`PostView` deserialisation is strict enough that one malformed item fails the whole
  page. That is pre-existing author-feed behaviour, unchanged here, but a feed generator's page is a
  page of strangers, so the exposure is wider than it was.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O5 are all SATISFIED on evidence the gate collected itself (16 of 16 bluesky tests
green, a mechanical test-module diff showing additions only, the grep proving the three filters and
`post_to_brick` live in `map_feed_page` alone, a fresh `just guard-wasm` recompile and a `just check`
that exited 0), the two named regression callers plus the `algo/fill.rs` one are PRESERVED, and the
residue's `&`-encoding case is asserted by `query_param` exactly as the author required.
