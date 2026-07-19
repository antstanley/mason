/// <reference types="@sveltejs/kit" />
/// <reference lib="webworker" />

// mortar-in-the-browser: this service worker runs the same Rust feed engine
// as the native server, compiled to wasm, and answers /api/feed without any
// backend. The SW may be reaped after ~30s idle, so the warm caches are
// persisted to IndexedDB after each served page and imported on startup -
// a wake-up costs an IDB read instead of a cold network fan-out. (The
// cursor's embedded seed covers correctness either way.)

import { build, files, version } from "$service-worker";
import init, {
  cache_names,
  dirty_cache_names,
  export_cache,
  feed_page,
  import_cache,
} from "$lib/mortar-wasm/pkg/mortar_wasm";
import wasmUrl from "$lib/mortar-wasm/pkg/mortar_wasm_bg.wasm?url";
import type { ErrorEnvelope, MortarErrorCode } from "$lib/types";

declare const self: ServiceWorkerGlobalScope;

// --- app shell cache --------------------------------------------------------
// mason installs as an app, and an installed app that dies without a network
// is a bad app. The shell and the wasm are precached, so an offline launch
// still opens the landing page and the demo wall, which needs no network at
// all: its bricks are fixtures compiled into the wasm.

const SHELL = `mason-shell-${version}`;
// `build` does NOT include the wasm engine (it rides in as a Vite `?url` asset,
// not a SvelteKit build output), so precache it explicitly. That is what lets a
// worker survive a deploy: the next deploy deletes this hashed wasm from S3, but
// this worker keeps serving the copy it cached here until it is itself replaced.
const PRECACHE = ["/", ...build, ...files, wasmUrl];

// --- IndexedDB: one store, one key per cache --------------------------------

const IDB_NAME = "mason";
const IDB_STORE = "kv";
// one key per mortar cache, so a page that only warmed one cache only writes
// one; the payload embeds mortar's persist VERSION, and mortar discards
// mismatches on import
const IDB_CACHE_PREFIX = "mortar-cache:";
// pre-v4 whole-bundle key; deleted on startup, its shape is no longer read
const IDB_LEGACY_KEY = "mortar-caches-v1";

function idbOpen(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(IDB_NAME, 1);
    req.addEventListener("upgradeneeded", () => req.result.createObjectStore(IDB_STORE));
    req.addEventListener("success", () => resolve(req.result));
    req.addEventListener("error", () => reject(req.error));
  });
}

async function idbGetMany(keys: string[]): Promise<unknown[]> {
  const db = await idbOpen();
  try {
    const store = db.transaction(IDB_STORE, "readonly").objectStore(IDB_STORE);
    return await Promise.all(
      keys.map(
        (key) =>
          new Promise((resolve, reject) => {
            const req = store.get(key);
            req.addEventListener("success", () => resolve(req.result));
            req.addEventListener("error", () => reject(req.error));
          }),
      ),
    );
  } finally {
    db.close();
  }
}

async function idbPutMany(entries: [string, string][]): Promise<void> {
  const db = await idbOpen();
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      const store = tx.objectStore(IDB_STORE);
      for (const [key, value] of entries) store.put(value, key);
      tx.addEventListener("complete", () => resolve());
      tx.addEventListener("error", () => reject(tx.error));
    });
  } finally {
    db.close();
  }
}

/** Delete keys this build will never read again: the pre-v4 whole-bundle key
 *  and any mortar-cache:* key whose cache mortar no longer persists (a renamed
 *  or dropped cache would otherwise orphan its key forever). */
async function idbSweepStale(validNames: Set<string>): Promise<void> {
  const db = await idbOpen();
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(IDB_STORE, "readwrite");
      const store = tx.objectStore(IDB_STORE);
      const req = store.getAllKeys();
      req.addEventListener("success", () => {
        for (const key of req.result) {
          if (typeof key !== "string") continue;
          const stale =
            key === IDB_LEGACY_KEY ||
            (key.startsWith(IDB_CACHE_PREFIX) &&
              !validNames.has(key.slice(IDB_CACHE_PREFIX.length)));
          if (stale) store.delete(key);
        }
      });
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
    try {
      // Load the engine from THIS worker's own precache, not the network. Every
      // deploy deletes the previous build's hashed assets, so a worker that
      // installed before a deploy would 404 fetching its old wasm URL and then
      // brick every /api/feed for its whole life (the rejected init below is
      // memoised). Its precache still holds the wasm it shipped with, so read
      // that; the network is only the cold-start fallback for a fresh install.
      const hit = await caches.open(SHELL).then((cache) => cache.match(wasmUrl));
      await init({ module_or_path: hit ? await hit.arrayBuffer() : wasmUrl });
      const names = cache_names();
      try {
        const saved = await idbGetMany(names.map((name) => IDB_CACHE_PREFIX + name));
        await Promise.all(
          names.map((name, i) => {
            const payload = saved[i];
            return typeof payload === "string" ? import_cache(name, payload) : undefined;
          }),
        );
      } catch {
        // no persisted caches (or unreadable); start cold
      }
      // reclaim keys this build will never read: the pre-v4 whole bundle and
      // any per-cache key orphaned by a cache rename or removal
      void idbSweepStale(new Set(names)).catch(() => {});
    } catch (e) {
      // never memoise a failed init: let the next request retry rather than
      // leaving the session bricked behind a permanently-rejected promise
      ready = null;
      throw e;
    }
  })());

const PERSIST_INTERVAL_MS = 4000;
let lastPersist = 0;
// persist cycles run one at a time: two overlapping cycles (two tabs share one
// SW) could otherwise interleave export and write so the OLDER payload commits
// last under a key whose dirty flag the newer export already cleared. Each new
// cycle chains behind the in-flight one, so a freeze arriving mid-persist
// still runs afterwards and captures the frozen state; nothing is dropped.
let persisting: Promise<void> = Promise.resolve();

/** Write the caches that changed since the last export, each under its own
 *  IDB key: cost scales with what this page touched, not with everything a
 *  long session has cached. Preview polls (350ms cadence, same data warming
 *  up) never persist; the freeze that ends them always does. */
function persistCaches(intent?: string): Promise<void> {
  if (intent === "preview") return Promise.resolve();
  const cycle = persisting.then(() => persistCycle(intent));
  // persistCycle never rejects, but never let the chain jam on a rejection
  persisting = cycle.catch(() => {});
  return cycle;
}

async function persistCycle(intent?: string): Promise<void> {
  // throttle is judged when the cycle actually runs, after any in-flight one
  if (intent !== "freeze" && Date.now() - lastPersist < PERSIST_INTERVAL_MS) return;
  lastPersist = Date.now();
  try {
    const exported = await Promise.all(
      dirty_cache_names().map(async (name): Promise<[string, string] | undefined> => {
        const payload = await export_cache(name);
        return typeof payload === "string" ? [IDB_CACHE_PREFIX + name, payload] : undefined;
      }),
    );
    const entries = exported.filter((entry) => entry !== undefined);
    if (entries.length > 0) await idbPutMany(entries);
  } catch {
    // persistence is best-effort; next page tries again
  }
}

self.addEventListener("install", (event) => {
  event.waitUntil(caches.open(SHELL).then((cache) => cache.addAll(PRECACHE)));
  void self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      // one cache per build; drop every older one
      const stale = (await caches.keys()).filter(
        (k) => k.startsWith("mason-shell-") && k !== SHELL,
      );
      await Promise.all(stale.map((k) => caches.delete(k)));
      await self.clients.claim();
    })(),
  );
});

/** Cache first for the immutable build assets, network first for the rest,
 *  and always fall back to the cached shell so an offline navigation lands on
 *  the app instead of the browser's error page. */
async function serveShell(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const cache = await caches.open(SHELL);

  // hashed build assets never change under their own name
  if (build.includes(url.pathname)) {
    const hit = await cache.match(url.pathname);
    if (hit) return hit;
  }

  try {
    const response = await fetch(request);
    if (response.ok && request.method === "GET" && files.includes(url.pathname)) {
      void cache.put(url.pathname, response.clone());
    }
    return response;
  } catch {
    const hit = (await cache.match(url.pathname)) ?? (await cache.match("/"));
    if (hit) return hit;
    throw new Error("offline and nothing cached for this request");
  }
}

async function serveFeed(request: Request): Promise<Response> {
  await ensureInit();
  const url = new URL(request.url);
  const actor = url.searchParams.get("actor");
  const cursor = url.searchParams.get("cursor") ?? undefined;
  const mode = url.searchParams.get("mode") ?? undefined;
  // "preview" / "freeze" drive the warm-then-commit first screen; absent is a
  // normal committed page (every page after the first).
  const intent = url.searchParams.get("intent") ?? undefined;
  if (!actor) {
    return json(
      {
        // typed as MortarErrorCode: mortar would answer this exact code, so
        // renaming it there (and in the contract fixture) fails here too
        error: "bad_request" satisfies MortarErrorCode,
        message: "missing required parameter: actor",
      } satisfies ErrorEnvelope,
      400,
    );
  }
  try {
    const body = await feed_page(actor, cursor, mode, intent);
    return new Response(body, {
      status: 200,
      headers: { "content-type": "application/json" },
    });
  } catch (raw) {
    // mortar throws the ErrorEnvelope JSON {error, message, status}; the exact
    // strings are pinned by a fixture test in mortar-core's error.rs
    try {
      const envelope = JSON.parse(String(raw)) as ErrorEnvelope;
      if (typeof envelope?.error !== "string") throw new Error("not an envelope");
      return json(envelope, envelope.status ?? 502);
    } catch {
      // anything else on this channel is a wasm-side failure, not a feed error
      return json({ error: "wasm", message: String(raw) } satisfies ErrorEnvelope, 500);
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
  if (event.request.method !== "GET" || url.origin !== self.location.origin) return;

  if (url.pathname === "/api/feed") {
    const response = serveFeed(event.request);
    event.respondWith(response);
    // keep the SW alive until the warm caches hit IndexedDB
    const intent = url.searchParams.get("intent") ?? undefined;
    event.waitUntil(response.then(() => persistCaches(intent)));
    return;
  }

  // navigations and same-origin assets: shell cache, with an offline fallback
  event.respondWith(serveShell(event.request));
});
