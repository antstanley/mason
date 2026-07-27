# 06 - Wire Contract

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

One endpoint, one response shape, one error shape, spoken over two transports.
This page defines them and the drift guard that keeps the Rust and TypeScript
sides in step. The entities on the wire are modelled in
[01-domain-model.md](01-domain-model.md) and formalised in
[`canonical-types.schema.json`](canonical-types.schema.json).

---

## Responsibilities

1. Define `/api/feed`: its query vocabulary, its success shape, its error shape.
2. Keep the same bytes flowing in local mode (a wasm throw through a service
   worker) and server mode (an HTTP response from axum).
3. Fail a build, on either side, when the two representations drift apart.

---

## The endpoint

```
GET /api/feed?(actor=<handle|did>|feed=<at-uri|bsky.app feed url>)
             [&cursor=<opaque>][&mode=glaze][&intent=preview|freeze][&refresh=1]
```

| Parameter | Required | Values | Meaning |
|---|---|---|---|
| `actor` | one of the two | a Bluesky handle, a DID, or the literal `demo` | Whose graph to lay |
| `feed` | one of the two | a feed generator AT-URI, or a `bsky.app/profile/<actor>/feed/<rkey>` URL | Which feed generator to lay |
| `cursor` | no | opaque base64url of `{seed, offset}` or `{feed}` | Where to continue; absent starts a fresh wall |
| `mode` | no | `glaze` | The image wall; it composes with either source |
| `intent` | no | `preview`, `freeze` | The warm-then-commit first screen; absent is a normal committed page |
| `refresh` | no | `1` | Lay a new wall and re-read the fast content caches. On a graph wall, honoured only when `cursor` is absent |

Exactly one of `actor` and `feed` is needed. `feed` wins when both are given,
because the two name different walls and one of them has to. Neither being
present is a 400, as a missing `actor` was before.

Unknown values for `mode` and `intent` still fall back to the default rather
than erroring. `feed` does **not**: it is a structured reference that reaches an
upstream query, and a value that will not parse is a `bad_request` rather than
a silent fallback to somebody's graph.

`refresh` is a single literal token for the same reason `mode` is: anything
other than exactly `1`, including absent, means no refresh. The safe direction
is the default one, so a hand-edited URL cannot make somebody re-fan-out a
hundred authors by accident. `refresh=1` beside a `cursor` is not an error
either: on a graph wall the flag is simply dropped, because a refresh is always
a first page (see [02](02-feed-engine.md)).

Server mode also serves `GET /api/health`.

### Response

```jsonc
{
  "items": [ /* Brick[] */ ],
  "cursor": "eyJzZWVkIjo…",       // null when the wall has no more bricks
  "warming": true                  // ONLY on a preview response; absent otherwise
}
```

Three canonical page shapes are pinned by the fixture: `committed` (mixed items,
a cursor, no `warming`), `preview` (the only shape carrying `warming`), and
`final` (cursor exhausted).

A brick is an internally-tagged union on `kind`:

| `kind` | Type |
|---|---|
| `post` | `PostBrick` |
| `blog` | `BlogBrick` |
| `video` | `VideoBrick`, with `source: "bluesky" \| "streamplace"` |

Field naming is camelCase throughout (`#[serde(rename_all = "camelCase")]`).

**Null versus absent is contract, not accident.** Optional-but-always-present
fields serialise as `null` (`displayName`, `avatar`, `cursor`, `external`,
`aspectRatio`, `poster`, `viewerCount`, `durationMs`, `activity`, `coverImage`,
`description`, `icon`). Three fields are *skipped* when empty and therefore
absent rather than null: `blur` (`skip_serializing_if = "Option::is_none"`),
`captions` (`skip_serializing_if = "Vec::is_empty"`), and `warming`. The TS side
models the first group as `T | null` and the second as `?:`, and the drift guard
checks that distinction in both directions.

### Errors

One envelope, both modes:

```jsonc
{ "error": "actor_not_found", "message": "actor not found: nobody.example.com", "status": 404 }
```

| `error` | HTTP status | Raised when |
|---|---|---|
| `bad_request` | 400 | Neither `actor` nor `feed` names a wall, or `feed` will not parse |
| `actor_not_found` | 404 | The AppView 400s or 404s on the actor |
| `feed_not_found` | 404 | The AppView 400s or 404s on the feed generator |
| `login_required` | 403 | The wall owner set `!no-unauthenticated` |
| `upstream` | 502 | An upstream read failed on a load-bearing path |

`error` is a machine code; the web classifies on it, so codes are wire contract,
not cosmetics. `message` is display-only and is never matched on.

`status` differs by transport, and this is the only difference between the two:

- **Server mode** omits it. The HTTP response line carries the status, and axum's
  `IntoResponse` sets it.
- **Local mode** carries it in-band. `mortar-wasm` throws the envelope as a JSON
  string; there is no HTTP layer at that point, so the service worker needs the
  status to build a `Response` with.

The service worker adds two out-of-band codes of its own for failures that never
reached mortar: `wasm` (a non-envelope throw, 500) and, in `api.ts`, `unknown`
(a non-JSON error body, which in local mode means a request escaped the worker
and hit the static host). This is why `ErrorEnvelope.error` is typed as a plain
`string` on the web side while `MortarErrorCode` names only mortar's own five.

`mortar-wasm` carries one more, and it is deliberately outside `MortarErrorCode`:
if serialising a `FeedResponse` ever fails, the export throws
`{"error": "internal", "message": …, "status": 500}` rather than a panic that
would cross the wasm boundary as an unreadable trap. It is not an `AppError`, it
is not in the fixture, and nothing pins it, because it is unreachable: `serde_json`
cannot fail on a `FeedResponse`, whose every field is an owned, finite value. It
is named here so the five-code table is not read as a promise the serializer
backstop quietly breaks.

---

## The two transports

```
LOCAL MODE
  page ──fetch("/api/feed?…")──▶ service worker
                                    │  feed_page(actor, feed, cursor, mode, intent)
                                    ▼
                                 mortar-wasm
                                    │  Ok  → JSON string  → Response 200
                                    │  Err → throw(JSON envelope with status)
                                    ▼
                                 service worker rebuilds a Response at that status

SERVER MODE
  page ──fetch("https://mortar…/api/feed?…")──▶ axum handler
                                                  │  Ok  → Json(FeedResponse) 200
                                                  └─ Err → (status, Json(envelope))
```

`api.ts` reads both the same way: on a non-ok response, parse the body as an
`ErrorEnvelope` and throw a `FeedError { code, status }`. A body that will not
parse as JSON is not mortar speaking and stays `unknown`.

Server mode's CORS allowlist is explicit, never a wildcard: comma-separated
`MASON_ALLOWED_ORIGINS`, defaulting to the local vite dev origins. Only `GET`,
never with credentials. A shipped server does not wave every origin through an
unauthenticated feed.

---

## The drift guard

`web/src/lib/types.ts` hand-mirrors mortar's serde output. Nothing used to catch
a rename on either side: `tsc` cannot see Rust and `nextest` cannot see
TypeScript. The guard closes that gap with one committed fixture and two checks
against it.

```
server/crates/mortar-core/tests/contract.rs
    pins  ──▶ tests/fixtures/contract.json  ◀──  imports and typechecks
  (cargo test)                                (tsc, via lib/contract-check.ts)
```

- A **Rust-side rename** changes the serialization, so `contract.rs` fails until
  the fixture is regenerated. Regenerating then fails `tsc` until `types.ts`
  follows.
- A **TypeScript-side rename** fails `tsc` against the committed fixture
  directly.

Regenerate after an intentional wire change with:

```sh
UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract
```

### What the fixture covers

| Fixture path | Covers |
|---|---|
| `bricks.{post,blog,video}.{full,bare}` | Every brick kind, with every optional field present and with none |
| `pages.{committed,preview,final}` | The three `FeedResponse` shapes |
| `errors.<code>.{server,wasm}` | Every error code in both envelope forms |
| `query.mode` / `query.intent` / `query.refresh` / `query.target` | The query vocabulary, as object keys (`target` holds `actor` and `feed`) |
| `vocab.videoSource` | `bluesky` / `streamplace`, as object keys |
| `vocab.hiddenLabels` | The five labels of the hidden tier, as object keys, so the feed picker's client-side copy cannot drift from mortar's |

**Vocabulary rides as object keys, not string values.** `tsc` widens an imported
JSON file's string values to `string`, but object keys stay literal, so `keyof`
on the web side sees the exact tokens. That is what lets `contract-check.ts`
assert `Equal<keyof typeof contract.errors, MortarErrorCode>` in both directions.

### What the TypeScript side asserts

- **Structure** with `satisfies` against `Wire<T>`, which is `T` with every
  literal string relaxed to `string`. Field names, optionality and
  null-versus-absent survive; the literals are checked separately.
- **Vocabulary** with a bidirectional `Equal<>` between fixture keys and the TS
  literal unions (`Brick["kind"]`, `MortarErrorCode`, `FeedIntent`, `FeedMode`,
  `FeedRefresh`, `FeedTargetKind`, `VideoBrick["source"]`, `HiddenLabel`).
- **Field sets** with `Equal<keyof full, keyof Interface>`, which is what catches
  a field mortar gained that `types.ts` does not know about, and renames of
  optional fields, in both directions.

The Rust side has a matching forcing chain for a new brick variant: the
`kind_key` match must gain an arm (exhaustiveness), that arm must index a new
slot of `ALL_KINDS` (an out-of-bounds constant index fails the build), and the
key-set assertion then fails until the fixture actually carries canonical
instances under the new key.

### Two tripwires

`contract-check.ts` is imported by nobody at runtime. Two things keep it alive:

- `types.ts` carries `import type {} from "./contract-check"`, which is erased at
  compile time but makes *deleting* the guard a `tsc` error rather than a silent
  loss of coverage.
- `knip.json` lists it as an entry point, so dead-code analysis does not flag it.

Separately, `error.rs` pins the exact envelope JSON strings per variant in a
fixture test, so the strings the service worker parses cannot change unnoticed
even without regenerating the contract fixture.

---

## Implementation layout

```
server/crates/mortar-core/src/model.rs         FeedResponse, Brick and friends
server/crates/mortar-core/src/error.rs         AppError, ErrorEnvelope, pinned strings
server/crates/mortar-core/src/mode.rs          Mode::from_query
server/crates/mortar-core/src/feed.rs          FeedTarget::from_query, FeedIntent::from_query,
                                               refresh_from_query, PAGE_SIZE
server/crates/mortar-core/tests/contract.rs    the fixture generator and pin
server/crates/mortar-core/tests/fixtures/contract.json
server/crates/mortar-server/src/routes/        axum wiring, CORS, IntoResponse
server/crates/mortar-wasm/src/lib.rs           feed_page, throw
web/src/lib/types.ts                           the TS mirror
web/src/lib/contract-check.ts                  the TS half of the guard
web/src/lib/api.ts                             fetchFeed, warmFeed, FeedError
web/src/service-worker.ts                      the local-mode responder
```

---

## Assumptions and open questions

**Assumptions**

- `tsc` widens imported JSON string values to `string` but preserves object keys
  as literals. The whole vocabulary check rests on this.
- A JSON body is always safe to parse defensively on the web side; a static host's
  HTML error document is a realistic response in local mode.

**Decisions**

- *One envelope, status in-band only on wasm.* **The body is identical
  otherwise.** The web then has one shape to parse regardless of mode, and the
  service worker can rebuild the right `Response`.
- *Machine code plus human message.* **Classification never reads `message`.**
  Rewording an error string is a cosmetic change and must not be a wire change.
- *A committed fixture, not codegen.* **Pin the wire, check both sides against
  it.** Generating TS from Rust would need a build step in the web toolchain and
  a generator to trust; a fixture is inert, reviewable in a diff, and fails both
  builds.
- *Vocabulary as object keys.* **The only way `keyof` sees literals.** String
  values widen and the check would pass on any string.
- *Unknown `mode` and `intent` fall back.* **A stray parameter cannot break a
  wall.** These arrive from URLs people share and edit.
- *Explicit CORS allowlist.* **Named origins, never `*`.** Server mode is called
  directly from the SPA's origin; there is no proxy to hide behind.

**Open questions**

- *`warming` on non-preview responses.* It is always absent today. If a future
  client wants warming state on a committed page, that is a wire change; nothing
  needs it yet.
- *Server mode error bodies and `intent`.* `mortar-server` accepts `preview` and
  `freeze` but no test covers those paths against the axum front. Open until
  server mode has a consumer.
