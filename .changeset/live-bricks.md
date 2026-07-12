---
"mason": minor
---

Streamplace video replaces Steam. The wall now carries atproto livestreams
from stream.place: archived streams from the people you follow (a 90-day
window, since an hours-long stream stays worth watching long after a skeet
about it would have expired), and anyone who is live right now.

A live stream is the only brick with a deadline, so it is the only one that
jumps the queue: it opens the wall, wears a LIVE badge and a viewer count, and
never ages out while it is running. Everything else is unchanged; video is
still click-to-play, and still never autoplays.

Steam is gone entirely. Its storefront API served no CORS headers, so trailers
never worked in the browser build at all; Streamplace is CORS-open, which means
the no-server build now reads exactly what the native one reads.

**A wall that actually arrives.** Chasing the live-stream work turned up three
things that were quietly starving the first page, and they are fixed here:

- The follow graph was fetched to completion before a single post was, and
  follows page 100 at a time with each request blocking the next. Someone with
  2000 follows waited **ten seconds** for a list nobody asked to see, and their
  first wall came back **empty**. The wall is now built from a head start of
  300 follows (the cohort only samples 100 authors anyway) while the rest of
  the graph is fetched behind their back for next time.
- standard.site documents refetched their publication record once per
  document, so one blogger cost 25 sequential requests. A blogger has one blog:
  it is fetched once now. The repo fan-out went from 21s to under 4s.
- Posts and repo reads shared a task per author, so an author's posts waited on
  plc.directory and two PDS reads before being admitted. They are fanned out
  separately now, and the per-author brick cap is per KIND, so a prolific
  poster's own blog is no longer turned away by a quota their skeets ate.

**Infinite scroll that keeps going.** The wall could stop dead with a cursor
still in its hand. `IntersectionObserver` fires on a *change* of intersection,
and a page that came back short did not grow the wall enough to push the
sentinel back out of its prefetch margin, so no second event ever arrived and
the scroll ended there. The wall now pulls rather than waits to be told: it
keeps laying while its bottom is within reach. And the AppView burst is raised
from 40 to 100 (the 10/s sustained ceiling is untouched), so a cold cohort goes
out at once instead of dripping, and a reader can no longer out-scroll their
own wall.
