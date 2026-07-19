---
"mason": patch
---

perf: a cold wall paints sooner. Five changes to the opening of a wall, none of
which move a brick once it is laid:

- one `getProfile` now resolves the handle AND reads the owner's logged-out
  opt-out, folding the two sequential AppView calls that gated every cold load
  into one round trip.
- the first wall waits for a single follow-graph page (100 follows, already more
  than the cohort samples) instead of three, so the fan-out starts two round
  trips sooner. The rest of the graph is still chased in the background.
- the first page's wait-for-a-better-mix deadline is now anchored to when the
  snapshot was created, so the first-paint wait counts against it rather than
  stacking on top of it: the opening wait is bounded, not doubled.
- the landing page warms the engine while you are still at the form — a
  remembered handle warms that wall's caches, and with no handle the demo wall
  at least compiles the wasm off the critical path.
- the roughly-first-screen bricks load their images eagerly and at high fetch
  priority; the rest of the wall stays lazy.
