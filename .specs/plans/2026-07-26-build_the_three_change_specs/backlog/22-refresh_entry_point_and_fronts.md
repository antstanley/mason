# Task 22 · refresh entry point and both fronts

**Plan:** [plan.md](../plan.md) · **Certificate:** [22-refresh_entry_point_and_fronts-certificate.md](22-refresh_entry_point_and_fronts-certificate.md)

**Implements:** [`changes/2026-07-26-refresh_the_wall.md`](../../../changes/2026-07-26-refresh_the_wall.md) §Proposed changes → `02-feed-engine.md` → Entry point and → Refresh, and → `06-wire-contract.md` → The endpoint; implementation notes 4 and 5. Also owns the cross-spec obligation [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) assigns to whoever merges second: `refresh` over a feed wall bypasses the `feed_pages` entry.
**Depends on:** 12, 21
**Produces:** `?refresh=1` re-reads the two fast caches on a graph wall, bypasses `feed_pages` on a feed wall, and is ignored mid-scroll and on the demo wall.
**Pointers:** `feed.rs:34` (`FeedIntent::from_query`, the parser pattern to copy), `:44` (`handle_feed`, already carrying `FeedTarget` from task 13), `:51` (the decode), `:56` (the demo short-circuit), `:86` and `:95` (the two forwarding sites). Five in-crate test call sites of `handle_feed`: `feed.rs:243`, `:327`, `:414`, `:436`, `:513`. `mortar-server/src/routes/feed.rs:15` (**`FeedParams`**), `:45` (beside `Mode::from_query`). `mortar-wasm/src/lib.rs:88` (`feed_page`, left five-parameter-shaped by task 13 as `(actor, feed, cursor, mode, intent)`; this task appends the sixth). `web/src/service-worker.ts:247` (the intent read), `:260` (the positional call).

## Steps

- [ ] Add `pub fn refresh_from_query(raw: Option<&str>) -> bool` to `feed.rs` beside `FeedIntent::from_query`. Exactly the token `"1"` is true. A **named function**, not an inline comparison, so task 23 has something to assert and neither front carries a second copy of the rule.
- [ ] Give `handle_feed` a `refresh: bool` argument, forced false when the cursor decodes and when the actor is `demo`, and otherwise forwarded to `ensure_snapshot` on the preview path and to `get_or_build`.
- [ ] On the feed path, make `refresh` skip the `feed_pages` read and insert as usual: two lines, and the obligation the feed spec assigned here.
- [ ] Add `pub refresh: Option<String>` to `FeedParams` and a **sixth** `refresh: Option<String>` to `feed_page`, after the five task 13 fixed, both parsed with `refresh_from_query`. `Option<String>` rather than `bool`, so the generated d.ts parameter stays optional and matches how `mode` and `intent` already cross that boundary.
- [ ] Read `url.searchParams.get("refresh") ?? undefined` in the service worker and pass it in the last positional slot.
- [ ] Update all five in-crate `handle_feed` test call sites.
- [ ] Extend task 13's Playwright case `the service worker binds every positional slot` with
      `&refresh=1` on the second fetch, and re-assert its three existing conditions unchanged. This
      is what covers the **last** slot: transposed with `intent`, `FeedIntent::from_query(Some("1"))`
      is `Normal`, so `warming` stops being `false` and the returned cursor stops echoing the one
      sent. The demo wall ignores `refresh` itself (task 22 forces it false there), so this case
      proves the binding, not the re-read; the re-read is the wiremock work above.

## Definition of done

- [ ] `refresh_from_query`'s negative space is covered: `None`, `"true"`, `"yes"`, `"0"`, `""` and `"1 "` all read as no refresh, and only `"1"` is true.
- [ ] A native test proves a cursorless refresh re-reads: lay a wall, then lay a second with `refresh = true` against the same wiremock server, and assert `getAuthorFeed` was called twice for the same author **inside** the 5 minute `author_feed` TTL.
- [ ] A native test proves a cursored refresh is ignored: page 2 with `refresh = true` issues no further `getAuthorFeed` calls. A test proves the demo wall ignores it.
- [ ] A native test proves the feed-wall bypass: a second `getFeed` for the same `(uri, cursor)` under `refresh = true` reaches the mock, where without the flag it would not.
- [ ] Playwright: the extended positional-slot case is green with `refresh=1` present and its three assertions unchanged, which is the only lane that can see `refresh` and `intent` swapped. Both are `Option<String>` and adjacent, so the tsc project from task 00 cannot.
- [ ] Meets the repo definition of done, plus the explicit `just wasm && cd web && pnpm check:ci` run. That run only means something because task 00 put `src/service-worker.ts` into a tsc project of its own; `web/.svelte-kit/tsconfig.json` excludes the file from the app project, so before task 00 the call site was typechecked against neither a fresh nor a stale `pkg/`. `just check` still does not run `just wasm`, so the rebuild stays a hand step here.
- [ ] Reviewable: `cd server && cargo nextest run` green, then `just dev-server` and two `curl`s at `?refresh=1` a second apart, with the mortar log showing two fan-outs.
