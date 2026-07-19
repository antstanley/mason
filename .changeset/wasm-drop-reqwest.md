---
"mason": patch
---

perf: the browser engine is 103 KB smaller. The wasm build talked to the network
through reqwest, which on wasm is only a thin wrapper over the browser's own
fetch — but it dragged the `url` crate's IDNA/ICU Unicode tables in with it, none
of which mason needs (every request URL is a plain ASCII atproto endpoint). The
browser build now uses gloo-net instead, a direct fetch wrapper with no such
tail. reqwest stays on the native server unchanged. The shared rate limiter and
429/5xx retry loop are untouched; only the one-shot GET underneath is split by
target. The shipped wasm drops from 389 KB to 286 KB gzipped (270 KB off the
raw binary), so a cold start downloads and compiles less on the very path that
gates first paint.
