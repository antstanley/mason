---
"mason": patch
---

fix: the wall no longer hangs on an endless skeleton when the service worker
fails to register. Waiting for the worker to be ready is now bounded by the same
short timeout as everything else, so a registration that never settles falls
through to a normal request instead of leaving you staring at a loading wall
forever.
