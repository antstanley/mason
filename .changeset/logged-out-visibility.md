---
"mason": minor
---

respect logged-out visibility. mason reads walls logged out, so it now honours the atproto `!no-unauthenticated` opt-out: a wall whose owner asked to be seen only by signed-in visitors is sealed behind a "sign in to view" panel, and any followed account that opted out is dropped from the wall whole (posts, blogs, archived streams, and live), not just their skeets. adult and graphic media (the porn, sexual, nudity, and graphic-media labels) is kept off the wall too, since a logged-out wall has no blur to tuck it behind.
