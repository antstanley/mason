# Development Guidelines

**Status:** Draft · **Date:** 2026-07-25 · **Owner:** Ant Stanley · **Scope:** Repo-wide

The rules of the road for everyone writing code in mason, human or agent. This
page covers the toolchain, the pervasive coding style, defensive coding, limits,
version control, the per-language conventions, testing, repository hygiene, and
the definition of done.

It sits beside [architecture-principles.md](architecture-principles.md), which
says how the code is *organised*; this page says how it is *written*. The
commands and CI gates named here are defined in
[10-build-release-deploy.md](10-build-release-deploy.md).

---

## Toolchain

| Tool | Version / channel | Notes |
|---|---|---|
| just | latest | The only command surface; local and CI run identical recipes |
| Rust | 1.97.0, pinned | `rust-toolchain.toml`, with the `wasm32-unknown-unknown` target |
| rustfmt | pinned channel | `just fmt` / `just fmt-check`; CI gate |
| clippy | pinned channel | `cargo clippy --workspace --all-targets -- -D warnings` |
| cargo-nextest | latest | The native Rust test runner |
| wasm-pack | 0.15.0 | Builds `mortar-wasm`, and runs the wasm test lane in headless chrome |
| TypeScript | 7.x, `strict` | `tsc --noEmit`; not `svelte-check` (see conventions) |
| Node | 22 | Tooling runtime |
| pnpm | 11.x | Package manager |
| oxlint | latest | The TS and Svelte linter; not ESLint |
| oxfmt | latest | The TS formatter; not Prettier |
| knip | latest | Dead-code and unused-dependency analysis |
| vitest | latest | Web unit tests |
| Playwright | latest | The service-worker end-to-end smoke, chromium only |
| changesets | latest | Owns the single repo version |
| jj (jujutsu) | latest | The version-control front end |

Tooling not in use, named so nobody reaches for it: ESLint, Prettier,
`svelte-check`, `cargo test` (nextest is the runner), npm and yarn.

---

## Tiger Style, as mason practises it

mason's pervasive style is **Tiger Style**: be defensive, validate everything,
bound everything. The design priorities are **safety, performance, developer
experience, in that order**; when they conflict, safety wins. Deviations need a
written reason in the change description.

The short form: assume any input you did not produce is wrong, assume any
invariant you did not check can be violated, make every limit explicit and named.

The load-bearing principles, in the form mason actually applies them:

- **Zero technical debt.** Do it right the first time. Several of the constants
  and workarounds in this repo carry a comment naming the bug they replaced; that
  is the intended shape of a fix.
- **Simple, explicit control flow.** No recursion in the engine. Loops carry an
  explicit bound or an explicit deadline. Combinator chains that hide a
  non-trivial branch lose to an explicit `match`.
- **Limits on everything.** Every fan-out, cache, retry, page, deadline, window
  and cap is a named constant with its units in the name. There are no magic
  numbers in `algo/`, `sources/` or `cache.rs`.
- **Always say why.** This is the rule mason enforces hardest. A comment that
  paraphrases the code is noise; a comment that names the bug a line prevents, or
  the constraint that forced a shape, is the most valuable text in the file. Read
  `algo/fill.rs` for the reference example.
- **Degrade, do not fail.** One flaky upstream out of a hundred must never cost
  a reader their wall. See [architecture-principles.md](architecture-principles.md),
  Failure philosophy.

**Where mason departs from canonical Tiger Style, deliberately:** the engine does
not carry runtime `assert!` density. Its correctness net is the type system, the
exhaustive `match`, a heavy unit-test suite over pure functions with an injected
clock, and a wire contract pinned by fixture on both sides. Assertions are used
where they earn their place; the two-per-function target is not a rule here. This
is recorded as a Decision below rather than left implicit.

---

## Defensive coding

### Where to validate

Validate at every boundary where data crosses from somewhere you do not control
into somewhere you do. mason's boundaries, and what each one checks:

| Boundary | What is validated | Where |
|---|---|---|
| Query string → engine | `actor` present; `mode` and `intent` fall back to defaults on anything unrecognised | `feed.rs`, `mode.rs` |
| Cursor → engine | Base64 and JSON both fallible; garbage yields `None` and a fresh wall, never a 500 | `algo/cursor.rs` |
| Cursor offset → arithmetic | `checked_add` / `saturating_add`; an attacker-writable offset must not overflow | `algo/snapshot.rs`, `feed.rs` |
| Upstream JSON → bricks | Unparseable records are logged and skipped, never best-effort patched | `sources/standardsite.rs`, `sources/streamplace.rs` |
| Upstream timestamps → scoring | Unparseable or more than 600 s in the future is treated as stale, not as age zero | `algo/score.rs` |
| Upstream counts → scoring | `saturating_add` / `saturating_mul`; a hostile pair near `u64::MAX` must not overflow | `algo/score.rs` |
| Third-party URL → `<a href>` | `http`/`https` only; `javascript:`, `data:`, `vbscript:` never reach the anchor | `sources/util.rs::is_http_url` |
| Any value → upstream query string | Percent-encoded to the RFC 3986 unreserved set | `sources/util.rs::urlencode` |
| DID document → outbound request | Scheme, userinfo, hostname and IP-literal vetting; DNS vetting natively | `sources/pds.rs` |
| Moderation labels → the wall | Hidden tier dropped, warn tier blurred, at three separate choke points | `sources/bluesky.rs`, `algo/cohort.rs`, `sources/fetch.rs` |
| Engine response → the client | Parsed defensively; a non-JSON body is not mortar speaking | `web/src/lib/api.ts` |

Treat third parties as adversarial. Never deserialise and trust the result.

### Assertions in Rust

- `assert!`, `debug_assert!` and `assert_eq!` are available and used where an
  invariant is worth crashing over. They are not sprinkled to hit a count.
- **Split compound assertions.** `assert!(a); assert!(b);` beats `assert!(a && b)`,
  so a failure points at the actual broken condition.
- **No `unwrap()` in production paths.** `expect()` is permitted only where the
  operation is genuinely infallible or is init-time, and it always carries a
  reason string: `expect("cursor serializes")`, `expect("reqwest client builds")`,
  `expect("nonzero")`.
- **No `panic!` for control flow.** A panic signals programmer error only. Every
  runtime failure the engine can meet is an `AppError`, an `HttpError`, or a
  degraded empty yield.
- Test bodies may `unwrap` and `expect` freely.

### Assertions in TypeScript

- **Exhaustiveness is enforced by types, not by a runtime check.** A `switch` or
  `{#if}` over `Brick["kind"]` is checked by `tsc` against the discriminated
  union mirrored from mortar.
- **Wire literals are asserted at type level.** Comparisons against mortar's error
  codes are written `e.code === ("login_required" satisfies MortarErrorCode)`, so
  a code renamed in Rust fails typechecking in the web app.
- **The wire contract is the validator.** Rather than a runtime schema library,
  mason pins the wire in `tests/fixtures/contract.json` and checks it from both
  sides (`contract.rs` in Rust, `contract-check.ts` in `tsc`). See
  [06-wire-contract.md](06-wire-contract.md).
- Anything that can fail at runtime is guarded where it is parsed:
  `res.json().catch(() => null)`, `JSON.parse` inside a `try`, `new URL()` inside
  a `try`.

### Errors are data, not exceptions

- Every engine error is a value with a typed reason: `AppError` and `HttpError`,
  both `thiserror`-derived, both `Clone` where they need to travel.
- **Errors carry a stable machine code.** `AppError::status_and_code` produces the
  code the web classifies on; the human message is display-only and never matched
  against. Rewording a message must not be a wire change.
- **Every error is handled or explicitly propagated.** Swallowing one is a bug.
  Where the engine deliberately degrades, it logs at `debug` with the context that
  makes the degradation diagnosable, and the comment says why degrading is right
  there.
- **Retry policy is explicit and bounded:** three attempts, retryable on transport
  errors, 429 and 5xx, backoff `500 ms × 2^attempt` or a `Retry-After` capped at
  30 seconds, with a 10 second ceiling per attempt.
- **Transient and permanent failures are distinguished, and only permanent ones
  are cached.** Caching a blip silences an author for a day.
- **Never log a secret.** mason holds no credentials, which is the strongest form
  of this rule, and it is worth keeping that way.

### Make invalid states unrepresentable

- Model state with enums and match exhaustively: `Brick`, `Mode`, `FeedIntent`,
  `VideoSource`, `AppError`, `HttpError`, `Bucket`. No free strings for state.
- Keep facts that mean different things in different types. `LiveStream` is
  deliberately not a `Brick`: one is true for every viewer and cacheable under a
  single key, the other is true for one viewer. Collapsing them would serve one
  reader's friends to the next.
- On the web side, `Brick` is a discriminated union on `kind`, and optional
  (`?:`) is kept distinct from nullable (`| null`) because mortar's
  `skip_serializing_if` makes that distinction real on the wire.

---

## Limits and bounds

**Every limit is a named constant, named with its units where units apply, and
referenced everywhere it applies.** No magic numbers. Reaching a limit is a
deliberate, commented behaviour, not a silent truncation.

The *existence* of a limit is non-negotiable. Concrete values are documented
where they are owned: [02](02-feed-engine.md) for the engine's thresholds,
[03](03-grout-and-mixer.md) for the scoring windows, [04](04-sources-and-moderation.md)
for fan-out and HTTP policy, [05](05-caching-and-persistence.md) for TTLs and
capacities, [08](08-wall-and-bricks.md) for the client's.

The kinds of thing that must be bounded here, all of which currently are:

- Fan-out concurrency, per source
- Page size, cohort size, wave size, follow-graph pages
- Every deadline and every wait
- Every cache TTL and every cache capacity
- Retry attempts and backoff
- Per-kind and per-author admission caps
- Age windows, half-lives, and skew allowances
- Client-side poll intervals, warm ceilings, and stall counts

A new loop, queue, retry, cache or buffer ships with a named constant for its
bound in the same change.

---

## Version control

This repo is **jj-managed** (jujutsu). Use `jj`, not `git`, even when jj resists.

### Shared core

- **Commits are small and well-described.** One coherent change per commit.
- **Empty descriptions are not accepted.** Describe the *why* before pushing.
- **Conventional Commits** for the subject line: `type(scope): subject`, from the
  standard set (`feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `build`, `ci`,
  `perf`, `style`), plus `release:` for the version-bump commit changesets makes.
- **`main` stays releasable**, and is branch-protected. Work lands via pull
  requests off `main`, never by pushing to it directly.
- **Do not rewrite published history** unless the change is yours and unmerged.
- **Destructive operations need explicit confirmation** even when they look like
  the cleanest path.
- **No em dashes anywhere**, including commit messages. `just guard-dashes`
  enforces it across tracked source, docs and config.

### jujutsu

- **`jj` is the sole front end.** Do not run `git commit`, `git add` or
  `git status` against the working copy; the index and working-copy mismatch is
  exactly what jj removes.
- `jj describe` sets the *why* before a push.
- Feature work happens on named bookmarks (`jj bookmark create feat/x`); pull
  requests go out with `jj git push`.
- Resolve conflicts with `jj resolve`, not by editing plain-text markers.
- `jj abandon`, `jj op restore`, force-fetches and bookmark deletion all need
  explicit confirmation.
- `.jj/` is local and is not committed.

### Changesets

Any user-visible change ships with a changeset (`pnpm changeset`). Bump
semantics for an app with no public API are in
[10-build-release-deploy.md](10-build-release-deploy.md). Infrastructure-only
changes need none; a forgotten one can be added in a follow-up PR.

---

## Rust conventions

### Formatting and linting

- `cargo fmt --all` clean before pushing (`just fmt`, checked by `just fmt-check`).
- `cargo clippy --workspace --all-targets -- -D warnings` clean before pushing
  (`just lint`).
- Both gates run in CI on every pull request.

### Code style

- **Modules over files.** Prefer many small files. `algo/snapshot.rs` is the
  largest module in the engine and is at the point where a further split is worth
  considering.
- **No feed logic in `main.rs` or in a handler.** Both fronts parse, call
  `handle_feed`, and shape the result for their transport.
- **Errors are `Result`**, with `thiserror`-derived enums per concern; `From` and
  explicit `map_err` translate at boundaries. The core never returns a vendor
  error type.
- **No `unsafe`.** There is none in the repo, and any addition needs a `// SAFETY:`
  comment justifying every invariant it relies on.
- **No recursion** in the engine. Iteration with an explicit bound or an explicit
  deadline.
- **Prefer an explicit `match` over a combinator chain** when the control flow is
  non-trivial. Simpler return types win: `()` over `bool` over `Option<T>` over
  `Result<T, E>`.
- **Platform differences live behind the seam.** New code does not add a
  `#[cfg(target_arch = "wasm32")]` outside `platform.rs`, `http.rs` or `pds.rs`
  without a stated reason.
- **Comments explain why,** in full sentences: the bug a line prevents, the
  constraint that forced a shape, the invariant a future reader would miss.

### Naming

- `snake_case` for functions, variables, modules and files; `CamelCase` for types
  and traits.
- **Acronyms in proper case:** `HttpClient`, not `HTTPClient`.
- **Units last in identifiers, descending significance:** `MAX_FUTURE_SKEW_SECS`,
  `GLAZE_MAX_AGE_HOURS`, `PERSIST_INTERVAL_MS`.
- **No abbreviations** beyond ecosystem-standard short names (`ctx`, `cfg`, `id`,
  loop counters) and mason's own vocabulary, which is not abbreviation: brick,
  mortar, grout, kiln, wall, pool, cohort, wave.

### Testing

- `cargo nextest run` is the native runner (`just test`).
- **Test modules are target-gated.** Native tests are
  `#[cfg(all(test, not(target_arch = "wasm32")))]`; browser-only tests are
  `#[cfg(all(test, target_arch = "wasm32"))]` and run under
  `wasm-pack test --headless --chrome` (`just test-wasm`). The wasm lane is not
  optional coverage: it is the only place the timer seam, the gloo-net transport
  and the hand-rolled throttle are exercised for real.
- **Determinism.** Pure functions take `now` as a parameter. Tests use a fixed
  anchor timestamp and a fixed seed; no `Instant::now()` in a test body's
  assertions.
- **Upstreams are mocked, never called.** Every base URL is a `Config` field so
  source tests point at wiremock.
- **Positive and negative space together.** Every "accepts a good input" test is
  paired with the rejection it implies: a future-dated brick sinks, an
  unparseable date is stale, an opted-out author is sampled out, a spent graph
  ends the wall.
- **Test names are sentences.** `a_transiently_failed_author_is_asked_again_by_the_next_wave`
  states the behaviour, and the doc comment above it states the bug it guards.
- **Regressions become tests.** Every fix that has a name in a comment has a test
  with the same name.
- **No flaky tests.** Timing assertions in the wasm lane are lenient by design,
  because browser timers are clamped and fuzzed; they check ordering, not
  precision.

### Documentation

- Every module opens with a `//!` doc comment saying what it is and where it sits
  in the layering.
- Public items in `mortar-core` carry doc comments; the ones that encode a
  decision say why, not just what.
- No bare `// TODO` without an owner and a tracking reference.

---

## TypeScript and Svelte conventions

### Formatting and linting

- `oxfmt src` clean before pushing (`just fmt`, checked by `just fmt-check`).
- `oxlint src` clean, with `correctness` at error and `suspicious` and `perf` at
  warn, plus the `typescript`, `import` and `unicorn` plugins.
- `knip` clean: no unused files, exports or dependencies.
- All three run in CI (`just lint`).

### Code style

- **No `any`.** Use `unknown` plus narrowing, or a typed parse. A cast needs a
  comment justifying it.
- **`strict` is on.** The typecheck is `tsc --noEmit`, not `svelte-check`:
  `svelte-check` crashes on TypeScript 7 (its programmatic API stabilises in
  TS 7.1, around October 2026). Do not swap it back yet.
- **Runes only.** `vite.config.ts` forces `runes: true` for everything outside
  `node_modules`. No legacy reactivity, no stores.
- **Wire types are mirrored once**, in `lib/types.ts`, and guarded by
  `lib/contract-check.ts`. No component redefines a brick shape.
- **One module owns the network.** `/api/feed` appears only in `lib/api.ts` and
  in the service worker.
- **Effects declare their cleanup.** Every `$effect` that adds a listener, an
  observer or a timer returns the teardown. Every async continuation that can
  outlive its trigger rechecks a generation counter or a cancellation flag.
- **`untrack` where an effect mutates state it also reads**, with a comment
  saying which loop it prevents.
- **Comments explain why.** The web half of this repo carries as much rationale
  as the Rust half, and it is expected to.

### Naming

- `camelCase` for functions and variables, `PascalCase` for types and components,
  `SCREAMING_SNAKE_CASE` for module-level constants.
- **Units last in identifiers:** `WARM_CEILING_MS`, `POLL_MS`,
  `PERSIST_INTERVAL_MS`, `PREFETCH_MARGIN`.
- Components are `PascalCase.svelte`; rune modules are `<name>.svelte.ts`.
- Card components are named for the rendered brick (`PostCard`, `BlogCard`,
  `VideoCard`, `GlazeCard`). The Rust model says brick, the web says card, and
  that is deliberate rather than drift.

### Testing

- `vitest run` for unit tests (`just test`), riding the app's own vite config so
  `.svelte.ts` rune modules compile as they do in the build.
- `playwright test` for the service-worker smoke against the real static build
  (`just test-e2e`), chromium only.
- **Determinism.** No wall-clock assertions; `FeedState` tests drive a stubbed
  `fetchFeed`.
- **Positive and negative space.** Every state-machine happy path is paired with
  the failure it implies: a superseded generation, a preview error, a sealed
  wall.

### Accessibility

Accessibility rules are code rules here, not review notes. See
[09-design-system.md](09-design-system.md) for the conformance target. In code:

- **No autoplay, ever.** `just guard-autoplay` fails on any `autoplay` in
  `web/src` and on any `.play(` outside `VideoPlayer.svelte`.
- **Every animation has a reduced-motion alternative**, either in `app.css` or
  behind a `motion-safe:` variant at the call site.
- **Every control is at least 44px** (`min-h-11`), keyboard-operable, and has a
  visible focus state from the one shared focus rule.
- **Custom widgets replace native ones only when a native one cannot do the job**,
  and then they carry the full keyboard contract the native one would have.

---

## Repository hygiene

- **`.specs/` is the canonical home for specs and decisions.** `README.md`,
  `PRODUCT.md` and `AGENTS.md` stay as the short-form entry points; anything
  structural belongs here.
- **`AGENTS.md` is the operational cheat sheet**, symlinked as `CLAUDE.md`. It is
  short on purpose; this page is the long form.
- **Generated code is not checked in.** `web/src/lib/mortar-wasm/pkg/` is
  gitignored and rebuilt by `just wasm`; CI builds it before the web checks, and
  the deploy zips it in through `sourceInclude`.
- **The wire fixture is checked in** (`tests/fixtures/contract.json`) and is
  regenerated deliberately with `UPDATE_FIXTURE=1`, never by hand.
- **Lockfiles are tracked and asserted fresh.** Every workflow that builds runs
  `cargo metadata --locked` first; web installs use `--frozen-lockfile`.
- **Secrets and domains never live in the repo.** CI injects them from
  per-environment GitHub secrets.
- **`just clean` exists because the cargo target directory reaches about 3 GB.**

---

## Guidelines for AI agents

These are not different rules. They are emphasis on the places agents slip.

1. **The pervasive style applies to you too.** Defensive validation and explicit
   named limits are not optional on a small change.
2. **Run `just wasm` after any Rust change to the engine**, or the browser is
   still running the old one. `just dev` and `just build` do it for you.
3. **Use `jj`, not `git`.** Even when jj resists, and even when a git command
   looks like the shorter path.
4. **No em dashes.** Anywhere: UI copy, code comments, commit messages, specs.
   `just guard-dashes` will catch it, but do not make it have to.
5. **Never add an autoplay attribute or a `.play()` call.** There is exactly one
   sanctioned player and it is click-gated.
6. **Stay inside the layering.** The most common slip is reaching for a source
   module from `algo/`, or adding a `#[cfg(target_arch)]` outside the platform
   seam. Go through `sources::fetch`.
7. **Take the lock in `snapshot.rs` or not at all.** Do not add a mutex
   acquisition to `fill.rs` or `cohort.rs`.
8. **Classify failures.** Before caching a failed fetch, decide whether it is
   transient or permanent, and say which in a comment.
9. **New bounds are named constants** in the same change, with units in the name.
10. **Do not invent wire fields.** Change `model.rs`, regenerate the fixture with
    `UPDATE_FIXTURE=1`, then update `types.ts` until `tsc` passes.
11. **No backwards-compat shims.** There is no published API. If a type changes,
    change every caller.
12. **Comments say why.** A comment that paraphrases the line above it is noise
    and will be removed.
13. **Run the tests before claiming complete.** "Compiles" is not "works". Report
    the actual output, including failures.
14. **Add a changeset for anything a visitor could notice.**
15. **Do not run destructive version-control operations without explicit
    confirmation.**

---

## Definition of done

A change is done when:

- The behaviour is exercised by a test at the right tier, and any regression it
  fixes has a test named for it.
- Negative-space tests cover every new validation or rejection path.
- Every new bound is a named constant with its units in the name.
- Every non-obvious line carries a comment saying *why*.
- `just fmt-check`, `just lint`, `just test`, `just guard-autoplay` and
  `just guard-dashes` all pass locally.
- If a Rust change touches the engine, `just wasm` has been run and the web app
  still typechecks against it.
- If the wire changed: `model.rs`, the regenerated `contract.json`, `types.ts`
  and this spec set all agree, and both `cargo test` and `tsc` pass.
- If the browser-only paths changed, `just test-wasm` passes.
- A changeset exists for any user-visible change.
- The commit description states the *why*, and the pull request describes what
  changed at the architecture level.

---

## Assumptions and open questions

**Assumptions**

- Contributors have `just`, the pinned Rust toolchain with the wasm32 target,
  `wasm-pack`, Node 22 and pnpm available locally.
- A Chromium is available for the wasm and Playwright lanes; `wasm-pack` fetches
  a matching chromedriver if none is found.
- CI is the enforcement point for every gate, since no local hook exists.

**Decisions**

- *Tiger Style, without the assertion-density rule.* **Defensive boundaries,
  named limits, errors as data; assertions where they earn their place.** mason's
  engine is mostly pure functions over untrusted upstream data, so the correctness
  net that pays here is exhaustive types plus a heavy unit suite with an injected
  clock, not runtime invariant checks in every function. The parts of Tiger Style
  that address untrusted input and unbounded work are applied in full.
- *Rust first, TypeScript second.* **The engine is the product.** The wall is a
  renderer for what mortar decides, and the wire contract runs from Rust outward.
- *`jj` as the sole front end.* **Never git.** Running both against one working
  copy reintroduces the index mismatch jj exists to remove.
- *oxlint and oxfmt over ESLint and Prettier.* **One fast toolchain family.** The
  lint and format lanes are fast enough to run on every change without thinking
  about it.
- *`tsc --noEmit` rather than `svelte-check`.* **`svelte-check` crashes on TS 7.**
  The programmatic API stabilises in TS 7.1; until then the typecheck covers
  `.ts` fully and `.svelte` through generated types.
- *A pinned wire fixture instead of a runtime schema validator.* **Two static
  checks, no runtime cost.** A Valibot or Zod layer would validate the same data
  mortar just produced, on a channel mason owns both ends of; the fixture catches
  drift at build time on both sides instead.
- *Guards are greps in CI.* **Autoplay and em dashes.** Both are rules a reviewer
  will eventually miss, and both are cheap to check mechanically.
- *Determinism as a testing rule.* **`now` is a parameter, seeds are fixed.** It
  is what makes the scoring and mixing tests exact rather than approximate.
- *Generated wasm is not checked in.* **Rebuilt by `just wasm`, zipped for
  deploy.** Checking in a binary artefact that changes on every engine edit would
  make every diff unreadable.

**Open questions**

- *No pre-push hook exists.* Every gate named here runs in CI and in `just`
  recipes, and nothing runs them automatically before a push. Open: is a jj-side
  hook worth adding, given how fast the lint and format lanes are?
- *Conventional Commits are followed but not enforced.* There is no commitlint
  and no CI check on the subject line. Open: enforce, or leave it to review?
- *`tsconfig.json` sets `strict` but not `noUncheckedIndexedAccess` or
  `exactOptionalPropertyTypes`.* Both would catch real classes of bug in the
  layout code, which indexes arrays freely. Open: turn them on and pay the
  migration, or leave them?
- *`api.ts` casts the response with `as FeedResponse`.* It is guarded by the
  contract fixture at build time, but a mortar that misbehaved at runtime would
  not be caught. Open: is a narrow runtime shape check on `items` worth it?
- *No `clippy.toml`.* Clippy runs at default plus `-D warnings`, without the
  pedantic-adjacent lints Tiger Style usually asks for. Open: adopt a pedantic
  set and annotate the opt-outs?
- *No property tests.* The mixer and the score are the obvious candidates
  (determinism, monotonicity, ratio convergence) and are currently covered by
  example-based tests only. Open: is `proptest` worth the dependency for those two
  modules?
- *`web/README.md` is still the `sv` scaffold default.* It describes creating a
  SvelteKit project rather than mason's web app.
