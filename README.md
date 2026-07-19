# mason 🧱

**One wall, every brick.** An atproto discovery app: a single vibrant masonry
grid with endless scroll, mixing three kinds of content recommended from your
Bluesky follow graph. No login; type your handle, we peek at your public
follows.

| brick | source | accent |
|---|---|---|
| 📱 posts | `app.bsky.feed.post` via the public AppView | sky |
| 📝 blogs | [standard.site](https://standard.site) documents (Leaflet, pckt.blog, Offprint, WordPress…) | tangerine |
| 🎬 video | Bluesky native video + [Streamplace](https://stream.place) streams, live and archived; all HLS, always click-to-play, **never autoplay** | violet |

A friend who is live right now opens the wall. It is the only brick with a
deadline, so it is the only one that jumps the queue.

## Two build modes, one Rust engine

```
web/                     SvelteKit SPA · Svelte 5 runes · Tailwind v4 · TS 7 · oxlint · oxfmt · knip
server/crates/
  mortar-core/           the feed engine; compiles native AND wasm32
  mortar-server/         native axum binary (server mode, future auth work)
  mortar-wasm/           the same engine for the browser
```

**local mode (default; no server at all):** mortar compiles to wasm and runs
inside a service worker that intercepts `/api/feed`. The static site deploys
anywhere; your browser talks directly to the public AppView, plc.directory,
each author's PDS, and stream.place (all CORS-open). Nobody's server sees
whose feed you browse, and every user spends their own rate-limit budget.
The browser build reads exactly what the native one reads; there is no
degraded mode.
Module service workers required: Chrome 91+, Safari 15+, Firefox 147+.

**server mode:** set `PUBLIC_MASON_SERVER_URL` in `web/.env` and the same SPA
calls a native mortar over CORS; the path for future authenticated features.

Either way, mortar builds a per-user **snapshot**: resolve handle → fetch
follows → sample a cohort of 100 authors (60 known-active + 40 seeded
exploration) → fan out under a global rate limiter with a first-paint
threshold (respond once bricks from 12 distinct authors have arrived, or 3 s)
→ keep filling in the background. A ~6 s mix deadline bounds the whole opening
wait, so a cold wall never waits longer than that for its first screen.

Bricks are laid by the **grout score**: within-kind recency decay
(half-lives: posts 12 h, blogs 3 d, archived streams 14 d; a Bluesky video
ages like a post) inside a hard age window (posts 72 h, blogs 14 d, archived
streams 90 d) × log engagement, then a weighted-round-robin mixer picks the
next *kind* by need (68/15/9/5/3 target across posts, blogs, Bluesky video,
archived streams, and live) and the best brick *within* that kind; kinds are
never compared by raw
score. Author-diversity window of 8, deterministic seeded jitter, opaque
`{snapshot, seed, offset}` cursor: endless scroll is stable and duplicate-free,
and every refresh is a fresh wall.

Everything is in-memory (hand-rolled TTL caches; wasm-compatible); no
database. The `sources/` boundary is the v2 seam for a Jetstream + SQLite
upgrade. A killed service worker (browsers reap them after ~30 s idle) is
just cache eviction: the cursor carries the seed, so the wall rebuilds
deterministically.

## Run it

```sh
just dev          # LOCAL mode: wasm SW + vite on :5173; no server
just build        # fully static site in web/build/
just dev-server   # server mode: native mortar :8787 + SPA against it
just test         # cargo nextest + typecheck
just test-wasm    # the wasm-only Rust paths, in headless chrome
just lint         # oxlint + knip + clippy
just guard-autoplay   # the video rule, enforced
just clean        # reclaim the cargo target dir (~3GB)
```

Try actor `demo` for an offline fixture wall.

## Releases

mason ships as one thing, so it carries one version. The root `package.json` is
the source of truth, owned by [changesets](https://github.com/changesets/changesets);
`pnpm version` propagates that number to `web/package.json`, the Rust workspace,
and `Cargo.lock`, so they cannot drift apart (they had: 0.0.1, 0.1.0 and v0.1.0
all at once).

Add a changeset with any user-visible change:

```sh
pnpm changeset      # describe the change, pick major/minor/patch
```

Commit the generated file. On merge to `main`, CI keeps a **"chore: version
mason"** PR open collecting every pending changeset. Merging *that* PR bumps the
version everywhere, writes `CHANGELOG.md`, tags, and cuts the GitHub release.
Nothing is published to npm: the release is the artifact.

**A release is a ship.** Merging the version PR bumps, changelogs, tags, cuts the
GitHub release, and then deploys to production, so the tag, the notes and the live
site always describe the same code. There is no such thing as a released version
that never shipped.

Merging an ordinary PR to `main` does not deploy: it only updates the pending
version PR. The deploy workflow can still be dispatched by hand for a hotfix, or
to re-deploy unchanged code.

What the numbers mean here, for an app with no public API:

| bump | when |
|---|---|
| **major** | the wall itself works differently, or a shared `?actor=` link stops meaning what it meant |
| **minor** | a new brick kind, a new surface, a visible capability (offline install, link previews) |
| **patch** | fixes and polish nobody has to relearn anything for |

Infrastructure-only changes (CI, deploy config, dependency bumps that change
nothing a visitor can see) need no changeset. Forgot one? Add it in a follow-up
PR; it joins the pending pile and lands in the next version.

## Deploy (AWS via blogwright)

The static local-mode build deploys to S3 + CloudFront with
[blogwright](https://github.com/antstanley/blogwright): the repo is zipped
(wasm pkg included via `sourceInclude`; Rust/wasm-pack stay in CI, out of
the builder MicroVM), built in a Lambda MicroVM (`pnpm install && pnpm build`
in `web/`), and synced with ETag-diffed uploads + minimal CloudFront
invalidations. Config lives in `config/{production,preview}.jsonc`
(`spa: true`, `paths: { app: "web", dist: "web/build" }`).

One-time setup with AWS credentials:

```sh
just bootstrap                             # production bucket/CDN/OIDC role
just bootstrap-preview preview.example.com # shared PR-preview stack (Route53 zone)

# per-environment GitHub secrets (domains never live in the repo):
gh secret set AWS_ACCOUNT_ID --env production --body <account-id>
gh secret set AWS_ACCOUNT_ID --env preview --body <account-id>
gh secret set PREVIEW_DOMAIN --env preview --body preview.example.com
gh secret set PRODUCTION_DOMAIN --env production --body example.com  # optional
```

Then CI (GitHub-OIDC; no stored keys):

- **PR previews** (`preview.yml`): every PR deploys to
  `https://pr-<n>.<preview-domain>` on open/update and is torn down on
  close; one shared distribution, so a preview is just an S3 prefix.
  (This very sentence shipped through the first preview PR. 🧱)
- **Production** (`deploy.yml`): manual workflow dispatch, gated by the
  `production` GitHub environment. `just deploy` does the same from a
  laptop. `blogwright status`, `history`, `logs <hash>`, and
  `rollback <hash>` cover day-2 operations.
