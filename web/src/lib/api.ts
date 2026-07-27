import { browser } from "$app/environment";
import type { ErrorEnvelope, FeedMode, FeedRefresh, FeedResponse } from "./types";

/** Empty → local mode: same-origin fetch, intercepted by the wasm service
 *  worker. Set → server mode: direct CORS call to that mortar instance.
 *  Injected at build time via vite `define` (defaults to '' when unset). */
const BASE: string = import.meta.env.PUBLIC_MASON_SERVER_URL ?? "";

export const localMode = BASE === "";

/** Interception only applies once the SW CONTROLS this page; `ready`
 *  resolves at activation, which can precede clients.claim() taking effect.
 *  Fetching in that gap goes to the network and 404s on a static host. */
async function swControlsPage(): Promise<void> {
  if (navigator.serviceWorker.controller) return;
  const controlled = new Promise<void>((resolve) =>
    navigator.serviceWorker.addEventListener("controllerchange", () => resolve(), {
      once: true,
    }),
  );
  // hard-reloaded pages stay uncontrolled by design; don't hang forever. A
  // rejected register() also leaves `ready` pending forever, so it must race the
  // timeout too, or every feed request would await it eternally.
  const timeout = new Promise<void>((resolve) => setTimeout(resolve, 2000));
  await Promise.race([navigator.serviceWorker.ready, timeout]);
  if (navigator.serviceWorker.controller) return;
  await Promise.race([controlled, timeout]);
}

/** A feed request's role in the warm-then-commit first screen. "preview" lays a
 *  non-committed first screen the client reflows while the wall warms; "freeze"
 *  commits it and begins paging; omitted is a normal committed page (every page
 *  after the first). Pinned by the contract fixture (see contract-check.ts). */
export type FeedIntent = "preview" | "freeze";

/** Which parameter names the wall a request asks for: whose graph, or which
 *  feed generator. Exactly one of the two is carried, and mortar decides which
 *  wins when both arrive. Mirrors FeedTarget::kind in
 *  server/crates/mortar-core/src/feed.rs; pinned by the contract fixture.
 *
 *  A named union rather than a `keyof` over the target object, because `keyof`
 *  across a union of object types yields the keys they SHARE, which is `never`
 *  for a `{actor} | {feed}` union. Comparing the fixture keys to that would
 *  typecheck without checking anything. */
export type FeedTargetKind = "actor" | "feed";

/** Which wall to lay: somebody's follow graph, or a feed generator. A union of
 *  two one-key objects rather than a `{kind, value}` pair, because the keys ARE
 *  the query parameters mortar reads, so a target can be written to the query
 *  string without a second table mapping kinds to names. Mirrors
 *  `FeedTarget` in server/crates/mortar-core/src/feed.rs. */
export type FeedTarget = { actor: string } | { feed: string };

/** `refresh` last, and a boolean rather than the token: the caller says what it
 *  wants and this function decides whether the wire carries it. */
export async function fetchFeed(
  target: FeedTarget,
  cursor?: string | null,
  mode?: FeedMode,
  intent?: FeedIntent,
  refresh?: boolean,
): Promise<FeedResponse> {
  if (localMode && browser && "serviceWorker" in navigator) {
    await swControlsPage();
  }
  // exactly one of the two is written, spelled out rather than spread, so a
  // widened object carrying both cannot put both on the wire. `feed` is tested
  // first for the same reason mortar prefers it: the two name different walls,
  // and the front and the engine have to pick the same one.
  const params = new URLSearchParams(
    "feed" in target ? { feed: target.feed } : { actor: target.actor },
  );
  if (cursor) params.set("cursor", cursor);
  if (mode) params.set("mode", mode);
  if (intent) params.set("intent", intent);
  // the cursorless half of the condition mirrors handle_feed, which ignores the
  // flag when a cursor decodes. Duplicated on purpose: the engine is already
  // correct without this, but sending a flag mortar would ignore makes the
  // network tab lie about what a mid-scroll refresh asked for. The literal is
  // `satisfies FeedRefresh` so renaming the token in mortar, and in the
  // regenerated contract fixture, fails typechecking here too.
  if (refresh && !cursor) params.set("refresh", "1" satisfies FeedRefresh);
  const res = await fetch(`${BASE}/api/feed?${params}`);
  if (!res.ok) {
    // in both modes the body is mortar's ErrorEnvelope; a non-JSON body (a
    // static host's error doc, say) is not mortar speaking and stays "unknown"
    const body = (await res.json().catch(() => null)) as Partial<ErrorEnvelope> | null;
    throw new FeedError(body?.error ?? "unknown", res.status);
  }
  return (await res.json()) as FeedResponse;
}

/** Warm the local engine before the wall is actually asked for. Ensures the
 *  service worker controls the page, then fires a feed request whose result is
 *  discarded. That moves the cold-start tax off the critical path: the wasm
 *  compiles and the persisted caches import ahead of time, and for a real
 *  handle the follow graph and author feeds land in their (did-keyed, seed
 *  independent) caches too, so the wall the reader actually opens reuses them
 *  and skips the network fan-out. A no-op in server mode; best-effort always.
 *
 *  Actor-only, and that is the whole of the decision: what warming lands is a
 *  follow graph and its author feeds, and a feed generator has neither. A feed
 *  wall simply does not call this. */
export async function warmFeed(actor: string, mode?: FeedMode): Promise<void> {
  if (!localMode || !browser || !("serviceWorker" in navigator)) return;
  try {
    await swControlsPage();
    const params = new URLSearchParams({ actor });
    if (mode) params.set("mode", mode);
    await fetch(`${BASE}/api/feed?${params}`);
  } catch {
    // warming is best-effort; the real request pays the cost if this didn't
  }
}

export class FeedError extends Error {
  constructor(
    public code: string,
    public status: number,
  ) {
    super(`feed error: ${code} (${status})`);
  }
}
