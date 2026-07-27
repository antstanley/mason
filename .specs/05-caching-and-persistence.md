# 05 - Caching and Persistence

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

mason has no database. All state is in-memory behind hand-rolled TTL caches,
because the same engine has to run inside a service worker that a browser may
reap at any idle moment. This page covers the cache primitive, every cache mason
keeps, and the IndexedDB persistence that makes a reaped worker wake up warm.

---

## Responsibilities

1. Hold every fetched artefact for a defined lifetime, bounded in size.
2. Serve fetch-or-create atomically for snapshots, so exactly one caller spawns a
   fill.
3. Export and import warm caches across a service-worker death.
4. Never persist anything whose staleness would be a lie.

The cache layer does **not** own: what to fetch on a miss (that is
`sources/fetch.rs`), or the source-shaped TTL *values*, which live at the seam
and are re-exported here as defaults.

---

## `TtlCache<K, V>`

A `HashMap<K, Entry<V>>` behind an async `Mutex`, with per-entry expiry, a soft
capacity, and a dirty flag. Values are `Arc`s everywhere, so `get` clones are
cheap.

```rust
TtlCache::new(default_ttl: Duration, max_capacity: usize)

get(&K) -> Option<V>                    // expired entries are removed on read
insert(K, V)                            // default TTL
insert_with_ttl(K, V, Duration)         // caller-chosen TTL
get_or_insert_with(K, impl FnOnce) -> (V, bool)   // atomic; exactly one caller sees true
export_map(unwrap) -> Vec<(K, T, u64)>  // live entries, absolute unix-ms expiry
import_map(entries, wrap)               // drops anything already expired
is_dirty() / take_dirty() -> bool
```

**Trimming.** Every insert path calls `trim` first: drop expired entries, and if
still at or over capacity, evict the soonest-to-expire in one batch of about 10%.
Removing a single minimum per insert costs an O(n) scan on every insert once a
20k cache is full; a batch amortises that over the inserts that follow. Snapshots
arrive only through `get_or_insert_with` and each pins an `Arc` of up to a few
hundred bricks, so an untrimmed path would grow without bound in a long-lived
server.

**Expiry across process death.** `Instant` does not survive a process, so
`export_map` converts each entry's remaining lifetime to an absolute unix-ms
timestamp and `import_map` converts it back, discarding anything already expired.

**Dirty tracking.** Every insert sets the flag; `take_dirty` clears it and returns
whether it was set. Imports do *not* set it: what was just read back from disk is
by definition already persisted. Expiry and eviction do not set it either, since
export filters expired entries anyway and a persisted copy of an evicted entry is
just a cache warmer.

---

## The caches

| Cache | Key | Value | TTL | Capacity | Persisted |
|---|---|---|---|---|---|
| `did` | handle | DID | 24 h | 10 000 | yes |
| `follows` | viewer DID | `Vec<Follow>` | 1 h | 1 000 | yes |
| `author_feed` | author DID | `AuthorYield` | 5 min | 20 000 | yes |
| `image_feed` | author DID | `AuthorYield` (deep media read) | 5 min | 20 000 | yes |
| `std_docs` | author DID | `StdDocs` | 15 min positive / 24 h negative | 20 000 | yes |
| `pds` | author DID | PDS endpoint | 24 h | 20 000 | yes |
| `streams` | author DID | `Vec<Brick>` | 30 min positive / 24 h negative | 20 000 | yes |
| `profiles` | wall-owner DID | opted out? | 1 h | 10 000 | yes |
| `activity` | `activity_key(viewer, mode)` | `Vec<String>` (yielding authors, max 300) | 24 h | 1 000 | yes |
| `live` | the constant `0u8` | `Vec<LiveStream>` | 60 s | 1 | **no** |
| `snapshots` | `snapshot_id` | `Arc<Snapshot>` | 30 min | 500 | **no** |
| `feed_pages` | `<feed uri>\u{1f}<limit>\u{1f}<upstream cursor>` | `Arc<AuthorYield>` plus the next cursor | 60 s | 500 | **no** |

Three shapes of TTL, each with a reason:

- **Identity is slow.** Handles and PDS endpoints move rarely, and every repo read
  needs the PDS answer, so both are cached for a day.
- **Content is warm.** Author feeds are five minutes: long enough to serve a whole
  cohort fan-out from one call and to survive a refresh, short enough that a wall
  reopened later is genuinely new.
- **Positive and negative differ.** Publishers and streamers get rechecked within
  the hour; the silent majority (most people have never blogged or streamed) is
  left alone for a day. Only a *successful* empty listing earns the negative TTL;
  a transient failure is not cached at all.

**A refresh bypasses two of these on a graph wall, and one on a feed wall.**
`author_feed` and `image_feed` are re-read on a `refresh=1` request; on a feed
wall it is `feed_pages` instead. Every other cache stays warm. The split is the
same reasoning the TTLs already encode: identity and repo contents move on a
scale of days, so re-reading them would spend a hundred PDS round trips to learn
nothing, while the AppView author feed is precisely the thing that has changed
since the reader last looked. A refreshed read overwrites its entry and marks
the cache dirty, so the next persist cycle captures the fresher data rather than
the data the refresh replaced. `feed_pages` is not persisted at all, so there
the overwrite buys the next reader a warm entry and nothing more.

The `live` cache is the one thing on the wall with a deadline, so it is the one
thing barely cached: sixty seconds, and the value stored is `LiveStream`, not
`Brick`, because what is cached there is true for every viewer and the per-viewer
filter happens downstream.

`author_feed` and `image_feed` are kept apart deliberately: the two walls read the
same author differently (`posts_no_replies&limit=30` versus
`posts_with_media&limit=100`), and one cache would let each clobber the other's.
`feed_pages` solves the same problem inside one cache by carrying the limit in
its key: the mixed views ask a generator for `PAGE_SIZE` and glaze asks for 100,
so a key of (uri, cursor) alone would serve a glaze request the 24-item page a
mixed request cached a moment earlier and the image wall would silently run a
quarter as deep.

A feed page is a ranked view with a deadline, like the live list, so it is
cached for a minute and never persisted. Sixty seconds is enough to make the
preview-then-freeze pair one network read and to survive a back/forward, and
short enough that reopening a feed wall shows the feed's current head. A
persisted feed page would be laid hours later as though it were fresh ranking,
which is exactly the lie the persistence layer exists to avoid.

---

## Persistence

Only the browser build persists. Browsers reap an idle service worker after
roughly 30 seconds, and without persistence every wake-up means a cold refetch of
the whole cohort. The worker exports dirty caches to IndexedDB after serving a
page and imports them on startup, turning the post-idle rebuild from seconds of
network fan-out into milliseconds.

```
service worker startup
  ├─ init(wasm)                       // from THIS worker's own precache, not the network
  ├─ cache_names()                    // 9 names, from mortar::persist::CACHE_NAMES
  ├─ idbGetMany("mortar-cache:" + name)
  ├─ import_cache(name, payload)      // per cache; version mismatch is discarded
  └─ idbSweepStale()                  // the pre-v4 bundle key, and orphaned per-cache keys

after each served page (event.waitUntil)
  ├─ intent == "preview" ? skip entirely
  ├─ chain behind any in-flight cycle
  ├─ throttle: 4 s, unless intent == "freeze"
  ├─ dirty_cache_names()              // only what this page actually touched
  ├─ export_cache(name) per dirty cache   // clears its dirty flag first
  └─ idbPutMany(entries)              // one IDB key per cache
```

### What is persisted, and what is not

Persisted: `did`, `follows`, `author_feed`, `image_feed`, `std_docs`, `pds`,
`streams`, `profiles`, `activity`. Each is a warm cache a cold wall would
otherwise repay in network round trips.

Not persisted:

- **The live list.** It is sixty seconds from being a lie, and one call rebuilds
  it.
- **Snapshots.** They hold locks and in-flight state, and the seed-carrying cursor
  already rebuilds them deterministically from the warm caches above.
- **Feed pages.** Somebody else's ranking, sixty seconds from being stale, and
  one call rebuilds it. `feed_pages` is deliberately absent from
  `persist::CACHE_NAMES`, so no exporter can reach it by name.

### Storage layout

- Database `mason`, version 1, one object store `kv`.
- One key per cache: `mortar-cache:<name>`.
- Each payload is `{version, entries: [[key, value, expiresUnixMs], …]}`, where
  `version` is `persist::VERSION` (currently 4). A payload written by a different
  version is discarded on import; it is only a cache.
- `idbSweepStale` deletes the pre-v4 whole-bundle key (`mason-caches-v1`) and any
  `mortar-cache:*` key whose cache mortar no longer persists, so a renamed or
  dropped cache does not orphan its key forever.

### Why per-cache keys

v3 and earlier wrote one payload under a single key, which meant a session
holding hundreds of warm `AuthorYield`s deep-cloned and re-serialised all of them
every time a handle resolved. v4 splits by cache and gates on the dirty flag, so
persistence cost scales with what the page actually changed.

### Ordering

Persist cycles run one at a time. Two tabs share one worker, and two overlapping
cycles could otherwise interleave export and write so the *older* payload commits
last under a key whose dirty flag the newer export already cleared. Each new cycle
chains behind the in-flight one, so a freeze arriving mid-persist still runs
afterwards and captures the frozen state.

Preview polls never persist: they run at a 350 ms cadence over the same data
warming up. The freeze that ends them always does, bypassing the throttle.

---

## Cache eviction as a correctness case

A killed service worker is, from the engine's point of view, cache eviction. The
system is built so that this is uneventful:

```
worker reaped mid-scroll
  ▼
next /api/feed request wakes a fresh worker
  ▼
persisted caches import: follows, author feeds, blogs, streams, identity   (warm)
  ▼
cursor carries {seed, offset}
  ▼
snapshot rebuilt with the same seed from the warm caches
  ▼
the mixer is pure, so the same pool and seed produce the same arrangement
```

Determinism of the jitter is exact. Continuity of the *pool* is best-effort, since
the underlying caches expire on their own schedules, so a rebuilt wall matches
closely rather than identically. A rebuilt snapshot is `warming` again, which is
why `get_page` does not re-pay the mix wait on any page but the first: the reader
is mid-scroll, and six seconds for a better blog-to-post ratio is a bad trade
there.

---

## Implementation layout

```
server/crates/mortar-core/src/
  cache.rs      TtlCache, Caches, the TTL and capacity table
  persist.rs    VERSION, CACHE_NAMES, dirty_cache_names, export_cache, import_cache
server/crates/mortar-wasm/src/lib.rs
                #[wasm_bindgen] cache_names / dirty_cache_names / export_cache / import_cache
web/src/service-worker.ts
                idbOpen · idbGetMany · idbPutMany · idbSweepStale · persistCaches
```

---

## Assumptions and open questions

**Assumptions**

- IndexedDB is available in the service worker and not in private-mode lockdown.
  Every path around it is wrapped in a `try`/`catch` that degrades to a cold start.
- A service worker instance may die between any two requests, and between an
  export and its write.
- Storage pressure may evict the whole database at any time; nothing depends on a
  key being there.

**Decisions**

- *Hand-rolled TTL cache.* **A `HashMap` behind an async mutex.** `moka` and every
  other mature cache crate fails to build for `wasm32-unknown-unknown`, and the
  engine must run there.
- *Batch eviction.* **Evict ~10% of capacity at once.** A single-minimum eviction
  is an O(n) scan on every insert once a 20k cache is full.
- *Absolute expiry on export.* **Unix milliseconds, not `Instant`.** `Instant` is
  meaningless across a process boundary.
- *Per-cache keys and a dirty flag.* **Export only what changed.** The v3
  whole-bundle export re-serialised every warm `AuthorYield` on every page.
- *Snapshots are not persisted.* **The cursor rebuilds them.** They hold locks and
  in-flight tasks, neither of which serialises, and the seed makes a rebuild
  deterministic.
- *The live list is not persisted.* **60 seconds and one call.** A restored live
  list is more likely to be wrong than useful.
- *Preview polls skip persistence.* **Only committed pages write.** A 350 ms poll
  loop writing to IndexedDB is pure waste on data that is still arriving.
- *Version mismatch discards silently.* **It is only a cache.** A migration path
  for warm data that rebuilds itself in seconds is not worth its own bug surface.

**Open questions**

- *Native-mode persistence.* `mortar-server` never exports; a restart is a fully
  cold engine. Nothing needs it today (the server is not the default mode), so
  the export path is exercised only by the browser build and by unit tests.
- *Capacity in the browser.* Capacities were chosen for a small server
  deployment. A long browsing session in one tab holds far less than 20 000
  author feeds, so the ceilings have never been reached in local mode; they are
  untested at scale.
- *Snapshot cache capacity.* 500 snapshots at up to a few hundred bricks each is a
  real memory ceiling for a server, and it has not been measured against a
  realistic concurrent load.
