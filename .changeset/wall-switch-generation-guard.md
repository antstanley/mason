---
"mason": patch
---

fix: switching walls mid-load no longer bleeds the old wall into the new one.
When you jump to a different wall while the first one is still fetching, a late
response from the old wall could shove its bricks into the new wall, overwrite
the cursor, or wrongly mark pagination finished (a wall stuck at one screen).
Both the load-more and freeze steps now bail out when the wall has moved on, and
resetting a wall clears the loading flag so the fresh wall starts clean.
