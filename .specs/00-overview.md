# mason - Design Overview

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley · **Scope:** Repo-wide

**mason** is an atproto discovery app: one masonry wall that mixes Bluesky
posts, standard.site blog documents, and video (Bluesky clips plus Streamplace
streams, live and archived) into a single endless scroll, built from a public
Bluesky follow graph with no login.

This document is the entry point for mason's design. It states the problem, the
goals, the system shape, and the scope. Detail pages are linked from each
section. The repo is a single project shipping a single version, so every spec
page lives in this directory rather than in a per-package layer.

---

## Problem

The atmosphere's content is scattered across lexicons. A Bluesky post lives in
`app.bsky.feed.post` and is readable from the public AppView; a blog post lives
in `site.standard.document` in the author's own repo; a livestream lives in
`place.stream.livestream` on stream.place. Each has an app that shows one of
them well and the others not at all, so the interesting work of the people you
already follow never shares a screen.

Existing discovery surfaces also require an account. A timeline is a logged-in
product, which means a shared link is a signup wall, and the reader's browsing
is visible to whoever runs the server. mason reads walls logged out, from the
reader's own browser, so a wall is a link you can hand to somebody.

---

## Goals

1. Turn a Bluesky handle into a mixed wall of posts, blogs and video, laid by
   the grout score and a kind-aware mixer, with no account and no sign-in.
2. Run the whole feed engine in the reader's browser by default: mortar compiles
   to wasm and answers `/api/feed` from a service worker, so the static site can
   be hosted anywhere and no server learns whose wall is being read.
3. Open fast on a cold graph: fan out under a rate limiter, paint once a dozen
   distinct authors have answered, and bound the whole opening wait to six
   seconds.
4. Extend the wall by scrolling: when the unlaid pool drains, fan out to the next
   hundred authors the wall has never asked, so the scroll quarries the whole
   follow graph and only ends when the graph is spent.
5. Offer a second wall shape from the same engine: `glaze`, an image-only wall
   built from Bluesky media posts, reached by the layout picker.
6. Honour a logged-out viewer's moderation reality: opted-out, hard-hidden and
   adult-labelled accounts and posts never reach the wall; `!warn` media is
   covered behind a per-brick reveal.
7. Keep the shipped artifact one thing at one version: root `package.json` is
   the source of truth and merging the release PR bumps, tags, releases and
   deploys together.
8. Let a reader read a brick without leaving the wall: an in-place reader
   renders the brick's own content at full size, and the trip to the source
   becomes a choice rather than the only option.
9. Split a wall into a source and a view. The source is either the reader's
   follow graph (mixed posts, blogs and video) or any Bluesky feed generator
   (`?feed=<at-uri>`, posts and Bluesky video only, in the feed's own order).
   All three views (bento, masonry, glaze) apply to either. On a feed wall the
   generator is the algorithm, so mason contributes the wall and nothing else.
10. Offer a feed picker as a second front door beside the handle box: browse what
    the network ranks, search it, list one person's feeds, or paste a link. Still
    no account and no sign-in.

## Non-goals

- **No accounts, no writes.** mason never authenticates, never posts, never
  likes. Every upstream read is an unauthenticated public read.
- **No database.** Everything is in-memory behind hand-rolled TTL caches, because
  the same code has to run inside a service worker.
- **No autoplay.** Videos require a click, always. `just guard-autoplay` enforces
  it in CI.
- **No server-side rendering.** The SPA is fully client-rendered; the shell in
  `web/src/app.html` carries the crawler-visible metadata.
- **No blog content rendering.** A blog brick is metadata plus a link out,
  on the wall and in the reader alike. The `site.standard.document` content
  union is platform-specific (Leaflet, pckt.blog, Offprint and WordPress all
  differ) and is never parsed, so the reader shows a blog's metadata at full
  size and makes the publication the primary destination.
- **No cross-kind score comparison.** Kinds compete only with themselves; the
  mixer decides the ratio.

---

## System shape

Local mode is the default and needs no server at all.

```
  ┌─────────────────────────────────────────────────────────────┐
  │ browser tab: SvelteKit SPA (static, no SSR)                 │
  │   /?actor=<handle>   layout picker · client picker · wall   │
  └───────────────┬─────────────────────────────────────────────┘
                  │ fetch /api/feed?actor=|feed=&cursor=&mode=&intent=
                  ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ service worker (web/src/service-worker.ts)                  │
  │   app-shell precache  ·  IndexedDB warm-cache persistence   │
  │   ┌───────────────────────────────────────────────────┐     │
  │   │ mortar-wasm  →  mortar-core (the feed engine)     │     │
  │   └───────────────────────────────────────────────────┘     │
  └───────────────┬─────────────────────────────────────────────┘
                  │ direct CORS reads, browser's own fetch
    ┌─────────────┼──────────────┬───────────────┬──────────────┐
    ▼             ▼              ▼               ▼              ▼
 public.api    plc.directory   each author's   stream.place   (blob reads
 .bsky.app     (did:plc docs)  PDS (blogs,     (live list,     on each PDS)
 (AppView)                     archived        HLS playback)
                               streams)
```

Server mode swaps only the box in the middle. Setting the
`PUBLIC_MASON_SERVER_URL` environment variable, which `just dev-server` does,
makes the same SPA call a native mortar over CORS instead of its own service
worker. It is a real environment variable read at build time, not a `.env` file:
see [07-web-client.md](07-web-client.md).

```
 browser tab (same SPA) ──CORS GET /api/feed──▶ mortar-server (axum, :8787)
                                                     │
                                                mortar-core
                                                     │
                                             the same upstreams
```

The engine is one Rust crate compiled twice. `mortar-core` is the whole feed
engine and compiles for both native and `wasm32-unknown-unknown`;
`mortar-server` is a thin axum binary around it; `mortar-wasm` is a thin
wasm-bindgen wrapper around it. Neither front holds feed logic: both call
`handle_feed`. The browser build reads exactly what the native one reads, so
there is no degraded mode.

A request builds or reuses a **snapshot**: one viewer's wall in progress, keyed
by (mode, viewer DID, seed). The snapshot fans out to a sampled cohort of the
follow graph, admits bricks into a pool, and lays them onto an append-only wall
in page-sized slices. Laid bricks never move.

---

## Detail pages

| Page | Topic |
|---|---|
| [01-domain-model.md](01-domain-model.md) | Bricks, authors, snapshots, cursors; identity and lifecycles |
| [02-feed-engine.md](02-feed-engine.md) | `handle_feed`, snapshot build, fill, extension waves, paging |
| [03-grout-and-mixer.md](03-grout-and-mixer.md) | The grout score and the weighted-round-robin mixer |
| [04-sources-and-moderation.md](04-sources-and-moderation.md) | The `sources/` seam, one page per upstream, and the label rules |
| [05-caching-and-persistence.md](05-caching-and-persistence.md) | TTL caches, their keys and lifetimes, IndexedDB persistence |
| [06-wire-contract.md](06-wire-contract.md) | `/api/feed`, `FeedResponse`, `ErrorEnvelope`, and the drift guard |
| [07-web-client.md](07-web-client.md) | The SPA: routes, reactive state, service-worker lifecycle |
| [08-wall-and-bricks.md](08-wall-and-bricks.md) | Layouts, card components, the player, warming reflow, empty states |
| [09-design-system.md](09-design-system.md) | Tokens, kiln palette, motion, focus, accessibility conformance |
| [10-build-release-deploy.md](10-build-release-deploy.md) | Build modes, `just` recipes, guards, CI, changesets, deploy |
| [architecture-principles.md](architecture-principles.md) | Layering rules, crate graph, the wasm32 constraint |
| [development-guidelines.md](development-guidelines.md) | Toolchain, code style, testing, definition of done |
| [canonical-types.schema.json](canonical-types.schema.json) | Wire shapes as JSON Schema |

---

## Scope summary

| Area | Implementation | Notes |
|---|---|---|
| Wall source | The reader's follow graph (default), or any Bluesky feed generator | `actor` or `feed`, exactly one of them |
| Wall views | `bento`, `masonry`, `glaze`, all three on either source | `Mode` carries glaze to the engine; bento and masonry are pure presentation |
| Brick kinds | Graph wall: post, blog, video (Bluesky and Streamplace, live and archived), five mix slots. Feed wall: post and Bluesky video only | A feed generator returns post URIs, so blogs and streams structurally cannot appear |
| Auth | None, in either direction | No sign-in; every upstream read is public |
| Storage | In-memory TTL caches; IndexedDB persistence for the browser build | No database, no server-side state |
| Pagination | Two opaque base64url cursor shapes: `{seed, offset}` for a graph wall, `{feed}` for a feed wall | Page size 24, except glaze on a feed wall; laid bricks never move |
| Moderation | Account and post labels; hidden tier dropped, `!warn` tier blurred | Mirrors what logged-out Bluesky shows |
| Captions | `CaptionTrack` modelled and rendered; no upstream supplies data | The one declared WCAG exception |
| Deployment | Static site to S3 + CloudFront via blogwright | PR previews per pull request, production by release |
| Server mode | `mortar-server` binary, CORS allowlist, no auth | The seam for future authenticated features |

---

## Assumptions and open questions

**Assumptions**

- The public Bluesky AppView (`public.api.bsky.app`), `plc.directory`, atproto
  PDS instances, and `stream.place` all serve permissive CORS. The browser build
  depends on this; it has no proxy to fall back to.
- The AppView's undocumented public rate limit is around 10 requests per second
  (3000 per 5 minutes), which is what the token bucket is sized against.
- Module service workers are available. mason states its floor as Chrome 91+,
  Safari 15+, Firefox 147+.
- `site.standard.document` and `place.stream.video` records are readable
  unauthenticated from their authors' repos via `com.atproto.repo.listRecords`.
- Browsers reap an idle service worker after roughly 30 seconds, so any state
  the engine holds can vanish between requests.

**Decisions**

- *Engine location.* **Wasm in a service worker, by default.** The static site
  then deploys anywhere, no server sees whose wall is browsed, and each reader
  spends their own upstream rate-limit budget instead of a shared one.
- *One engine, two fronts.* **`mortar-core` compiles native and wasm32.** A
  separate browser implementation would drift; sharing the crate means the
  browser build reads exactly what the server build reads.
- *Logged-out only.* **No sign-in anywhere.** It keeps the shareable `?actor=`
  link honest and removes the entire class of credential handling from a
  client-side app. The cost is accepted: opted-out accounts are unreadable, and
  mason seals those walls rather than working around them.
- *No database.* **In-memory TTL caches with hand-rolled expiry.** `moka` and
  every other mature cache crate is unavailable on `wasm32-unknown-unknown`,
  and the engine has to run there.
- *Snapshot immutability.* **Bricks are laid once and never move.** Endless
  scroll with a stable cursor is only possible if the wall behind the reader is
  append-only.
- *Kind ratio over global ranking.* **A weighted-round-robin mixer picks the
  kind, then the best brick within it.** Posts carry engagement counts that
  blogs and archived streams do not, so a single global score buries every kind
  but posts.

**Open questions**

- *The `sources/` seam's second implementation.* The seam exists so the
  ingestion layer can be swapped for a Jetstream plus SQLite backend without
  touching `algo/`. Nothing consumes it that way yet, so the seam is unproven
  against a second implementation.
- *Server mode's purpose.* `mortar-server` ships and is tested, but nothing in
  the product currently requires it; it is described as the path for future
  authenticated features. Open: what feature justifies it, and does that feature
  change the wire contract?
