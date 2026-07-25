# mason specs

The canonical design specification for **mason**: an atproto discovery app that
lays a Bluesky follow graph into one masonry wall of posts, blogs and video.

These pages describe **what exists in the current branch**. Aspirational content
lives only in each page's closing `Assumptions and open questions` block, never
in the body. If a page describes something the code does not do, that is a
divergence to fix, in one or the other.

mason ships as one thing at one version, so there is no per-package layer: every
spec page lives in this directory.

Start at [00-overview.md](00-overview.md).

## Spec set

| Page | Topic |
|---|---|
| [00-overview.md](00-overview.md) | Problem, goals, non-goals, system shape, scope |
| [01-domain-model.md](01-domain-model.md) | Bricks, authors, snapshots, cursors; identity and lifecycles |
| [02-feed-engine.md](02-feed-engine.md) | `handle_feed`, snapshot build, the fill, extension waves, paging |
| [03-grout-and-mixer.md](03-grout-and-mixer.md) | The grout score and the weighted-round-robin mixer |
| [04-sources-and-moderation.md](04-sources-and-moderation.md) | The `sources/` seam, each upstream, the label rules, outbound safety |
| [05-caching-and-persistence.md](05-caching-and-persistence.md) | TTL caches, their keys and lifetimes, IndexedDB persistence |
| [06-wire-contract.md](06-wire-contract.md) | `/api/feed`, `FeedResponse`, `ErrorEnvelope`, the drift guard |
| [07-web-client.md](07-web-client.md) | The SPA: routes, reactive state, service-worker lifecycle |
| [08-wall-and-bricks.md](08-wall-and-bricks.md) | Layouts, cards, the player, warming reflow, wall states |
| [09-design-system.md](09-design-system.md) | Tokens, kiln palette, motion, focus, accessibility conformance |
| [10-build-release-deploy.md](10-build-release-deploy.md) | Build modes, `just` recipes, guards, CI, changesets, deploy |
| [architecture-principles.md](architecture-principles.md) | Layering rules, crate graph, the wasm32 constraint, security posture |
| [development-guidelines.md](development-guidelines.md) | Toolchain, coding style, defensive coding, testing, definition of done |
| [canonical-types.schema.json](canonical-types.schema.json) | The wire shapes as JSON Schema (Draft 2020-12) |

## Change specs

A change spec proposes a delta to the canonical pages above. Its body describes
code that does **not** yet exist, which is exactly why it is a separate document:
the canonical pages keep describing the current branch until the change lands.
Lifecycle is `Proposed` → `Accepted` → `Implemented` → `Merged`; a merged spec
moves to [`changes/merged/`](changes/merged/) and is kept as dated history.

| Pending change spec | Status | Proposes |
|---|---|---|
| [2026-07-25-drop_snapshot_from_cursor.md](changes/2026-07-25-drop_snapshot_from_cursor.md) | Proposed | Remove the `snapshot` field the cursor carries and never reads |
| [2026-07-25-tighten_typescript_and_add_a_prepush_gate.md](changes/2026-07-25-tighten_typescript_and_add_a_prepush_gate.md) | Proposed | `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, and a local pre-push gate |

## Related documents

These are not specs, and they are deliberately short. Anything structural belongs
in the pages above.

| Document | What it is |
|---|---|
| [`../README.md`](../README.md) | The repo's front door: what mason is and how to run it |
| [`../PRODUCT.md`](../PRODUCT.md) | Product intent: users, purpose, positioning, brand, accessibility stance |
| [`../AGENTS.md`](../AGENTS.md) | The operational cheat sheet, symlinked as `CLAUDE.md` |
| [`../CHANGELOG.md`](../CHANGELOG.md) | Written by changesets on each release |

## Reading paths

- **New to the codebase:** 00, then 02, then 07.
- **Changing how the wall is composed:** 03, then 02.
- **Adding or changing an upstream:** 04, then 05, then 01.
- **Changing anything on the wire:** 06 first, then the schema, then 01.
- **Working on the client:** 07, then 08, then 09.
- **Shipping:** 10, then development-guidelines.

## Conventions

- Pages are numbered for reading order. The two unnumbered pages are standalone
  references rather than a step in that order.
- Every page opens with a status header and closes with
  `Assumptions and open questions`.
- No em dashes, anywhere. `just guard-dashes` enforces it in the tracked source
  tree.
