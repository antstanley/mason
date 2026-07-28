---
"mason": minor
---

a laid wall is no longer final. one control in the header lays it again in
place: the bricks you were reading stay on screen and reflow into the new
arrangement, rather than collapsing to skeletons or throwing away the engine,
the warm caches and any playing video the way a reload does. it does not take
you back to the top, because the reflow is the thing you asked to see.

it is disabled while a wall is being laid, and that is the whole rate limit: one
refresh is one burst of upstream requests, so a double tap cannot become two.
the wall keeps its single polite announcement, which already says "laying
bricks" while it warms, and a refresh is a warm.

the header bar carries four controls at 375px now, so its gaps and the layout
picker's padding are tighter below sm. it fits a 360px phone for the first time
too, which it did not before the fourth control arrived.
