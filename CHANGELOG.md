# mason

## 0.5.0

### Minor Changes

- [#15](https://github.com/antstanley/mason/pull/15) [`8f22066`](https://github.com/antstanley/mason/commit/8f22066ec1dc04aa36099f6c9cfba752abbcf5a1) Thanks [@antstanley](https://github.com/antstanley)! - respect logged-out visibility and moderation labels. mason reads walls logged out, so it now mirrors what Bluesky itself shows a logged-out viewer.

  - a wall whose owner set `!no-unauthenticated` is sealed behind a "sign in to view" panel, and any followed account that opted out is dropped from the wall whole (posts, blogs, archived streams, and live), not just their skeets.
  - adult media (`porn`, `sexual`, `graphic-media`) and moderator hard-hides (`!hide`) are kept off the wall, exactly as a logged-out Bluesky viewer would find them. `nudity`, which Bluesky shows to logged-out viewers, is shown here too.
  - a `!warn` label covers a brick's image or video poster behind a "show anyway" reveal, chosen per brick and forgotten on reload. nothing hard-hidden ever reaches this tier, so a covered brick can always be uncovered.

## 0.4.0

### Minor Changes

- [#13](https://github.com/antstanley/mason/pull/13) [`96f7eba`](https://github.com/antstanley/mason/commit/96f7ebab673729695e914b4ac44d6f28afd71b73) Thanks [@antstanley](https://github.com/antstanley)! - lay the wall as a bento grid: feature bricks (videos, and blogs or posts with a landscape image) span two columns, and smaller bricks backfill the gaps with dense grid flow. a segmented layout picker in the header switches between the bento wall and the original masonry columns, and the choice is remembered. the client picker becomes an icon dropdown carrying each service's own mark (Bluesky, Mu Social, Blacksky), the layout picker slides between its two states, and the switch button wears the wall owner's avatar and drops an inline switcher below itself, so opening it and changing your mind never leaves the wall you are on. on a narrow screen the header sheds the wordmark and becomes a sticky bottom bar: an icon-only layout slider, the client picker, and an avatar-only switch button, evenly spread, with its menus opening upward.

## 0.3.0

### Minor Changes

- [#11](https://github.com/antstanley/mason/pull/11) [`e51775c`](https://github.com/antstanley/mason/commit/e51775c794c7fc463ff86125801121ca8a226268) Thanks [@antstanley](https://github.com/antstanley)! - Streamplace video replaces Steam. The wall now carries atproto livestreams
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
  still in its hand. `IntersectionObserver` fires on a _change_ of intersection,
  and a page that came back short did not grow the wall enough to push the
  sentinel back out of its prefetch margin, so no second event ever arrived and
  the scroll ended there. The wall now pulls rather than waits to be told: it
  keeps laying while its bottom is within reach. And the AppView burst is raised
  from 40 to 100 (the 10/s sustained ceiling is untouched), so a cold cohort goes
  out at once instead of dripping, and a reader can no longer out-scroll their
  own wall.

- [#9](https://github.com/antstanley/mason/pull/9) [`05fa72e`](https://github.com/antstanley/mason/commit/05fa72eee8231ab20bf5faaaee9c25100f637152) Thanks [@antstanley](https://github.com/antstanley)! - An atmosphere client picker, and a wall that is actually fresh.

  **Open posts in the client you use.** bsky.app, mu.social and blacksky.community share a URL structure, so the picker in the header rewrites the host and nothing else. Blog and stream links are left exactly as they are, because they are not Bluesky posts. The choice persists.

  **A stronger recency bias.** Posts and Bluesky videos now live for 72 hours, blogs for 14 days, and nothing older is admitted to the wall at all: a hard window, not a soft preference, because decay alone leaves week-old content eligible and on a quiet follow graph it surfaces. Half-lives are steeper to match (posts 12h, blogs 3d).

  **A wall could belong to one person.** First paint gated on the number of bricks in the pool, and a single prolific account returns thirty of them, so the wall could open before anyone else's feed arrived. It now gates on distinct authors, no author may hold more than four bricks, and when the diversity rule truly cannot be honoured the mixer falls back to the least represented author rather than re-picking the loudest.

## 0.2.0

### Minor Changes

- [#5](https://github.com/antstanley/mason/pull/5) [`23e5e10`](https://github.com/antstanley/mason/commit/23e5e10fd7e7a6c08d88a7c9c97d975d9a59e7ec) Thanks [@antstanley](https://github.com/antstanley)! - Link previews, a kiln-fired Open Graph card, a favicon that is a wall, and offline install.

  Shared mason links previewed as a bare URL: crawlers do not run JavaScript and never boot the service worker that is the feed engine, so the shell carried no title, description or image. It does now, with a 1200x630 card built from mason's own dark tokens (source in `web/scripts/og-template.html`, rendered by `pnpm og`).

  The tab showed SvelteKit's Svelte logo. mason now has its own mark: a staggered bond of colour-coded bricks, a wall rather than a letterform, because an "m" turns to mush at 16px.

  mason also installs as a desktop app and survives offline. The service worker precaches the shell and the wasm, and the demo wall needs no network at all, because its bricks are fixtures compiled into the wasm.

### Patch Changes

- [#4](https://github.com/antstanley/mason/pull/4) [`a5f3f74`](https://github.com/antstanley/mason/commit/a5f3f74368b033537f72bd0edf4d66c0901c887b) Thanks [@antstanley](https://github.com/antstanley)! - Serve `site.webmanifest` with the right content type.

  S3 was returning `application/octet-stream`, because blogwright had no entry for the extension. Fixed upstream in blogwright 0.3.1, which also grants the CI build role the `s3:PutObjectTagging` permission its object tagging needs, and which now creates the preview stack's wildcard DNS record and its CloudFront log delivery instead of asking for them by hand.

  S3 writes object metadata only on a PUT, so a normal deploy skips content-identical files and a corrected header never reaches an object already live. Both workflows can now pass `--refresh` to re-upload everything: preview always does, production takes it as an input.
