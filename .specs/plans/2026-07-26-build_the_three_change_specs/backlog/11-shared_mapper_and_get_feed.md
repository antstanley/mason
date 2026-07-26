# Task 11 · shared mapper and get_feed

**Plan:** [plan.md](../plan.md) · **Certificate:** [11-shared_mapper_and_get_feed-certificate.md](11-shared_mapper_and_get_feed-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `04-sources-and-moderation.md` → Per source → Bluesky; implementation note 1. Targets [`04-sources-and-moderation.md`](../../../04-sources-and-moderation.md) §Per source.
**Depends on:** none
**Produces:** one mapping path, shared, so a feed wall inherits moderation, `!warn` blur, video unwrapping and repost dropping rather than growing a second copy of them.
**Pointers:** `server/crates/mortar-core/src/sources/bluesky.rs:208` (`author_feed`), `:224` / `:228` / `:232` (the three filters), `:250` (`struct AuthorFeed`, which reads only the `feed` array today), `:201` (`get_image_feed`, the neighbour the new function sits beside). `sources/util.rs:8` holds `urlencode`. **Seven** existing author-feed tests live at `bluesky.rs:464`, `:509`, `:535`, `:564`, `:594`, `:622` and `:654`. The last one, `author_feed_drops_hidden_posts`, is the one that covers the hidden-label drop, which is the exact moderation behaviour the shared mapper exists to preserve on a wall where every author is a stranger; an earlier draft of this task listed six and left that one out.

## Steps

- [ ] Extract the mapping half of `author_feed` into `fn map_feed_page(page: AuthorFeed) -> Vec<Brick>`, holding the three filters and `post_to_brick`. `author_feed` then builds a URL, fetches, and calls it.
- [ ] Add `#[serde(default)] cursor: Option<String>` to `AuthorFeed`; both author-feed reads ignore it.
- [ ] Add `get_feed(http, base, feed_uri, cursor, limit) -> Result<(AuthorYield, Option<String>), HttpError>` requesting `app.bsky.feed.getFeed?feed=<urlencoded>&limit=<n>[&cursor=<urlencoded>]` on `Bucket::Appview`.
- [ ] Add wiremock cases for `get_feed`: upstream order preserved, a repost dropped, an opted-out author's post dropped, a `!warn` post blurred, and the upstream cursor returned.
- [ ] Add a case asserting a feed uri containing `&` reaches the mock as one percent-encoded query value, using `query_param`.

## Definition of done

- [ ] All **seven** existing author-feed tests pass **unchanged**, `author_feed_drops_hidden_posts` at `:654` included; that is the proof the extraction preserved behaviour, so no test in that set may be edited in this diff.
- [ ] `map_feed_page` is the only place the three filters and `post_to_brick` appear, proven by grep, so there is no second place for the `!warn` tier to be forgotten on a surface where every author is a stranger.
- [ ] `get_feed` returns the upstream cursor alongside the yield, and returns `None` for it when upstream sends none.
- [ ] Meets the repo definition of done (wiremock cases for every filter, `just guard-wasm` green so the new tests do not break the wasm32 build, `just check` green).
- [ ] Reviewable: `cd server && cargo nextest run -p mortar-core bluesky` is green with a diff that touches no existing test body.
