---
"mason": patch
---

fix: the wasm service worker survives a deploy. Each deploy deletes the previous
build's hashed assets, so a worker that installed before a deploy would 404
fetching its old wasm engine and then brick every `/api/feed` for the life of
that session (the rejected init was memoised) — a 500 that only hit visitors who
had loaded the app before. The worker now precaches its own wasm and loads the
engine from that cache, so it keeps serving until it is itself replaced; a failed
init is never memoised. The client also revalidates the worker script on load
(`updateViaCache: 'none'` + `update()`) and reloads once when a new engine takes
control, so a deploy is picked up promptly instead of leaving a stale worker.
