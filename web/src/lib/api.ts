import { browser } from "$app/environment";
import type { ErrorEnvelope, FeedResponse } from "./types";

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
 *  after the first). */
export type FeedIntent = "preview" | "freeze";

export async function fetchFeed(
  actor: string,
  cursor?: string | null,
  mode?: string,
  intent?: FeedIntent,
): Promise<FeedResponse> {
  if (localMode && browser && "serviceWorker" in navigator) {
    await swControlsPage();
  }
  const params = new URLSearchParams({ actor });
  if (cursor) params.set("cursor", cursor);
  if (mode) params.set("mode", mode);
  if (intent) params.set("intent", intent);
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
 *  and skips the network fan-out. A no-op in server mode; best-effort always. */
export async function warmFeed(actor: string, mode?: string): Promise<void> {
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
