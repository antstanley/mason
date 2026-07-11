import type { FeedResponse } from "./types";

export async function fetchFeed(actor: string, cursor?: string | null): Promise<FeedResponse> {
  const params = new URLSearchParams({ actor });
  if (cursor) params.set("cursor", cursor);
  const res = await fetch(`/api/feed?${params}`);
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
