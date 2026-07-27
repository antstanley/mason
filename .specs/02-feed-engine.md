# 02 - Feed Engine

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

This page covers `mortar-core`: the crate that turns a handle into pages of
bricks. Scoring and mixing are in [03-grout-and-mixer.md](03-grout-and-mixer.md);
ingestion is in [04-sources-and-moderation.md](04-sources-and-moderation.md); the
caches this engine reads and writes are in
[05-caching-and-persistence.md](05-caching-and-persistence.md).

---

## Responsibilities

1. Resolve an actor to a DID and refuse to lay a wall its owner sealed.
2. Build and own snapshots: one per (mode, viewer, seed), created on demand,
   filled in the background, paged immutably.
3. Fan out to the follow graph under a global rate limiter, admit what comes back
   under per-kind and per-author caps, and keep fanning out as the scroll drains
   the pool.
4. Serve a page: lay bricks from the pool onto the wall and slice the requested
   window out of it.
5. Bound every wait. No request blocks indefinitely on a slow upstream.

The engine does **not** own: HTTP transport policy (`http.rs`), per-source
parsing (`sources/`), or presentation. It never renders and never stores.

---

## Entry point

```rust
pub async fn handle_feed(
    state: &Arc<AppState>,
    target: FeedTarget<'_>,
    cursor: Option<&str>,
    mode: Mode,
    intent: FeedIntent,
) -> Result<FeedResponse, AppError>
```

Both fronts are thin wrappers around it. `mortar-server`'s axum handler parses
the query string into these arguments; `mortar-wasm`'s `feed_page` does the same
from JS strings.

`FeedTarget` is `Actor(&str)` or `Feed(&str)`, built by each front from the
query string: `feed` wins when both parameters are present, and neither being
present is a `bad_request`. Page size is a constant for both:
`PAGE_SIZE = 24`.

### Modes

| `?mode=` | `Mode` | Wall |
|---|---|---|
| absent, or anything unrecognised | `Wall` | Posts, blogs and video, mixed |
| `glaze` | `Glaze` | Bluesky image posts only |

An unrecognised value can never break a request: `Mode::from_query` falls back to
`Wall`. The mode's `tag()` is folded into the snapshot id and the per-viewer
activity key, so the two walls occupy separate cache namespaces. The author-feed
cache underneath is shared, moderation and blur intact.

### Intents

| `?intent=` | `FeedIntent` | Behaviour |
|---|---|---|
| absent | `Normal` | Commit a page; the first page waits for a decent mix |
| `preview` | `Preview` | Lay a throwaway first screen from a clone of the pool; never commit, never wait |
| `freeze` | `Freeze` | Commit the first screen immediately, without re-paying the mix wait |

The wasm front polls `Preview` while a wall warms and asks `Freeze` exactly once
to commit. `Normal` is what a client without a preview loop asks, and it is why
server mode still opens on a proper mix rather than on nothing but posts.

### Request flow

```
handle_feed(Actor(actor), cursor, mode, intent)
  │
  ├─ decode cursor ──▶ Some{seed, offset} | None (garbage decodes to None, and
  │                                               a feed cursor is treated as one)
  │
  ├─ actor == "demo" ? ──▶ fixture page from compiled-in bricks, return
  │
  ├─ resolve_and_gate(actor) ──▶ DID  |  Err(ActorNotFound | LoginRequired | Upstream)
  │
  ├─ seed = cursor.seed  or  fresh_seed(did)
  │
  ├─ intent == Preview ?
  │     ensure_snapshot ▸ preview_page(clone of pool) ▸ return {items, cursor(offset 0), warming}
  │
  ├─ get_or_build(did, seed, mode)          // blocks for first paint
  ├─ get_page(offset, PAGE_SIZE, wait_for_mix = intent == Normal)
  └─ return {items, cursor: has_more ? encode{seed, offset + items.len()} : null}
```

A preview's cursor points at the **current** screen (offset 0), not the next
page, so the freeze that follows commits from there.

---

## A feed wall

A feed generator is an algorithm somebody else published, and mason's job on a
feed wall is to lay it, not to re-rank it. So a feed wall skips almost the whole
engine: no snapshot, no pool, no admission caps, no cohort, no extension waves,
no grout and no mixer.

```
handle_feed(Feed(ref), cursor, mode, intent)
  │
  ├─ FeedRef::parse(ref) ──▶ AtUri  |  Err(BadRequest)
  │     a bsky.app feed URL resolves its profile segment to a DID first
  │
  ├─ decode cursor ──▶ Some{feed} | None   (a graph cursor here decodes to None:
  │                                         a fresh wall, not an error)
  ├─ feed_page_cached(uri, upstream cursor)
  │     ──▶ (bricks, next upstream cursor)  |  Err(FeedNotFound | Upstream)
  │
  ├─ Mode::Glaze ? ──▶ keep only is_image_post()   (and lay every survivor)
  ├─ Mode::Wall  ? ──▶ truncate to PAGE_SIZE
  │
  └─ intent == Preview ? {items, cursor: the INCOMING cursor, warming: false}
                       : {items, cursor: next.map(encode)}
```

Four consequences, and each one is the point:

- **There is nothing to warm.** One AppView call answers a page, so a preview
  reports itself already settled and echoes the cursor it was given, exactly as
  the demo wall does. The client freezes on its first poll, and the 60 second
  `feed_pages` cache makes the freeze that follows a cache hit rather than a
  second round trip. What it echoes is the position it actually read, re-encoded
  rather than copied from the request, so a graph cursor handed to a feed wall
  is dropped here instead of being returned as though it meant something; for a
  feed cursor the two are byte identical.
- **The page size follows the view, because glaze's filter is aggressive.** The
  mixed views ask for `limit = PAGE_SIZE` and may come back a few short, since
  reposts and moderated posts are dropped after the request; serving short is
  already normal and the pump retries. Glaze asks for `limit = 100`
  (`getFeed`'s ceiling) and lays **every** image post that survives, not the
  first `PAGE_SIZE` of them. Most posts in a general feed carry no image, so
  asking for 24 and filtering would lay three or four bricks per network call
  and spend a dozen calls filling one screen. Laying all of them rather than
  truncating is not an optimisation but a correctness requirement: there is no
  pool to hold a remainder in, and the cursor mason hands back belongs to the
  call that fetched them, so a truncated page throws the rest away.
- **The wall ends when the feed does.** `getFeed` returning no cursor is the
  whole end condition. There is no `graph_spent`, no `has_more()` and no pool
  to drain.
- **Only posts and Bluesky videos can appear.** A feed generator returns post
  URIs, so blogs and Streamplace bricks are structurally absent from a feed
  wall. The mix ratio has nothing to balance.

**All three views work on a feed wall, and glaze means something different on
each source.** A view is the reader's choice about the wall in front of them, so
it does not depend on where the bricks came from:

| View | Graph wall | Feed wall |
|---|---|---|
| Bento, Masonry | Presentation only; one mixed wall packed two ways | Presentation only; one feed packed two ways |
| Glaze | `Mode::Glaze` re-reads each author deep (`posts_with_media`, 100) and admits image posts alone | `Mode::Glaze` filters the feed's own posts to those carrying an image |

One `mode` value carries both, because `Mode` selects kinds and never a source.
The layout picker therefore needs no new state, no fourth option and no disabled
cases: three views, always, whichever door the reader came in through.

---

## Resolution and the wall-owner gate

A wall is somebody's social graph on display. If the owner set
`!no-unauthenticated`, a logged-out mason must not lay it. Their own posts never
reach the fill, so `resolve_and_gate` is the one place that preference is
checked.

One `getProfile` call does double duty: its response carries both the DID and the
label, so a cold handle load pays one AppView round trip on this path instead of
a `resolveHandle` and then a `getProfile`.

The failure direction depends on what is already known:

| Situation | On `getProfile` failure |
|---|---|
| Cold handle: the call is load-bearing for resolution | Fail closed: `Upstream` (or `ActorNotFound` on 400/404) |
| DID already known (a `did:` actor, or a cached handle) | Fail open: treat as not opted out |

A flaky profile read must never seal a wall by accident, but an unresolvable
handle has no wall to lay either way.

A feed wall has no owner to gate. `!no-unauthenticated` is a request about a
person's own social graph being put on display; a feed generator is a published
service, and the feed's creator has not asked anybody not to read it. Individual
posts and their authors are still filtered: a feed wall runs the same post
mapper as an author feed, which drops a hidden or opted-out author's posts and
blurs the `!warn` tier (see [04](04-sources-and-moderation.md)). That per-post
filter is complete coverage on a feed wall, where the cohort filter has nothing
to do, because a feed cannot yield a blog or a stream.

---

## Snapshot construction

`ensure_snapshot` is a fetch-or-create under one cache lock. Exactly one
concurrent caller observes `inserted == true` and spawns the background fill;
everyone gets the same `Arc<Snapshot>`. It never waits, which is what the preview
loop needs: it wants whatever pool exists right now, however thin.

`get_or_build` is `ensure_snapshot` plus a block on the first-paint threshold. It
is used by the committing paths so a page is never laid from an empty pool.

### Timings and thresholds

| Constant | Value | Meaning |
|---|---|---|
| `FIRST_PAINT_AUTHORS` | 12 | Distinct authors pooled before `get_or_build` returns |
| `FIRST_PAINT_DEADLINE` | 3 s | Or this much time, whichever comes first |
| `MIX_DEADLINE` | 6 s **from snapshot creation** | How long the first page defers laying to wait for the rare kinds |
| `get_page` hard deadline | 8 s | After which a short page is served rather than hang |
| `POOL_LOW_WATER` | 48 (two pages) | Below this, a settled snapshot starts an extension wave |
| `WAVE_DEAD_AFTER` | 60 s | A wave that has not reported back is presumed dead |
| Snapshot TTL | 30 min | See [05](05-caching-and-persistence.md) |

First paint counts **distinct authors**, not bricks. Counting bricks let one
chatty account clear the gate with thirty of its own posts before anyone else's
feed had arrived.

`MIX_DEADLINE` is anchored to snapshot creation, not to the page request, so the
first-paint wait is spent *inside* this budget rather than stacked on top of it.
That bounds a cold wall's whole opening wait to six seconds.

The mix wait applies only when all of: the caller asked for it (`Normal`), it is
the first page (`offset == 0`), the snapshot is still warming, a slow fan is
outstanding, and the deadline has not passed. Nobody watches a blank screen
decide what it wants to be on page four.

### Admission

Every brick enters through `Inner::admit`, and is rejected unless all four hold:

1. **Fresh.** Within `max_age_hours` if the snapshot set one (glaze: 30 days),
   otherwise within the per-kind window from `score::is_fresh`.
2. **Under its kind's cap.** `KIND_CAPS = [500, 60, 30, 20, 5]` for
   post / blog / Bluesky video / archived stream / live. Posts arrive by the
   thousand and must not crowd out the rarer kinds.
3. **Under its author's share.** `max_per_author` bricks of **one kind**:
   4 on the full wall, 8 on glaze. Per kind, not per author, because posts arrive
   from a fast endpoint and blogs from slow ones, so a flat cap would be spent
   entirely on skeets before a prolific author's blog had even been fetched.
4. **Not already seen.** The `seen` set is the dedup and the suppression list at
   once.

Glaze raises the per-author cap and widens the window because it reads one source
and admits one kind: a chatty account there can only crowd out other image
posters, and an image wall is a gallery rather than a timeline.

---

## The fill

`fill::fill` runs detached, spawned by `ensure_snapshot`. A follow-graph failure
leaves an empty but terminated snapshot rather than an error; actor existence was
already established by resolution.

```
fill(state, snapshot)
  │
  ├─ get_follows_cached(viewer)            // 1 page eagerly, the rest chased in background
  ├─ cohort = sample_cohort(activity_key(viewer, mode), follows, seed)
  │
  ├─ Mode::Glaze  ──▶ fan_out_authors(deep media read, keep only image posts)
  │
  └─ Mode::Wall   ──▶ join!(
        fan_out_authors(shallow read, keep everything),   // posts, FAN_OUT = 16
        fan_out_repos(cohort),                            // blogs + archived streams, REPO_FAN_OUT = 32
        live_fill                                         // one call for the whole network
      )
  │
  ├─ record_fanned(authors that ANSWERED)   // before warming ends, so no wave races a half-recorded set
  ├─ finish_warming()
  └─ record_activity(authors that YIELDED)  // warm-starts the next cohort
```

**Posts and repo reads are fanned out separately, and that split is why a cold
wall paints at all.** They used to share one task per author, so an author's
posts were not admitted until `plc.directory` and two PDS `listRecords` had also
answered for them. Posts are 68% of the wall and come from one fast rate-limited
endpoint; blogs and archived streams are a handful of bricks from a hundred
different PDSes at a hundred different speeds. Coupled, a 100-author fill took
17 seconds and the first page arrived empty.

The live list runs alongside rather than per author: who is live is one call for
the whole network, and it does not depend on the cohort. A friend streaming right
now belongs on the wall whether or not this snapshot's sample happened to pick
them.

### Cohort sampling

```
COHORT_SIZE = 100   KNOWN_ACTIVE = 60
```

The cohort is up to 60 authors that yielded content in recent snapshots for this
viewer and mode, topped up to 100 with a seeded-random sample of the rest, so a
refresh rotates through the whole follow graph. Hidden follows are filtered out
**before** sampling, which is the single choke point that keeps an opted-out or
adult-flagged account off every source at once: no posts, no blogs, no archived
streams, and (via a matching filter in `followed_live`) no live stream either.

`record_activity` merges the authors that yielded into the front of the existing
list, dedupes, and truncates at 300.

Follow-graph pages are 100 records each and strictly sequential. The first wall
waits for `FOLLOW_PAGES_EAGER = 1` page, already more than the cohort samples,
and the remaining pages up to `FOLLOW_PAGES_MAX = 20` are chased in a background
task. The partial head is deliberately **not** cached: only the completed graph
is, so a partial list can never masquerade as the whole one.

---

## Extension waves

The wall extends itself. When `get_page` finds the fill settled, no wave in
flight, the graph not spent, and the pool below `POOL_LOW_WATER`, it claims a
wave under the snapshot lock and spawns `fill::extend` outside it.

```
extend(state, snapshot)
  ├─ follows (cached)                      // failure ⇒ finish_extension(false): a later page retries
  ├─ wave = next_wave(follows, seed, fanned)
  │     empty ⇒ finish_extension(graph_spent = true); the wall can now genuinely end
  ├─ raise_caps()                          // one KIND_CAPS increment per wave
  ├─ fan out (posts ‖ repos; the live list is NOT re-read)
  ├─ record_fanned(answered)
  └─ finish_extension(false) ▸ record_activity(yielded)
```

Four rules make this safe:

- **Waves run single-file.** `extending` is claimed under the lock. A claim older
  than 60 seconds is treated as a dead task's, so a panicked or reaped wave
  cannot block every future wave for the snapshot's half-hour life.
- **At most one wave per page request.** The claim keeps waves single-file across
  requests; this keeps one request from spinning up wave after wave inside its
  own wait loop. Scroll retries are the pacing.
- **Caps rise with each wave.** They were budgeted for one cohort, and a second
  hundred authors must not be turned away at a door the first hundred filled.
- **Only authors that *answered* count as fanned.** An author whose fetch failed
  transiently was never really asked, nothing was cached, and the next wave asks
  them again. The alternative lost them for the wall's whole life.

The live list is not re-read by a wave: a wave feeds a wall long past its first
paint, and ended streams are pruned separately.

---

## Serving a page

```
get_page(state, snapshot, offset, size, wait_for_mix)
  │
  ├─ drop_ended_streams()      // only if a live brick is actually pooled
  ├─ wanted = offset.checked_add(size)?   // an attacker-writable offset must not overflow
  │
  └─ loop, waking on a brick arriving or on whichever deadline is next:
       ├─ claim an extension wave if the pool has run low  (once per request)
       ├─ awaiting_mix? ──▶ defer laying until the mix deadline
       ├─ lay (wanted - wall.len()) bricks from the pool          [03]
       ├─ ready if:  wall.len() >= wanted
       │          |  exhausted   (settled + pool empty + no wave + graph spent)
       │          |  wave_spent  (this request's wave came and went, pool dry)
       └─ hard deadline: lay whatever arrived during the wait, serve short
```

Serving short is a deliberate outcome, not a failure: the cursor stays alive and
the client's scroll pump retries. The three readiness conditions exist so a page
never waits out a deadline for bricks that are not coming.

### Ended streams

A live brick is admitted once, during the fill, and then waits in the pool for
the snapshot's whole half-hour life. `drop_ended_streams` prunes any live brick
no longer on the network's live list, so a stream that finished twenty minutes
ago is not laid with a LIVE badge and a playlist that 404s.

Two constraints shape it. It takes nothing at all when no live brick is pooled,
which is the overwhelming majority of walls. And it prunes only against a live
list already in cache: on a lapse it kicks off a background refresh for the next
page and lets this one through, so a network round trip never lands mid-scroll.

Already-laid live bricks stay where they are, because the wall never moves. The
player is the last line of defence for those, and says "this stream has ended"
when the playlist fails.

`kind_counts` and `author_counts` are left alone by the prune. They are admission
budgets, not a census: an ended stream does not buy its author a fresh slot on a
wall already laid around them.

---

## Concurrency rules

- **Every acquisition of the snapshot's inner mutex lives in `algo/snapshot.rs`.**
  `fill` and `cohort` mutate the pool only through methods on `Snapshot`, or not
  at all.
- A `Notify` waiter is registered and enabled **before** the state check it
  guards, in both `get_or_build` and `get_page`. `Notify` only wakes futures that
  already exist, so a notification slipping in between the lock release and the
  await would otherwise be lost and stall the wait until the deadline.
- Batches are admitted under one lock hold (`admit_all`, `admit_repo_yield`), and
  the caller decides when progress is worth announcing.
- `platform::spawn` is `tokio::spawn` natively and `spawn_local` on wasm, so
  nothing in the engine requires `Send`.

---

## The demo wall

`actor == "demo"` short-circuits everything before resolution and serves from
`fixtures::pool()`, a deterministic set of bricks compiled into the binary. It
obeys the mode (glaze narrows it to image posts) and pages with the same cursor.
A preview against it reports itself already settled, so the client freezes on the
first poll. It needs no network at all, which is what makes an offline launch of
the installed app useful.

---

## Implementation layout

```
server/crates/mortar-core/src/
  feed.rs          handle_feed, FeedIntent, resolve_and_gate, the demo page
  mode.rs          Mode, from_query, tag
  state.rs         AppState { config, http, caches }
  config.rs        upstream base URLs, all overridable for tests
  error.rs         AppError, ErrorEnvelope, the pinned wire strings
  algo/
    snapshot.rs    Snapshot, admission, get_or_build, preview_page, get_page
    fill.rs        the background fill and the extension waves
    cohort.rs      cohort sampling, next_wave, activity
    mix.rs         the mixer                       [03]
    score.rs       the grout score                 [03]
    cursor.rs      encode / decode
  sources/         ingestion                       [04]
  cache.rs         TtlCache and Caches             [05]
  persist.rs       browser-build cache persistence [05]
  platform.rs      the native / wasm seam
  fixtures.rs      the demo wall's bricks
```

---

## Assumptions and open questions

**Assumptions**

- A background task spawned by the engine may be killed at any moment (the
  service worker is reaped), so no invariant may depend on a spawned task
  finishing.
- `getFollows` returns at most 100 records per page and pages sequentially.
- Six seconds is an acceptable ceiling for a cold wall's first screen on a
  reasonable connection.

**Decisions**

- *First paint counts authors.* **12 distinct authors, or 3 seconds.** A
  brick-count gate let one chatty account own the entire first screen.
- *Mix deadline anchored to creation.* **6 seconds from when the snapshot was
  born.** Anchoring it to the page request stacked two waits, so a cold wall
  could stare at skeletons for the sum of both.
- *Posts and repos fanned out separately.* **Two independent fan-outs.** Coupled,
  the fast source was held hostage to the slow one and the first page arrived
  empty.
- *Only answering authors count as fanned.* **A transient failure is not
  recorded.** Otherwise one blip silenced an author for the wall's whole life.
- *Waves have a dead man's switch.* **60 seconds.** A panicked or reaped wave
  cannot clear its own claim, and a stuck flag would block every later wave.
- *Per-kind, per-author admission cap.* **4 of one kind (8 on glaze).** A flat
  per-author cap was spent on posts before the slower sources answered.
- *Serving short is normal.* **The cursor survives and the client retries.**
  Making a reader wait out a deadline for bricks that are not coming is worse
  than a short page.
- *AppView burst of 100.* **Sustained 10/s, burst 100.** At a burst of 40 the
  cohort queued behind the drip and a reader could out-scroll their own wall.

**Open questions**

- *Server mode and the preview loop.* `mortar-server` honours `preview` and
  `freeze` because it serves the same SPA, but nothing exercises that path in
  tests. Open until server mode has a consumer.
- *Extension waves and the live list.* A wave never re-reads who is live, so a
  friend who goes live an hour into a long scroll never appears. Open: is a
  refresh worth the call, given the ended-stream prune already touches that
  cache?
- *`Snapshot.mode` and glaze waves.* Glaze waves re-read `posts_with_media` for
  new authors but never widen the existing pool's age window. Open only if the
  glaze wall is observed running dry early.
