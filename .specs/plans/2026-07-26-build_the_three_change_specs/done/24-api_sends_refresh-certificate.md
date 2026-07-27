# Done Certificate · Task 24: api sends refresh

**Task:** [24-api_sends_refresh.md](24-api_sends_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-27

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
  - *Collected:* `api.test.ts:87-97`, "asks for a refresh on a cursorless request", calls
    `fetchFeed({ actor: "demo" }, null, "glaze", "preview", true)` and asserts
    `toHaveBeenCalledWith("/api/feed?actor=demo&mode=glaze&intent=preview&refresh=1")`. That is
    `toHaveBeenCalledWith` over the whole single argument, not `toContain` and not a substring, and
    it follows the two pre-existing exact-URL cases in the same describe block.
    `cd web && pnpm vitest run src/lib/api.test.ts --reporter=verbose`: 8 passed, this case named
    among them. `cd web && pnpm test`: 6 files, 72 tests passed.
  - *Check result:* insertion order in `api.ts` is target (`:70-72`), `cursor` (`:73`), `mode`
    (`:74`), `intent` (`:75`), then `refresh` (`:82`). `URLSearchParams` serialises in insertion
    order, so `refresh=1` lands last, exactly where the expected string puts it. Trace on the test's
    own input: `{actor:"demo"}` seeds `actor=demo`, `cursor` is `null` so nothing is written, `mode`
    writes `mode=glaze`, `intent` writes `intent=preview`, `refresh && !cursor` is `true && true` so
    `refresh=1` is appended, and `fetch` is called with
    `/api/feed?actor=demo&mode=glaze&intent=preview&refresh=1`.
  - *Status:* SATISFIED

- **O2 · A refreshed call with a cursor omits the flag entirely.**
  - *Claim:* `refresh` does not appear in the URL when a cursor is passed, asserted by its own case.
  - *Evidence to collect:* run the named case. Confirm the assertion is absence of the key, not a
    different value.
  - *Checks:* this is the negative-space half and the reason the rule lives in `api.ts` as well as in
    mortar. Trace the condition: it must be `refresh === true && !cursor`, not `refresh === true`
    alone.
  - *Collected:* `api.test.ts:99-113`, "drops the refresh flag when a cursor rides along", calls
    `fetchFeed({ actor: "demo" }, "cur1", "glaze", "preview", true)` and asserts the whole URL
    `"/api/feed?actor=demo&cursor=cur1&mode=glaze&intent=preview"`. Because the assertion is the
    entire string, the key is absent rather than merely reordered, and neither `refresh=0` nor an
    empty `refresh=` could pass it. The case passes in the verbose run.
  - *Check result:* `api.ts:82` reads
    `if (refresh && !cursor) params.set("refresh", "1" satisfies FeedRefresh);`. `refresh` is typed
    `boolean | undefined`, so the truthiness test is `refresh === true` for every value the type
    admits, and the second conjunct is the cursorless half. Trace: `cursor` is `"cur1"`, so `:73`
    writes `cursor=cur1` and `!cursor` is `false`, the `set` at `:82` never runs, and `fetch` sees a
    URL with no `refresh` key. The engine rule it mirrors is
    `server/crates/mortar-core/src/feed.rs:167`, `let refresh = refresh && decoded.is_none();`.
  - *Status:* SATISFIED

- **O3 · The literal is pinned to the wire vocabulary.**
  - *Claim:* the token is written `("1" satisfies FeedRefresh)`, so a rename in mortar and the
    regenerated fixture fails typechecking here.
  - *Evidence to collect:* read `api.ts`. Confirm the `satisfies` form, matching the existing uses at
    `feed.svelte.ts:200` and `service-worker.ts:253`.
  - *Collected:* `api.ts:82` writes `"1" satisfies FeedRefresh`, and `api.ts:2` widens the type
    import to `import type { ErrorEnvelope, FeedMode, FeedRefresh, FeedResponse } from "./types"`.
    `FeedRefresh` resolves by step 4 of the resolution sequence to `types.ts:138`,
    `export type FeedRefresh = "1"`, the type task 23 pinned against the fixture's `query.refresh`
    key. The form is not decorative, and this gate proved it bites rather than assuming: with the
    literal temporarily changed to `"2"`, `cd web && pnpm check:ci` failed with
    `src/lib/api.ts(82,53): error TS1360: Type '"2"' does not satisfy the expected type '"1"'`. The
    file was then restored from a byte copy and its sha256 matches the pre-mutation value, so the
    diff under review is untouched by the experiment.
  - *Status:* SATISFIED

- **O4 · `warmFeed` and every existing assertion are untouched.**
  - *Claim:* `warmFeed` is unchanged, and no existing assertion in `api.test.ts` or `feed.test.ts`
    changed in this task.
  - *Evidence to collect:* read the diff. Confirm `warmFeed`'s body is identical and that the only
    test changes are additions. Warming is a cache-filling request and must never trigger a
    hundred-author burst.
  - *Collected:* `jj diff --stat` is two files, `web/src/lib/api.ts` (+12/-1) and
    `web/src/lib/api.test.ts` (+28/-0). `jj diff --git` shows four hunks: `@@ -83,6 +83,34 @@` in the
    test file, purely additive, and `@@ -1,5 +1,5 @@`, `@@ -51,11 +51,14 @@`, `@@ -70,6 +73,13 @@` in
    `api.ts`. The single removed line in the whole diff is the old type import. The string `warmFeed`
    appears nowhere in the diff, so its signature and body are unchanged and warming still fires the
    same one cursorless, refresh-free request. `feed.test.ts` is not in the diff at all, and the two
    pre-existing exact-URL assertions in `api.test.ts` appear only as unmodified context.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd web && pnpm test`, `pnpm check:ci` and `just check`.
  - *Collected:* `cd web && pnpm test`: 6 files, 72 tests passed. `cd web && pnpm check:ci`: green
    across both projects, `tsconfig.json` and `tsconfig.worker.json`. `just check` on the final
    restored tree: exit 0, covering `guard-dashes`, `guard-autoplay`, `guard-toolchain`,
    `fmt-check`, `guard-wasm`, `lint` and `test`, with 154 rust tests passed and 72 vitest passed.
    The only oxlint output is the four pre-existing warnings in `FeedGrid.svelte` and
    `service-worker.ts`, neither file in this diff.
  - *Status:* SATISFIED

- **O6 · Reviewable: the two assertions state when mason sends the flag.**
  - *Claim:* a reviewer runs `cd web && pnpm test` and reads the two new exact-URL assertions, which
    together are the whole statement of when the flag is sent.
  - *Evidence to collect:* the test run plus the read.
  - *Collected:* exercised as written. `cd web && pnpm test` passes, and the verbose run names both
    cases, "asks for a refresh on a cursorless request" and "drops the refresh flag when a cursor
    rides along". Read together the pair is the complete statement: the flag rides only on a
    cursorless request, and its absence with a cursor is asserted by whole-string equality rather
    than left unsaid. The comment at `api.test.ts:100-102` gives the reason in the reader's terms,
    the network tab, and `api.ts:76-81` says the same beside the line it governs.
  - *Status:* SATISFIED

## Regression check

- `feed.svelte.ts:97`, `:131` and `:160` all call `fetchFeed`. Trace: none passes the new argument
  yet, so all three still build the same URLs and `feed.test.ts`'s recorded tuples keep their length :
  PRESERVED. The three calls now sit at `feed.svelte.ts:116` (`"preview"`), `:154` (`"freeze"`) and
  `:184` (three arguments, no intent); the line numbers moved with earlier tasks, the calls did not.
  Each leaves `refresh` `undefined`, so `refresh && !cursor` short-circuits false, no `refresh` key
  is written, and the built URLs are byte-identical to before. `feed.test.ts` is not in the diff and
  its suite is green inside the 72.
- `LandingWall.svelte:16` and `HandleForm.svelte:21`. Trace: both still work with the shorter
  argument list : PRESERVED. `LandingWall.svelte:16` calls `fetchFeed({ actor: 'demo' })` with one
  argument, and the new parameter is trailing and optional, so it stays `undefined` and the URL is
  `/api/feed?actor=demo` as before. `HandleForm.svelte:21` calls `warmFeed`, which the diff does not
  touch. Neither file is typechecked, since tsc cannot parse `.svelte`, so both were read by hand
  rather than trusted to `check:ci`.
- Not named in the certificate, checked anyway: `web/src/service-worker.ts:254` already reads
  `url.searchParams.get("refresh")` and forwards the raw token to `feed_page` at `:271`. The only
  value this task can put on the wire is the `"1"` it already handles, and the file is not in the
  diff : PRESERVED.

## Residue

- Nothing calls `fetchFeed` with `refresh: true` until task 25. Until then the parameter is dead in
  production, which is intentional and is what keeps this task reviewable on its own.
- One deliberate asymmetry with the engine, noted rather than faulted: mortar suppresses the flag
  when the cursor **decodes** (`decoded.is_none()`), while the client suppresses it whenever a
  cursor string is present. A cursor that failed to decode would therefore be honoured by mortar but
  never asked for by mason. The divergence is conservative, and no client path can produce an
  undecodable cursor, because every cursor the client holds came out of a mortar response.
- An empty-string cursor behaves the same on both sides: `api.ts:73` already omits it from the
  query, and `:82` treats it as cursorless, which is exactly what mortar then sees.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED on evidence this gate collected itself, including both new cases
asserting the whole query string (the second by absence of the key), a mutation test that turned the
literal into `"2"` and made `pnpm check:ci` fail with TS1360 before the file was restored to its
original sha256, a diff in which `warmFeed` and `feed.test.ts` never appear and the only deletion is
the widened type import, and `just check` at exit 0; the named regression traces for the three
`feed.svelte.ts` calls, `LandingWall.svelte` and `HandleForm.svelte` are all PRESERVED because the
new parameter is trailing and optional.
