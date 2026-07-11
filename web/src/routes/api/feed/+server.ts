import { json } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import type { RequestHandler } from "./$types";

const MORTAR_BASE = env.MASON_SERVER_URL ?? "http://localhost:8787";

export const GET: RequestHandler = async ({ url, fetch }) => {
  const upstream = new URL("/api/feed", MORTAR_BASE);
  for (const key of ["actor", "cursor"]) {
    const value = url.searchParams.get(key);
    if (value) upstream.searchParams.set(key, value);
  }

  try {
    const res = await fetch(upstream, { signal: AbortSignal.timeout(15_000) });
    const body = await res.json();
    return json(body, { status: res.status });
  } catch {
    return json(
      { error: "mortar_unreachable", message: "feed server not responding" },
      { status: 502 },
    );
  }
};
