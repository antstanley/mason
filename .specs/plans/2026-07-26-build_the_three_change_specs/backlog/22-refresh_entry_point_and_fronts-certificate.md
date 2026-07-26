# Done Certificate · Task 22: refresh entry point and both fronts

**Task:** [22-refresh_entry_point_and_fronts.md](22-refresh_entry_point_and_fronts.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · A cursorless refresh genuinely re-reads.**
  - *Claim:* laying a wall, then laying a second with `refresh = true` against the same wiremock
    server, calls `getAuthorFeed` twice for the same author inside the 5 minute `author_feed` TTL.
  - *Evidence to collect:* run the named test. Confirm the assertion is on the mock's hit count for
    a specific author, and that the two calls are inside the TTL (no clock advance between them).
  - *Checks:* trace the flag from `handle_feed` through `ensure_snapshot` or `get_or_build` into
    `Snapshot::new` and out through `fill::fill` to `fan_out_authors`. The fill is spawned detached,
    so the flag can only reach it on the snapshot.
  - *Status:* unverified

- **O3 · The three ignore cases hold.**
  - *Claim:* page 2 with `refresh = true` issues no further `getAuthorFeed` calls; the demo wall
    ignores the flag; and the flag is forced false whenever the cursor decodes.
  - *Evidence to collect:* run the two named tests (cursored, demo). Read `handle_feed` and confirm
    the force-to-false happens at both `decoded.is_some()` and the `demo` short-circuit.
  - *Status:* unverified

- **O4 · The feed-wall bypass exists and is asserted.**
  - *Claim:* on a feed wall, `refresh` skips the `feed_pages` cache read and inserts as usual: a
    second `getFeed` for the same `(uri, cursor)` under `refresh = true` reaches the mock, where
    without the flag it would not.
  - *Evidence to collect:* run the named test. Confirm the same test without the flag asserts
    `expect(1)`, so the pair proves the bypass rather than just the call.
  - *Checks:* resolve which cache the flag bypasses on the feed path. It must be `feed_pages`, not
    `author_feed` or `image_feed`, which a feed wall never touches.
  - *Status:* unverified

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
  - *Status:* unverified

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
  - *Status:* unverified

- **O7 · Reviewable: two refreshes, two fan-outs.**
  - *Claim:* `just dev-server`, then two `curl`s at `?refresh=1` a second apart, with the mortar log
    showing two fan-outs.
  - *Evidence to collect:* the two commands and the log lines.
  - *Status:* unverified

## Regression check

- `feed.rs`'s five in-crate `handle_feed` test call sites. Trace: all updated for the new argument
  and still passing with the same meaning : (PRESERVED / REGRESSION)
- `mortar-server/src/routes/feed.rs`: a request with no `refresh` parameter. Trace: still lays the
  same wall : (PRESERVED / REGRESSION)
- `web/src/service-worker.ts`: a request with no `refresh` parameter. Trace: `just test-e2e` green,
  including `web/tests/service-worker-smoke.test.ts` : (PRESERVED / REGRESSION)

## Residue

- The `follows` cache stays warm on a refresh, so a newly followed account cannot appear until its
  hour expires. Deliberate, per the plan's "what this plan does not do".

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
