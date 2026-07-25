# 01 - Domain Model

**Status:** Draft · **Date:** 2026-07-25 · **Owner:** Ant Stanley

This page defines the entities mason manages, how they are identified, how they
relate, and what lookups the cache layer must serve. The wire shape of the
externally visible entities is formalised in
[`canonical-types.schema.json`](canonical-types.schema.json); the caches that
hold them are described in
[05-caching-and-persistence.md](05-caching-and-persistence.md).

---

## ID scheme

mason mints no identifiers. Every brick's `id` is the atproto AT-URI of the
record it was built from, copied verbatim from upstream:

```
at://<did>/<collection>/<rkey>
```

| Brick kind | Collection in the id |
|---|---|
| Post | `app.bsky.feed.post` |
| Video (Bluesky) | `app.bsky.feed.post` (the post carrying the video embed) |
| Blog | `site.standard.document` |
| Video (archived stream) | `place.stream.video` |
| Video (live) | `place.stream.livestream` |

Three consequences follow, and all three are load-bearing:

- **Dedup is free and cross-source.** A snapshot's `seen` set is a set of
  AT-URIs, so the same record arriving twice from two fans is admitted once.
- **Suppression works across lexicons.** A `site.standard.document` carrying a
  `bskyPostRef` names the post URI it cross-posted to, and the blog brick
  withdraws that exact post from the pool.
- **A brick id is stable across snapshots and sessions.** Client-side dedup, DOM
  keying and the masonry column memory all key off it.

Authors are identified by DID (`did:plc:…` or `did:web:…`). Handles are display
data and are resolved to a DID once, then cached; they are never a key.

Cursors carry no identity of their own: they are an opaque base64url encoding of
`{snapshot, seed, offset}`.

### Wire primitives

Six primitive shapes recur across the entities below and are named in
[`canonical-types.schema.json`](canonical-types.schema.json) so the constraints
live in one place:

| Primitive | Shape | Constraint that matters |
|---|---|---|
| `Did` | `did:plc:…` or `did:web:…` | The only identity mason keys on |
| `Handle` | a lowercased Bluesky handle | Display data and a request parameter, never a key |
| `AtUri` | `at://<did>/<collection>/<rkey>` | Every brick id is one, verbatim from upstream |
| `Timestamp` | RFC3339 | Author-supplied and untrusted; unparseable or far-future values are treated as stale |
| `HttpUrl` | `http(s)://…`, or empty | A third-party URL that fails `is_http_url` becomes empty rather than reaching an anchor |
| `AspectRatio` | `{width, height}` | Lets a card reserve space before its media loads |

---

## Entities

### Brick

The unit of content on the wall. Serialised as an internally-tagged union on
`kind`, so the web client receives a discriminated union.

```
Brick = Post(PostBrick) | Blog(BlogBrick) | Video(VideoBrick)
```

Every variant carries `id`, `url` (where a reader goes to see it at the source),
and `author`. Two behaviours are defined on the union itself:

- `is_image_post()` is true only for a `Post` carrying at least one image. It is
  the whole definition of what belongs on the glaze wall; a native-video post and
  a text- or link-only post are both excluded.
- `set_blur()` covers a brick's media behind a reveal. Only posts and videos can
  carry a blur: blogs come from a source Bluesky's labels never reach.

### PostBrick (`kind: "post"`)

A Bluesky post that is not a native video.

- `text` - the record's text, rendered as the card body
- `createdAt` - RFC3339, author-supplied and therefore untrusted
- `likeCount`, `repostCount` - the engagement inputs to the grout score
- `images` - zero or more `ImageEmbed` (`src` is the AppView `thumb`, plus `alt`
  and an optional `aspectRatio`)
- `external` - an optional `ExternalEmbed` (`uri`, `title`, `description`,
  `thumb`) rendered as a link preview inside the card
- `blur` - present only when a `!warn` label covers the media

### BlogBrick (`kind: "blog"`)

A `site.standard.document` record from an author's own repo.

- `title`, optional `description`, optional `coverImage` (a blob URL on the
  author's PDS)
- `publication` - `{name, url, icon}`, resolved from the document's `site`
  pointer; one fetch per publication per author, not per document
- `tags` - the record's tags, of which the card renders at most four
- `publishedAt` - RFC3339
- `url` - `publication.url` joined with the document `path`, and empty unless the
  result is an `http(s)` URL

A blog brick is metadata and a link out. The `content` union of the lexicon is
platform-specific (Leaflet, pckt.blog, Offprint, WordPress all differ) and is
never parsed or rendered.

### VideoBrick (`kind: "video"`)

Three different things share one shape, distinguished by `source` and `live`:

| `source` | `live` | What it is | Half-life / window |
|---|---|---|---|
| `bluesky` | `false` | A post whose embed is `app.bsky.embed.video#view` | 12 h / 72 h |
| `streamplace` | `false` | A `place.stream.video` archived stream | 14 d / 90 d |
| `streamplace` | `true` | A `place.stream.livestream` happening now | Exempt from the age window |

- `playlist` - an HLS `.m3u8` URL: Bluesky's `playlist`, or
  `place.stream.playback.getLivePlaylist` / `getVideoPlaylist`
- `poster` - Bluesky's `thumbnail`, or a blob URL on the streamer's PDS
- `aspectRatio` - Bluesky's if present; Streamplace bricks are always 16:9
- `viewerCount` - live only; it is also the engagement signal for a live brick
- `durationMs` - archived streams only
- `activity` - what the streamer says they are doing
- `captions` - a list of `CaptionTrack` (`src`, `lang`, `label`). Modelled and
  rendered, but empty on every brick today: neither upstream carries caption data
- `blur` - only ever set on native Bluesky videos

### Author

`{did, handle, displayName, avatar}`. Denormalised onto every brick, because
bricks are the only thing the wire carries and the client never fetches authors.

### Follow

The engine-internal view of one edge of the follow graph: an `Author` plus the
account's `labels`. `Follow::hidden()` is the single choke point that keeps an
opted-out or adult-flagged account out of the cohort, and therefore off every
source at once.

### Profile

A profile view reduced to `{did, opted_out}`. One `getProfile` call resolves a
handle to a DID and reads the wall owner's logged-out opt-out at the same time.

### Snapshot

One viewer's wall in progress. Not serialised; it lives only in the engine.

Identity: `snapshot_id = "<mode-tag>-<xxh3_64(did, seed) as 16 hex>"`, so a glaze
wall and a full wall for the same actor and seed can never collide.

State it owns:

| Field | Meaning |
|---|---|
| `pool` | Admitted bricks not yet laid |
| `wall` | Laid bricks, append-only, in final order |
| `seen` | Brick ids admitted or suppressed |
| `kind_counts` / `kind_caps` | Per-mix-slot population and its admission ceiling |
| `author_counts` | Bricks held per (author, kind), against `max_per_author` |
| `fanned` | Author DIDs already asked, so waves never repeat one |
| `warming` | The initial fill is still running |
| `extending` | When the in-flight extension wave was claimed (a dead man's switch) |
| `graph_spent` | A wave found nobody left to ask |
| `slow_fans` | Rare-kind fans not yet finished (repo reads, live list) |
| `max_age_hours` | Admission window override; `None` uses the per-kind default |
| `max_per_author` | 4 on the full wall, 8 on glaze |

### Cursor

The `CursorPayload` is `{snapshot: String, seed: u64, offset: usize}`,
JSON-serialised and base64url (no padding) encoded into the opaque `Cursor`
string the client sees. It is attacker-writable, so every consumer treats it
defensively: a garbage or tampered cursor decodes to `None` and falls back to a
fresh wall, and the offset is added with `checked_add` / `saturating_add`.

`seed` is the load-bearing field. It drives the cohort shuffle and the mixer's
jitter, so a snapshot evicted mid-scroll rebuilds into a closely-matching wall
from the same seed and the still-warm per-author caches.

---

## Relationships

```
        Viewer (DID)
            │ 1
            │ follows *
            ▼
         Follow ──────has──────▶ Label[]
            │                       │
            │ sampled into          └─▶ hidden()  drops the author entirely
            ▼
         Author  ◀──────denormalised onto──────┐
            │ 1                                │
            │ authors *                        │
            ▼                                  │
          Brick ────────────────────────────────
         ╱  │  ╲
     Post  Blog  Video
       │     │     │
       │     │     ├── ImageEmbed[] · ExternalEmbed?   (Post)
       │     │     ├── Publication                     (Blog)
       │     │     └── CaptionTrack[]                  (Video)
       │     │
       │     └── bskyPostRef ──suppresses──▶ Post (by AT-URI)
       │
       ▼
    Snapshot (pool ──laid──▶ wall) ──paged by──▶ Cursor{snapshot, seed, offset}
       │
       └── one per (Mode, viewer DID, seed)
```

---

## Snapshot lifecycle

```
   request with no cursor
            │  fresh_seed(did)
            ▼
     ┌─────────────┐   get_or_insert_with (exactly one caller wins)
     │   CREATED   │──────────────► background fill spawned
     └──────┬──────┘
            │ follows ▸ cohort ▸ fan out (posts ‖ repos ‖ live)
            ▼
     ┌─────────────┐  first paint: 12 distinct authors, or 3 s
     │   WARMING   │  mix wait:    slow fans done, or 6 s from creation
     └──────┬──────┘
            │ fill finishes (or fails)
            ▼
     ┌─────────────┐  pool < 48 and graph not spent
     │   SETTLED   │──────────────► EXTENDING ──► SETTLED (caps raised)
     └──────┬──────┘   ▲                │
            │          └────────────────┘  waves run single-file,
            │                              presumed dead after 60 s
            │ a wave finds nobody left to ask
            ▼
     ┌─────────────┐
     │ GRAPH SPENT │  the wall can genuinely end: pool empty + no wave
     └─────────────┘  ▸ the page returns cursor: null
            │
            ▼  30 min TTL, or the service worker is reaped
     ┌─────────────┐
     │   EVICTED   │  the cursor's seed rebuilds a matching wall from warm caches
     └─────────────┘
```

`has_more()` is the negation of the end condition, and it is deliberately
generous: more can still arrive if the pool is non-empty, the fill is running, a
wave is in flight, **or** the graph has never been declared spent. Only an empty
pool on a settled snapshot with a spent graph ends the wall.

---

## Required query patterns

Every lookup below is served by an in-memory TTL cache; none touches disk except
through the browser build's IndexedDB export.

| Query | Key | Served from |
|---|---|---|
| Handle to DID | handle | `caches.did` |
| Wall owner opted out? | owner DID | `caches.profiles` |
| Viewer's follow graph | viewer DID | `caches.follows` |
| Authors that yielded content recently | `activity_key(viewer, mode)` | `caches.activity` |
| One author's recent posts | author DID | `caches.author_feed` |
| One author's recent media posts (glaze) | author DID | `caches.image_feed` |
| Where an author's repo lives | author DID | `caches.pds` |
| One author's blog documents | author DID | `caches.std_docs` |
| One author's archived streams | author DID | `caches.streams` |
| Who is live, network-wide | the single key `0u8` | `caches.live` |
| A wall in progress | `snapshot_id` | `caches.snapshots` |

The live list is the only viewer-independent cache, which is what makes a single
key safe. Turning it into bricks is per-viewer and happens downstream of the
cache, in `fetch::live_bricks`.

---

## Assumptions and open questions

**Assumptions**

- AT-URIs are globally unique and stable for the life of a record, so they are
  safe as brick ids and as dedup keys.
- `createdAt` on a post record is author-controlled and may be wrong in either
  direction.
- An `rkey` on a `place.stream.video` record is a TID, so its top bits are
  microseconds since the epoch. Records missing `createdAt` recover their
  timestamp from it.
- A publication record's `url` is a third-party string and may be any scheme.

**Decisions**

- *Brick ids.* **The upstream AT-URI, verbatim.** Minting synthetic ids would
  need a mapping table to dedup against, and cross-lexicon suppression
  (`bskyPostRef`) only works if the blog record's pointer and the post's id are
  the same string.
- *Author denormalisation.* **Copied onto every brick.** The wire carries only
  bricks, and a client-side author lookup would mean a second request shape and a
  second cache for data that is a hundred bytes.
- *One `VideoBrick` for three things.* **`source` plus `live` discriminate.** The
  player is identical for all three (they are all HLS), and the differences that
  matter are scoring inputs, not rendering.
- *Cursor is opaque and untrusted.* **Base64url JSON, validated defensively.** It
  needs to survive a URL and a page reload; a signed cursor would need a key,
  which a client-side engine has nowhere to keep.
- *Seed in the cursor.* **Carried on every page.** It is what lets an evicted
  snapshot rebuild deterministically rather than roll a new wall mid-scroll.

**Open questions**

- *Caption tracks.* `CaptionTrack` is modelled end to end and rendered by the
  player, but no upstream populates it: the `app.bsky.embed.video` record can
  carry VTT blobs and the AppView's `#view` omits them, and Streamplace has none.
  Open until an upstream carries the data.
- *`Publication.icon`.* Modelled and rendered nowhere; `fetch_publication` always
  sets it to `None` because the publication record read does not ask for it.
  Open: populate it, or drop the field.
- *Blog engagement.* `engagement()` returns 0 for every blog, so blogs are ranked
  purely by recency within their kind. Open: is there a signal worth reading (the
  cross-posted skeet's likes, say), or is recency correct for the medium?
