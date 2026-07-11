/// <reference types="@sveltejs/kit" />
/// <reference lib="webworker" />

// mortar-in-the-browser: this service worker runs the same Rust feed engine
// as the native server, compiled to wasm, and answers /api/feed without any
// backend. The SW may be reaped after ~30s idle, so the warm caches are
// persisted to IndexedDB after each served page and imported on startup —
// a wake-up costs an IDB read instead of a cold network fan-out. (The
// cursor's embedded seed covers correctness either way.)

import init, { export_caches, feed_page, import_caches } from "$lib/mortar-wasm/pkg/mortar_wasm";
import wasmUrl from "$lib/mortar-wasm/pkg/mortar_wasm_bg.wasm?url";

declare const self: ServiceWorkerGlobalScope;

// --- IndexedDB: one store, one key -----------------------------------------

const IDB_NAME = "mason";
const IDB_STORE = "kv";
const IDB_KEY = "mortar-caches-v1";

function idbOpen(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1);
    req.addEventListener("upgradeneeded", () => req.result.createObjectStore(IDB_STORE));
    req.addEventListener("success", () => resolve(req.result));
    req.addEventListener("error", () => reject(req.error));
  });
}

async function idbGet(key: string): Promise<unknown> {
  const db = await idbOpen();
  try {
    return await new Promise((resolve, reject) => {
      const req = db.transaction(IDB_STORE, "readonly").objectStore(IDB_STORE).get(key);
      req.addEventListener("success", () => resolve(req.result));
      req.addEventListener("error", () => reject(req.error));
    });
  } finally {
    db.close();
  }
}

async function idbPut(key: string, value: string): Promise<void> {
  const db = await idbOpen();
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      tx.objectStore(IDB_STORE).put(value, key);
      tx.addEventListener("complete", () => resolve());
      tx.addEventListener("error", () => reject(tx.error));
    });
  } finally {
    db.close();
  }
}

// --- wasm lifecycle ---------------------------------------------------------

let ready: Promise<unknown> | null = null;
const ensureInit = () =>
  (ready ??= (async () => {
    await init({ module_or_path: wasmUrl });
    try {
      const saved = await idbGet(IDB_KEY);
      if (typeof saved === "string") await import_caches(saved);
    } catch {
      // no persisted caches (or unreadable) — start cold
    }
  })());

const PERSIST_INTERVAL_MS = 4000;
let lastPersist = 0;

async function persistCaches(): Promise<void> {
  if (Date.now() - lastPersist < PERSIST_INTERVAL_MS) return;
  lastPersist = Date.now();
  try {
    await idbPut(IDB_KEY, await export_caches());
  } catch {
    // persistence is best-effort; next page tries again
  }
}

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
    const response = serveFeed(event.request);
    event.respondWith(response);
    // keep the SW alive until the warm caches hit IndexedDB
    event.waitUntil(response.then(() => persistCaches()));
  }
  // everything else falls through to the network
});
