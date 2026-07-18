---
"mason": minor
---

respect logged-out visibility and moderation labels. mason reads walls logged out, so it now mirrors what Bluesky itself shows a logged-out viewer.

- a wall whose owner set `!no-unauthenticated` is sealed behind a "sign in to view" panel, and any followed account that opted out is dropped from the wall whole (posts, blogs, archived streams, and live), not just their skeets.
- adult media (`porn`, `sexual`, `graphic-media`) and moderator hard-hides (`!hide`) are kept off the wall, exactly as a logged-out Bluesky viewer would find them. `nudity`, which Bluesky shows to logged-out viewers, is shown here too.
- a `!warn` label covers a brick's image or video poster behind a "show anyway" reveal, chosen per brick and forgotten on reload. nothing hard-hidden ever reaches this tier, so a covered brick can always be uncovered.
