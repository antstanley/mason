---
"mason": minor
---

a feed wall now says whose feed it is: the switcher in the header carries the
generator's own avatar and name, read from the appview the way a wall owner's
face already is, and falls back to the feed's rkey rather than an empty button
when the appview has nothing to say.

a feed link that names no feed also stops asking you to check your handle. it
gets its own panel, reading "no such feed", and an empty feed wall says "this
feed has no bricks yet" instead of calling it a wall.
