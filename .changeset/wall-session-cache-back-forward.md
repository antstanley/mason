---
"mason": patch
---

fix: going back to a wall you already scrolled now returns it exactly as you left
it. mason keeps each wall you visit for the length of the session, so browser
back/forward rehydrates the same bricks in the same order (and your scroll lands
where it should) instead of rolling a fresh arrangement that dropped you back at
a single screen.
