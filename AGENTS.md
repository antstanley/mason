# AGENTS.md

Guidance for AI agents working in **mason**, an atproto discovery app. One wall,
every brick. Read `README.md` for the full architecture and `PRODUCT.md` for
product intent; this file is the operational cheat sheet. `.specs/` holds the
canonical design spec, and it is the long form of everything below: start at
`.specs/README.md`, and read `.specs/development-guidelines.md` before writing
code.

## Layout

```
web/                     SvelteKit SPA · Svelte 5 runes · Tailwind v4 · TS 7 · oxlint · oxfmt · knip
  src/lib/state/         *.svelte.ts rune modules: the app's logic and its unit tests
  src/lib/components/    the wall, the cards, the reader, the pickers
  src/service-worker.ts  local mode's mortar host; its own tsc project
  tests/                 playwright specs, driven against the built static site
server/crates/
  mortar-core/           feed engine; compiles native AND wasm32
  mortar-server/         native axum binary (server mode)
  mortar-wasm/           the same engine, built for the browser
.specs/                  the canonical spec, plus changes/merged/ and plans/
```

Two build modes, one Rust engine. **Local mode (default)**: mortar compiles to
wasm and runs in a service worker that intercepts `/api/feed`; the browser talks
directly to the AppView, plc.directory, each PDS, and stream.place. **Server
mode**: set the `PUBLIC_MASON_SERVER_URL` environment variable, which
`just dev-server` does, and the SPA calls a native mortar over CORS. It is a real
environment variable read at build time by a vite `define`, not a `.env` file:
vite's default `envPrefix` is `VITE_`, so a `PUBLIC_`-prefixed name in `web/.env`
never reaches `import.meta.env` and the SPA quietly stays in local mode.

Two wall sources, three views. `?actor=<handle>` lays a follow graph, which is
the whole snapshot machinery: cohort, fill, grout, mixer, extension waves.
`?feed=<at-uri or bsky.app link>` lays somebody else's Bluesky feed generator in
its own order, in FRONT of that machinery, with no snapshot, grout or mixer
behind it. `feed` wins when both are in the URL, and `FeedTarget::from_query` in
`mortar-core/src/feed.rs` is the one copy of that rule, because it is the only
place both fronts and the contract fixture can test. The layout picker's three
views (bento, masonry, glaze) apply to either source, and glaze is a view AND an
algorithm: it also asks mortar for `?mode=glaze`, the image wall of Bluesky media
only. The first screen warms rather than blocking: the client polls
`intent=preview` and reflows, then commits once with `intent=freeze` (or the
moment the reader scrolls). `refresh=1` lays the wall again in place, stepping
over the fast content caches. A plain click opens a brick in the reader over an
inert wall, held in `page.state.brick`, so the back gesture closes it. See
`.specs/02-feed-engine.md` and `.specs/08-wall-and-bricks.md`.

## Commands (via `just`)

```sh
just dev          # local mode: builds wasm, runs vite on :5173
just dev-server   # server mode: native mortar :8787 + SPA against it
just wasm         # rebuild web/src/lib/mortar-wasm/pkg (gitignored, generated)
just build        # static site → web/build/ (rebuilds wasm first)
just test         # cargo nextest + two tsc projects + vitest
just test-e2e     # service-worker smoke: the static build driven in chromium
just test-wasm    # wasm-bindgen tests in headless chrome
just lint         # oxlint + knip + clippy
just fmt          # oxfmt + cargo fmt (just fmt-check to verify only)
just guard-autoplay   # enforces the no-autoplay rule
just guard-dashes     # enforces the no-em-dash rule
just guard-toolchain  # the pinned rust channel satisfies the declared MSRV
just guard-wasm       # mortar still compiles for wasm32, the default build mode
just check        # the local gate: the four guards + fmt-check + lint + test
just push         # just check, then jj git push. THE way to push.
just deploy       # blogwright deploy to production (CI usually does this)
just clean        # cargo clean; target dir grows to ~3GB
```

Any Rust change to the engine needs a `just wasm` (or `just dev`/`just build`,
which run it) before the browser sees it. `test` and `lint` each declare `wasm`
as a dependency and each says why in the justfile: the generated pkg is
gitignored, so on a fresh clone tsc and knip both fail on an unresolved module
rather than on anything the change touched. Try actor `demo` for an offline
fixture wall.

CI runs the same gate by calling `just check` as one step, so a gate added there
is enforced in CI for free. The two browser lanes (`test-wasm`, `test-e2e`) stay
out of the local gate and run as their own CI jobs.

**`pnpm exec playwright test` does not build.** It starts `vite preview`, which
SERVES `web/build/`, so running a spec straight after touching `web/src` (or the
Rust behind the wasm) drives the previous build and answers about code you no
longer have: green, fast and wrong. `just test-e2e` builds first and never had
the problem. A `globalSetup` guard now refuses the run when the build is older
than what it was made from, and names the files, so the shortcut is safe too.
Editing a spec in `web/tests/` never trips it: specs are not built.

## Conventions & gotchas

- **Naming is the brand.** brick (a content card), mortar (the feed engine),
  grout (the ranking score), kiln (tones). Keep the metaphor; voice is
  lowercase, brick-punning, brief.
- **Card is the web name for a brick.** The Rust engine models content as
  `Brick`; the Svelte renderers in `web/src/lib/components/cards/` are
  `*Card.svelte`. The two vocabularies are deliberate, not drift: a brick is
  the model, a card is the rendered brick.
- **No em dashes.** Anywhere: UI copy, code comments, commits, specs.
  `just guard-dashes` greps the whole tree (a denylist of generated dirs, not an
  allowlist of paths) and fails if one appears.
- **Videos never autoplay.** `just guard-autoplay` fails on the word `autoplay`
  anywhere in `web/src` and on any `.play(` outside `VideoPlayer.svelte`, where
  it is gated behind a click. It is an accessibility stance, not a preference.
- **Logic belongs in `.svelte.ts`, not in a component body.** Nothing in this
  repo typechecks or unit tests a `.svelte` file, so a decision with a wrong
  answer (the reader's open rule, the picker's parse, the url scheme guard)
  lives in a rune module or a plain module beside its test, and the component
  renders what it is told.
- **TypeScript 7, and it does not see your component.** `tsc` cannot parse
  `.svelte`, so zero of them enter the program and a green typecheck says
  nothing about a component body. `pnpm check:ci` runs two projects, the app and
  `tsconfig.worker.json` for the service worker, which `svelte-kit sync`
  excludes from the first. `svelte-check` would close the component gap and
  crashes on TS 7 (programmatic API stabilizes in TS 7.1, ~Oct 2026). Do not
  swap `check` back to `svelte-check` yet, and do not read a green run as
  coverage.
- **The wire is pinned by a fixture.** `web/src/lib/types.ts` hand-mirrors
  mortar's serde output; `contract-check.ts` (tsc-only, never bundled) asserts
  it against `mortar-core/tests/fixtures/contract.json`, which
  `tests/contract.rs` pins byte for byte. After an intentional wire change,
  regenerate with `UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract`,
  then update `types.ts` until `pnpm check:ci` is green again.
- **Formatting is oxfmt, linting is oxlint, dead-code is knip**, not prettier /
  eslint. Run `just check` before finishing, and `just push` to push.
- `mortar-core` must stay `wasm32`-compatible: everything in-memory, hand-rolled
  TTL caches, no database, no threads. The `sources/` boundary is the v2 seam.
  `just guard-wasm` enforces it, and it is the ONLY gate that can: `lint` and
  `test` run on the host, so a dependency that builds natively and dies on
  wasm32 passes both. That is not hypothetical, `rand` 0.10 did it.

## Specs, changes, plans

`.specs/` describes what exists on the current branch; anything aspirational
sits in a page's closing `Assumptions and open questions` block. A delta that is
not built yet is a change spec under `.specs/changes/`, and lands in
`changes/merged/` (dated, kept as history) once shipped. A plan under
`.specs/plans/` is a kanban board of task packages: a task's status is the
subfolder it sits in. If a spec page and the code disagree, that is a divergence
to fix in one or the other, not something to write around.

## Version control

This repo is **jj-managed** (jujutsu), not raw git. Use `jj`, not `git`, even
when jj resists. Land work as PRs off `main`.

## Releases

mason ships as one thing, one version. Root `package.json` is the source of
truth, owned by changesets; `pnpm version` propagates it to `web/package.json`,
the Rust workspace, and `Cargo.lock`. Add a changeset (`pnpm changeset`) with any
user-visible change. A release is a ship: merging the release PR (named
"release: mason v<next version>") bumps, changelogs, tags, cuts the GitHub
release, and deploys to production.
