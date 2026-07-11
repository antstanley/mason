/// <reference types="@sveltejs/kit" />
/// <reference lib="webworker" />

// mortar-in-the-browser: this service worker runs the same Rust feed engine
// as the native server, compiled to wasm, and answers /api/feed without any
// backend. Bundled by SvelteKit as a CLASSIC script in prod — so: no
// top-level await; wasm init is lazy and memoized per SW instance. The SW
// may be killed after ~30s idle; the cursor's embedded seed makes the
// rebuilt wall deterministic.

import init, { feed_page } from "$lib/mortar-wasm/pkg/mortar_wasm";
import wasmUrl from "$lib/mortar-wasm/pkg/mortar_wasm_bg.wasm?url";

declare const self: ServiceWorkerGlobalScope;

let ready: Promise<unknown> | null = null;
const ensureInit = () => (ready ??= init({ module_or_path: wasmUrl }));

self.addEventListener("install", () => {
  void self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});

async function serveFeed(request: Request): Promise<Response> {
  await ensureInit();
  const url = new URL(request.url);
  const actor = url.searchParams.get("actor");
  const cursor = url.searchParams.get("cursor") ?? undefined;
  if (!actor) {
    return json({ error: "bad_request", message: "missing required parameter: actor" }, 400);
  }
  try {
    const body = await feed_page(actor, cursor);
    return new Response(body, {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  } catch (raw) {
    // mortar throws a JSON envelope {status, error, message}
    try {
      const envelope = JSON.parse(String(raw)) as { status?: number };
      return json(envelope, envelope.status ?? 502);
    } catch {
      return json({ error: "wasm", message: String(raw) }, 500);
    }
  }
}

function json(body: unknown, status: number): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });
}

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (
    event.request.method === "GET" &&
    url.origin === self.location.origin &&
    url.pathname === "/api/feed"
  ) {
    event.respondWith(serveFeed(event.request));
  }
  // everything else falls through to the network
});
