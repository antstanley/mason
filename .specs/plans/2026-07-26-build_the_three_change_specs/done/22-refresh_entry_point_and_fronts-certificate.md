# Done Certificate · Task 22: refresh entry point and both fronts

**Task:** [22-refresh_entry_point_and_fronts.md](22-refresh_entry_point_and_fronts.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 22. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task owns
> the cross-spec obligation the feed spec assigned to whoever merges second.

## Definition

DONE(Task 22) is every obligation O1 to O7 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** `?refresh=1` re-reads the two fast caches on a graph wall, bypasses `feed_pages` on
  a feed wall, and is ignored mid-scroll and on the demo wall.
- **P2 · Obligations.** Done iff O1 to O7 all hold; O7 is the Reviewable item.
- **P3 · Invariants.** Must not break the graph wall, the demo wall, the feed wall, the preview and
  freeze intents, or the service worker's existing five-parameter behaviour (task 13's shape) for
  requests that carry no `refresh`.

## Obligations

- **O1 · The parser is named, single, and safe in the default direction.**
  - *Claim:* `pub fn refresh_from_query(raw: Option<&str>) -> bool` lives in `feed.rs` beside
    `FeedIntent::from_query`; exactly `"1"` is true; `None`, `"true"`, `"yes"`, `"0"`, `""` and
    `"1 "` are all false; and neither front carries a second copy of the rule.
  - *Evidence to collect:* run the named unit test covering the six negative cases and the one
    positive. Run
    `grep -rn 'refresh' server/crates/mortar-server/src server/crates/mortar-wasm/src` and confirm
    each front calls `refresh_from_query` rather than comparing inline.
  - *Checks:* resolve `refresh_from_query`'s visibility. `tests/contract.rs` is an integration test
    and can only see `pub` items; task 23 asserts this function from there, so a private function
    would compile now and block task 23.
  - *Status:* SATISFIED. `feed.rs:132` defines `pub fn refresh_from_query(raw: Option<&str>) -> bool`
    immediately after `FeedIntent::from_query`, body `raw == Some("1")`. Ran
    `cargo nextest run -E 'test(only_the_literal_token_asks_for_a_refresh)'` → PASS; the case asserts
    `Some("1")` true and loops over exactly `None`, `"true"`, `"yes"`, `"0"`, `""`, `"1 "` asserting
    false. It sits in a plain `#[cfg(test)] mod refresh_query_tests`, so it also compiles for wasm32,
    where one of the two callers lives (`just guard-wasm` builds `--all-targets` and is green).
    The grep over both fronts returns only `use mortar_core::feed::{… refresh_from_query}` plus one
    call each (`routes/feed.rs:59`, `mortar-wasm/src/lib.rs:116`) and doc comments: no second copy of
    the token rule. Visibility is proved by construction rather than inspection: `pub mod feed` in
    `lib.rs`, and mortar-server is a *separate crate* that imports the item and compiles, which is
    exactly the reach `tests/contract.rs` needs.

- **O2 · A cursorless refresh genuinely re-reads.**
  - *Claim:* laying a wall, then laying a second with `refresh = true` against the same wiremock
    server, calls `getAuthorFeed` twice for the same author inside the 5 minute `author_feed` TTL.
  - *Evidence to collect:* run the named test. Confirm the assertion is on the mock's hit count for
    a specific author, and that the two calls are inside the TTL (no clock advance between them).
  - *Checks:* trace the flag from `handle_feed` through `ensure_snapshot` or `get_or_build` into
    `Snapshot::new` and out through `fill::fill` to `fan_out_authors`. The fill is spawned detached,
    so the flag can only reach it on the snapshot.
  - *Status:* SATISFIED. `feed::refresh_tests::a_cursorless_refresh_re_reads_the_author_feed_inside_its_ttl`
    → PASS, and PASS on five consecutive runs of the module (no flake). The assertion is
    `author_feed_reads(&server)`, a count of received requests whose path is
    `/xrpc/app.bsky.feed.getAuthorFeed`, against a one-follow graph so the count is that one author's.
    Nothing advances a clock: the only wait is a real `tokio::time::sleep(2ms)`, present because
    `fresh_seed` is wall-clock-derived in whole milliseconds and two walls inside one millisecond
    would be one snapshot. Both walls therefore land far inside the five minute TTL. That the entry
    really was still warm is not inferred: I mutated both graph forwarding sites
    (`ensure_snapshot`/`get_or_build`) to pass a literal `false`, re-ran, and the count fell to 1 with
    the second fill served from the cache; restoring the file (sha1 `6a26a97…`) restored PASS.
    Flag trace: `handle_feed` (`feed.rs:227` preview, `:239` committing) → `snapshot::get_or_build`
    → `ensure_snapshot` → `Snapshot::new(…, refresh)` → detached `fill::fill`, which reads
    `snapshot.refresh` (`fill.rs:67`, `:104`) → `fan_out_authors(…, refresh, …)` →
    `fetch::author_feed_cached(state, did, refresh)`, whose `if !refresh && let Some(cached)` skips
    the entry.

- **O3 · The three ignore cases hold.**
  - *Claim:* page 2 with `refresh = true` issues no further `getAuthorFeed` calls; the demo wall
    ignores the flag; and the flag is forced false whenever the cursor decodes.
  - *Evidence to collect:* run the two named tests (cursored, demo). Read `handle_feed` and confirm
    the force-to-false happens at both `decoded.is_some()` and the `demo` short-circuit.
  - *Status:* SATISFIED. `feed::refresh_tests::a_cursored_refresh_is_ignored` and
    `feed::refresh_tests::the_demo_wall_ignores_a_refresh` both PASS. The cursored case holds the
    author-feed count at `DEEP_FOLLOWS` (10) across page two *and* across a second cursor naming a
    seed no snapshot exists under, which is the case that really does build and fill; a second,
    differently tagged mount sits behind the first, so a wrongly honoured flag fails on the tag it
    laid rather than merely 404ing. Mutating `let refresh = refresh && decoded.is_none();` to
    `let refresh = refresh;` fails it (20 reads against an expected 10), so the guard is load-bearing
    and not incidentally true. The demo case asserts `server.received_requests()` is *empty* for a
    refreshed demo wall and that it lays bricks byte-identical to the unrefreshed one.
    One divergence from the evidence line as written, recorded rather than waved through: the
    force-to-false is present at `decoded.is_some()` (`feed.rs:158`) but **not** written at the demo
    short-circuit. The demo arm returns before `refresh` is read by anything, so an explicit force
    there would be unreachable code; the observable claim ("the demo wall ignores the flag") is what
    the test pins, and it holds. Mechanism differs, behaviour is the one the claim names.

- **O4 · The feed-wall bypass exists and is asserted.**
  - *Claim:* on a feed wall, `refresh` skips the `feed_pages` cache read and inserts as usual: a
    second `getFeed` for the same `(uri, cursor)` under `refresh = true` reaches the mock, where
    without the flag it would not.
  - *Evidence to collect:* run the named test. Confirm the same test without the flag asserts
    `expect(1)`, so the pair proves the bypass rather than just the call.
  - *Checks:* resolve which cache the flag bypasses on the feed path. It must be `feed_pages`, not
    `author_feed` or `image_feed`, which a feed wall never touches.
  - *Status:* SATISFIED. `feed::feed_wall_tests::a_refresh_over_a_feed_wall_steps_over_the_cached_page`
    → PASS. The unflagged half is genuinely asserted, not assumed: request 2 is an unflagged repeat of
    request 1 and asserts the count is *still* 1 ("without the flag the second read of a page never
    reaches the AppView, which is what makes the refresh below provable"). Request 3 under
    `refresh = true` asserts 2 and lays the newer rkey; request 4, unflagged, asserts the count holds
    at 2 and still lays the newer rkey, so the fresh answer was inserted as usual and the freeze
    behind a refreshed preview stays one network read. Mutating `feed_wall`'s forward to a literal
    `false` fails the test at request 3 (1 against an expected 2). Cache resolution: the flag reaches
    only `fetch::feed_page_cached`, which guards `state.caches.feed_pages`, not `author_feed` and
    `image_feed` are untouched on this path.

- **O5 · The last positional slot is asserted behaviourally.**
  - *Claim:* task 13's Playwright case `the service worker binds every positional slot` now sends
    `&refresh=1` on its second fetch, and its three assertions (all items `kind === 'post'`,
    `warming === false`, the returned cursor byte-identical to the one sent) still hold.
  - *Evidence to collect:* run `just test-e2e` and confirm the case passes with `refresh=1` present.
    Read the diff of `web/tests/` and confirm the flag was added to the existing case rather than a
    new case being written that drops one of the three assertions.
  - *Checks:* resolve what the `warming === false` assertion pins here. Swap `intent` and `refresh`
    and `FeedIntent::from_query(Some("1"))` returns `Normal`, so `feed.rs:60` never runs, `warming`
    is absent and the cursor is a next-page cursor rather than the echo. Both slots are
    `Option<String>` and adjacent, so no compiler in the repo can see this. Note that the demo wall
    forces `refresh` false, so this case proves the **binding** and not the re-read; O2 and O4 own
    the re-read.
  - *Status:* SATISFIED. The diff adds `&refresh=1` to the *existing* case's second fetch URL and
    changes nothing else inside the test body: the three `expect`s are byte-identical to task 13's.
    Green, 3/3. **Methodology note, because it changed the answer:** `web/playwright.config.ts`
    hardcodes port 4173 with `reuseExistingServer: !CI`, and another workspace on this machine was
    holding 4173 with its own `vite preview`, so my first two `just test-e2e` runs silently drove
    *that* build and are not evidence about this diff. Re-run against a gate-local config on port
    4199 whose `webServer` has `cwd` set to this workspace and `reuseExistingServer: false`:
    3/3 green with `refresh=1` present. The transposition check was then run properly: swapping the
    last two arguments at the `service-worker.ts` call site to
    `feed_page(actor, feed, cursor, mode, refresh, intent)`, rebuilding wasm and the site (the
    transposed order verified in the emitted `build/service-worker.js`) and re-running makes the case
    **FAIL** at `expect(preview.body.warming).toBe(false)` with `Received: undefined`. Restoring the
    file (sha1 `eb3802a…`) and rebuilding returns it to 3/3. The last slot is bound by a real lane.

- **O6 · Meets the repo definition of done, including the explicit wasm rebuild.**
  - *Claim:* the gates are green and the service worker was typechecked against a freshly generated
    `mortar_wasm.d.ts`.
  - *Evidence to collect:* run `cd server && cargo nextest run`, `just guard-wasm`, then
    `just wasm && cd web && pnpm check:ci`, then `just check`.
  - *Checks:* first confirm task 00's project still covers the file:
    `cd web && pnpm exec tsc -p tsconfig.worker.json --listFiles | grep service-worker` must hit.
    `web/.svelte-kit/tsconfig.json` excludes `../src/service-worker.ts` from the app project, so a
    green `tsconfig.json` run alone is not evidence about this call site. Then read
    `service-worker.ts:260` and confirm the `refresh` argument sits in the last positional slot,
    after `intent`, matching `feed_page(actor, feed, cursor, mode, intent, refresh)`. Confirm the
    parameter is `Option<String>` rather than `bool`, so the generated d.ts parameter stays optional.
  - *Status:* SATISFIED. `cargo nextest run` → 154 passed, 0 skipped. `just guard-wasm` → exit 0
    (`cargo check -p mortar-core -p mortar-wasm --target wasm32-unknown-unknown --all-targets`, which
    is what covers the new test modules' gating). `just wasm` → the generated
    `web/src/lib/mortar-wasm/pkg/mortar_wasm.d.ts:44` reads
    `feed_page(actor?, feed?, cursor?, mode?, intent?, refresh?): Promise<string>`: six slots, the
    sixth optional, because `mortar-wasm/src/lib.rs:107` declares `refresh: Option<String>` and not
    `bool`. `cd web && pnpm check:ci` → exit 0 (svelte-kit sync, `tsc -p tsconfig.json`,
    `tsc -p tsconfig.worker.json`). `tsc -p tsconfig.worker.json --listFiles` includes both
    `src/service-worker.ts` and the freshly generated `pkg/mortar_wasm.d.ts`, so the six-argument call
    really is inside a tsc program. The call site is
    `feed_page(actor, feed, cursor, mode, intent, refresh)`, with `refresh` last, after `intent`.
    `just check` → exit 0 (guard-dashes, guard-autoplay, guard-toolchain, fmt-check, guard-wasm,
    oxfmt, oxlint with only the four pre-existing warnings, knip, `clippy -D warnings`, nextest
    154/154, check:ci, vitest 45/45). No `contract.json` movement and no `.specs/`, `api.ts` or
    `feed.svelte.ts` in the diff, so the scope edges hold. No changeset, which matches the change
    spec's own implementation-notes ordering (`pnpm changeset` is its step 11, after the control) and
    all fourteen preceding task commits on the bookmark.
    One repo-DoD bullet could not be discharged here and is recorded as an evidence gap rather than a
    failure: "if the browser-only paths changed, `just test-wasm` passes" exits 1 with
    `Error: http status: 404` out of `wasm-bindgen-test-runner`. I ran it on the pristine `main`
    checkout as a control and it fails identically, so the chromedriver runner is broken on this
    machine and it is not caused by this diff. It is not part of `just check` and not named in this
    task's definition of done; the browser lane that *does* cover the changed worker is O5's, and it
    is green.

- **O7 · Reviewable: two refreshes, two fan-outs.**
  - *Claim:* `just dev-server`, then two `curl`s at `?refresh=1` a second apart, with the mortar log
    showing two fan-outs.
  - *Evidence to collect:* the two commands and the log lines.
  - *Status:* SATISFIED. Exercised for real against the live AppView (reachability confirmed first:
    `getProfile` 200, `plc.directory` 200). Port 8787 is held on this machine by an unrelated app, so
    `just dev-server` was run as `PORT=8797`; vite came up on 5173 unchanged. Two
    `curl 'http://localhost:8797/api/feed?actor=pfrazee.com&refresh=1'` one second apart, both 200
    with 24 bricks and different cursors, and the log shows two fan-outs:
    `snapshot wall-1e56c773d97d66c2: 99 follows, cohort of 96` at 07:59:22 and
    `snapshot wall-a9672c9b1b5a6305: 636 follows, cohort of 100` at 07:59:29.
    Because a fan-out line alone cannot tell a re-read from a cache hit, I also ran a second mortar
    with `APPVIEW_BASE` pointed at a counting pass-through in front of `public.api.bsky.app`, and
    counted `getAuthorFeed` per request: a cold wall 96, a second **unrefreshed** wall for the same
    actor 38 (the rest of its cohort served out of the five minute cache), and the same wall under
    `?refresh=1` **100**. That is the flag surviving the whole route through the axum front. All
    background servers were torn down afterwards; no listeners remain on 8797/8798/8899/5173.

## Regression check

- `feed.rs`'s five in-crate `handle_feed` test call sites. Trace: all updated for the new argument
  and still passing with the same meaning : **PRESERVED**. There are 33 `handle_feed(` sites in
  `feed.rs` after tasks 13 and 22; every one takes the sixth argument as a literal `false`, and
  `cargo nextest run` is 154/154 with no test's meaning changed.
- `mortar-server/src/routes/feed.rs`: a request with no `refresh` parameter. Trace: still lays the
  same wall : **PRESERVED**. `params.refresh` is `None` → `refresh_from_query(None)` is `false` →
  `handle_feed` behaves exactly as before. Confirmed live: the unrefreshed second wall in O7 served
  62 of its 100 authors out of the warm `author_feed` cache.
- `web/src/service-worker.ts`: a request with no `refresh` parameter. Trace: `just test-e2e` green,
  including `web/tests/service-worker-smoke.test.ts` : **PRESERVED**. `searchParams.get("refresh")`
  is `null` → `undefined` → `None` → `false`. The two cases that send no `refresh` (the demo
  round-trip and the two offline 400s) are green.
- Task 21's invariant, checked because this task is what first makes `snapshot.refresh` ever true:
  `grep -n 'snapshot.refresh' src/algo/fill.rs` still shows it only inside `fill::fill` (`:67`,
  `:104`, plus the doc comment at `:28`); `extend` still passes two literal `false`s, and
  `a_wave_of_a_refreshed_wall_never_re_reads_an_author_feed` is green : **PRESERVED**.

## Residue

- The `follows` cache stays warm on a refresh, so a newly followed account cannot appear until its
  hour expires. Deliberate, per the plan's "what this plan does not do".
- `feed_wall` receives `refresh` **before** the cursorless rule is applied, so a refresh over a feed
  wall is honoured on any page. This follows the change spec's cross-spec section ("bypass the
  `feed_pages` entry for the page being asked for", "a two-line addition on the feed path"), and it
  is documented at the function with its reasoning: the cursorless rule exists because re-reading a
  hundred rate-limited author feeds to serve page nine changes nothing, and a feed page is one
  AppView call whichever page it is. It sits askew of the unqualified wire-contract row the same spec
  proposes ("Honoured only when `cursor` is absent"). No failure mode: one extra AppView call, a
  correct fresher page, and the shipped client (tasks 24 and 25) only ever sends the flag cursorless.
  Named here for task 27, the spec merge, which owns reconciling the two sentences. The one line is
  the `feed_wall(state, reference, cursor, mode, intent, refresh)` call at the top of `handle_feed`.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O7 are all SATISFIED on evidence I collected myself: the five named tests run and, for
the two claims the gate asked to break, mutated-and-failed then restored; the Playwright slot case
green with `refresh=1` and red when `intent` and `refresh` are transposed; `just check` exit 0; and
the live two-curl review backed by a counting proxy showing 100 upstream author-feed reads under the
flag against 38 without it, with all four regression traces PRESERVED, one evidence gap
(`just test-wasm` is broken identically on pristine `main`) and one residue note handing the
feed-wall cursor question to the spec merge.
