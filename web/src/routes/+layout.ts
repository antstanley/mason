// Full SPA: no SSR, no prerendered routes — the wall is client-rendered and
// the feed comes from the wasm service worker (local mode) or a CORS call to
// mortar (server mode).
export const ssr = false;
export const prerender = false;
