import { browser } from "$app/environment";
import { PUBLIC_MASON_SERVER_URL } from "$env/static/public";
import type { FeedResponse } from "./types";

/** Empty → local mode: same-origin fetch, intercepted by the wasm service
 *  worker. Set → server mode: direct CORS call to that mortar instance. */
const BASE = PUBLIC_MASON_SERVER_URL;

export const localMode = BASE === "";

export async function fetchFeed(actor: string, cursor?: string | null): Promise<FeedResponse> {
  if (localMode && browser && "serviceWorker" in navigator) {
    // interception only applies once the SW controls this page
    await navigator.serviceWorker.ready;
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
