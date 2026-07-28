# 04 - Sources and Moderation

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

Everything mason reads from the network enters through `sources/`. Each submodule
reads one upstream and maps it into bricks; `sources/fetch.rs` is the seam the
rest of the crate consumes, one fetch-and-cache function per source. `algo/`
never names a source module, and the cache and persistence structs take only the
yield types re-exported from `sources/mod.rs`. Swapping an ingestion backend is
meant to touch this directory and nothing else.

---

## Responsibilities

1. Speak each upstream's wire format and map it into `Brick` and the small set of
   engine-internal yield types.
2. Apply logged-out moderation at ingestion, so nothing downstream has to.
3. Sanitise every third-party string that will reach an `<a href>` or an outbound
   request.
4. Degrade to empty yields rather than errors, and distinguish a transient
   failure (do not cache, ask again) from an honest refusal (cache as empty).

The seam does **not** own: rate limiting or retry (that is `http.rs`), TTL policy
enforcement (that is `cache.rs`, though the source-shaped TTL *values* live here),
or any scoring.

---

## The upstreams

| Source | Base | Endpoints | Bucket |
|---|---|---|---|
| Bluesky AppView | `https://public.api.bsky.app` | `app.bsky.actor.getProfile`, `app.bsky.graph.getFollows`, `app.bsky.feed.getAuthorFeed`, `app.bsky.feed.getFeed` | `Appview` (rate limited) |
| PLC directory | `https://plc.directory` | `GET /<did>` (DID documents) | `Unmetered` |
| Each author's PDS | resolved per author | `com.atproto.repo.listRecords`, `com.atproto.repo.getRecord`, `com.atproto.sync.getBlob` | `Unmetered` |
| Streamplace | `https://stream.place` | `place.stream.live.getLiveUsers`, `place.stream.playback.*` | `Unmetered` |

All three bases are fields on `Config` and are overridable, so tests point mortar
at a wiremock server. In server mode they are additionally overridable by
`APPVIEW_BASE`, `PLC_BASE` and `STREAMPLACE_BASE` environment variables.

Those three are the whole of the server's upstream configuration. It reads two
more environment variables that are not upstreams and belong here only so the
list is complete: `PORT`, which is the port it binds and defaults to 8787, and
`MASON_ALLOWED_ORIGINS` (see [06](06-wire-contract.md)). Anything unparseable
falls back to the default rather than failing to start, on the same reasoning as
the query vocabulary: the safe direction is the default one.

`did:web` identities skip plc.directory and read
`https://<domain>/.well-known/did.json` instead.

---

## HTTP policy

One shared `Http` (`http.rs`) sits under every source. The rate limiter and the
retry loop are transport-agnostic; only the one-shot GET underneath is split by
target. Native drives `reqwest` (hyper + rustls); the browser drives `gloo-net`,
a thin wrapper over the fetch the service worker already has, so the wasm build
carries no HTTP stack of its own and the browser owns the user agent, TLS, gzip
and connection limits.

| Policy | Value |
|---|---|
| AppView bucket | 10 requests/second sustained, burst 100 |
| Other hosts | Unmetered; per-source fan-out bounds concurrency instead |
| Attempts | 3 |
| Retryable | Transport errors, 429, and 5xx |
| Backoff | `Retry-After` seconds capped at 30, else `500 ms × 2^attempt` |
| Request ceiling | 10 s per attempt, applied in shared code so both transports share one policy |
| User agent (native only) | `mason/<crate version> (+https://github.com/antstanley/mason)`, derived from `env!("CARGO_PKG_VERSION")` |

Two details are load-bearing. A retryable status on the *final* attempt returns
the real status rather than a generic `RetriesExhausted`, because no further
request will be made and sleeping only delays the answer. And on wasm the bucket
wait is a hand-rolled check-then-sleep loop through `platform::sleep`, because
governor's own async wait builds a timer that reaches for `std::time::Instant`
and `std::thread`, both of which trap on `wasm32-unknown-unknown`. The gate is
still `check()`, so a burst is never let through unthrottled.

The burst of 100 is sized so a whole 100-author cohort goes out at once. At a
burst of 40 the remaining sixty queued behind the drip, the pool grew at ten
bricks a second, and a reader could out-scroll their own wall.

The browser build sends the browser's own user agent and cannot override it,
which is correct: in local mode the request genuinely is the reader's browser
making it. The native string carries a contact URL because mason reads public
endpoints unauthenticated, and the user agent is the only channel an upstream
operator has to reach whoever is generating the traffic.

---

## Per source

### Bluesky (`sources/bluesky.rs`)

- **`get_profile(actor)`** returns `{did, opted_out}`. Accepts a handle or a DID,
  which is what lets a cold handle load skip a separate `resolveHandle` hop.
- **`get_follows(did, from, max_pages)`** pages the graph 100 at a time and
  returns `(follows, next_cursor)`, so a caller can take a head start and finish
  later. Each `Follow` carries the account's `labels`.
- **`get_author_feed(did)`** reads `filter=posts_no_replies&limit=30`: the full
  wall's shallow skim.
- **`get_image_feed(did)`** reads `filter=posts_with_media&limit=100`: the glaze
  wall's deep read. It narrows to posts carrying an image or video (replies
  included, which the full wall omits), so one request reaches much further back
  and returns far more images than skimming 30 mostly-text posts would.

  Both feed reads take a `refresh` argument. When it is set, the cache is not
  consulted and the AppView answer overwrites whatever was there. A refreshed
  read that fails *transiently* falls back to the cached yield rather than
  returning `None`, so a refresh can never lay a thinner wall than the one it
  replaced: the author did answer, just earlier. A refreshed read with nothing
  cached behind it behaves exactly like a cold one.

- **`get_feed(feed_uri, cursor, limit)`** pages a feed generator through the
  AppView and returns `(AuthorYield, Option<String>)`: the mapped bricks and the
  upstream cursor. It shares the *exact* mapping path with both author-feed
  reads, which is the whole reason a feed wall inherits moderation, `!warn`
  blur, video-embed unwrapping and repost dropping without a second
  implementation of any of them. `getFeed` hydrates its results into the same
  `PostView` shape `getAuthorFeed` returns, labels included, so the shared
  mapper needs no branch.

  A 400 or 404 is an unknown or withdrawn feed and becomes `FeedNotFound`. Any
  other failure is `Upstream`. Unlike an author feed, this is the wall's only
  source: there is no hundred-author quorum to degrade into, so a failure is
  the request failing rather than a thin wall.

  This read takes a `refresh` argument too, and steps over its `feed_pages`
  entry exactly as the two above step over theirs: on a feed wall that entry is
  the whole of what "new posts" means. What does not carry over is the fallback.
  There is no hundred-author quorum to degrade into here, so a refreshed read
  that fails is the request failing, exactly as an unrefreshed one is.

All three feed reads share one mapping path (`map_feed_page`): drop reposts
(`reason != null`), drop anything a logged-out viewer must not see, blur the
soft-warn tier, then map to bricks. A post whose embed is
`app.bsky.embed.video#view` becomes a video brick; `recordWithMedia` is unwrapped
to its media half first; everything else becomes a post brick carrying images or
an external embed.

### PDS resolution (`sources/pds.rs`)

Every repo-reading source needs the author's PDS endpoint (blogs, archived
streams, and the blobs behind both), so resolution lives here and is cached for a
day: one author costs one identity lookup no matter how many collections are
read from their repo.

`blob_url(pds, did, cid)` builds
`{pds}/xrpc/com.atproto.sync.getBlob?did=…&cid=…`, which is how blog covers and
stream posters reach the browser.

### standard.site (`sources/standardsite.rs`)

`com.atproto.repo.listRecords?collection=site.standard.document&limit=25` against
the author's own PDS. Each document points at a publication, and it is nearly
always the *same* publication (a blogger has one blog), so publications are
memoised per author within the call. Fetching one per document meant 25
sequential `getRecord` calls for one author, which is what made the repo fan-out
take twenty seconds and left the first wall with no blogs on it.

Two shapes are handled defensively because the lexicon is young:

- `site` may be an AT-URI or a plain `https://` URL. A plain URL implies the
  publication (host as name), no fetch needed.
- `bskyPostRef` is officially a strongRef `{uri, cid}` but some publishers write a
  bare string, either an AT-URI or a bare rkey. All three resolve to a post
  AT-URI; anything else yields nothing.

A record that will not parse is logged and skipped, and a document with no `site`
is skipped entirely (a card that links nowhere is not wall-worthy). A 400 or 404
from `listRecords` means "no blog here" (some PDS implementations 400 on a
collection the repo has never held), not an error.

**Suppression.** A document's `bskyPostRef` names the skeet it was cross-posted
to. `Snapshot::admit_repo_yield` withdraws that post from the pool if it is
already there and inserts its id into `seen` if it is not, so the blog card wins
whether the post arrived first or later. Posts race ahead of the repo reads, so
"later" is the common case.

### Streamplace (`sources/streamplace.rs`)

Two shapes reached two ways:

- **Live now.** One call to `place.stream.live.getLiveUsers?limit=100` returns
  everyone streaming anywhere on the network, and the caller intersects that with
  the viewer's follow graph. Far cheaper than asking every author in turn, and one
  page covers the network.
- **Archived.** `place.stream.video` records in the author's own repo, listed
  exactly like blog documents, capped at 10 per author.

`LiveStream` is deliberately *not* a brick. It is a fact about Streamplace,
identical for every viewer, which is what makes it safe to cache under a single
key. Turning it into a brick is a fact about one viewer (do they follow this
person, where does that person's poster live), and conflating the two would serve
the first viewer's friends to the second.

Two record-shape workarounds:

- A livestream record is created once and reused for months, so its `createdAt` is
  roughly the day the streamer signed up. `lastSeenAt`, the heartbeat, is
  preferred; without it a four-month-old record would age out of the wall while
  its owner is live on it.
- Archived `place.stream.video` records may predate the server-side `createdAt`.
  An atproto rkey is a TID whose top bits are microseconds since the epoch, so
  `tid_created_at` recovers the timestamp from the key. The record carries its own
  birthday whether or not anyone wrote it down.

Playback is HLS from `place.stream.playback.getLivePlaylist` (live) or
`getVideoPlaylist` (archived). Both serve `access-control-allow-origin: *`, so
the wasm build reaches them straight from the browser and one hls.js player
handles all three video kinds.

---

## Moderation

mason reads logged out, so it mirrors what a logged-out Bluesky viewer is shown.
Labeler labels (from the default moderation service) and self-labels both land in
the same `labels` array, so one check covers both.

### The two tiers

| Tier | Labels | Effect |
|---|---|---|
| Hidden | `!hide`, `!no-unauthenticated`, `porn`, `sexual`, `graphic-media` | The subject never reaches the wall |
| Warn | `!warn` | The subject's media is covered behind a per-brick reveal |

`nudity` is deliberately absent: it carries no adult flag and Bluesky shows it to
logged-out viewers, so mason does too.

Because the hidden tier is applied first, anything that reaches the warn tier can
always be revealed. The client never has to decide whether a blur is
unrevealable.

### Where each check happens

```
                        ┌─ the wall's OWNER: getProfile label      ──▶ 403 login_required
                        │     (feed.rs, resolve_and_gate)
labels are checked at ──┤
                        ├─ each FOLLOW: Follow::hidden()           ──▶ dropped from the cohort
                        │     (cohort.rs, sample_cohort + next_wave)
                        │     …and from the live filter (fetch.rs, followed_live)
                        │
                        └─ each POST and its AUTHOR: in the feed mapper
                              (bluesky.rs, author_feed)             ──▶ dropped, or blurred
```

Dropping a hidden account from the **cohort** is the single choke point that
keeps every one of their sources off the wall at once: posts, blogs, archived
streams. The author-feed label filter alone would miss the blogs and streams,
because those come from the author's own repo where the AppView's labels never
reach. The live filter repeats the check for the same reason: a live stream comes
from Streamplace, which never sees an AppView label.

`!no-unauthenticated` is a request, not a guarantee: the public AppView still
serves the content. That is exactly why the client has to honour it, because
nothing upstream does.

---

## Outbound safety

Two classes of untrusted string leave `sources/`, and both are vetted here.

### URLs that will reach an `<a href>`

`is_http_url` accepts only `http://` and `https://`, case-insensitively and after
trimming. Third-party records carry arbitrary strings in their url fields, and
`javascript:`, `data:` and `vbscript:` must never survive the trip to the anchor.
A blog's canonical URL falls back to empty, a live stream's record URL falls back
to the stream.place watch page, and a post's external embed is dropped **whole**
(`external_embed` in `sources/bluesky.rs`): a link card with nowhere to go is a
headline for a page nobody can open, and a post carrying nothing else then fails
the wall-worthiness check and never reaches the wall at all.

That embed's `thumb` is vetted by the same rule for a different reason, since it
reaches an `<img src>` rather than an anchor and no browser runs script from an
image source. The AppView resolves that picture itself and hands back its own CDN
link every time, so anything else is either a picture mason cannot draw or bytes
carried inline past the page's own network rules. The thumb alone is dropped and
the embed stays, through the fallback a link that brought no picture already
takes.

`urlencode` percent-encodes everything outside the RFC 3986 unreserved set for
any value interpolated into a query string, so a handle, DID or cursor containing
`&`, `#`, `?` or a space cannot rewrite or truncate the upstream query, or poison
a cache key derived from it.

A `feed` parameter is a third class of untrusted string: it does not reach an
anchor, it reaches an *upstream query*. `FeedRef::parse` is what vets it. It
requires the `at://` scheme and the exact `app.bsky.feed.generator` collection,
or a `bsky.app` feed URL it rebuilds an AT-URI from, and rejects everything else
as `bad_request` rather than forwarding it. The result is `urlencode`d into the
`getFeed` query like every other interpolated value, so a reference carrying
`&` or `#` cannot rewrite the upstream request or poison the cache key derived
from it. There is no SSRF surface: the host is always the configured AppView
base, never anything the reference names.

### Requests mortar itself makes (SSRF)

Both the `did:web` domain and the `serviceEndpoint` come out of an untrusted DID
document, and in server mode the native binary fetches them verbatim. A hostile
document could otherwise aim mortar at the machine's loopback, the cloud metadata
endpoint, or an internal host.

```
validate_endpoint:  require https://          (http, file, data, … rejected)
                    reject any userinfo (@)   (it hides the real host from a naive read)
                    ▼
validate_host:      reject localhost, *.localhost, *.local
                    reject IP literals in private / loopback / link-local / ULA /
                           unspecified / broadcast / 0.0.0.0-0.255.255.255 ranges
                           (IPv4-mapped IPv6 unwrapped first)
                    ▼
native only:        resolve the name and reject if ANY answer is a blocked address
```

The literal and string checks run in both builds. The DNS check is native-only:
wasm has no `std::net` resolver, and there the browser is the client and does the
fetching, so SSRF is not mortar's to prevent. An unresolvable host is allowed
through unchanged, because a host the resolver cannot reach is one the follow-up
fetch cannot reach either.

---

## Failure semantics

The rule is that a single author, or a single source, failing must never sink the
wall. But a blip must not be remembered as a fact either.

| Outcome | Meaning | Cached? | Caller sees |
|---|---|---|---|
| Transport error, `RetriesExhausted`, 429, 5xx | Transient | No | `None` from `author_feed_cached` / `image_feed_cached`: the author never answered, so they are not recorded as fanned and a later wave asks again |
| Transient failure on a refreshed read | Transient | No (the older entry survives) | The previously cached yield, so the refreshed wall is never thinner than the one it replaced |
| Other 4xx on an author feed | The AppView's honest answer (suspended or deleted repo) | Yes, as an empty yield | An author who genuinely yields nothing |
| 400/404 on a repo `listRecords` | The collection has never existed here | Yes, empty | "This person does not blog / does not stream" |
| Any other repo failure | Transient | No | An empty yield this time; the next snapshot asks again |
| Live list failure | Transient | Yes, as empty (60 s TTL) | No live bricks this minute |
| PDS resolution failure | Transient | No | That author's repo is skipped this round |

The positive/negative TTL split follows from this: publishers and streamers get
rechecked within the hour, the silent majority is left alone for a day, and only
a *successful* empty listing ever earns the long negative TTL. See
[05-caching-and-persistence.md](05-caching-and-persistence.md) for the values.

---

## Implementation layout

```
server/crates/mortar-core/src/sources/
  mod.rs           the seam's exports: AuthorYield, Follow, StdDocs, LiveStream
  fetch.rs         one fetch-and-cache function per source; the ONLY door algo/ uses
  bluesky.rs       profile, follows, author feeds, feed generator pages, labels,
                   post → brick
  feedref.rs       the ?feed= parameter → an AT-URI, or a rejection
  pds.rs           DID document → PDS endpoint, SSRF vetting, blob_url
  standardsite.rs  site.standard.document → BlogBrick, publications, suppression
  streamplace.rs   live list and place.stream.video → VideoBrick, TID timestamps
  util.rs          urlencode, is_http_url
```

---

## Assumptions and open questions

**Assumptions**

- Labeler and self-labels arrive in the same `labels` array on profile views,
  author views and post views.
- `com.atproto.repo.listRecords` is readable unauthenticated on public repos.
- Streamplace's live network fits in one 100-record page.
- Streamplace playback endpoints and the AppView's CDN serve permissive CORS.
- A DID document's `serviceEndpoint` for `#atproto_pds` is an `https` URL.

**Decisions**

- *One HTTP surface, two transports.* **Shared limiter and retry, split
  one-shot GET.** `reqwest` on wasm was only a fetch wrapper anyway, and it
  dragged the `url` crate's IDNA and ICU tables in with it; `gloo-net` does not.
- *User agent derived, not written.* **`env!("CARGO_PKG_VERSION")`.** A literal
  version string is a version declaration nothing propagates to, and it drifted
  by six minor releases before anyone noticed.
- *Hidden accounts filtered at the cohort.* **Before any content is fetched.** It
  is the only point where all four sources are covered at once.
- *`nudity` is not hidden.* **Bluesky shows it to logged-out viewers.** mason
  mirrors the platform rather than inventing a stricter policy.
- *The live list is cached un-filtered.* **`LiveStream`, not `Brick`.** A cache of
  already-filtered bricks under one key would serve one viewer's friends to the
  next.
- *One eager follow page.* **The cohort samples 100; a page is 100.** Every
  further page is a sequential round trip that fetches nothing while it blocks; a
  2000-follow graph cost twenty of them and ten seconds of empty wall.
- *Partial follow graphs are not cached.* **Only the completed list is.** A
  partial graph in the cache would masquerade as the whole one for an hour.
- *Publications memoised per author.* **One `getRecord` per publication.** Per
  document it was 25 sequential calls and the first wall had no blogs on it.
- *TID-derived timestamps.* **Recover `createdAt` from the rkey.** Dropping
  archived streams that predate the server-side field would throw away real
  content.
- *SSRF vetting is native-only for DNS.* **String checks in both builds, resolver
  checks natively.** In the browser the fetch is the browser's, so the threat
  model is the browser's.

**Open questions**

- *Author-feed depth.* The full wall reads the last 30 posts per author with no
  paging. A very chatty author's older-but-better brick is unreachable. Open: is
  depth worth a second round trip per author, given the per-author admission cap
  is 4?
- *`Publication.icon`.* The publication record read asks only for `name` and
  `url`, so the icon is always `None`. Open: is there an icon field in the
  lexicon worth reading?
- *Streamplace beyond 100 live.* If the network outgrows one page, the live list
  silently truncates. Open until the network is that big.
