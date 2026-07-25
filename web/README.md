# web

mason's front end: a SvelteKit SPA, Svelte 5 runes, Tailwind v4, TypeScript 7,
built to a fully static site with `adapter-static`.

In the default **local mode** this app has no backend. The service worker
(`src/service-worker.ts`) loads mortar compiled to wasm and answers `/api/feed`
itself, and the browser reads the AppView, plc.directory, each author's PDS and
stream.place directly. In **server mode** the same app calls a native mortar over
CORS instead.

## Running it

Drive this app from the repo root, not from here. The recipes build the wasm feed
engine first, and without it the app will not typecheck, let alone run.

```sh
just dev          # local mode: wasm service worker + vite on :5173
just dev-server   # server mode: native mortar on :8787, SPA against it
just build        # static site → web/build/
just test         # includes this app's vitest and tsc lanes
just test-e2e     # the service-worker smoke, chromium
just lint         # oxlint + knip (+ clippy for the engine)
just fmt          # oxfmt (+ cargo fmt)
```

The `pnpm` scripts in `package.json` are what those recipes call. Run them
directly only when you already have `src/lib/mortar-wasm/pkg/` built.

Try actor `demo` for an offline fixture wall: its bricks are compiled into the
wasm, so it needs no network at all.

## Layout

```
src/
  app.html              the shell: crawler-visible metadata, icons, theme colours
  app.css               Tailwind v4 @theme tokens and the base layer
  service-worker.ts     local mode's feed responder, shell cache, IndexedDB persistence
  routes/               one route: /?actor=<handle>
  lib/
    api.ts              the only module that names /api/feed
    types.ts            the wire mirror, guarded by contract-check.ts
    columns.ts          colsForWidth: the one column-count source
    state/*.svelte.ts   rune singletons: feed, layout, client, handle, profile, player
    components/         the wall, the cards, the chrome
scripts/                icon and OG-image generation
tests/                  the Playwright service-worker smoke
```

## Gotchas

- **Rebuild the wasm after any Rust change** (`just wasm`), or the browser keeps
  running the old engine.
- **The typecheck is `tsc --noEmit`, not `svelte-check`**, which crashes on
  TypeScript 7. Do not swap it back yet.
- **Videos never autoplay.** `just guard-autoplay` enforces it.
- **No em dashes**, in copy or comments. `just guard-dashes` enforces it.
- `src/lib/mortar-wasm/pkg/` is generated and gitignored.

## Specs

The canonical design spec is in [`../.specs/`](../.specs/README.md). The pages
that cover this app are
[07-web-client.md](../.specs/07-web-client.md) (routes, state, service-worker
lifecycle), [08-wall-and-bricks.md](../.specs/08-wall-and-bricks.md) (layouts,
cards, the player), [09-design-system.md](../.specs/09-design-system.md)
(tokens, motion, accessibility) and
[06-wire-contract.md](../.specs/06-wire-contract.md) (what `/api/feed` speaks).
