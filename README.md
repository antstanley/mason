# mason 🧱

**One wall, every brick.** An atproto discovery app: a single vibrant masonry
grid with endless scroll, mixing three kinds of content recommended from your
Bluesky follow graph. No login — type your handle, we peek at your public
follows.

| brick | source | accent |
|---|---|---|
| 📱 posts | `app.bsky.feed.post` via the public AppView | sky |
| 📝 blogs | [standard.site](https://standard.site) documents (Leaflet, pckt.blog, Offprint, WordPress…) | tangerine |
| 🎬 video | Bluesky native video + Steam trailers — both HLS, always click-to-play, **never autoplay** | violet |

## Architecture

```
web/     SvelteKit · Svelte 5 runes · Tailwind v4 · TypeScript 7 · oxlint · oxfmt · knip
server/  "mortar" — Rust · axum · moka · governor · cargo-nextest
```

The SvelteKit app proxies `/api/feed` to **mortar**, which builds a per-user
**snapshot**: resolve handle → fetch follows → sample a cohort of 100 authors
(60 known-active + 40 seeded exploration) → fan out under a global rate
limiter with a first-paint threshold (respond at 40 authors or 3 s) → keep
filling in the background.

Bricks are laid by the **grout score**: within-kind recency decay
(posts 24 h · blogs 7 d · trailers 30 d) × log engagement, then a
weighted-round-robin mixer picks the next *kind* by need (70/15/10/5 target)
and the best brick *within* that kind — kinds are never compared by raw
score. Author-diversity window of 8, deterministic seeded jitter, opaque
`{snapshot, seed, offset}` cursor: endless scroll is stable and duplicate-free,
and every refresh is a fresh wall.

Everything is in-memory (moka TTL caches) — no database. The `sources/`
boundary is the v2 seam for a Jetstream + SQLite upgrade.

## Run it

```sh
just dev        # mortar on :8787 + vite on :5173
just test       # cargo nextest + typecheck
just lint       # oxlint + knip + clippy
just guard-autoplay   # the video rule, enforced
```

Try actor `demo` for an offline fixture wall.
