# AGENTS.md

Guidance for AI agents working in **mason** — an atproto discovery app. One wall,
every brick. Read `README.md` for the full architecture and `PRODUCT.md` for
product intent; this file is the operational cheat sheet.

## Layout

```
web/                     SvelteKit SPA · Svelte 5 runes · Tailwind v4 · TS 7 · oxlint · oxfmt · knip
server/crates/
  mortar-core/           feed engine; compiles native AND wasm32
  mortar-server/         native axum binary (server mode)
  mortar-wasm/           the same engine, built for the browser
```

Two build modes, one Rust engine. **Local mode (default)**: mortar compiles to
wasm and runs in a service worker that intercepts `/api/feed`; the browser talks
directly to the AppView, plc.directory, each PDS, and stream.place. **Server
mode**: set `PUBLIC_MASON_SERVER_URL` in `web/.env` and the SPA calls a native
mortar over CORS.

## Commands (via `just`)

```sh
just dev          # local mode: builds wasm, runs vite on :5173
just dev-server   # server mode: native mortar :8787 + SPA against it
just build        # static site → web/build/ (rebuilds wasm first)
just test         # cargo nextest + tsc typecheck
just lint         # oxlint + knip + clippy
just fmt          # oxfmt + cargo fmt
just guard-autoplay   # enforces the no-autoplay rule
just clean        # cargo clean — target dir grows to ~3GB
```

Any Rust change to the engine needs a `just wasm` (or `just dev`/`just build`,
which run it) before the browser sees it. Try actor `demo` for an offline
fixture wall.

## Conventions & gotchas

- **Naming is the brand.** brick (a content card), mortar (the feed engine),
  grout (the ranking score), kiln (tones). Keep the metaphor; voice is
  lowercase, brick-punning, brief.
- **No em dashes.** Anywhere — UI copy, code comments, commits.
- **Videos never autoplay.** `just guard-autoplay` greps `web/src` for the word
  and fails if present; it is an accessibility stance, not a preference.
- **TypeScript 7.** `svelte-check` crashes on TS 7 (programmatic API stabilizes
  in TS 7.1, ~Oct 2026); typecheck is plain `tsc --noEmit`. Do not swap
  `check` back to `svelte-check` yet.
- **Formatting is oxfmt, linting is oxlint, dead-code is knip** — not prettier /
  eslint. Run `just fmt` and `just lint` before finishing.
- `mortar-core` must stay `wasm32`-compatible: everything in-memory, hand-rolled
  TTL caches, no database, no threads. The `sources/` boundary is the v2 seam.

## Version control

This repo is **jj-managed** (jujutsu), not raw git. Use `jj`, not `git`, even
when jj resists.

## Releases

mason ships as one thing, one version. Root `package.json` is the source of
truth, owned by changesets; `pnpm version` propagates it to `web/package.json`,
the Rust workspace, and `Cargo.lock`. Add a changeset (`pnpm changeset`) with any
user-visible change. A release is a ship: merging the "chore: version mason" PR
bumps, changelogs, tags, cuts the GitHub release, and deploys to production.
