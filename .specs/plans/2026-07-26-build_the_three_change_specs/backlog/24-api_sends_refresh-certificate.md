# Done Certificate · Task 24: api sends refresh

**Task:** [24-api_sends_refresh.md](24-api_sends_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 24. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 24) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The client never sends a flag mortar would ignore, and the cursorless rule is
  visible to anybody reading the network tab.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not change any existing `fetchFeed` call's built URL, must not change
  `warmFeed`, and must not lengthen any recorded call tuple in `api.test.ts` or `feed.test.ts` until
  task 25.

## Obligations

- **O1 · A cursorless refreshed call produces the exact expected query string.**
  - *Claim:* the URL is asserted whole, not by substring, for a cursorless refreshed call.
  - *Evidence to collect:* run `cd web && pnpm test` and read the new assertion in `api.test.ts`;
    confirm it compares the full string, following the existing style at `api.test.ts:51`.
  - *Checks:* resolve the parameter order in the built `URLSearchParams`. The exact-URL assertion is
    order-sensitive, so confirm `refresh` is appended after `intent` and that the assertion's
    expected string matches the insertion order in `api.ts`.
  - *Status:* unverified

- **O2 · A refreshed call with a cursor omits the flag entirely.**
  - *Claim:* `refresh` does not appear in the URL when a cursor is passed, asserted by its own case.
  - *Evidence to collect:* run the named case. Confirm the assertion is absence of the key, not a
    different value.
  - *Checks:* this is the negative-space half and the reason the rule lives in `api.ts` as well as in
    mortar. Trace the condition: it must be `refresh === true && !cursor`, not `refresh === true`
    alone.
  - *Status:* unverified

- **O3 · The literal is pinned to the wire vocabulary.**
  - *Claim:* the token is written `("1" satisfies FeedRefresh)`, so a rename in mortar and the
    regenerated fixture fails typechecking here.
  - *Evidence to collect:* read `api.ts`. Confirm the `satisfies` form, matching the existing uses at
    `feed.svelte.ts:200` and `service-worker.ts:253`.
  - *Status:* unverified

- **O4 · `warmFeed` and every existing assertion are untouched.**
  - *Claim:* `warmFeed` is unchanged, and no existing assertion in `api.test.ts` or `feed.test.ts`
    changed in this task.
  - *Evidence to collect:* read the diff. Confirm `warmFeed`'s body is identical and that the only
    test changes are additions. Warming is a cache-filling request and must never trigger a
    hundred-author burst.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci` and `just check`.
  - *Status:* unverified

- **O6 · Reviewable: the two assertions state when mason sends the flag.**
  - *Claim:* a reviewer runs `cd web && pnpm test` and reads the two new exact-URL assertions, which
    together are the whole statement of when the flag is sent.
  - *Evidence to collect:* the test run plus the read.
  - *Status:* unverified

## Regression check

- `feed.svelte.ts:97`, `:131` and `:160` all call `fetchFeed`. Trace: none passes the new argument
  yet, so all three still build the same URLs and `feed.test.ts`'s recorded tuples keep their length :
  (PRESERVED / REGRESSION)
- `LandingWall.svelte:16` and `HandleForm.svelte:21`. Trace: both still work with the shorter
  argument list : (PRESERVED / REGRESSION)

## Residue

- Nothing calls `fetchFeed` with `refresh: true` until task 25. Until then the parameter is dead in
  production, which is intentional and is what keeps this task reviewable on its own.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
