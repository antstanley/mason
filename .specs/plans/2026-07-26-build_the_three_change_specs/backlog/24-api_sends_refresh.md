# Task 24 · api sends refresh

**Plan:** [plan.md](../plan.md) · **Certificate:** [24-api_sends_refresh-certificate.md](24-api_sends_refresh-certificate.md)

**Implements:** [`changes/2026-07-26-refresh_the_wall.md`](../../../changes/2026-07-26-refresh_the_wall.md) implementation note 7, and → `06-wire-contract.md` → The endpoint (the cursorless rule). Targets [`07-web-client.md`](../../../07-web-client.md) §The feed state machine.
**Depends on:** 15, 23
**Produces:** the client never sends a flag mortar would ignore, and the cursorless rule is visible to anybody reading the network tab.
**Pointers:** `api.ts:36` (`fetchFeed`, already taking a `FeedTarget` from task 15), `:45` (the params build), `:66` (`warmFeed`). `api.test.ts:51` (the existing exact-URL assertion, the pattern to follow). `feed.svelte.ts:200` and `service-worker.ts:253` are the existing `satisfies MortarErrorCode` uses this mirrors.

## Steps

- [ ] Add a trailing `refresh?: boolean` parameter to `fetchFeed`, after the target, cursor, mode and intent that task 15 settled.
- [ ] Write `refresh=1` into the URL only when the parameter is true **and** `cursor` is falsy, mirroring `handle_feed`'s cursorless rule.
- [ ] Write the literal as `("1" satisfies FeedRefresh)`, so a token renamed in mortar and in the regenerated fixture fails typechecking here.
- [ ] Leave `warmFeed` unchanged: warming is a cache-filling request and must never trigger a hundred-author burst.
- [ ] Add two `api.test.ts` cases: the exact URL for a cursorless refreshed call, and the same call with a cursor omitting `refresh` entirely.

## Definition of done

- [ ] A cursorless refreshed call produces the exact expected query string, asserted whole rather than by substring.
- [ ] A refreshed call **with** a cursor omits `refresh` from the URL, which is the negative-space half and the reason the rule lives here rather than only in mortar.
- [ ] `warmFeed` is untouched, proven by the diff.
- [ ] No existing assertion in `api.test.ts` or `feed.test.ts` changes in this task: every current caller still passes fewer arguments, so recorded call tuples keep their length until task 25.
- [ ] Meets the repo definition of done (`cd web && pnpm test`, `pnpm check:ci` and `just check` green).
- [ ] Reviewable: `cd web && pnpm test` and read the two new exact-URL assertions; they are the whole statement of when mason sends the flag.
