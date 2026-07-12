---
"mason": minor
---

Link previews, a kiln-fired Open Graph card, a favicon that is a wall, and offline install.

Shared mason links previewed as a bare URL: crawlers do not run JavaScript and never boot the service worker that is the feed engine, so the shell carried no title, description or image. It does now, with a 1200x630 card built from mason's own dark tokens (source in `web/scripts/og-template.html`, rendered by `pnpm og`).

The tab showed SvelteKit's Svelte logo. mason now has its own mark: a staggered bond of colour-coded bricks, a wall rather than a letterform, because an "m" turns to mush at 16px.

mason also installs as a desktop app and survives offline. The service worker precaches the shell and the wasm, and the demo wall needs no network at all, because its bricks are fixtures compiled into the wasm.
