---
"mason": patch
---

a stale build can no longer answer for the tree.

the browser specs drive the real static site, and the preview server that hosts
them only serves what is already in `web/build/`. it never compiles. running one
spec straight after a source edit therefore reported on the build before it:
green, fast, and about code that no longer existed.

the run now refuses to start when the build is older than anything it was made
from, and names the files. this is developer tooling and changes nothing about
the wall itself.
