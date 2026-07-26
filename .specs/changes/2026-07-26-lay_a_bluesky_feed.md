# Change: Lay a Bluesky feed as a wall

**Status:** Proposed · **Date:** 2026-07-26 · **Owner:** Ant Stanley · **Target:** Repo-wide

mason lays exactly one algorithm: a sampled follow graph, ranked by grout and
laid by the mixer. This change splits a wall into a **source** and a **view**,
and adds a second source. The follow graph becomes one option (mixed posts, blogs
and video, still the default); any Bluesky feed generator becomes another
(`?feed=at://…`, posts and Bluesky video only, in the feed's own order). All
three views, bento, masonry and glaze, apply to either source.

A feed generator **is** an algorithm, so grout and the mixer sit out: mason pages
`app.bsky.feed.getFeed` and lays what comes back. There is no snapshot, no pool
and no cohort behind a feed wall, which makes it the cheapest wall mason can lay,
one AppView call per page instead of a hundred-author fan-out. A **feed picker**
screen is the way in, standing beside the handle box as a peer front door rather
than as an extra field on it.

---

## Motivation

mason's wall answers one question well: what are the people this person follows
making. That is the product, and it stays the default. But it is the only
question mason can be asked, and the atmosphere's own answer to "show me
something else" is the feed generator: a published, addressable, unauthenticated
ranking service that anybody can point at. There are thousands of them, readers
already have favourites, and mason currently cannot show one.

The fit is unusually good. A feed generator returns a ranked list of post URIs,
hydrated by the AppView into exactly the `PostView` shape mason's post mapper
already consumes, labels and all. So the whole ingestion side is a rename: one
new endpoint, the existing mapper unchanged, the existing moderation inherited.
And because the feed's order is authoritative, the entire snapshot machinery
(the pool, the caps, the waves, the warming reflow) has nothing to do. A feed
wall is the same wall drawn from a different faucet.

Discovery is the front door's problem, and it is the other half of this change. A
handle is a question that assumes its own answer: it asks the reader who they
already want to read. Somebody arriving with nobody in mind has nothing to type,
and mason currently has nothing to show them but a demo. A feed picker answers a
different question, from what the network already ranks, which is why it is a
screen beside the handle box rather than a second input on it.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/00-overview.md`](../00-overview.md) | Goals, system shape, scope summary |
| [`.specs/01-domain-model.md`](../01-domain-model.md) | A `FeedRef` entity; the `Cursor` entity gains a second shape; the query-pattern table |
| [`.specs/02-feed-engine.md`](../02-feed-engine.md) | `handle_feed` gains `feed`; a feed wall's request flow, which builds no snapshot |
| [`.specs/03-grout-and-mixer.md`](../03-grout-and-mixer.md) | Neither module runs for a feed wall |
| [`.specs/04-sources-and-moderation.md`](../04-sources-and-moderation.md) | `getFeed` joins the Bluesky source; the feed reference is vetted like any untrusted string |
| [`.specs/05-caching-and-persistence.md`](../05-caching-and-persistence.md) | A `feed_pages` cache, not persisted |
| [`.specs/06-wire-contract.md`](../06-wire-contract.md) | The `feed` parameter, an optional `actor`, `feed_not_found`, two cursor shapes |
| [`.specs/07-web-client.md`](../07-web-client.md) | `?feed=` as a second routing surface; feed identity in the header; a fourth error state |
| [`.specs/08-wall-and-bricks.md`](../08-wall-and-bricks.md) | The feed box on the landing form; what a feed wall's chrome shows |
| [`.specs/canonical-types.schema.json`](../canonical-types.schema.json) | `FeedRef` and `HiddenLabel` added; `CursorPayload` and `MortarErrorCode` replaced |

---

## Proposed changes

Each block below is the prose as it should read once merged, so links *inside* a
quoted block are relative to the canonical page it lands on, not to this
directory. Links outside the blocks resolve from `.specs/changes/` as usual.

### `.specs/00-overview.md` → Goals (Add)

> 8. Split a wall into a source and a view. The source is either the reader's
>    follow graph (mixed posts, blogs and video) or any Bluesky feed generator
>    (`?feed=<at-uri>`, posts and Bluesky video only, in the feed's own order).
>    All three views (bento, masonry, glaze) apply to either. On a feed wall the
>    generator is the algorithm, so mason contributes the wall and nothing else.
> 9. Offer a feed picker as a second front door beside the handle box: browse what
>    the network ranks, search it, list one person's feeds, or paste a link. Still
>    no account and no sign-in.

### `.specs/00-overview.md` → System shape (Modify)

Extend the fetch line under the browser-tab box:

> ```
>   │ fetch /api/feed?actor=|feed=&cursor=&mode=&intent=
> ```

### `.specs/00-overview.md` → Scope summary (Modify)

Replace the `Brick kinds`, `Walls` and `Pagination` rows with four:

> | Wall source | The reader's follow graph (default), or any Bluesky feed generator | `actor` or `feed`, exactly one of them |
> | Wall views | `bento`, `masonry`, `glaze`, all three on either source | `Mode` carries glaze to the engine; bento and masonry are pure presentation |
> | Brick kinds | Graph wall: post, blog, video (Bluesky and Streamplace, live and archived), five mix slots. Feed wall: post and Bluesky video only | A feed generator returns post URIs, so blogs and streams structurally cannot appear |
> | Pagination | Two opaque base64url cursor shapes: `{seed, offset}` for a graph wall, `{feed}` for a feed wall | Page size 24, except glaze on a feed wall; laid bricks never move |

### `.specs/01-domain-model.md` → Entities → FeedRef (Add)

> ### FeedRef
>
> A validated pointer to a Bluesky feed generator record: an `AtUri` whose
> collection is exactly `app.bsky.feed.generator`. It is the one request
> parameter besides `actor` and `cursor` that reaches an upstream query, so it is
> parsed rather than forwarded.
>
> Two spellings are accepted, because one of them is what people have in their
> clipboard:
>
> | Given | Handled |
> |---|---|
> | `at://<did>/app.bsky.feed.generator/<rkey>` | Used as is |
> | `https://bsky.app/profile/<handle\|did>/feed/<rkey>` | The profile segment is resolved to a DID (the `did` cache, then `getProfile`) and the AT-URI is built from it |
>
> Anything else, including an AT-URI naming a different collection, is a
> `bad_request`. A `FeedRef` is not an identity mason keys on: it is a cache key
> component and a query parameter, and the bricks it yields carry their own
> author DIDs as always.

### `.specs/01-domain-model.md` → Entities → Cursor (Modify)

> ### Cursor
>
> The `CursorPayload` has two shapes, one per wall source, JSON-serialised and
> base64url (no padding) encoded into the opaque `Cursor` string the client sees.
> It is attacker-writable, so every consumer treats it defensively: a garbage or
> tampered cursor decodes to `None` and falls back to a fresh wall.
>
> | Shape | Wall | Carries |
> |---|---|---|
> | `{seed: u64, offset: usize}` | A graph wall (`wall` or `glaze`) | The seed and the next offset into the snapshot's wall |
> | `{feed: String}` | A feed wall | The upstream `getFeed` cursor, verbatim and opaque |
>
> The two are distinguished structurally (`#[serde(untagged)]`, the feed shape
> tried first), so no discriminator field has to be carried and a cursor issued
> before this change still decodes to the graph shape.
>
> On a graph wall, `seed` is the load-bearing field: it drives the cohort shuffle
> and the mixer's jitter, so a snapshot evicted mid-scroll rebuilds into a
> closely-matching wall from the same seed and the still-warm per-author caches.
> The snapshot id itself is not carried: `handle_feed` derives it from the
> resolved DID, the seed and the mode.
>
> On a feed wall there is nothing to rebuild. The feed generator holds the order
> and the upstream cursor is the whole of mason's position in it, so the offset
> and the seed have no meaning and are not carried. The `offset` is not preserved
> across an eviction because there is no snapshot to evict.

### `.specs/01-domain-model.md` → Required query patterns (Add)

> | One page of a feed generator | `(feed uri, upstream cursor)` | `caches.feed_pages` |

### `.specs/02-feed-engine.md` → Entry point (Modify)

> ```rust
> pub async fn handle_feed(
>     state: &Arc<AppState>,
>     target: FeedTarget<'_>,
>     cursor: Option<&str>,
>     mode: Mode,
>     intent: FeedIntent,
> ) -> Result<FeedResponse, AppError>
> ```
>
> `FeedTarget` is `Actor(&str)` or `Feed(&str)`, built by each front from the
> query string: `feed` wins when both parameters are present, and neither being
> present is a `bad_request`. Page size is a constant for both:
> `PAGE_SIZE = 24`.

### `.specs/02-feed-engine.md` → A feed wall (Add, new section after Modes)

> ## A feed wall
>
> A feed generator is an algorithm somebody else published, and mason's job on a
> feed wall is to lay it, not to re-rank it. So a feed wall skips almost the whole
> engine: no snapshot, no pool, no admission caps, no cohort, no extension waves,
> no grout and no mixer.
>
> ```
> handle_feed(Feed(ref), cursor, mode, intent)
>   │
>   ├─ FeedRef::parse(ref) ──▶ AtUri  |  Err(BadRequest)
>   │     a bsky.app feed URL resolves its profile segment to a DID first
>   │
>   ├─ decode cursor ──▶ Some{feed} | None   (a graph cursor here decodes to None:
>   │                                         a fresh wall, not an error)
>   ├─ feed_page_cached(uri, upstream cursor)
>   │     ──▶ (bricks, next upstream cursor)  |  Err(FeedNotFound | Upstream)
>   │
>   ├─ Mode::Glaze ? ──▶ keep only is_image_post()   (and lay every survivor)
>   ├─ Mode::Wall  ? ──▶ truncate to PAGE_SIZE
>   │
>   └─ intent == Preview ? {items, cursor: the INCOMING cursor, warming: false}
>                        : {items, cursor: next.map(encode)}
> ```
>
> Four consequences, and each one is the point:
>
> - **There is nothing to warm.** One AppView call answers a page, so a preview
>   reports itself already settled and echoes the cursor it was given, exactly as
>   the demo wall does. The client freezes on its first poll, and the 60 second
>   `feed_pages` cache makes the freeze that follows a cache hit rather than a
>   second round trip.
> - **The page size follows the view, because glaze's filter is aggressive.** The
>   mixed views ask for `limit = PAGE_SIZE` and may come back a few short, since
>   reposts and moderated posts are dropped after the request; serving short is
>   already normal and the pump retries. Glaze asks for `limit = 100`
>   (`getFeed`'s ceiling) and lays **every** image post that survives, not the
>   first `PAGE_SIZE` of them. Most posts in a general feed carry no image, so
>   asking for 24 and filtering would lay three or four bricks per network call
>   and spend a dozen calls filling one screen. Laying all of them rather than
>   truncating is not an optimisation but a correctness requirement: there is no
>   pool to hold a remainder in, and the cursor mason hands back belongs to the
>   call that fetched them, so a truncated page throws the rest away.
> - **The wall ends when the feed does.** `getFeed` returning no cursor is the
>   whole end condition. There is no `graph_spent`, no `has_more()` and no pool
>   to drain.
> - **Only posts and Bluesky videos can appear.** A feed generator returns post
>   URIs, so blogs and Streamplace bricks are structurally absent from a feed
>   wall. The mix ratio has nothing to balance.
>
> **All three views work on a feed wall, and glaze means something different on
> each source.** A view is the reader's choice about the wall in front of them, so
> it does not depend on where the bricks came from:
>
> | View | Graph wall | Feed wall |
> |---|---|---|
> | Bento, Masonry | Presentation only; one mixed wall packed two ways | Presentation only; one feed packed two ways |
> | Glaze | `Mode::Glaze` re-reads each author deep (`posts_with_media`, 100) and admits image posts alone | `Mode::Glaze` filters the feed's own posts to those carrying an image |
>
> One `mode` value carries both, because `Mode` selects kinds and never a source.
> The layout picker therefore needs no new state, no fourth option and no disabled
> cases: three views, always, whichever door the reader came in through.

### `.specs/02-feed-engine.md` → Resolution and the wall-owner gate (Add)

Append to that section:

> A feed wall has no owner to gate. `!no-unauthenticated` is a request about a
> person's own social graph being put on display; a feed generator is a published
> service, and the feed's creator has not asked anybody not to read it. Individual
> posts and their authors are still filtered: a feed wall runs the same post
> mapper as an author feed, which drops a hidden or opted-out author's posts and
> blurs the `!warn` tier (see [04](04-sources-and-moderation.md)). That per-post
> filter is complete coverage on a feed wall, where the cohort filter has nothing
> to do, because a feed cannot yield a blog or a stream.

### `.specs/03-grout-and-mixer.md` → Responsibilities (Add)

Append to the "does not own" paragraph:

> Neither module runs for a feed wall. A feed generator publishes an order and
> mason lays it in that order; re-ranking somebody else's algorithm by grout would
> produce a wall that is neither theirs nor mason's.

### `.specs/04-sources-and-moderation.md` → The upstreams (Modify)

> | Bluesky AppView | `https://public.api.bsky.app` | `app.bsky.actor.getProfile`, `app.bsky.graph.getFollows`, `app.bsky.feed.getAuthorFeed`, `app.bsky.feed.getFeed` | `Appview` (rate limited) |

### `.specs/04-sources-and-moderation.md` → Per source → Bluesky (Add)

> - **`get_feed(feed_uri, cursor, limit)`** pages a feed generator through the
>   AppView and returns `(AuthorYield, Option<String>)`: the mapped bricks and the
>   upstream cursor. It shares the *exact* mapping path with both author-feed
>   reads, which is the whole reason a feed wall inherits moderation, `!warn`
>   blur, video-embed unwrapping and repost dropping without a second
>   implementation of any of them. `getFeed` hydrates its results into the same
>   `PostView` shape `getAuthorFeed` returns, labels included, so the shared
>   mapper needs no branch.
>
>   A 400 or 404 is an unknown or withdrawn feed and becomes `FeedNotFound`. Any
>   other failure is `Upstream`. Unlike an author feed, this is the wall's only
>   source: there is no hundred-author quorum to degrade into, so a failure is
>   the request failing rather than a thin wall.

### `.specs/04-sources-and-moderation.md` → Outbound safety → URLs that will reach an `<a href>` (Add)

> A `feed` parameter is a third class of untrusted string: it does not reach an
> anchor, it reaches an *upstream query*. `FeedRef::parse` is what vets it. It
> requires the `at://` scheme and the exact `app.bsky.feed.generator` collection,
> or a `bsky.app` feed URL it rebuilds an AT-URI from, and rejects everything else
> as `bad_request` rather than forwarding it. The result is `urlencode`d into the
> `getFeed` query like every other interpolated value, so a reference carrying
> `&` or `#` cannot rewrite the upstream request or poison the cache key derived
> from it. There is no SSRF surface: the host is always the configured AppView
> base, never anything the reference names.

### `.specs/05-caching-and-persistence.md` → The caches (Add)

> | `feed_pages` | `<feed uri>\u{1f}<upstream cursor>` | `Arc<AuthorYield>` plus the next cursor | 60 s | 500 | **no** |

And append to the TTL discussion:

> A feed page is a ranked view with a deadline, like the live list, so it is
> cached for a minute and never persisted. Sixty seconds is enough to make the
> preview-then-freeze pair one network read and to survive a back/forward, and
> short enough that reopening a feed wall shows the feed's current head. A
> persisted feed page would be laid hours later as though it were fresh ranking,
> which is exactly the lie the persistence layer exists to avoid.

### `.specs/06-wire-contract.md` → The endpoint (Modify)

> ```
> GET /api/feed?(actor=<handle|did>|feed=<at-uri|bsky.app feed url>)
>              [&cursor=<opaque>][&mode=glaze][&intent=preview|freeze]
> ```
>
> | Parameter | Required | Values | Meaning |
> |---|---|---|---|
> | `actor` | one of the two | a Bluesky handle, a DID, or the literal `demo` | Whose graph to lay |
> | `feed` | one of the two | a feed generator AT-URI, or a `bsky.app/profile/<actor>/feed/<rkey>` URL | Which feed generator to lay |
> | `cursor` | no | opaque base64url of `{seed, offset}` or `{feed}` | Where to continue; absent starts a fresh wall |
> | `mode` | no | `glaze` | The image wall; it composes with either source |
> | `intent` | no | `preview`, `freeze` | The warm-then-commit first screen; absent is a normal committed page |
>
> Exactly one of `actor` and `feed` is needed. `feed` wins when both are given,
> because the two name different walls and one of them has to. Neither being
> present is a 400, as a missing `actor` was before.
>
> Unknown values for `mode` and `intent` still fall back to the default rather
> than erroring. `feed` does **not**: it is a structured reference that reaches an
> upstream query, and a value that will not parse is a `bad_request` rather than
> a silent fallback to somebody's graph.

### `.specs/06-wire-contract.md` → Errors (Modify)

Add a row:

> | `feed_not_found` | 404 | The AppView 400s or 404s on the feed generator |

### `.specs/06-wire-contract.md` → What the fixture covers (Modify)

> | `query.mode` / `query.intent` / `query.target` | The query vocabulary, as object keys (`target` holds `actor` and `feed`) |
> | `vocab.hiddenLabels` | The five labels of the hidden tier, as object keys, so the feed picker's client-side copy cannot drift from mortar's |

### `.specs/07-web-client.md` → Responsibilities (Modify)

> 1. Turn a URL into a wall: `/?actor=<handle>` lays a graph wall and
>    `/?feed=<at-uri>` lays a feed generator's. Those two parameters are the whole
>    routing surface.

### `.specs/07-web-client.md` → Shape (Modify)

The tree names `lib/` modules individually, so two lines change in it:

> ```
>     api.ts                  fetchFeed, warmFeed, FeedError, localMode
>     appview.ts              the public AppView base, for the header and picker
> ```
>
> ```
>     +page.svelte            actor or feed ? wall : landing form
> ```

And append to the description under the tree:

> There is still one route. `?actor=` and `?feed=` are the source of truth for
> which wall is showing and are mutually exclusive, with `feed` winning if both
> appear; everything else (layout, client, last handle, recent feeds) is a local
> preference in `localStorage`, never in the URL.

### `.specs/07-web-client.md` → Reactive state (Modify)

Add a row, and extend the `lastHandle` row's storage note:

> | `state/feedinfo.svelte.ts` | `feedInfo` | The feed generator's name, avatar and creator, for the header | none |
> | `state/handle.svelte.ts` | `lastHandle` | The last handle typed, and the last feeds opened | `mason:handle`, `mason:feeds` |

And add to the paragraphs beneath:

> - **`feedInfo` exists for the same reason `profile` does.** The feed never
>   carries the generator's own identity, so the header asks
>   `app.bsky.feed.getFeedGenerator` on the public AppView directly, exactly as
>   `profile` asks `getProfile` for a wall owner's avatar. A miss leaves the
>   header showing the feed's rkey and nothing else, which is ugly but never
>   blocking.

### `.specs/07-web-client.md` → Reactive state → The picker is history, not a URL (Add)

> The feed picker is a screen, not a route. It opens with
> `pushState('', { picker: 'feeds' })`, so the address bar keeps showing whatever
> is behind it and the back gesture closes the picker rather than leaving mason.
> `page.state.picker` is the single source of truth for whether it is open,
> declared alongside the rest of `App.PageState` in `app.d.ts`. The reasoning is
> the same as everywhere else on this page: the URL identifies the wall, and a
> picker is not a wall.

### `.specs/07-web-client.md` → The feed state machine (Modify)

> `FeedState` drives the warm-then-commit first screen and then paginates, for
> either kind of wall. `reset(target, mode)` takes a `FeedTarget`
> (`{actor}` or `{feed}`), and the session wall cache is keyed by the target and
> the mode together, so a graph wall and a feed wall never rehydrate into each
> other.
>
> A feed wall runs the same three states with the warming phase collapsed:
> mortar reports `warming: false` on the first preview, so the loop freezes
> immediately and pagination begins. Nothing in `FeedState` branches on the
> target beyond building the request.

### `.specs/07-web-client.md` → Error classification (Modify)

Add a row:

> | `feed_not_found` | `feed-not-found` | "no such feed", with a way into the feed picker |

### `.specs/07-web-client.md` → Decisions (Modify)

Replace the *Glaze is a layout that changes the algorithm* decision with two:

> - *A wall is a source and a view.* **`actor` or `feed` picks the source; the
>   layout picker picks the view.** Readers do not think in query parameters, and
>   they should not have to learn that one of mason's three views works on only one
>   of its two sources.
> - *Glaze is a view that changes the algorithm.* **One control, two effects, on
>   either source.** On a graph wall it re-fetches an images-only wall; on a feed
>   wall it filters the feed's own posts. `bento` and `masonry` stay pure
>   presentation, so switching them never re-mixes.

### `.specs/08-wall-and-bricks.md` → Layouts (Modify)

Replace the opening paragraph and the layout table's Glaze row:

> One control (`LayoutPicker`) offers three views, and all three work on either
> wall source. `glaze` is also an algorithm: picking it sets `mode=glaze`, which
> on a graph wall re-fetches an images-only wall and on a feed wall filters the
> feed's own posts to those carrying an image.
>
> | Glaze | `Bento.svelte` with `filler` | The same dense grid, laid on a muted field |

### `.specs/08-wall-and-bricks.md` → Wall states (Add)

> A feed wall changes the chrome, not the wall. `SwitchWall` shows the feed
> generator's avatar and name where it would show the owner's face, and the empty
> state reads "this feed has no bricks yet". Every other state (the skeletons, the
> stall, the ending, the three error panels) is unchanged, and all three views are
> offered.

### `.specs/08-wall-and-bricks.md` → Implementation layout (Modify)

That block enumerates every component by name, so the two new ones join it after
`LandingWall.svelte`:

> ```
>     FeedPicker.svelte        the second front door: browse, search, paste
>     FeedCard.svelte          one feed generator as a result
> ```

### `.specs/08-wall-and-bricks.md` → The feed picker (Add, new section after Wall states)

> ## The feed picker
>
> A handle assumes its own answer: it asks the reader who they already want to
> read. The feed picker is mason's other front door, and it is a screen rather
> than a field, standing beside the handle box as a peer.
>
> `FeedPicker` is reached two ways: from the landing page, where it sits under the
> handle box, and from `SwitchWall` on a laid wall, which is already the
> switch-walls affordance. Either way it opens over the current content and is held
> in history state (`pushState('', { picker: 'feeds' })`), so the back gesture
> returns the reader to where they were instead of out of mason.
>
> Five ways to reach a feed, because they answer five different states of knowing
> what you want:
>
> | Section | Source | Shown when |
> |---|---|---|
> | Recent | `mason:feeds` in `localStorage`, most recent first, capped at 12 | The reader has opened a feed before |
> | Search | `app.bsky.unspecced.getPopularFeedGenerators?query=<q>` | The reader types a search term |
> | By creator | `app.bsky.feed.getActorFeeds?actor=<handle>` | The input parses as a handle |
> | Popular | The same popular endpoint with no query, paged by its cursor | Always, as the resting state |
> | Paste | The value is handed to mortar as `?feed=`, which parses it | The input is a `bsky.app` feed link or an AT-URI |
>
> One input serves search, creator and paste: what the reader typed decides which
> question is being asked. That is the same input the handle box takes, and **by
> creator is the bridge between the two front doors**: a handle here means "show me
> the feeds this person made" rather than "show me their wall".
> `getActorFeeds` accepts a bare handle, so it costs no resolution hop.
>
> Each result is a card carrying the feed's avatar, its display name, its
> creator's handle, its description clamped to two lines, and its like count
> (hidden at zero, like every tally on the wall). Activating one opens
> `/?feed=<uri>` and writes it to `mason:feeds`.
>
> ### The picker reads the AppView directly, and that is chrome rather than moderation
>
> The picker is a directory listing, so it asks the public AppView from the browser
> exactly as `profile` already does for a wall owner's avatar. Content moderation
> stays where it belongs: every brick a chosen feed yields is filtered by mortar
> when the wall is laid.
>
> The picker does owe one check of its own, because a feed generator's own record
> carries labels. A feed whose view or whose creator carries the hidden tier is not
> listed, so mason never advertises a feed it would then refuse to lay properly.
> That puts the hidden-label list on both sides of the wire, which is normally the
> shape of a bug, so it is pinned rather than copied: the contract fixture carries
> `vocab.hiddenLabels` as object keys and `contract-check.ts` asserts the
> TypeScript list equals it, the same mechanism that already keeps the error codes
> and the video sources in step (see [06-wire-contract.md](06-wire-contract.md)).
>
> ### States
>
> | Condition | What the reader sees |
> |---|---|
> | Loading | Six card skeletons at the picker's own column count |
> | A search with no results | "no feeds by that name", and the paste hint |
> | A handle with no feeds | "that person has not made any feeds", with a link to lay their wall instead |
> | The AppView unreachable | Recents and the paste box, plus a quiet line saying browsing is unavailable. Paste is the load-bearing path and always works |
> | A pasted value that will not parse | The input says so in place; nothing navigates |
>
> The picker is a dialog in the same language as mason's other overlays:
> `role="dialog"`, `aria-modal="true"`, focus into the input on open, Escape and
> back to close, `inert` behind it, and the results as a list so a screen reader is
> told how many there are. Touch targets stay at 44px and the cards' hover lift is
> `motion-safe:`, like a brick's.

---

## Type changes

```json
{
  "$comment": "Fragment for 2026-07-26-lay_a_bluesky_feed. FeedRef is new; CursorPayload and MortarErrorCode replace their current defs in .specs/canonical-types.schema.json on merge. The opaque Cursor string def is unchanged.",
  "$defs": {
    "FeedRef": {
      "description": "The ?feed= request parameter: a pointer to a Bluesky feed generator record. Parsed, never forwarded. Either an AT-URI whose collection is exactly app.bsky.feed.generator, or a bsky.app feed URL whose profile segment mason resolves to a DID. Anything else is bad_request.",
      "oneOf": [
        {
          "$comment": "The DID authority form, which is what mason ultimately queries with.",
          "type": "string",
          "pattern": "^at://did:(plc|web):[A-Za-z0-9._:%-]+/app\\.bsky\\.feed\\.generator/[A-Za-z0-9._~-]+$"
        },
        {
          "$comment": "The handle authority form. A legal AT-URI spelling that people do paste; the handle is resolved to a DID before the upstream query.",
          "type": "string",
          "pattern": "^at://[A-Za-z0-9.-]+/app\\.bsky\\.feed\\.generator/[A-Za-z0-9._~-]+$"
        },
        {
          "$comment": "A bsky.app feed link, whose profile segment may be a handle or a DID.",
          "type": "string",
          "pattern": "^https://bsky\\.app/profile/[A-Za-z0-9._:%-]+/feed/[A-Za-z0-9._~-]+$"
        }
      ]
    },

    "CursorPayload": {
      "description": "What a decoded cursor carries. Engine-internal: the client never inspects it. Two shapes, one per wall source, distinguished structurally rather than by a discriminator field, so a cursor issued before feed walls existed still decodes to the graph shape.",
      "oneOf": [
        {
          "title": "Feed wall",
          "description": "Mason's position in a feed generator's own ordering. There is no snapshot to rebuild, so no seed and no offset are carried.",
          "type": "object",
          "required": ["feed"],
          "additionalProperties": false,
          "properties": {
            "feed": {
              "description": "The upstream app.bsky.feed.getFeed cursor, verbatim and opaque.",
              "type": "string",
              "minLength": 1
            }
          }
        },
        {
          "title": "Graph wall",
          "description": "A position in a snapshot's append-only wall.",
          "type": "object",
          "required": ["seed", "offset"],
          "additionalProperties": false,
          "properties": {
            "seed": {
              "description": "The seed for the cohort shuffle and the mixer's jitter. Carrying it is what lets an evicted snapshot rebuild deterministically mid-scroll.",
              "type": "integer",
              "minimum": 0
            },
            "offset": {
              "description": "The next item offset within the snapshot's wall.",
              "type": "integer",
              "minimum": 0
            }
          }
        }
      ]
    },

    "MortarErrorCode": {
      "description": "The machine codes mortar itself emits. Classification reads these, never the message, so they are wire contract. The web adds out-of-band codes of its own (wasm, unknown) for failures that never reached mortar.",
      "type": "string",
      "enum": ["bad_request", "actor_not_found", "feed_not_found", "login_required", "upstream"]
    },

    "HiddenLabel": {
      "description": "The hidden moderation tier: a subject carrying any of these never reaches the wall. Mirrors HIDDEN_LABELS in server/crates/mortar-core/src/sources/bluesky.rs. It is on the wire only as a pinned vocabulary (vocab.hiddenLabels in the contract fixture), because the feed picker reads a feed generator's own labels client-side and must not list a feed mason would refuse to lay. `nudity` is deliberately absent: Bluesky shows it to logged-out viewers, so mason does too.",
      "type": "string",
      "enum": ["!hide", "!no-unauthenticated", "porn", "sexual", "graphic-media"]
    }
  }
}
```

---

## Implementation notes

The engine work is additive and lands in front of the snapshot machinery rather
than inside it. Nothing in `algo/` changes except the cursor.

```
1. server/crates/mortar-core/src/sources/bluesky.rs:208
     Extract the mapping half of `author_feed` (the three filters plus
     post_to_brick, :221 to :245) into `fn map_feed_page(page: AuthorFeed) ->
     Vec<Brick>`. `author_feed` then builds a URL and calls it. This refactor is
     the change: get_feed must not grow a second copy of the moderation filters.

     Add `get_feed(http, base, feed_uri, cursor, limit) -> Result<(AuthorYield,
     Option<String>), HttpError>`, hitting
     app.bsky.feed.getFeed?feed=<urlencoded>&limit=<n>[&cursor=<urlencoded>] on
     Bucket::Appview, and returning map_feed_page's output plus page.cursor.

     `AuthorFeed` (:250) needs one added field for this: `cursor:
     Option<String>`. Its `feed` array is already exactly getFeed's, but the
     struct currently reads only that array. Adding the field is harmless for
     both author-feed reads, which ignore the cursor they were already being
     sent.

2. server/crates/mortar-core/src/sources/feedref.rs   (new, ~60 lines)
     Pure string work; the DID resolution for a handle-form URL is the caller's,
     so this module stays testable without an AppState. Export it from
     sources/mod.rs, not from a source submodule directly.

     The return must be TWO-CASE, not Option<String>: the caller cannot
     otherwise tell a finished AT-URI from a bsky.app URL whose profile segment
     is a handle still needing resolution, without re-inspecting the string the
     parser just parsed.

       pub enum FeedRef { Uri(String), NeedsDid { profile: String, rkey: String } }
       pub fn parse(raw: &str) -> Option<FeedRef>

     An AT-URI spelled with a handle authority (at://alice.bsky.social/
     app.bsky.feed.generator/x) is a legal spelling people do paste. It is
     accepted as NeedsDid rather than rejected, and the schema pattern in Type
     changes is widened to match; the AT-URI mason then queries with is always
     the DID form.

3. server/crates/mortar-core/src/feed.rs:130
     Factor `resolve_did` out of `resolve_and_gate` (the cold-handle branch at
     :162 minus the gate) so a bsky.app feed URL can resolve its profile segment
     without inheriting the wall-owner gate. resolve_and_gate keeps calling it.

4. server/crates/mortar-core/src/sources/fetch.rs
     feed_page_cached(state, uri, cursor) -> Result<(Arc<AuthorYield>,
     Option<String>), AppError>. Key: format!("{uri}\u{1f}{}", cursor.
     unwrap_or_default()). 400/404 -> AppError::FeedNotFound; anything else ->
     AppError::Upstream. Note the cached value has to carry the next cursor, so
     the cache value is a small struct, not a bare AuthorYield.

5. server/crates/mortar-core/src/cache.rs:194
     Add the feed_pages field (60s, 500). Do NOT add it to persist::CACHE_NAMES.
     `idbSweepStale` already deletes orphaned per-cache keys, so nothing rots.

5b. server/crates/mortar-core/src/feed.rs
     `pub fn FeedTarget::from_query<'a>(actor: Option<&'a str>, feed:
     Option<&'a str>) -> Result<FeedTarget<'a>, AppError>` holding the whole
     precedence rule (feed wins; neither present is bad_request), plus
     `kind() -> &'static str` for the wire token. BOTH fronts call it rather
     than each spelling the rule out. This is not tidiness: contract.rs is a
     mortar-core integration test and cannot reach either front, so a rule
     living in the axum route has nothing for step 10's fixture assert to check
     against, and mortar-server has no test module at all (grep for cfg(test)
     under its src returns nothing). An explicit lifetime is needed; two input
     refs do not elide.

6. server/crates/mortar-core/src/algo/cursor.rs:8
     Cursor becomes #[serde(untagged)] enum { Feed { feed: String }, Wall { seed,
     offset } }, Feed FIRST. Order is load-bearing: {seed, offset} cannot match
     Feed, so putting Feed first is unambiguous and keeps every legacy cursor
     decoding to Wall (there is no deny_unknown_fields; the existing test at :44
     proves it for the dropped `snapshot` key). Both existing tests must still
     pass unchanged, including garbage_is_none's {"seed":42} case.

     THREE call sites break, not two. The demo wall is a consumer ahead of both
     real paths: feed.rs:57 reads `decoded.map(|c| c.offset)` and feed.rs:64
     constructs `Cursor { seed: 0, offset }`. Add a test that a feed cursor
     handed to the demo wall lays from offset 0 rather than panicking.

7. server/crates/mortar-core/src/error.rs:6
     AppError::FeedNotFound(String) -> (404, "feed_not_found"). Add it to the
     variant list at :78 so the pinned envelope-string fixture covers it.

     While here, BadRequest needs attention: it Displays as "missing required
     parameter: {0}", so an unparseable feed reference would read "missing
     required parameter: feed" when the parameter was present. Either give it a
     second form for a malformed value or reword the Display. Note also that
     web/src/service-worker.ts:254 carries its OWN hardcoded copy of the string
     ("missing required parameter: actor"), and nothing in the repo compares the
     two: the pinned fixture strings come from a literal BadRequest("actor") in
     contract.rs:238 and error.rs:78, not from either front. Change both copies
     together.

8. server/crates/mortar-core/src/feed.rs:44
     handle_feed takes FeedTarget instead of &str actor, and branches to a new
     `feed_wall` path before resolve_and_gate. The demo branch at :56 stays on
     the Actor arm.

9. Both fronts, each calling FeedTarget::from_query from step 5b:
     server/crates/mortar-server/src/routes/feed.rs:15   the struct is
       `FeedParams`, NOT FeedQuery. It gains `feed`; the ok_or(BadRequest(...))
       at :42 is replaced by the shared parser.
     server/crates/mortar-wasm/src/lib.rs:88             feed_page gains a feed
       argument (Option<String> from JS).
     web/src/service-worker.ts                           forward the parameter.

10. Regenerate the wire fixture, which now needs the new error code and the
    target and label vocabularies:
      UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract

    Two prerequisites, or the new keys are retyped literals rather than pins:
    - Make HIDDEN_LABELS pub in sources/bluesky.rs:68 and re-export it from
      sources/mod.rs (contract.rs is an integration test and can only see pub
      items), then GENERATE vocab.hiddenLabels from it rather than retyping the
      five labels.
    - Bind the actor/feed tokens once as consts used for both the fixture keys
      and an assert against FeedTarget::kind(), the way contract.rs:347 already
      does for glaze/preview/freeze.

    Then follow it in web/src/lib/types.ts (MortarErrorCode) and
    web/src/lib/contract-check.ts. The target assertion needs a named union to
    compare against: `keyof FeedTarget` is `never` for a `{actor} | {feed}`
    union, so api.ts also exports `FeedTargetKind = "actor" | "feed"` and
    contract-check compares the fixture keys to THAT in both directions, the way
    ModeVocabularyMatches does at contract-check.ts:80.

11. Web, the wall. FIVE call sites of the changed APIs, and tsc can see exactly
    one of them; the other four are .svelte:
      lib/api.ts:36            fetchFeed(target: FeedTarget, cursor?, mode?,
                               intent?); warmFeed likewise. api.test.ts follows.
      lib/state/feed.svelte.ts:59    reset(target, mode); #key(target, mode)
      lib/state/feedinfo.svelte.ts   new, modelled on state/profile.svelte.ts
      routes/+page.svelte:10         derive both parameters; feed wins
      components/FeedGrid.svelte:55  feed.reset(handle, currentMode)
      components/FeedGrid.svelte:263 feed.reset(currentActor, currentMode)
      components/LandingWall.svelte:16   fetchFeed('demo')
      components/HandleForm.svelte:21    warmFeed(handle || 'demo')
      components/SwitchWall.svelte   the generator's face, and a way into the
                                     picker, on a feed wall
      components/FeedGrid.svelte     the feed-not-found error panel

    warmFeed stays ACTOR-ONLY and a feed target skips it. Its purpose is to land
    a follow graph and author feeds ahead of the wall; a feed target has neither,
    so the only thing left to warm is the wasm compile, which the picker screen
    has already paid by the time a feed is chosen.

11b. web/src/routes/+layout.svelte:13, :101, :110, :111, :129
    THE HEADER IS GATED ON `{#if actor}` AT :111. The skip link, the bottom
    padding, LayoutPicker, ClientPicker and SwitchWall are all inside it, and the
    document title at :101 is the same ternary, so a wall opened as /?feed=...
    renders today with no chrome at all. That contradicts this spec's own claim
    that all three views are offered on a feed wall. Derive both parameters at
    :13, gate on `actor || feed`, and give the title a feed-wall branch.

12. Web, the picker:
      app.d.ts:8               App.PageState is a COMMENTED-OUT placeholder, so
                               it is created rather than extended: uncomment and
                               declare `picker?: 'feeds'`
      lib/appview.ts           new: hoist the APPVIEW base out of
                               state/profile.svelte.ts:6. There are three
                               client-side AppView readers after this change
                               (profile, feedinfo, the picker) and one hardcoded
                               constant each is two too many.
      lib/state/feeds.svelte.ts      new: the mason:feeds recents list, the
                               search/creator/popular queries, their loading and
                               error state, and the hidden-tier filter
      components/FeedPicker.svelte   new: the screen
      components/FeedCard.svelte     new: one result. NOT in cards/, which is
                               brick renderers; a feed is not a brick
      components/HandleForm.svelte   the way into the picker, under the box

13. just check && just test-e2e
    pnpm changeset   (minor: a new surface)
```

### Two interactions with the other pending change specs

[`2026-07-26-read_a_brick_in_place.md`](2026-07-26-read_a_brick_in_place.md) also
adds a field to `App.PageState` (`brick`), and also opens a dialog held in
history state. Whichever merges second inherits two small obligations: the
interface gains both fields rather than being replaced, and the two overlays need
one rule about what happens when both could be open (the picker is a landing-page
surface and the reader is a wall surface, so the simple answer is that opening
either closes the other).

[`2026-07-26-refresh_the_wall.md`](2026-07-26-refresh_the_wall.md) names its own
half of the feed interaction: `refresh` over a feed wall bypasses the
`feed_pages` entry rather than the author-feed caches.

### Tests worth writing before the code

- `feedref.rs`: all three accepted spellings and which variant each returns, an
  AT-URI naming `app.bsky.feed.post` (rejected), a `javascript:` string, a URL
  on a lookalike host, and a reference carrying `&`.
- `FeedTarget::from_query`: feed wins over actor, actor alone, feed alone,
  neither (bad_request), and `kind()` for both. This is a mortar-core unit test
  precisely because neither front can host one.
- `cursor.rs`: a feed cursor round-trips; a graph cursor still round-trips; a
  legacy cursor with a stray `snapshot` key still decodes to `Wall`; a feed
  cursor on the graph path decodes to something the graph path treats as a fresh
  wall rather than a panic.
- A wiremock feed wall in `feed.rs`'s test module: the page keeps upstream order
  (which is the whole claim), drops a repost and an opted-out author's post,
  ends when `getFeed` returns no cursor, and under `Mode::Glaze` lays only the
  image posts.

---

## Merge plan

1. Apply each `Proposed changes` block to its canonical page; bump that page's
   `**Date:**` to the merge date.
2. Fold the `Type changes` `$defs` into
   `.specs/canonical-types.schema.json`: add `FeedRef` and `HiddenLabel`, replace
   `CursorPayload` and `MortarErrorCode`.
3. Flip this file's `**Status:**` to `Merged`, add `**Merged:** YYYY-MM-DD`, and
   move it to `.specs/changes/merged/`.
4. Update `.specs/README.md`: remove it from the pending list and add it to the
   merged table.

---

## Assumptions and open questions

**Assumptions**

- `app.bsky.feed.getFeed` is readable unauthenticated on the public AppView,
  serves `access-control-allow-origin: *`, and pages with its own cursor.
  Verified on 2026-07-26 against `at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot`:
  200 with a wildcard CORS header, and a second page under the returned cursor.
- `getFeed` hydrates its results into the same `PostView` shape
  `getAuthorFeed` returns, with `labels` on both the post and its author. The
  shared mapper, and therefore the whole moderation inheritance, rests on this.
- `app.bsky.feed.getFeedGenerator` is readable unauthenticated (verified, 200),
  so the client can name a feed in the header without an engine round trip.
- An unknown feed generator produces a 400 from `getFeed` (verified), not a 200
  with an empty list.
- Feed generators are third-party services with their own uptime. A feed that
  times out is an upstream failure, not a mason failure, and the reader is told
  so.

**Decisions**

- *No snapshot for a feed wall.* **Page the feed and lay it.** The snapshot
  exists to accumulate an order out of a hundred concurrent sources and to hold
  it still under a cursor. A feed generator already published an order, so a pool,
  admission caps, extension waves and a warming reflow would be machinery
  defending an invariant that upstream is maintaining.
- *The feed's order is the algorithm.* **Grout and the mixer sit out.**
  Re-ranking somebody's published feed by mason's recency-dominant score would
  produce a wall that is neither theirs nor mason's, and the reader picked theirs.
- *A source and a view, not a list of walls.* **`actor` or `feed` picks the
  source, and all three views apply to either.** The alternative was a flat menu
  of walls (mixed, glaze, this feed, that feed) which grows by multiplication and
  would have to explain why glaze is missing from half of it. Two independent
  choices is also what the code already wanted: `Mode` selects kinds and has never
  known where a brick came from.
- *Glaze over a feed asks for 100 and lays every survivor.* **The filter is
  aggressive, and there is nowhere to put a remainder.** A general feed is mostly
  text, so a 24-brick request yields three or four images; and because the cursor
  belongs to the call that fetched them, truncating a page to `PAGE_SIZE` would
  discard the rest rather than defer it.
- *One mapping path, not two.* **Extract `map_feed_page` and share it.** A second
  copy of the moderation filters is a second place for the `!warn` tier to be
  forgotten, on a surface where the content is by definition from strangers.
- *A parse, not a fallback.* **An unparseable `feed` is a `bad_request`.** `mode`
  and `intent` fall back because they are optional decorations on a wall that
  exists either way. A malformed `feed` names no wall at all, and quietly laying
  the reader's graph instead would be a different product than the one they asked
  for.
- *A new error code.* **`feed_not_found`, not `actor_not_found`.** The client
  maps `actor_not_found` to "no wall for that handle" and hands back a handle box.
  Reusing it would tell somebody with a bad feed link to fix their handle.
- *No owner gate on a feed wall.* **`!no-unauthenticated` is about a person's
  graph, not a published service.** Per-post and per-author filtering still runs,
  which is what keeps an opted-out account's posts off a feed wall.
- *The generator's identity is the client's read.* **`getFeedGenerator` from the
  browser, like `getProfile` already is.** Returning it on the wire would mean a
  new response field present on some walls and null on others, to carry two
  strings the header alone consumes.
- *Feed pages cached for 60 seconds, never persisted.* **A ranking has a
  deadline.** It buys the preview-then-freeze pair and back/forward for one call,
  and refuses to lay hour-old ranking as fresh.
- *`limit = PAGE_SIZE` on the mixed views, and short pages are fine.* **A handful
  of reposts dropped out of 24 is not worth over-fetching for.** Serving short is
  already normal and the pump already retries.
- *A picker screen, not a second input.* **The two front doors are peers.** A feed
  field beside the handle box would ask the reader to already have a feed URI in
  their clipboard, which is the same assumption the handle box makes and the one
  the picker exists to remove. One input inside the picker still serves search, a
  creator's feeds and a pasted link, because what somebody types is enough to tell
  which of the three they meant.
- *A feed wall carries no blogs and no streams.* **A feed generator returns post
  URIs, and mason lays what it returns.** A hybrid (a feed for the posts, the
  reader's own graph for the blogs and streams around them) would need the mixer
  back, an `actor` alongside the `feed`, and a snapshot to mix into, which is
  precisely the machinery this change gets to skip. The reader who wants blogs and
  streams has the graph wall, one control away.
- *The picker filters labelled feeds, and the list is pinned rather than copied.*
  **`vocab.hiddenLabels` in the contract fixture.** A directory listing is chrome,
  so it reads the AppView directly like `profile` does, but mason should not
  advertise a feed it would refuse to lay. That needs the hidden tier on both sides
  of the wire, and the fixture already has the exact mechanism for keeping a Rust
  list and a TypeScript one in step.

**Open questions**

- *Reposts are dropped.* The shared mapper drops `reason != null`, which is right
  on an author feed (a repost is not that author's brick) and arguable on a feed
  wall, where a repost is content the feed deliberately surfaced. Keeping them
  would need `Brick` to carry reposted-by attribution, or they would render as
  the original author's post with no context. Open: is the attribution worth a
  wire field?
- *Browse and search both ride on unspecced endpoints.*
  `getPopularFeedGenerators` serves both the popular list and, with its `query`
  parameter, the search. Both answer 200 today and `app.bsky.unspecced.*` carries
  no stability promise whatsoever, while the stable-looking
  `app.bsky.feed.searchFeedGenerators` answers 501 on the public AppView. Recents
  and paste are the load-bearing paths and degrade cleanly, so this is not a
  blocker; it is a dependency that will break one day without a deprecation
  notice. Open: is a cached mirror of the popular list worth carrying so the
  picker is never empty?
- *The popular list is the same for everybody.* Logged out there is nothing to
  personalise it with, so every reader sees the network's own ranking. That is
  honest, and it also means the picker's resting state is the most mainstream
  possible view of the atmosphere, which is an odd first impression for an app
  whose pitch is the content the big apps do not show. Open: should the resting
  state be something else, and what?
- *A feed's own view is not a wall.* A feed generator record carries a
  description, a like count and a creator, all of which the picker shows and none
  of which appears once the wall is laid. Open: does a feed wall want a header
  line naming what the reader is reading, beyond the avatar and name in
  `SwitchWall`?
- *Recent feeds in `localStorage`.* Logged out, there is no saved-feeds list to
  read, so `mason:feeds` is a purely local memory. It will not follow a reader to
  another browser, and mason has nowhere to put it that would.
