# 10 - Build, Release, Deploy

**Status:** Draft · **Date:** 2026-07-25 · **Owner:** Ant Stanley

How mason is built, checked, versioned and shipped. Toolchain choices and code
style are in [development-guidelines.md](development-guidelines.md); the
architectural reason there are two build modes is in
[architecture-principles.md](architecture-principles.md).

---

## Responsibilities

1. Produce a fully static site from a Rust engine and a SvelteKit app.
2. Run one check suite locally and in CI, with the same commands.
3. Keep one version across the whole repo.
4. Make a release and a deploy the same event.

---

## Two build modes

| | Local mode (default) | Server mode |
|---|---|---|
| Trigger | `PUBLIC_MASON_SERVER_URL` unset | `PUBLIC_MASON_SERVER_URL` set |
| Feed engine | `mortar-wasm` in a service worker | `mortar-server` over CORS |
| Artefact | A static site, deployable anywhere | The same static site, plus a binary |
| Dev command | `just dev` | `just dev-server` |

`just wasm` is the seam between them, and it is not optional: any Rust change to
the engine needs a `just wasm` (or `just dev` / `just build`, which run it)
before the browser sees it.

```sh
cd server && wasm-pack build crates/mortar-wasm --target web --no-pack \
    --out-dir ../../../web/src/lib/mortar-wasm/pkg
```

The output is gitignored and imported by both the service worker and, through
Vite's `?url` asset handling, as a precached binary.

---

## Commands

Every command in the repo goes through `just`.

| Recipe | What it does |
|---|---|
| `just wasm` | Build the wasm feed engine into `web/src/lib/mortar-wasm/pkg` |
| `just dev` | Local mode: build wasm, run vite on :5173 |
| `just dev-server` | Server mode: native mortar on :8787 and the SPA against it |
| `just build` | Static site into `web/build/` (rebuilds wasm first) |
| `just test` | `cargo nextest run`, `pnpm check:ci` (tsc), `pnpm test` (vitest) |
| `just test-e2e` | Build, then the Playwright service-worker smoke in chromium |
| `just test-wasm` | The wasm-only Rust paths in a headless browser |
| `just lint` | `oxlint`, `knip`, `cargo clippy --workspace --all-targets -D warnings` |
| `just fmt` / `just fmt-check` | `oxfmt` and `cargo fmt` |
| `just guard-autoplay` | The video rule, enforced |
| `just guard-dashes` | No U+2014 em dash in tracked source, docs or config |
| `just check` | The local gate: `fmt-check`, `lint`, both guards, `test` |
| `just push [args]` | `just check`, then `jj git push` with those args |
| `just deploy [env]` | blogwright deploy (rebuilds wasm first) |
| `just bootstrap [env]` / `just bootstrap-preview <domain>` | One-time infrastructure creation |
| `just clean` | `cargo clean`; the target directory grows to about 3 GB |

`just dev-server` polls both child PIDs rather than using `wait -n`, which needs
bash 4+; macOS ships bash 3.2.

### Test lanes

Four lanes, because the engine compiles two ways and the client is a browser app.

| Lane | Runner | Scope |
|---|---|---|
| Native Rust | `cargo nextest` | The engine: scoring, mixing, cohorts, admission, sources against wiremock, the wire-contract pin |
| Wasm Rust | `wasm-pack test --headless --chrome` | The browser-only paths: the timer and spawn seam, the gloo-net transport, the hand-rolled throttle |
| Web unit | vitest | `FeedState` transitions and `api.ts` |
| Web end to end | Playwright | The real static build: the worker intercepts `/api/feed` and lays the demo wall |

Test modules in `mortar-core` are gated `cfg(all(test, not(target_arch = "wasm32")))`
so the native suite (tokio, wiremock) stays off the browser build, and the wasm
tests are gated the other way. Upstream base URLs are all `Config` fields, so
every source test points at a wiremock server rather than the network.

### Guards

Two rules are enforced by grep rather than by review, because a convention nobody
can check is a convention that erodes:

- **`guard-autoplay`** fails on any `autoplay` or `autostartload` in `web/src`,
  and on any `.play(` outside `VideoPlayer.svelte`.
- **`guard-dashes`** fails on a U+2014 em dash anywhere in the tree. The recipe
  builds the pattern from bytes so it holds no literal em dash itself, and it
  works by **denylist**: it scans everything and excludes the generated trees
  (`target`, `node_modules`, `build`, `dist`, `.svelte-kit`, the wasm `pkg`).
  An allowlist of scanned paths is the shape this recipe used to have, and it
  fails silently: a new tracked directory is simply never read, and the gate
  keeps exiting 0 over it. `.specs/` went a whole spec set unchecked that way.

Both use a filesystem grep rather than `git grep`, so new unsnapshotted files in
this jj-managed repo are seen too. `guard-dashes` additionally passes `-I` so a
binary that happens to hold the byte sequence is not a finding.

### Local gates

`just check` runs `just guard-dashes`, `just guard-autoplay`, `just fmt-check`,
`just lint` and `just test`, in that order. It is the same set CI runs, so a red
pull request from a formatting slip is not a thing that has to happen. The recipe
is dependencies-only: `just` runs the five before an empty body, so there is
exactly one list of gates and no second copy to drift out of step with it.

The order is cheapest first, and it is measured rather than asserted: the two
greps are about 0.2 s each, `fmt-check` about 1 s, `lint` about 2 s warm (it is
clippy, so minutes on a cold `target/`), `test` about 3.5 s. An earlier ordering
put the greps behind `lint`, which meant an em dash in a comment cost a full
clippy pass before anything reported it.

`just push` depends on `check` and then runs `jj git push`, which makes the
ordinary way to push the gated way. It is a **wrapper recipe, not a hook**. jj
has no hook system, and `jj git push` does not fire the colocated repo's
`.git/hooks/pre-push` either, so nothing here can intercept a push. What makes
the gate hard to skip is position rather than enforcement: `just push` is shorter
than `jj git push` and is the only push path these pages and `AGENTS.md`
recommend. A bare `jj git push` still works and skips the gate; these pages name
it only to say that, and to name what it costs (a red pull request rather than a
bad merge, since CI is the authority).

`test-e2e` and `test-wasm` stay out of `check` deliberately. Both need a real
browser, which takes the run past a minute, and a gate that slow is one people
learn to work around.

CI remains the authority. It re-runs the same set plus `just test-e2e` and the
wasm lane, and it is what a merge is gated on, so a push that went around the
local gate costs a red pull request rather than a bad merge.

---

## CI

Four workflows, all with SHA-pinned actions.

### `ci.yml`

Runs on every pull request and every push to `main`. No secrets, so it works on
fork PRs where the preview deploy cannot.

```
check job:      Rust 1.97.0 + wasm32 + clippy + rustfmt · just · nextest · wasm-pack · pnpm
                ▸ cargo metadata --locked            (Cargo.lock is fresh)
                ▸ just wasm                          (also the wasm compilability gate)
                ▸ just test · just lint · just fmt-check · just guard-autoplay

e2e job:        the same toolchain plus chromium
                ▸ just build · pnpm test:e2e
```

Building the wasm before the web checks is not incidental: the web app imports
the generated pkg, so `tsc` and `knip` need it present, and compiling it doubles
as the wasm32 gate.

`cargo metadata --locked` fails fast on a stale `Cargo.lock` rather than letting
a build silently repair it and ship an unpinned dependency graph. Every workflow
that builds repeats this check.

### `preview.yml`

Every PR deploys to `https://pr-<n>.<preview-domain>` on open, update and reopen,
and is torn down on close. One shared CloudFront distribution serves them all, so
a preview is just an S3 prefix. A sticky PR comment carries the URL and is
updated to a removal notice on teardown.

Fork PRs and dependabot PRs are skipped: both run without the OIDC token and the
preview secrets, so the deploy can only fail. They are covered by the secretless
`ci.yml` instead.

### `release.yml`

Runs on every push to `main`.

```
release job:
  ▸ compute the version the pending changesets add up to, and name the PR
    "release: mason v<next>"
  ▸ changesets/action:
       version:  pnpm run version   (changeset version + scripts/sync-version.mjs)
       publish:  pnpm run release   (scripts/release.mjs: tag, push, gh release)
       createGithubReleases: false
  ▸ outputs.released

deploy job:  needs release, if released == 'true'  ──▶ calls deploy.yml
```

`createGithubReleases` is off deliberately. `release.mjs` already tags and cuts
the release from the changelog; left on, the action cuts a *second* release for
the same tag, GitHub rejects it as an immutable tag name, and the job fails
**after** the release is live. The tag ships, `published` is never set, and the
deploy that depends on it is silently skipped. A release that does not deploy is
the one outcome this pipeline exists to prevent.

### `deploy.yml`

Two entry points, one definition: called by `release.yml`, or dispatched by hand
for a hotfix or to re-deploy unchanged code. Gated by the `production` GitHub
environment. Deploys are serialised (`concurrency: deploy-production`,
`cancel-in-progress: false`), so a manual dispatch queues behind a
release-triggered deploy rather than interleaving S3 uploads and CloudFront
invalidations with it.

An optional `refresh` input re-uploads every file including unchanged ones. S3
writes object metadata only on a PUT, so content-type and tag fixes never reach
objects the ETag check would skip; it invalidates the whole CDN, so it is off by
default.

---

## Versioning

**mason ships as one thing, so it carries one version.** Root `package.json` is
the source of truth, owned by changesets. `pnpm version` runs `changeset version`
and then `scripts/sync-version.mjs`, which propagates the number to:

- `web/package.json`
- `server/Cargo.toml` (`[workspace.package] version`, inherited by every crate)
- `server/Cargo.lock` (tracked, and `cargo build --locked` fails on a stale one)

The list is short on purpose. Anything that can read the version at compile time
should, rather than join it: the native user agent reads
`env!("CARGO_PKG_VERSION")` and so needs no propagation step, which is why it
cannot drift the way the literal it replaced did. Adding a fourth regex to
`sync-version.mjs` is the last resort, not the pattern.

The lockfile is patched by regex rather than by running cargo, so the release job
needs no Rust toolchain. Before changesets, the repo held three versions that all
disagreed: `web/package.json` at 0.0.1, the Cargo workspace at 0.1.0, and a
v0.1.0 release.

Add a changeset with any user-visible change:

```sh
pnpm changeset
```

Semantics for an app with no public API:

| Bump | When |
|---|---|
| **major** | The wall itself works differently, or a shared `?actor=` link stops meaning what it meant |
| **minor** | A new brick kind, a new surface, a visible capability |
| **patch** | Fixes and polish nobody has to relearn anything for |

Infrastructure-only changes (CI, deploy config, dependency bumps that change
nothing a visitor can see) need no changeset. A forgotten one can be added in a
follow-up PR; it joins the pending pile and lands in the next version.

Nothing is published to npm. The GitHub release is the artifact.

### A release is a ship

Merging the release PR bumps, changelogs, tags, cuts the GitHub release, and then
deploys to production, so the tag, the notes and the live site always describe
the same code. There is no such thing as a released version that never shipped.

Merging an ordinary PR to `main` does **not** deploy; it only updates the pending
release PR.

---

## Deploy

The static local-mode build deploys to S3 and CloudFront via
[blogwright](https://github.com/antstanley/blogwright).

```
CI: just wasm  ──▶ web/src/lib/mortar-wasm/pkg  (gitignored, but zipped via sourceInclude)
      │
      ▼
  repo zipped ──▶ Lambda MicroVM: pnpm install && pnpm build in web/
      │
      ▼
  ETag-diffed upload to S3  +  minimal CloudFront invalidation
```

Rust and wasm-pack stay in CI, out of the builder MicroVM, which is why the
generated pkg rides into the deploy zip through `sourceInclude` despite being
gitignored.

Config lives in `config/production.jsonc` and `config/preview.jsonc`:
`spa: true`, `paths: { app: "web", dist: "web/build" }`, region `eu-west-1`, and
`githubRepo` for provisioning the OIDC deploy role at bootstrap.

**Domains are never committed.** CI injects them from per-environment GitHub
secrets (`PRODUCTION_DOMAIN`, `PREVIEW_DOMAIN`), and `--domain` is passed only
when the secret is set; otherwise the CloudFront default domain serves the site.

Authentication is GitHub OIDC with no stored keys: `production-mason-gh` and
`preview-mason-gh` roles, assumed per job.

One-time setup, with AWS credentials in hand:

```sh
just bootstrap                             # production bucket, CDN, OIDC role
just bootstrap-preview preview.example.com # shared PR-preview stack (a Route53 zone)

gh secret set AWS_ACCOUNT_ID   --env production --body <account-id>
gh secret set AWS_ACCOUNT_ID   --env preview    --body <account-id>
gh secret set PREVIEW_DOMAIN   --env preview    --body preview.example.com
gh secret set PRODUCTION_DOMAIN --env production --body example.com   # optional
```

Day-2 operations are `blogwright status`, `history`, `logs <hash>` and
`rollback <hash>`.

---

## Version control

The repo is **jj-managed** (jujutsu), not raw git. Use `jj`, not `git`, even when
jj resists. Work lands via pull requests off `main`, which is branch-protected,
and goes out with `just push` rather than a bare `jj git push`, so the local gate
runs first (see Local gates above).

---

## Implementation layout

```
justfile                    every command
rust-toolchain.toml         channel 1.97.0, wasm32 target
package.json                the source-of-truth version; changesets scripts
.changeset/config.json      changelog-github, private packages versioned and tagged
scripts/sync-version.mjs    propagate the version everywhere
scripts/release.mjs         tag, push, gh release from the changelog section
config/{production,preview}.jsonc   blogwright
.github/workflows/          ci · preview · release · deploy
web/vite.config.ts          adapter-static, SW register: false, the mode define
web/playwright.config.ts    the service-worker smoke
web/vitest.config.ts        merged with the app's vite config
```

---

## Assumptions and open questions

**Assumptions**

- The pinned Rust channel in `rust-toolchain.toml` and the version in every
  workflow are kept in step by hand; nothing enforces it.
- `wasm-pack` 0.15.0 produces output the Vite `?url` import and the service worker
  can consume unchanged.
- The blogwright builder MicroVM has Node and pnpm but no Rust toolchain.
- A Route53 hosted zone exists for the preview domain, so the wildcard certificate
  and DNS are automatic.

**Decisions**

- *`just` as the only command surface.* **Local and CI run identical recipes.**
  Two sets of commands drift, and the one that drifts is always the local one.
- *Build the wasm before the web checks.* **`tsc` and `knip` need the pkg.** It
  also makes wasm compilability a CI gate for free.
- *Assert `Cargo.lock` is fresh in every building workflow.* **Fail fast.** A
  build that silently repairs the lock ships an unpinned dependency graph.
- *Guards are greps in CI.* **Autoplay and em dashes.** Both are rules a reviewer
  will eventually miss.
- *The local gate is a recipe you invoke.* **`just push`, not a hook.** jj has no
  hook system and does not fire git's, so an enforced local gate is not on the
  menu here; a shorter, documented push path that runs `just check` first is.
- *One version, changesets-owned.* **Root `package.json` propagates everywhere.**
  Three disagreeing versions is the state this replaced.
- *`createGithubReleases: false`.* **`release.mjs` cuts the release.** The action
  doing it too fails the job after the tag ships, and silently skips the deploy.
- *A release is a ship.* **The release workflow calls the deploy workflow.** A tag
  whose code is not live makes the tag meaningless.
- *Ordinary merges do not deploy.* **Only a release does.** Production changes on
  a decision, not on a merge.
- *Domains in secrets, not config.* **`--domain` only when set.** The repo is
  public and a domain is deployment detail, not source.
- *SHA-pinned actions.* **Every `uses:` carries a commit SHA.** A tag is mutable
  and a workflow with write permissions is worth pinning.

**Open questions**

- *Toolchain pin duplication.* `1.97.0` appears in `rust-toolchain.toml`, the
  workspace `rust-version`, and three workflows. Open: read it from the file in
  CI, or accept the duplication?
- *No native mortar deploy path.* Server mode has no packaging, image or deploy
  workflow; it exists as a `cargo run` target. Open until server mode has a
  consumer.
- *`web/README.md` is the `sv` scaffold default.* It still describes creating a
  SvelteKit project rather than mason's web app. Open: replace it, or point it at
  these specs?
