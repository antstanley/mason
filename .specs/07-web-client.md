# 07 - Web Client

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

`web/` is a SvelteKit SPA built with Svelte 5 runes and Tailwind v4, shipped as a
fully static site. This page covers its shape: routes, reactive state, the feed
state machine, and the service-worker lifecycle. What the wall looks like is in
[08-wall-and-bricks.md](08-wall-and-bricks.md); the tokens it draws with are in
[09-design-system.md](09-design-system.md).

---

## Responsibilities

1. Turn a URL into a wall: `/?actor=<handle>` lays a graph wall and
   `/?feed=<at-uri>` lays a feed generator's. Those two parameters are the whole
   routing surface.
2. Drive the warm-then-commit first screen and the endless-scroll pagination
   against `/api/feed`.
3. Register and manage the service worker that *is* the feed engine in local
   mode, including what happens to an open tab when a deploy lands.
4. Hold reader preferences (layout, client, last handle, recent feeds) locally.
5. Hold the open brick as history state, so a reader opens one in place and the
   back gesture closes it.

The client does **not** own: feed composition, ordering, dedup within a page, or
moderation. Those are all mortar's, and the client re-implements none of them.

---

## Shape

```
web/src/
  app.html                shell: crawler-visible metadata, icons, theme colours
  app.css                 Tailwind v4 @theme tokens and base layer      [09]
  service-worker.ts       local mode's feed responder and shell cache
  routes/
    +layout.ts            ssr = false, prerender = false
    +layout.svelte        header, SW registration, deploy-reload policy
    +page.svelte          actor or feed ? wall : landing form
  lib/
    api.ts                fetchFeed, warmFeed, FeedError, localMode
    appview.ts            the public AppView base, for the header and picker
    types.ts              the wire mirror                               [06]
    contract-check.ts     the tsc-side drift guard                      [06]
    columns.ts            colsForWidth: the one column-count source
    feedref.ts            what the picker's one input is asking for
    format.ts             runtime and date labels for the cards
    state/*.svelte.ts     rune singletons
    components/           the wall, the cards, the chrome               [08]
```

There is still one route. `?actor=` and `?feed=` are the source of truth for
which wall is showing and are mutually exclusive, with `feed` winning if both
appear; everything else (layout, client, last handle, recent feeds) is a local
preference in `localStorage`, never in the URL.

`ssr = false` and `prerender = false`: the wall is client-rendered, and the
adapter is `adapter-static` with `fallback: 'index.html'`. Crawlers do not run
JavaScript and never boot the service worker, so whatever a link preview shows
has to be in `app.html` or it does not exist.

---

## Build-mode switch

```ts
const BASE = import.meta.env.PUBLIC_MASON_SERVER_URL ?? "";
export const localMode = BASE === "";
```

Injected at build time by a vite `define` reading `process.env`, with a hard
default of `''`. It is a real environment variable rather than a `.env` file, so
there is no file to forget in a deploy zip; `just dev-server` sets it.

Local mode fetches same-origin and the service worker intercepts. Server mode
fetches the absolute URL and the worker is never registered at all.

### Waiting for control

Interception only applies once the worker **controls** the page.
`navigator.serviceWorker.ready` resolves at activation, which can precede
`clients.claim()` taking effect, and fetching in that gap goes to the network and
404s on a static host. `swControlsPage()` therefore waits for either
`controller` to be set or a `controllerchange` event, and races both against a
2 second timeout: a hard-reloaded page stays uncontrolled by design, and a
rejected `register()` leaves `ready` pending forever.

### Warming

`warmFeed(actor, mode)` fires a feed request whose result is discarded, from the
landing form, before the reader has asked for anything. It ensures the worker
controls the page, compiles the wasm, imports the persisted caches, and for a
real handle lands the follow graph and author feeds in their DID-keyed,
seed-independent caches. The wall the reader then opens reuses them and skips the
network fan-out. It is a no-op in server mode and best-effort always.

It stays **actor-only**, and a feed target skips it. Its purpose is to land a
follow graph and author feeds ahead of the wall; a feed target has neither, so
the only thing left to warm is the wasm compile, which the picker screen has
already paid for by the time a feed is chosen.

---

## Reactive state

Rune singletons, each a class holding `$state` fields and exported as a module
instance, apart from `revealed`, which is a reactive set. Preferences persist to
`localStorage`; nothing else does.

| Module | Singleton | Holds | Storage key |
|---|---|---|---|
| `state/feed.svelte.ts` | `feed` | The wall: items, cursor, loading, warming, done, error | none (session `Map` for back/forward) |
| `state/layout.svelte.ts` | `layout` | `bento` \| `masonry` \| `glaze` | `mason:layout` |
| `state/client.svelte.ts` | `client` | Which atmosphere client bricks open in | `mason:client` |
| `state/handle.svelte.ts` | `lastHandle` | The last handle typed, to prefill forms | `mason:handle` |
| `state/feeds.svelte.ts` | `feeds` | The feeds opened recently, most recent first | `mason:feeds` |
| `state/profile.svelte.ts` | `profile` | The wall owner's avatar and opt-out, for the header | none |
| `state/feedinfo.svelte.ts` | `feedInfo` | The feed generator's name, avatar and creator, for the header | none |
| `state/player.svelte.ts` | `player` | The id of the one video allowed to play | none |
| `state/reader.svelte.ts` | `reader` | The brick being read in place; its position on the wall is derived by id | none (history state) |
| `state/sensitive.svelte.ts` | `revealed` | Brick ids whose `!warn` media the reader uncovered | none (session set) |

Three of these are worth naming:

- **`layout` is also an algorithm.** Choosing `glaze` sets `mode=glaze` on the
  feed request, so it re-fetches an images-only wall, exactly as switching actor
  does. `bento` and `masonry` are pure presentation and do not re-mix, because
  `+page.svelte` derives `mode` as `glaze ? 'glaze' : undefined` and `$derived`
  only propagates on a real change.
- **`profile` exists because the feed never carries the actor's own profile.**
  Their bricks may not appear on their own wall at all, so the header asks the
  public AppView directly for an avatar. An opted-out owner shows no face, to
  match the sealed wall behind it.
- **`feedInfo` exists for the same reason `profile` does.** The feed never
  carries the generator's own identity, so the header asks
  `app.bsky.feed.getFeedGenerator` on the public AppView directly, exactly as
  `profile` asks `getProfile` for a wall owner's avatar. A miss leaves the
  header showing the feed's rkey and nothing else, which is ugly but never
  blocking.

`clientUrl(url, host)` rewrites only `bsky.app` links to the chosen client, and
only when the chosen client is not `bsky.app`. Blog links and stream.place pages
are not Bluesky posts and no other client knows how to show them, so they pass
through untouched. Anything that is not `http(s)` returns empty.

### The reader is history, not a URL

**The reader is history state, not a URL.** Opening a brick calls SvelteKit's
`pushState('', { brick: id })`, so the address bar keeps showing `?actor=` and
the back button (and the phone's back gesture) closes the reader instead of
leaving the wall. `page.state.brick` is the half of that answer history owns,
declared as `App.PageState` in `app.d.ts`; `reader` holds the brick itself and
derives its position on the wall from it when it needs to step.

A URL parameter was the alternative and it cannot be honest here. A
`?brick=<at-uri>` link would be shareable and would not work: the recipient's
wall is built from a different seed, that brick is almost certainly not on it,
and mason has no way to fetch one brick by id. History state promises exactly
what it delivers, and leaves `?actor=` as the only thing a shared link carries.

**Open is page state and the held brick agreeing.** `ReaderState.showing` is the
one predicate, and it answers with a brick only when `page.state.brick` names one
*and* the rune is holding that same brick; `isOpen` is that answer as a boolean.
The two halves can disagree, because a history entry outlives the rune that
pushed it: open a reader, go back, reload, then go forward, and the entry returns
with its id while the rune is empty. Page state alone would then make the
layout's content wrapper `inert` under a reader rendering nothing, which is a
wall frozen under nothing at all, so `+layout.svelte` and the dialog both read
the one predicate.

**Opening the reader freezes the wall.** A click is engagement, so
`reader.open` calls `feed.freeze()` before anything else. That is not only
consistency with the wheel, touch and focus signals: while the wall is warming
the arrangement reorders between preview polls, so a position on the wall would
go stale under an open reader.

**The reader holds the brick, and locates it by id.** `reader` stores the
`Brick` itself and derives its position with
`feed.items.findIndex(b => b.id === id)` when it needs to step. Threading an
index down from the wall was the alternative and it is worse in every direction:
`FeedGrid`'s `brick` snippet takes `(item, priority)` and nothing else, so an
index would mean editing that signature, both layout components' prop types,
both render call sites, all four cards and `BrickShell`, none of which any lane
typechecks. Locating by id is O(n) once per click on a list of hundreds, it
keeps `FeedGrid` out of this change entirely, and a replaced or reordered
`feed.items` returns -1 rather than silently pointing at the wrong brick.

Page state does not survive a reload, so reloading with a reader open reopens
the wall without it. That is the intended behaviour rather than a gap to work
around: the reader is a view of a brick on a wall that is itself being rebuilt.

### The picker is history, not a URL

The feed picker is a screen, not a route. It opens with
`pushState('', { picker: 'feeds' })`, so the address bar keeps showing whatever
is behind it and the back gesture closes the picker rather than leaving mason.
`page.state.picker` is the single source of truth for whether it is open,
declared alongside the rest of `App.PageState` in `app.d.ts`. The reasoning is
the same as everywhere else on this page: the URL identifies the wall, and a
picker is not a wall.

---

## The feed state machine

`FeedState` drives the warm-then-commit first screen and then paginates, for
either kind of wall. `reset(target, mode)` takes a `FeedTarget`
(`{actor}` or `{feed}`), and the session wall cache is keyed by the target and
the mode together, so a graph wall and a feed wall never rehydrate into each
other. Its public fields are what the wall renders from: `items`, `cursor`,
`loading`, `initialLoad`, `warming`, `done`, `error`.

A feed wall runs the same three states with the warming phase collapsed:
mortar reports `warming: false` on the first preview, so the loop freezes
immediately and pagination begins. Nothing in `FeedState` branches on the
target beyond building the request.

```
reset(target, mode)
   │
   ├─ cached wall for this (target, mode) this session?
   │     ▸ rehydrate items, cursor, done, seen; warming = false. Back/forward
   │       returns the same arrangement and scroll instead of a fresh seed.
   │
   └─ otherwise: clear, warming = true, generation++, spawn #warm(generation)
                                │
      ┌─────────────────────────┘
      ▼
   #warm loop (up to WARM_CEILING_MS = 8000)
      ├─ fetchFeed(target, cursor, mode, "preview")
      ├─ adopt page.cursor  (it carries the seed, so the next poll and the
      │                      freeze land on this same warming snapshot)
      ├─ #replace(page.items)     ▸ the wall reflows in place, deduped
      ├─ page.warming && within ceiling ?  sleep(POLL_MS = 350)  : freeze()
      └─ on error: freeze(generation, e), so a real error surfaces properly
      ▼
   freeze()   ← also called by the first scroll, wheel, touch, nav key, or focus
      ├─ generation++ (supersedes the preview loop)
      ├─ fetchFeed(target, cursor, mode, "freeze")
      ├─ #replace(items); adopt cursor; done = !cursor; #save()
      └─ finally: warming = false, loading = false, initialLoad = false
                   ▸ set in ONE synchronous continuation with the committed order
      ▼
   loadMore()  ← the scroll pump, repeatedly
      ├─ refuses while loading, done, warming, or targetless
      ├─ fetchFeed(target, cursor, mode)     (no intent: a normal committed page)
      ├─ dedupe against #seen, append, adopt cursor, done = !cursor, #save()
      └─ on error: classify
```

Three mechanisms hold it together:

- **A generation counter.** Bumped on every reset and freeze. Every async
  continuation rechecks it and bows out if a newer wall took over, so a late
  preview or a superseded page can never land on the current wall.
- **`#replace`, not append, during warming.** The preview lays a fresh
  arrangement each poll. The grid keys bricks by id, so shared bricks reorder in
  place and only genuinely new ones animate in. `#seen` is rebuilt from the
  replacement, so pagination after the freeze dedupes against exactly what is on
  the wall.
- **A session cache keyed by `target + mode`.** The key carries the target's
  *kind* as well as its value, so a feed reference spelled like a handle cannot
  rehydrate that handle's graph wall. Only settled walls are saved.
  Returning to a wall already laid this session rehydrates it exactly rather than
  rolling a new snapshot and landing the reader on a skeleton.

`warming` flips off in the same synchronous continuation that delivers the
committed order, so the wall sees a single update that both ends warming and
carries the final arrangement. The masonry layout depends on this to tell a
freeze apart from an append.

### Error classification

`#fail` maps a `FeedError` to one of four strings, and the comparisons are typed
`satisfies MortarErrorCode`, so a code renamed in mortar fails typechecking here:

| Code | `feed.error` | Rendering |
|---|---|---|
| `login_required` | `login-required` | "this wall is sealed", with a handle box (cleared) |
| `actor_not_found` | `handle-not-found` | "no wall for that handle", with a handle box (prefilled to correct) |
| `feed_not_found` | `feed-not-found` | "no such feed", with a way into the feed picker |
| anything else | `feed-unavailable` | "the wall wouldn't load", with a retry button |

Only mortar's own `actor_not_found` means the handle is bad. In local mode a
request that escapes the service worker hits the static host and 404s with a
non-JSON error document, which arrives as `unknown`; that must not be mistaken
for a missing handle.

`feed_not_found` carries its own code for exactly that reason in the other
direction: reusing `actor_not_found` would hand somebody with a bad feed link a
handle box, which repairs nothing they typed. Its panel therefore offers no
handle box and no retry. The way on is the header's wall switcher, which on a
feed wall is also the door to the feed picker, plus the demo link every panel
carries.

---

## Service-worker lifecycle

Registered manually from `+layout.svelte`, local mode only, always as
`type: 'module'` (the wasm-bindgen glue contains `import.meta`, which a classic
script rejects at parse time).

```
install   ▸ precache ["/", ...build, ...files, wasmUrl]; skipWaiting
activate  ▸ delete every mason-shell-* cache but this build's; clients.claim
fetch     ▸ /api/feed        → serveFeed  (+ waitUntil persistCaches)
          ▸ everything else  → serveShell (cache-first for hashed build assets,
                                network-first otherwise, cached shell offline)
```

`build` does not include the wasm engine, which rides in as a Vite `?url` asset
rather than a SvelteKit build output, so it is precached explicitly. That is what
lets a worker survive a deploy: the next deploy deletes this hashed wasm from S3,
but this worker keeps serving the copy in its own cache until it is itself
replaced. `ensureInit` therefore loads the engine from the precache first and
falls back to the network only for a fresh install. A failed init is never
memoised, so the next request retries instead of leaving the session bricked
behind a permanently-rejected promise.

### Deploy-reload policy

A deploy swaps the whole engine (a new wasm hash), so a page must not keep
running an old worker against new assets. But a hard reload mid-session drops the
laid wall, the scroll position and any playing video.

```
registration: updateViaCache: 'none' + registration.update()
              ▸ always revalidate the worker script rather than trust an HTTP cache

controllerchange fires:
   first flip while previously uncontrolled ─▶ adoption, not a deploy: ignore once
   otherwise ─▶ tab hidden?  reload now
                tab visible? pendingReload = true

pendingReload is applied at:
   visibilitychange to hidden   ─▶ location.reload()
   the next client-side nav     ─▶ location.href = destination  (a full page load,
                                    so page and engine leave together)
```

An uncontrolled tab splits into two states, and the registration path
discriminates them. A true first install produces one adoption flip that is not a
deploy. A shift-reload beside an already-active worker re-registers a
byte-identical script with no install, activate or claim, so no adoption flip
ever fires and the first flip that tab sees *is* a deploy: if a registration is
already active while the tab is uncontrolled, `hadController` is set up front.
This is checked before `register()`, so a deploy landing in the sliver before it
resolves still reads as adoption, and the next deploy converges.

### Offline

mason installs as an app (`site.webmanifest`, `display: standalone`), and an
installed app that dies without a network is a bad app. The shell and the wasm
are precached, so an offline launch opens the landing page and the demo wall,
whose bricks are fixtures compiled into the wasm and need no network at all.

---

## Testing

| Lane | Runner | Covers |
|---|---|---|
| `web/src/**/*.test.ts` | vitest, node environment | `FeedState` transitions, `api.ts` request shaping and error mapping |
| `web/tests/*.test.ts` | Playwright, chromium | The real static build: the worker intercepts `/api/feed` and lays the demo wall |
| `pnpm check:ci` | `tsc --noEmit` | Types in `.ts` and `.svelte.ts`, including the wire drift guard. **Not `.svelte` component bodies** |

vitest rides the app's own vite config through `mergeConfig`, so `.svelte.ts`
rune modules compile in tests exactly as they do in the build.

**No lane in this table typechecks a component.** `tsc` cannot parse `.svelte`
and drops those files silently, so a green run says nothing about one; the vite
build strips their types without checking them; and both vitest suites are `.ts`
that import no component. This is stated here as well as in
[development-guidelines.md](development-guidelines.md) because this is the page
somebody working on a component reaches for first.

---

## Assumptions and open questions

**Assumptions**

- `localStorage` is available. Every access is guarded by SvelteKit's `browser`
  flag but not by a `try`/`catch`, so a hard-blocked storage would throw.
- The reader's browser supports module service workers (Chrome 91+, Safari 15+,
  Firefox 147+). Below that, local mode has no feed engine.
- One tab at a time is the common case, but two tabs sharing one worker is
  handled (the persist chain, the reload policy).

**Decisions**

- *One route, `?actor=` or `?feed=` as truth.* **A wall is a URL.** Shared links
  are the growth loop, and back/forward has to mean something.
- *No SSR.* **`ssr = false`, static adapter.** The feed engine lives in a service
  worker; there is nothing a server could render, and the shell carries the
  crawler metadata instead.
- *A wall is a source and a view.* **`actor` or `feed` picks the source; the
  layout picker picks the view.** Readers do not think in query parameters, and
  they should not have to learn that one of mason's three views works on only one
  of its two sources.
- *Glaze is a view that changes the algorithm.* **One control, two effects, on
  either source.** On a graph wall it re-fetches an images-only wall; on a feed
  wall it filters the feed's own posts. `bento` and `masonry` stay pure
  presentation, so switching them never re-mixes.
- *Preferences in `localStorage`, not the URL.* **The URL identifies the wall,
  not the reader.** A shared link should show the recipient's own preferences.
- *Generation counter over cancellation.* **Every continuation rechecks it.**
  `AbortController` would cancel the request but not the reflow it drives, and
  the state machine has three concurrent entry points (preview, freeze, pump).
- *Replace during warming, append after.* **The freeze is the switchover.**
  Appending a reflowed screen would duplicate; replacing after the freeze would
  drop the scroll.
- *Deferred deploy reload.* **Reload when the reader is not looking.** A reload
  mid-session drops the wall, the scroll and any playing video; running an old
  engine against new assets is worse, so it is deferred, not skipped.
- *Warm from the landing form.* **Fire and discard a feed request.** It moves the
  wasm compile and the cache import off the critical path; the caches are
  DID-keyed and seed-independent, so the real wall reuses them.
- *The reader is history state, not a URL.* **`pushState('', { brick })`.** The
  back gesture has to close an overlay on a phone, and a `?brick=` link would
  advertise a deep link mason cannot serve: a single brick is not fetchable, so
  the recipient would get the wall and a dropped parameter.
- *Opening the reader freezes the wall.* **A click is engagement.** The reader
  locates its brick in `feed.items` by id, and a warming wall reorders between
  polls.
- *No index is threaded down to the cards.* **The reader locates the brick by
  id.** An index would edit the snippet signature, both layouts, all four cards
  and `BrickShell` to save a `findIndex` over a few hundred items, and every one
  of those files is invisible to `tsc`.
- *The reveal choice follows the brick.* **A session set of brick ids, shared
  by the card and the reader.** Uncovering a brick on the wall and finding it
  covered again one click later reads as a bug. It is still forgotten on
  reload: the set lives in a rune, not in storage.

**Open questions**

- *`localStorage` in lockdown.* Safari's private mode and some enterprise
  policies throw on access. Nothing wraps these calls; unproven in practice.
- *Multi-tab preference sync.* Two tabs with different layout choices do not
  observe each other's `localStorage` writes. Harmless today, but it means the
  preference is per-tab-session rather than per-browser after the first load.
- *The session wall cache is unbounded.* `FeedState.#cache` grows one entry per
  `(target, mode)` visited and is never trimmed. It dies with the page, so it is
  a ceiling on a very long session rather than a leak.
