---
"mason": patch
---

fix: switching between bento and masonry no longer throws away your loaded wall.
Only picking (or leaving) glaze re-fetches, since glaze is a different feed; the
two grid-only layouts now just relay the same bricks instead of wiping the wall
and refetching from the top.
