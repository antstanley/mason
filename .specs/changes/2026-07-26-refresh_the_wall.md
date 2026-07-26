# Change: Refresh the wall without reloading the page

**Status:** Proposed · **Date:** 2026-07-26 · **Owner:** Ant Stanley · **Target:** Repo-wide

A laid wall is final. Once the first screen freezes, the only thing that grows it
is the scroll pump, and the only way to see anything newer is a browser reload,
which throws away the wasm engine, the warm caches, the laid arrangement and any
playing video to rebuild all of it. This change adds a refresh control: one
button in the header that lays a new wall in place, and a `refresh=1` request
flag that makes the fill re-read the two fast content caches so the new wall is
genuinely newer rather than the same bricks reshuffled.

---

## Motivation

Reload is not a refresh, it is a restart. It drops the service worker's
in-memory caches and every snapshot, reboots the wasm module, refetches the
follow graph if its hour has passed, and lands the reader on skeletons. mason
went to real trouble to make a reaped service worker uneventful, and then left
the reader with no way to ask for new bricks except the one action that undoes
all of it.

The second half matters as much as the first. A new wall on its own would only
reshuffle: author feeds are cached for five minutes and persisted across a worker
death, so a wall re-laid within that window is the same bricks in a different
arrangement from a different cohort sample. That is not nothing on a large follow
graph, and it is nothing at all on a small one. Getting *newer posts* means
re-reading the source they come from, which is one burst of a hundred requests
against the reader's own rate-limit budget: exactly what a cold wall already pays,
and no more.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/01-domain-model.md`](../01-domain-model.md) | The `Snapshot` state table gains `refresh` |
| [`.specs/02-feed-engine.md`](../02-feed-engine.md) | `handle_feed` gains `refresh`; when it is honoured and what it reaches |
| [`.specs/04-sources-and-moderation.md`](../04-sources-and-moderation.md) | The two fast content reads become bypassable, with a failure fallback |
| [`.specs/05-caching-and-persistence.md`](../05-caching-and-persistence.md) | What a refresh re-reads and what it deliberately leaves warm |
| [`.specs/06-wire-contract.md`](../06-wire-contract.md) | The `refresh` parameter and its fixture entry |
| [`.specs/07-web-client.md`](../07-web-client.md) | `FeedState.refresh()`, the session-cache invalidation, the one-shot flag |
| [`.specs/08-wall-and-bricks.md`](../08-wall-and-bricks.md) | The header control, the wall's appearance during a refresh, the live region |
| [`.specs/canonical-types.schema.json`](../canonical-types.schema.json) | A `FeedRefresh` query-vocabulary def |

---

## Proposed changes

Each block below is the prose as it should read once merged, so links *inside* a
quoted block are relative to the canonical page it lands on, not to this
directory. Links outside the blocks resolve from `.specs/changes/` as usual.

### `.specs/01-domain-model.md` → Entities → Snapshot (Modify)

Add a row to the state table:

> | `refresh` | This wall was asked for explicitly, so the fill re-reads the fast content caches instead of trusting them |

### `.specs/02-feed-engine.md` → Entry point (Modify)

> ```rust
> pub async fn handle_feed(
>     state: &Arc<AppState>,
>     actor: &str,
>     cursor: Option<&str>,
>     mode: Mode,
>     intent: FeedIntent,
>     refresh: bool,
> ) -> Result<FeedResponse, AppError>
> ```

### `.specs/02-feed-engine.md` → Refresh (Add, new section after Intents)

> ### Refresh
>
> `?refresh=1` says the reader asked for this wall on purpose. It does two things
> and refuses to do a third.
>
> It is honoured **only on a cursorless request**. A refresh is always a first
> page: mid-scroll it would mean re-reading a hundred author feeds to serve page
> nine, whose bricks were admitted from the old ones anyway. With a cursor
> present the flag is ignored, not an error.
>
> On a cursorless request there is already a fresh seed (that is what no cursor
> means), so a refresh always lands on a brand new snapshot id and can never
> disturb the one it replaces. What the flag adds is carried into that snapshot
> and read by its fill: `author_feed` and `image_feed` are re-read from the
> AppView rather than served from cache. Nothing else is. The follow graph, PDS
> endpoints, blog documents, archived streams, the owner's opt-out and the
> per-viewer activity list all stay warm, because none of them is where "new
> posts" lives and every one of them is expensive to re-read (see
> [05](05-caching-and-persistence.md)).
>
> Because the flag rides on the snapshot rather than on the request, it is
> idempotent per wall: `ensure_snapshot` creates a snapshot and spawns its fill
> exactly once, so the preview polls and the freeze that follow a refresh land on
> the already-refreshing snapshot whether or not they repeat the flag.
>
> The demo wall ignores it. Its bricks are fixtures compiled into the binary and
> there is nothing to re-read.

### `.specs/04-sources-and-moderation.md` → Per source → Bluesky (Modify)

Append to the two feed-read bullets:

> Both feed reads take a `refresh` argument. When it is set, the cache is not
> consulted and the AppView answer overwrites whatever was there. A refreshed
> read that fails *transiently* falls back to the cached yield rather than
> returning `None`, so a refresh can never lay a thinner wall than the one it
> replaced: the author did answer, just earlier. A refreshed read with nothing
> cached behind it behaves exactly like a cold one.

### `.specs/04-sources-and-moderation.md` → Failure semantics (Add)

Add a row to the table:

> | Transient failure on a refreshed read | Transient | No (the older entry survives) | The previously cached yield, so the refreshed wall is never thinner than the one it replaced |

### `.specs/05-caching-and-persistence.md` → The caches (Add)

Append after the three TTL shapes:

> **A refresh bypasses exactly two of these.** `author_feed` and `image_feed` are
> re-read on a `refresh=1` request; the other nine are not. The split is the same
> reasoning the TTLs already encode: identity and repo contents move on a scale of
> days, so re-reading them would spend a hundred PDS round trips to learn nothing,
> while the AppView author feed is precisely the thing that has changed since the
> reader last looked. A refreshed read overwrites its entry and marks the cache
> dirty, so the next persist cycle captures the fresher data rather than the data
> the refresh replaced.

### `.specs/06-wire-contract.md` → The endpoint (Modify)

Add a row to the parameter table:

> | `refresh` | no | `1` | Lay a new wall and re-read the fast content caches. Honoured only when `cursor` is absent |

And append to the fallback paragraph:

> `refresh` is a single literal token for the same reason `mode` is: anything
> other than exactly `1`, including absent, means no refresh. The safe direction
> is the default one, so a hand-edited URL cannot make somebody re-fan-out a
> hundred authors by accident.

### `.specs/06-wire-contract.md` → What the fixture covers (Modify)

> | `query.mode` / `query.intent` / `query.refresh` | The query vocabulary, as object keys |

### `.specs/07-web-client.md` → Responsibilities (Modify)

> 2. Drive the warm-then-commit first screen, the endless-scroll pagination, and
>    the in-place refresh against `/api/feed`.

### `.specs/07-web-client.md` → The feed state machine (Add)

Add after the `loadMore()` block in the diagram:

> ```
>    refresh()   ← the header control
>       ├─ refuses while loading, warming, or actorless
>       ├─ drop this (actor, mode) from the session cache: back/forward must not
>       │  resurrect the wall the reader just replaced
>       ├─ cursor = null, done = false, warming = true, generation++
>       ├─ items are LEFT ON THE WALL and initialLoad stays false
>       ├─ arm a one-shot refresh flag
>       └─ spawn #warm(generation), whose first request carries the flag
> ```
>
> Three details hold it together:
>
> - **The outgoing wall stays visible.** `refresh()` does not clear `items`, so
>   the reader keeps looking at bricks while the new wall warms, and the first
>   `#replace` reflows the old arrangement into the new one. That is the same
>   machinery the warming reflow already uses; clearing to skeletons would make a
>   refresh look like a failure for two seconds.
> - **The flag is one-shot and consumed by whichever request goes out first.**
>   `#warm`'s first poll normally carries it, but a reader who engages
>   immediately can make `freeze()` the first request instead, and that request is
>   cursorless too. Both call sites consume the same flag, so the refresh is
>   never silently dropped by a race with the reader's own scroll.
> - **`FeedState` still touches no DOM.** Scrolling back to the top of the wall is
>   the control's job, not the state machine's, which is what keeps the vitest
>   lane running the real module in node.

### `.specs/07-web-client.md` → Decisions (Add)

> - *A refresh lays a new wall from the top.* **It does not weave new bricks into
>   the old one.** Laid bricks never move, which is what makes the cursor
>   meaningful; prepending would shift every brick behind the reader and
>   invalidate the offset they are holding.
> - *The outgoing wall stays on screen during a refresh.* **Reflow, do not clear.**
>   The preview loop already replaces an arrangement in place, and skeletons in
>   the middle of a session read as something breaking.
> - *No auto-refresh and no unread count.* **The reader asks.** A wall that
>   restacks itself while somebody reads it, or wears a badge counting what they
>   have not seen, is the doomscroll mechanic mason is positioned against.

### `.specs/08-wall-and-bricks.md` → Wall states (Add)

Add a row to the table, and a paragraph:

> | `warming` with items already laid | The wall the reader was reading, reflowing into the refreshed one. No skeletons |
>
> ### Refreshing
>
> `RefreshWall` sits in the header beside the layout and client pickers: one
> control, no confirmation, no count. It scrolls the wall back to the top (with
> `behavior: 'auto'` under `prefers-reduced-motion`) and then asks `feed.refresh()`.
>
> It is disabled while the feed is loading or warming, and that is the whole rate
> limit: one refresh can be in flight, so a double tap cannot start two
> hundred-author fan-outs. The live region announces "refreshing the wall" and
> then the wall's usual settled message; the announcement of *new* bricks is
> suppressed for free, because the region's count already resets while warming so
> a reflow's churn never reads as new bricks.

### `.specs/08-wall-and-bricks.md` → Accessibility behaviours (Modify)

Append to the "Custom controls stay keyboard-honest" bullet:

> `RefreshWall` is a plain `<button>` with an accessible name and a disabled state
> that is real rather than styled, so a screen-reader user is told the wall is
> already refreshing instead of pressing a control that silently does nothing.

### `.specs/08-wall-and-bricks.md` → Implementation layout (Modify)

Add to the `components/` list, after `LayoutPicker.svelte`:

> ```
>     RefreshWall.svelte       lay this wall again, now
> ```

---

## Type changes

```json
{
  "$comment": "Fragment for 2026-07-26-refresh_the_wall. FeedRefresh is new; it joins FeedMode and FeedIntent as a query-vocabulary def in .specs/canonical-types.schema.json on merge. No entity changes: FeedResponse, Brick and the cursor are untouched.",
  "$defs": {
    "FeedRefresh": {
      "description": "The ?refresh= request parameter. Exactly the token \"1\" asks for a new wall laid from re-read content caches; any other value, including absent, is no refresh. Honoured only when cursor is absent, because a refresh is always a first page. Mirrors the refresh parse in server/crates/mortar-core/src/feed.rs.",
      "type": "string",
      "const": "1"
    }
  }
}
```

---

## Implementation notes

The engine change is a boolean threaded from the query string to two cache reads.
Most of the work is on the web side.

```
1. server/crates/mortar-core/src/sources/fetch.rs:143 and :177
     author_feed_cached / image_feed_cached take `refresh: bool`. When set, skip
     the `state.caches.*.get` at the top. In the transient-failure arm (:155,
     :188) return the cached entry instead of None when refreshing, so the wall
     is never thinner than the one it replaced. Every other caller passes false.

2. server/crates/mortar-core/src/algo/snapshot.rs:88
     Snapshot gains `pub refresh: bool`, set in `new` (:105) from a new argument.
     ensure_snapshot (:346) and get_or_build (:375) take and forward it.

3. server/crates/mortar-core/src/algo/fill.rs:56, :89, :151, :153, :235
     fan_out_authors takes the flag (or reads snapshot.refresh) and passes it to
     the two seam functions. The extension waves at :151 and :153 pass FALSE
     even on a refreshed snapshot: a wave asks authors this wall has never asked,
     so there is nothing cached for them to bypass, and re-reading the cohort
     again on every wave would multiply the cost of a refresh by the length of
     the scroll.

4. server/crates/mortar-core/src/feed.rs:44
     handle_feed takes `refresh: bool`. Ignore it when `decoded.is_some()`, and
     when actor == "demo" (:56). Pass it to get_or_build and to the preview
     path's ensure_snapshot (:86), so a refresh that begins with a preview poll
     still refreshes.

5. Both fronts and the worker:
     server/crates/mortar-server/src/routes/feed.rs:16   FeedQuery gains
       `refresh: Option<String>`; parse it beside Mode::from_query at :45.
     server/crates/mortar-wasm/src/lib.rs:88             feed_page gains the
       argument.
     web/src/service-worker.ts                           forward the parameter.

6. Regenerate the wire fixture for the new query vocabulary:
      UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract
    Then add `FeedRefresh` to web/src/lib/types.ts and its Equal<> assertion to
    web/src/lib/contract-check.ts, beside the FeedMode and FeedIntent ones.

7. web/src/lib/api.ts:36
     fetchFeed gains a `refresh?: boolean` argument; it sets refresh=1 only when
     true AND no cursor is passed, so the client never sends a flag mortar would
     ignore. api.test.ts gains a case for both.

8. web/src/lib/state/feed.svelte.ts
     refresh() per the state-machine block above. The one-shot flag is a private
     field with a #takeRefresh() that returns and clears it; call it from #warm's
     fetch (:97) and from freeze's fetch (:131). Delete the session-cache entry
     (#cache, :41) before bumping the generation.
     feed.test.ts: a refresh keeps the outgoing items until the first preview
     lands, sends the flag exactly once across a preview-plus-freeze cycle, and
     drops the session cache entry so a later reset does not rehydrate.

9. web/src/lib/components/RefreshWall.svelte    (new)
     A button; scrollTo then feed.refresh(). Mount it in
     web/src/routes/+layout.svelte:126, beside LayoutPicker.

10. just check
    pnpm changeset   (minor: a visible capability)
```

### Interaction with the other pending change specs

If [`2026-07-26-lay_a_bluesky_feed.md`](2026-07-26-lay_a_bluesky_feed.md) lands,
a feed wall has no snapshot and no author-feed caches, so `refresh` there means
one thing instead: bypass the `feed_pages` entry for the page being asked for. It
is a two-line addition on the feed path (skip the cache read, insert as usual),
and whichever of these two change specs merges second owns writing it. The client
side needs nothing extra: the control and the flag are the same.

The reader from [`2026-07-26-read_a_brick_in_place.md`](2026-07-26-read_a_brick_in_place.md)
holds an index into `feed.items`, which a refresh replaces wholesale. Whichever
merges second must make `refresh()` close an open reader, for the same reason
`refresh()` refuses while warming: an index into a list that is being replaced is
not an index.

---

## Merge plan

1. Apply each `Proposed changes` block to its canonical page; bump that page's
   `**Date:**` to the merge date.
2. Add `FeedRefresh` to `.specs/canonical-types.schema.json` beside `FeedMode`
   and `FeedIntent`.
3. Flip this file's `**Status:**` to `Merged`, add `**Merged:** YYYY-MM-DD`, and
   move it to `.specs/changes/merged/`.
4. Update `.specs/README.md`: remove it from the pending list and add it to the
   merged table.

---

## Assumptions and open questions

**Assumptions**

- A hundred-author fan-out is an acceptable cost for an explicit, reader-initiated
  action. It is the same cost a cold wall already pays, spent from the reader's own
  AppView budget in local mode, and the burst of 100 is sized for exactly one
  cohort.
- A refresh is rare compared with a scroll. The control is the only trigger, and
  it is disabled while one is in flight.
- The snapshot cache absorbs the replaced snapshots. Each refresh strands one for
  up to 30 minutes; the cache holds 500 and trims the soonest-to-expire first, so
  a long session of refreshes evicts its own leftovers.

**Decisions**

- *Refresh re-reads the two fast caches, and nothing else.* **`author_feed` and
  `image_feed`.** They are where new posts live and they cost one rate-limited
  burst. Blogs, streams and PDS endpoints move on a scale of days and cost a
  hundred round trips to a hundred different hosts.
- *Refresh is a cursorless request only.* **Mid-scroll it is ignored.**
  Re-reading the cohort to serve page nine would re-fetch the source of bricks
  that were already admitted, to change nothing about the pages already laid.
- *The flag rides on the snapshot, not on each request.* **`ensure_snapshot`
  spawns a fill once.** That makes the refresh idempotent for the preview loop
  and the freeze without either of them having to know they are in a refresh.
- *A refreshed read falls back to its cached yield on a transient failure.* **A
  refresh must never make the wall worse.** Returning `None` would drop that
  author from the wall entirely and leave them unfanned, so a flaky moment during
  a refresh would visibly thin the wall the reader just asked to improve.
- *Extension waves are never refreshed.* **A wave asks authors nobody has
  asked.** There is nothing cached to bypass, and honouring the flag per wave
  would make a refresh cost more the longer the reader scrolls.
- *One literal token, `refresh=1`.* **Anything else is no refresh.** Falling back
  to the safe direction matches `mode` and `intent`, and here the unsafe direction
  costs a hundred upstream requests.
- *The control is the rate limit.* **Disabled while loading or warming.** A
  server-side throttle would need per-viewer state, which a client-side engine has
  nowhere to keep and a shared server should not keep on an unauthenticated read.

**Open questions**

- *The follow graph stays warm for an hour.* Follow somebody, refresh, and they
  cannot appear until the `follows` cache expires. Bypassing it too is one extra
  AppView call, but `get_follows_cached` deliberately does not cache a partial
  graph and chases the tail in a background task, so a bypass has to be written
  carefully rather than added as an argument. Open: is "my new follow is not
  here yet" a complaint worth that care?
- *Pull to refresh.* A touch reader's instinct is to drag down, and the wall
  currently has nothing there. It competes with the browser's own overscroll
  refresh and needs a gesture threshold, a rubber band and a reduced-motion path.
  Open.
- *Whether a refresh should hold the reader's position.* It does not: the wall is
  a new arrangement from a new seed, so there is no old position to hold. A
  refresh that appended instead would preserve it, and would need the wall to
  stop being append-only. Not open so much as decided against, and recorded here
  because it is the first thing anybody asks.
- *Server mode.* The flag works identically over axum, and as with `intent`
  nothing exercises it there. Open until server mode has a consumer.
