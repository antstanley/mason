# mason

## 0.2.0

### Minor Changes

- [#5](https://github.com/antstanley/mason/pull/5) [`23e5e10`](https://github.com/antstanley/mason/commit/23e5e10fd7e7a6c08d88a7c9c97d975d9a59e7ec) Thanks [@antstanley](https://github.com/antstanley)! - Link previews, a kiln-fired Open Graph card, a favicon that is a wall, and offline install.

  Shared mason links previewed as a bare URL: crawlers do not run JavaScript and never boot the service worker that is the feed engine, so the shell carried no title, description or image. It does now, with a 1200x630 card built from mason's own dark tokens (source in `web/scripts/og-template.html`, rendered by `pnpm og`).

  The tab showed SvelteKit's Svelte logo. mason now has its own mark: a staggered bond of colour-coded bricks, a wall rather than a letterform, because an "m" turns to mush at 16px.

  mason also installs as a desktop app and survives offline. The service worker precaches the shell and the wasm, and the demo wall needs no network at all, because its bricks are fixtures compiled into the wasm.

### Patch Changes

- [#4](https://github.com/antstanley/mason/pull/4) [`a5f3f74`](https://github.com/antstanley/mason/commit/a5f3f74368b033537f72bd0edf4d66c0901c887b) Thanks [@antstanley](https://github.com/antstanley)! - Serve `site.webmanifest` with the right content type.

  S3 was returning `application/octet-stream`, because blogwright had no entry for the extension. Fixed upstream in blogwright 0.3.1, which also grants the CI build role the `s3:PutObjectTagging` permission its object tagging needs, and which now creates the preview stack's wildcard DNS record and its CloudFront log delivery instead of asking for them by hand.

  S3 writes object metadata only on a PUT, so a normal deploy skips content-identical files and a corrected header never reaches an object already live. Both workflows can now pass `--refresh` to re-upload everything: preview always does, production takes it as an input.
