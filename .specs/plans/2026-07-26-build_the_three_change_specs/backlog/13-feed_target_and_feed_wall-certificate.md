# Done Certificate · Task 13: FeedTarget and the feed wall

**Task:** [13-feed_target_and_feed_wall.md](13-feed_target_and_feed_wall.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O1b · The parse exposes the wire token task 14 pins.**
  - *Claim:* `FeedTarget::kind()` returns `"actor"` or `"feed"` and is `pub`, so `tests/contract.rs`
    can bind each token once and assert it against a real parse result.
  - *Evidence to collect:* read the method. Confirm it is `pub` and that the returned strings are the
    only place mortar-core names the two tokens.
  - *Checks:* without this, task 14's `query.target` keys are retyped literals and the const-bound
    mechanism at `tests/contract.rs:347` degrades to the one-sided rename it exists to prevent. This
    obligation is task 14's precondition, discharged here.
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O5 · The two offline rejections are asserted in Playwright.**
  - *Claim:* `/api/feed?feed=nonsense` answers 400 `bad_request` and `/api/feed` with neither
    parameter answers 400, both with no network.
  - *Evidence to collect:* run `just test-e2e` and confirm both cases pass. Confirm they run offline
    because a malformed reference is rejected before any upstream call.
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O8 · Reviewable: a real feed lays over the wire.**
  - *Claim:* `just dev-server`, then
    `curl 'localhost:8787/api/feed?feed=at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot'`
    returns a page in the feed's own order.
  - *Evidence to collect:* the command output, with the item order compared against the same feed
    read directly from the public AppView.
  - *Status:* unverified

## Regression check

- `feed.rs:56` demo short-circuit. Trace: `?actor=demo` still returns the fixture wall with the same
  cursor behaviour : (PRESERVED / REGRESSION)
- `feed.rs:74` graph path. Trace: `?actor=<handle>` still builds a snapshot and pages it; the five
  in-crate `handle_feed` tests pass : (PRESERVED / REGRESSION)
- `web/src/service-worker.ts` `serveFeed`. Trace: an `?actor=` request with no `feed` still reaches
  `feed_page` and returns 200 : (PRESERVED / REGRESSION)
- `web/tests/service-worker-smoke.test.ts`. Trace: still green : (PRESERVED / REGRESSION)
- Task 00's worker tsc project. Trace: `cd web && pnpm check:ci` still runs both projects and both
  are green against the freshly built `pkg/` : (PRESERVED / REGRESSION)

## Residue

- Under glaze a feed wall can hand the client a page several times `PAGE_SIZE`. Whether the client's
  `#replace` and dedupe path is comfortable at that size is proven at task 15, not here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
