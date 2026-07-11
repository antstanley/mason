import { browser } from "$app/environment";
import type { FeedResponse } from "./types";

/** Empty → local mode: same-origin fetch, intercepted by the wasm service
 *  worker. Set → server mode: direct CORS call to that mortar instance.
 *  Injected at build time via vite `define` (defaults to '' when unset). */
const BASE: string = import.meta.env.PUBLIC_MASON_SERVER_URL ?? "";

export const localMode = BASE === "";

/** Interception only applies once the SW CONTROLS this page — `ready`
 *  resolves at activation, which can precede clients.claim() taking effect.
 *  Fetching in that gap goes to the network and 404s on a static host. */
async function swControlsPage(): Promise<void> {
  if (navigator.serviceWorker.controller) return;
  await navigator.serviceWorker.ready;
  if (navigator.serviceWorker.controller) return;
  await Promise.race([
    new Promise<void>((resolve) =>
      navigator.serviceWorker.addEventListener("controllerchange", () => resolve(), {
        once: true,
      }),
    ),
    // hard-reloaded pages stay uncontrolled by design — don't hang forever
    new Promise<void>((resolve) => setTimeout(resolve, 2000)),
  ]);
}

export async function fetchFeed(actor: string, cursor?: string | null): Promise<FeedResponse> {
  if (localMode && browser && "serviceWorker" in navigator) {
    await swControlsPage();
  }
  const params = new URLSearchParams({ actor });
  if (cursor) params.set("cursor", cursor);
  const res = await fetch(`${BASE}/api/feed?${params}`);
  if (!res.ok) {
    const body = (await res.json().catch(() => null)) as { error?: string } | null;
    throw new FeedError(body?.error ?? "unknown", res.status);
  }
  return (await res.json()) as FeedResponse;
}

export class FeedError extends Error {
  constructor(
    public code: string,
    public status: number,
  ) {
    super(`feed error: ${code} (${status})`);
  }
}
