# Done Certificate · Task 13: FeedTarget and the feed wall

**Task:** [13-feed_target_and_feed_wall.md](13-feed_target_and_feed_wall.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 13. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This task
> fixes the `feed_page` argument order for the whole plan; task 22 appends to it.

## Definition

DONE(Task 13) is every obligation O1 to O8 below holding, O1b included, each backed by the evidence
it names.

## Premises

- **P1 · Goal.** `GET /api/feed?feed=<at-uri>` answers a page in the feed's own order, through the
  axum route, the wasm export and the service worker, with no snapshot, pool, cohort or mixer.
- **P2 · Obligations.** Done iff O1, O1b and O2 to O8 all hold; O8 is the Reviewable item.
- **P3 · Invariants.** Must not break: the graph wall's whole request path, the demo short-circuit,
  the wall-owner gate (`an_opted_out_owner_seals_their_wall` at `feed.rs:226`), or the service
  worker's existing `?actor=` handling.

## Obligations

- **O1 · Target selection is a parse, in mortar-core, not a fallback in each front.**
  - *Claim:* `FeedTarget::from_query(actor, feed)` exists in `mortar-core`'s `feed.rs` and holds the
    whole rule: `feed` wins when both parameters are present, neither present is a `bad_request`
    whose message names both, and an unparseable `feed` is a `bad_request` that never falls back to
    laying somebody's graph. Both fronts call it.
  - *Evidence to collect:* run the mortar-core unit tests for those three cases, the negative-space
    one included. Read `handle_feed`'s branch and confirm the `Feed` arm returns an error rather than
    falling through to the `Actor` path. Run
    `grep -n 'BadRequest' server/crates/mortar-server/src server/crates/mortar-wasm/src` and expect
    no construction of the missing-parameter error in either.
  - *Checks:* resolve where the rule can be tested from. `grep -rn 'cfg(test)' server/crates/mortar-server/src`
    returns nothing, so a copy of the rule in the axum route is untestable in place, and
    `tests/contract.rs` is a mortar-core integration test that can reach neither front crate: a rule
    written in a front is a rule with no lane. Then resolve which parse runs first. If the mode or
    intent parse ran before the target parse and swallowed the error, the fallback would be silent.
    Trace one malformed request end to end.
  - *Status:* SATISFIED. `FeedTarget::from_query` is `feed.rs:69` to `:80`, one `if let` per
    parameter with `feed` read first and `Err(AppError::BadRequest(NO_TARGET))` as the only fall
    through. Four unit tests in the new plain `#[cfg(test)] mod target_tests` pass:
    `feed_wins_when_both_parameters_are_present`, `one_parameter_alone_names_its_own_wall`,
    `neither_parameter_present_is_a_bad_request_naming_both` (asserts `(400, "bad_request")` and
    that the message contains both tokens), `kind_is_the_query_parameter_it_came_from`. The
    unparseable half is its own case,
    `feed_wall_tests::an_unparseable_feed_reference_is_a_bad_request_and_asks_nobody`: five
    malformed references each a 400, then `from_query(Some("did:plc:viewer"), Some("nonsense"))`
    driven through `handle_feed` for a 400 rather than that actor's wall, then
    `assert!(upstream_paths(&server).await.is_empty())`. `handle_feed:130` to `:135` returns out
    of the `Feed` arm, so the error propagates by `?` from `feed_uri` and cannot fall through to
    the `Actor` path. `grep -n 'BadRequest' server/crates/mortar-server/src
    server/crates/mortar-wasm/src` exits 1 with no output; `grep -rn 'cfg(test)'
    server/crates/mortar-server/src` likewise. Parse order: both fronts call
    `FeedTarget::from_query` before `Mode::from_query` and `FeedIntent::from_query`
    (`routes/feed.rs:50`, `mortar-wasm/src/lib.rs:105`), and neither of those two can error, so
    nothing swallows the target error. End to end against a native mortar:
    `/api/feed?feed=nonsense&actor=demo` answered `400 {"error":"bad_request","message":"missing
    required parameter: feed"}`, not the demo wall.

- **O1b · The parse exposes the wire token task 14 pins.**
  - *Claim:* `FeedTarget::kind()` returns `"actor"` or `"feed"` and is `pub`, so `tests/contract.rs`
    can bind each token once and assert it against a real parse result.
  - *Evidence to collect:* read the method. Confirm it is `pub` and that the returned strings are the
    only place mortar-core names the two tokens.
  - *Checks:* without this, task 14's `query.target` keys are retyped literals and the const-bound
    mechanism at `tests/contract.rs:347` degrades to the one-sided rename it exists to prevent. This
    obligation is task 14's precondition, discharged here.
  - *Status:* SATISFIED. `pub fn kind(&self) -> &'static str` at `feed.rs:85` to `:90` returns
    `"actor"` for the `Actor` arm and `"feed"` for the `Feed` arm, and
    `target_tests::kind_is_the_query_parameter_it_came_from` pins both. It is the only place
    mortar-core's own query vocabulary is spelled as a token in non-test code: a grep for both
    literals over `src/` returns otherwise only `UNPARSEABLE_FEED` (a `bad_request` payload that
    deliberately names the same parameter), upstream AppView query names and JSON body keys
    (`getAuthorFeed?actor=`, `{"feed": [...]}`), and test fixtures. `tests/contract.rs:239` still
    carries `BadRequest("actor")`, which is task 14's to move.

- **O2 · A feed wall touches nothing in `algo/` except the cursor, and the gate is intact.**
  - *Claim:* no `ensure_snapshot`, no `get_or_build`, no mixer runs on a feed wall; `resolve_did`
    carries no owner gate; `resolve_and_gate` still calls it and
    `an_opted_out_owner_seals_their_wall` passes unchanged.
  - *Evidence to collect:* run the wiremock feed-wall test that mocks **only** `getFeed` and
    `getProfile`; a snapshot build would fail on the unmocked `getFollows`. Run
    `cargo nextest run -p mortar-core an_opted_out_owner_seals_their_wall`, expect green with no
    edit to its body.
  - *Checks:* resolve `resolve_did` versus `resolve_and_gate` at each call site. The feed path must
    call the former; calling the latter would apply a wall-owner gate to a feed generator's creator,
    which is not the same person and not the same question.
  - *Status:* SATISFIED. `feed_wall` is `feed.rs:218` to `:288` and its whole body is
    `feed_uri`, `cursor::decode`, `fetch::feed_page_cached`, `Brick::is_image_post` and
    `cursor::encode`; the words snapshot, pool, cohort and mixer appear in it only inside
    comments. `snapshot::ensure_snapshot` and `snapshot::get_or_build` are reached only at
    `:182` and `:191`, both on the `Actor` path, and the `Feed` arm returns at `:132`, before
    `resolve_and_gate` at `:167`. Behaviourally,
    `feed_wall_tests::a_feed_wall_lays_the_generators_own_order` asserts the exact upstream
    path list `vec!["/xrpc/app.bsky.feed.getFeed"]`, and every test in the module points all
    three `Config` bases at the mock, so a `getFollows` (or anything else) would fail rather
    than escape. Call sites resolved: `resolve_and_gate` is called once, at `:167`, the graph
    path; `resolve_did` is called at `:304` (the feed path, inside `feed_uri`) and at `:376`
    (inside `resolve_and_gate`). `resolve_did` (`:335` to `:362`) contains no call to `gate`.
    `cargo nextest run -p mortar-core an_opted_out_owner_seals_their_wall` passes, and the diff
    shows its body untouched apart from `"did:plc:owner"` becoming
    `FeedTarget::Actor("did:plc:owner")`. The positive side is proved too:
    `a_sealed_creator_does_not_seal_their_feed` lays a feed whose creator's profile carries
    `!no-unauthenticated`.

- **O3 · The feed wall's behaviour is asserted, including the glaze non-truncation.**
  - *Claim:* upstream order preserved, a repost dropped, an opted-out author's post dropped, the
    wall ending when `getFeed` returns no cursor, a preview reporting `warming: false` and echoing
    the incoming cursor, and glaze laying only image posts **without** truncating to `PAGE_SIZE`.
  - *Evidence to collect:* run the six named wiremock tests. For the glaze case, confirm the test
    feeds more than `PAGE_SIZE` image posts and asserts all of them are laid; a test with fewer than
    `PAGE_SIZE` survivors would pass under a truncating implementation too.
  - *Checks:* trace the truncate call site. The change spec's ASCII flow puts it on the `Mode::Wall`
    arm alone, and the paragraph below it says laying every glaze survivor is a correctness
    requirement rather than an optimisation, because there is no pool to hold a remainder in and the
    cursor belongs to the call that fetched it. Confirm the code truncates on the `Mode::Wall` path
    only.
  - *Status:* SATISFIED. Eight `feed_wall_tests` pass in `just check`'s 146.
    `a_feed_wall_lays_the_generators_own_order` lays three survivors oldest-first and
    least-liked-first (the order grout inverts), drops a repost and an author carrying
    `!no-unauthenticated`, decodes the next cursor to `Cursor::Feed { feed: "page2" }` and
    asserts `warming.is_none()`. `a_feed_wall_ends_when_the_generator_does` pages to a second
    page with a different brick and no cursor.
    `a_feed_wall_preview_is_already_settled_and_echoes_its_cursor` asserts `Some(false)` and a
    byte-identical cursor. `glaze_over_a_feed_lays_every_image_post_rather_than_a_page_of_them`
    feeds 30 image posts interleaved with 30 text posts and asserts `items.len() == 30`,
    `items.len() > PAGE_SIZE` (30 > 24, so a truncating implementation fails it), all
    `is_image_post`, and that the first laid brick is `image-0`; its mock matches `limit=100`,
    so a `PAGE_SIZE` ask would find no mock at all. Truncation resolved to its call site:
    `.take(PAGE_SIZE)` appears once, on the `Mode::Wall` arm at `feed.rs:248`; the `Mode::Glaze`
    arm at `:254` is `filter(is_image_post).cloned().collect()` with no bound. Two further
    cases the DoD did not name are present and pass:
    `a_graph_cursor_on_a_feed_wall_lays_from_the_head` (the mock matches the absence of a cursor
    parameter) and `a_feed_link_naming_nobody_is_a_feed_that_is_not_there`.

- **O4 · The `bad_request` message exists once per language, reads honestly, and the pinned Rust copy follows.**
  - *Claim:* the message lives in `FeedTarget::from_query`'s `Err` arm and in
    `web/src/service-worker.ts:254`'s hardcoded string, they match character for character,
    `error.rs`'s canonical `variants()` instance plus both pinned envelope arrays (`:90`, `:106`)
    carry the same wording, and the wording is chosen to read correctly for **both** callers once
    task 14 rewords `BadRequest`'s Display at `error.rs:5`.
  - *Evidence to collect:* read `feed.rs`'s error construction, `service-worker.ts` around
    `:248`-`:258`, and `error.rs:75`-`:110`; compare the strings. Run
    `cargo nextest run -p mortar-core error` and expect green.
  - *Checks:* nothing in the repo compares the Rust and TypeScript copies, so that half is a read,
    not a test. Confirm `tests/contract.rs`'s `errors()` is **unchanged** here: editing it without a
    fixture regeneration makes `cargo test` red, and this task is not one of the plan's three
    regenerations. Task 14 changes it inside its own regeneration, so the fixture pins the older
    wording for exactly one task and the PR says so. The Display itself is task 14's for the same
    reason, so read the payload strings this task picks against the wording task 14 will put in front
    of them: `BadRequest` carries a `&'static str`, so "missing required parameter: {0}" cannot be
    made honest for a `?feed=` that was present and malformed, and a payload chosen to read well only
    under the old prefix leaves task 14 rewording twice.
  - *Status:* SATISFIED. One Rust place: `const NO_TARGET: &str = "actor or feed"` at
    `feed.rs:38`, used only in `from_query`'s `Err` arm at `:79`. One TypeScript place:
    `service-worker.ts:259`, `"missing required parameter: actor or feed"`, which is
    character for character what `BadRequest`'s Display (`error.rs:5`, `"missing required
    parameter: {0}"`) produces for that payload. `error.rs`'s canonical instance and both
    pinned envelope strings moved together inside one tuple of `pinned_wire()` (`:96` to
    `:103`), and `variants()` is derived from that table, so the three cannot drift.
    `cargo nextest run -p mortar-core error` passes all three envelope tests. The scope edge
    holds: `tests/contract.rs:239` still reads `AppError::BadRequest("actor")` and
    `tests/fixtures/contract.json` still pins `"missing required parameter: actor"`; neither
    file appears in `jj diff --stat`, and `wire_contract_matches_the_committed_fixture` is
    green, so the one-task window is exactly the window the plan described and no fourth
    regeneration happened. Honesty under task 14's reword: the two payloads are `"actor or
    feed"` (a request naming no wall) and `"feed"` (a `?feed=` that was present and would not
    parse). Both name a parameter rather than asserting an absence, so a Display of the shape
    `"bad request: {0}"` reads correctly for both and task 14 rewords the prefix once rather
    than the payloads again. The second payload does read oddly under today's prefix
    ("missing required parameter: feed" for a parameter that was present); that is the
    transitional cost the task file explicitly accepts, and it is display only, since
    classification reads the `bad_request` code.

- **O5 · The two offline rejections are asserted in Playwright.**
  - *Claim:* `/api/feed?feed=nonsense` answers 400 `bad_request` and `/api/feed` with neither
    parameter answers 400, both with no network.
  - *Evidence to collect:* run `just test-e2e` and confirm both cases pass. Confirm they run offline
    because a malformed reference is rejected before any upstream call.
  - *Status:* SATISFIED. `just test-e2e` run three times (baseline, and again after the
    mutation work below was reverted): `a request naming no wall answers 400 without touching
    the network` passes both times, asserting `status === 400` and `body.error ===
    "bad_request"` for `/api/feed` and for `/api/feed?feed=nonsense`. Neither can reach the
    network: the first returns from the worker's own `!actor && !feed` guard at
    `service-worker.ts:251` before `feed_page` is called, and the second is rejected inside
    `feed_uri` by `feedref::parse` before any URL is built, which
    `an_unparseable_feed_reference_is_a_bad_request_and_asks_nobody` proves in Rust by
    asserting the mock server received zero requests.

- **O6 · The positional order is asserted behaviourally, not read.**
  - *Claim:* a Playwright case drives `/api/feed?actor=demo&mode=glaze&intent=preview&cursor=<a
    cursor taken from a prior glaze page>` and asserts three independent effects: every item is
    `kind === 'post'`, `warming === false`, and the returned `cursor` is byte-identical to the one
    sent.
  - *Evidence to collect:* run `just test-e2e` and confirm the case passes. Read its body and confirm
    all three assertions are present and that the cursor is taken from a live first response rather
    than hardcoded.
  - *Checks:* resolve what each assertion pins. Glaze filtering happens in `demo_page`
    (`feed.rs:197`) and only under `Mode::Glaze`, so it pins the `mode` slot; `warming: Some(false)`
    is set only on the demo preview branch (`feed.rs:60`), so it pins the `intent` slot; the demo
    preview re-encodes the **incoming** offset (`feed.rs:64`), so a byte-identical cursor pins the
    `cursor` slot. Drop any one assertion and the corresponding transposition goes unseen. Confirm
    the case is offline: `demo` never leaves the wasm.
  - *Status:* SATISFIED, and mutation tested rather than read. `the service worker binds every
    positional slot` (`web/tests/service-worker-smoke.test.ts:90`) passes in `just test-e2e`.
    It takes the cursor from a live first response (`wall.body.cursor`, guarded by a
    `typeof !== "string"` throw) rather than hardcoding one, and carries all three assertions:
    `kinds.filter((kind) => kind !== "post")` equals `[]`, `warming` is `false`, and
    `preview.body.cursor` is `cursor`. Each assertion pins the slot the certificate names:
    glaze filtering happens only under `Mode::Glaze` in `demo_page` (`feed.rs:417` to `:420`),
    `warming = Some(false)` is set only on the demo preview branch (`feed.rs:153`), and that
    branch re-encodes the incoming offset (`feed.rs:157`). All four adjacent transpositions
    were applied by hand at `service-worker.ts:267`, each rebuilt through `just test-e2e`, and
    each was caught: `(cursor, mode)` fails at line 109, `(mode, intent)` fails at line 109,
    `(feed, cursor)` fails at line 104 on the second fetch's status, `(actor, feed)` fails all
    three cases on the first fetch. The file was restored from a byte copy afterwards
    (`shasum` identical, `jj diff --stat` unchanged) and the suite re-run green, 3 passed.
    Offline: the case drives only `actor=demo`, which the wasm answers from compiled-in
    fixtures.

- **O7 · Meets the repo definition of done, including the explicit wasm rebuild.**
  - *Claim:* the gates are green and the service worker was typechecked against a freshly generated
    `mortar_wasm.d.ts`.
  - *Evidence to collect:* run `just wasm && cd web && pnpm check:ci` explicitly, then `just check`
    and `cd server && cargo nextest run`.
  - *Checks:* first confirm task 00 landed, because without it this evidence is empty: run
    `cd web && pnpm exec tsc -p tsconfig.worker.json --listFiles | grep service-worker` and expect a
    hit. `web/.svelte-kit/tsconfig.json` excludes `../src/service-worker.ts`, so the app project has
    never contained the call site and a green `pnpm check:ci` on `tsconfig.json` alone says nothing
    about it, fresh `pkg/` or stale. Then read `service-worker.ts:260` and confirm the positional
    order matches `feed_page(actor, feed, cursor, mode, intent)` exactly. The reading is a
    convenience; O6 is the evidence, because a transposition of two `Option<String>` parameters
    compiles.
  - *Status:* SATISFIED. Task 00 landed: `cd web && pnpm exec tsc -p tsconfig.worker.json
    --listFiles | grep service-worker` returns exactly one hit,
    `web/src/service-worker.ts`, and the same listing carries
    `web/src/lib/mortar-wasm/pkg/mortar_wasm.d.ts`, so the call site is compiled against the
    generated declarations. `just wasm && cd web && pnpm check:ci` run explicitly, green (both
    `tsconfig.json` and `tsconfig.worker.json`). `just check` green end to end, exit 0:
    guard-dashes, guard-autoplay, guard-toolchain, fmt-check, guard-wasm (`cargo check
    --target wasm32-unknown-unknown --all-targets`, which is what compiles the ungated
    `target_tests` module for wasm32), oxlint, knip, `clippy -D warnings`, 146 Rust tests, both
    tsc projects, 45 vitest. `service-worker.ts:267` reads `feed_page(actor, feed, cursor,
    mode, intent)` and the regenerated `mortar_wasm.d.ts:37` declares `feed_page(actor?, feed?,
    cursor?, mode?, intent?)`. The repo definition of done's changeset bullet is not owed here:
    task 19 owns "add a minor changeset describing the new surface", and this task ships no
    surface a visitor can reach. New bounds are named constants with why comments
    (`GLAZE_FEED_LIMIT`, `PAGE_SIZE_LIMIT`, `NO_TARGET`, `UNPARSEABLE_FEED`), and the new
    wiremock module is gated `#[cfg(all(test, not(target_arch = "wasm32")))]`.

- **O8 · Reviewable: a real feed lays over the wire.**
  - *Claim:* `just dev-server`, then
    `curl 'localhost:8787/api/feed?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot'`
    returns a page in the feed's own order.
  - *Evidence to collect:* the command output, with the item order compared against the same feed
    read directly from the public AppView.
  - *Status:* SATISFIED. The network was available. A native mortar was run as `PORT=8799 cargo
    run -p mortar-server` rather than through `just dev-server`, because an unrelated node
    process of the user's holds 8787 and the vite half of that recipe is not needed for a curl;
    the route under test is the same one. `curl
    'localhost:8799/api/feed?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot'`
    answered HTTP 200 with 19 bricks, kinds `post` and `video` only, no `warming` key, and a
    cursor that decodes to the feed shape. The same feed read straight from
    `public.api.bsky.app` with `limit=24` returned 24 items; mason's 19 ids are an exact
    ordered subsequence of those 24, with 5 dropped (reposts and moderated posts), so the order
    is the generator's own. Paging with that cursor returned 20 further bricks with zero
    overlap and a cursor of its own. Also exercised: `mode=glaze` on the same feed laid 48
    image posts, twice `PAGE_SIZE`, so the non-truncation rule holds over the wire;
    `intent=preview` reported `warming: false`; the bsky.app link spelling of the same feed
    answered 200 with 19 bricks, proving the resolution hop; `/api/feed` answered 400 `missing
    required parameter: actor or feed`; `?feed=nonsense&actor=demo` answered 400; an unknown
    rkey answered 404 `feed_not_found`; and `?actor=` on a real handle still answered 200 with
    posts, blogs and videos.

## Regression check

- `feed.rs:56` demo short-circuit. Trace: `?actor=demo` still returns the fixture wall with the same
  cursor behaviour : **PRESERVED**. The short-circuit stayed on the `Actor` arm (`feed.rs:142`),
  reached only after the `Feed` arm returns. `the_demo_wall_pages_and_previews` passes with only
  its arguments adapted, the Playwright demo smoke passes, and a live
  `/api/feed?actor=demo` answered 200.
- `feed.rs:74` graph path. Trace: `?actor=<handle>` still builds a snapshot and pages it; the five
  in-crate `handle_feed` tests pass : **PRESERVED**. All in-crate `handle_feed` tests pass
  unchanged apart from the `FeedTarget::Actor(...)` argument. `resolve_and_gate` was restructured
  into `resolve_did` plus a profiles-cache read plus `gate`, and every path keeps its old outcome
  and its old network-call count: a cold handle's one `getProfile` now populates the profiles
  cache inside `resolve_did`, so the read that follows is a cache hit rather than a second round
  trip, and the fail directions are unchanged (resolution closed, opt-out open). A live
  `?actor=` on a real handle answered 200 with posts, blogs and videos.
- `web/src/service-worker.ts` `serveFeed`. Trace: an `?actor=` request with no `feed` still reaches
  `feed_page` and returns 200 : **PRESERVED**. The guard is now `!actor && !feed`, so an
  actor-only request passes it; `feed` arrives as `undefined` and `FeedTarget::from_query` takes
  the `Actor` arm. The shell cache and the fetch handler are untouched by the diff.
- `web/tests/service-worker-smoke.test.ts`. Trace: still green : **PRESERVED**. All three cases
  pass, including the pre-existing demo round trip, which was refactored onto the shared
  `underServiceWorker` and `apiFeed` helpers without changing what it asserts.
- Task 00's worker tsc project. Trace: `cd web && pnpm check:ci` still runs both projects and both
  are green against the freshly built `pkg/` : **PRESERVED**. Run explicitly after `just wasm`,
  green, and `--listFiles` confirms the worker project still contains `src/service-worker.ts` and
  the generated `mortar_wasm.d.ts`.

## Residue

- Under glaze a feed wall can hand the client a page several times `PAGE_SIZE`. Whether the client's
  `#replace` and dedupe path is comfortable at that size is proven at task 15, not here.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1, O1b and O2 to O8 are all SATISFIED with collected evidence rather than reading:
the ten new mortar-core cases and the 146-test suite pass, the four adjacent transpositions of
the positional `feed_page` call were each applied and each caught by the Playwright case (which
was then restored byte for byte), `just check`, `just wasm && pnpm check:ci` and `just test-e2e`
are green, and the Reviewable curl laid a live `whats-hot` page in an order that is an exact
ordered subsequence of the same feed read straight from the AppView; all five named regression
traces are PRESERVED.

Three notes that are not obligations. First, a feed-wall preview at the head hands back
`cursor: null` rather than a cursor, and a graph cursor handed to a feed-wall preview is dropped
rather than echoed; the change spec's flow line says "the INCOMING cursor", and for the only
case a client can produce (a feed cursor) the two are byte identical, which the test asserts.
The freeze that follows a null cursor re-reads the head out of the 60 second `feed_pages` entry
and commits the same page, so `FeedState` is unaffected. Second, `Mode::Wall`'s `.take(PAGE_SIZE)`
would discard a remainder if a generator ignored `limit`, but that truncation is exactly what
the change spec's flow prescribes and the mapper only ever drops, so it cannot fire against a
conforming generator. Third, `?feed=` with an empty value produces a different message text on
the two fronts (the worker's guard treats an empty string as absent, mortar does not); the code
and status are identical on both and the message is display only.
