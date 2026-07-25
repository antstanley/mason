# 08 - The Wall and Its Bricks

**Status:** Draft · **Date:** 2026-07-25 · **Owner:** Ant Stanley

This page covers what a reader sees and touches: the three wall layouts, the card
components that render each brick kind, the video player, and the states a wall
can be in. The state machine driving it is in
[07-web-client.md](07-web-client.md); the tokens it draws with are in
[09-design-system.md](09-design-system.md).

---

## Responsibilities

1. Lay bricks into a wall at three layouts, without ever replaying an entrance
   animation on a brick already on the wall.
2. Render each brick kind at its own shape and accent.
3. Keep the endless scroll moving, and say so honestly when it cannot.
4. Play video only on an explicit click, one at a time.
5. Keep DOM order equal to feed order, whatever the visual packing does.

---

## Layouts

One control (`LayoutPicker`) offers three walls. `glaze` is also an algorithm:
picking it sets `mode=glaze` and re-fetches an images-only wall.

| Layout | Component | Packing |
|---|---|---|
| Bento (default) | `Bento.svelte` | CSS grid, `grid-auto-flow: row dense`, feature bricks span two columns |
| Masonry | `Masonry.svelte` | Absolute placement into the shortest column, transforms only |
| Glaze | `Bento.svelte` with `filler` | The same dense grid, laid on a muted field |

Column count comes from one function for every surface, `colsForWidth`, measured
on the **container** and never the viewport, so skeletons and laid bricks always
agree:

| Container width | Columns |
|---|---|
| `< 340px` | 1 |
| `< 640px` | 2 |
| `< 1024px` | 3 |
| `< 1392px` | 4 |
| otherwise | 5 |

A phone gets two columns, not one. A single column is the doomscroll feed mason
positioned against.

### Bento

Every brick keeps its natural height by spanning a whole number of 4px row
tracks, measured with a `ResizeObserver` on its content (so a late-loading image
or a webfont reflows it). A **feature** brick earns a second column: a video
always, a blog with a cover image, or a post whose first image is landscape.
`grid-auto-flow: dense` backfills the holes wide bricks leave.

Glaze passes `filler`, which wraps the grid in a muted padded panel so every gap
the dense packing leaves reads as solid grout between the pictures. The container
is measured on its **border** box for exactly this reason: the filler's padding
must not shrink the measured width, or glaze would drop a column at a viewport
where bento and masonry keep it.

### Masonry

One keyed list in feed order, placed with transforms rather than split across
per-column blocks, so a brick's node survives re-places and column moves and its
in-card state and keyboard focus stay put. Heights come from one shared
`ResizeObserver`; entries arrive after the browser has laid out, so placement
never forces a layout.

- A brick **keeps the column it first landed in**, so an append or a late height
  change shifts bricks down their own column instead of reshuffling the wall.
- A column-count change lays out at once with the pre-resize heights (so the wall
  moves immediately) and re-places when the remeasured heights land.
- **Warming is a distinct mode.** While the first screen reflows, the arrangement
  can reorder between updates, so kept columns mean nothing and every update
  re-places from scratch. A `wasWarming` flag catches the freeze, because the feed
  flips `warming` off in the same update that delivers the committed order, and
  that update must be re-placed rather than misread as an append.
- A brick with no measurement yet stays `invisible` and slots in a frame later.

### Visual order versus DOM order

Dense packing and transform placement both mean a later brick can be *painted*
before an earlier one. DOM order stays the feed order on purpose, so tab order
and screen-reader order follow the feed rather than the packed layout.

### Entrance animation

Both layouts keep an `entered` set of brick ids. A re-measure, a column change or
a re-place must not replay the drop-in on a brick already on the wall; only a
shrink (a feed reset) clears the set so a fresh wall drops in again.

---

## Cards

Every card is wrapped in `BrickShell`, which owns the shared brick chrome: a 2px
accent border keyed to the kind, `rounded-card`, `shadow-brick`, and a hover lift
with a 0.6 degree tilt (gated behind `motion-safe:`). It takes an `aria-label` so
every brick's `<article>` carries a consistent accessible name, and wraps its
children in the outbound link when the card has one.

| Component | Brick | Shape |
|---|---|---|
| `PostCard` | post | Optional image at its own aspect, text, optional external-link preview, author, like and repost counts |
| `BlogCard` | blog | Optional 8:5 cover, publication chip, title, clamped description, up to four tags, author |
| `VideoCard` | video | Poster with a play button, source or LIVE badge, runtime, title, activity or viewer count, author, watch-at-source link |
| `GlazeCard` | post (glaze wall only) | The picture *is* the brick |
| `SkeletonCard` | none | Three height and tint variants, cycled by index |

Details worth pinning:

- **Counts hide at zero.** A fresh brick has no tallies yet, and zeros read as
  neglect.
- **`priority`** is passed to roughly the first screen's worth of bricks
  (`EAGER_BRICKS = 6` in both layouts). Those load their image `eager` at
  `fetchpriority="high"`; the rest of the wall stays lazy, so the tail costs
  nothing until it is scrolled to.
- **Icons are inline SVG, never glyphs.** Glyph characters sit on a font baseline
  and render unevenly across platforms; `Icon.svelte` holds every lucide path the
  app draws.
- **Outbound links** are `target="_blank" rel="noopener noreferrer"` and go
  through `clientUrl`, which rewrites `bsky.app` to the reader's chosen client and
  passes everything else through.

### GlazeCard

The image count decides the shape:

| Images | Shape |
|---|---|
| 1 | Full-card image at its own aspect |
| 2 or 3 | A 3:2 grid, `gap-1` so the card behind shows through as grout |
| 4+ | A full-frame filmstrip |

The filmstrip is a scroll-snap-free horizontal strip you swipe or page with
arrows, committing to a neighbour only once **more than 60%** of it has been
dragged in; anything less snaps back. `scrollend` is the clean settle signal with
a 140 ms debounced fallback, and both defer to an in-flight programmatic
correction. A live region announces "image N of M".

Each image is its own link and the controls sit outside them, so a tap opens the
post while a drag scrolls the strip and the arrows never trip navigation. At rest
the card is just the picture; on hover an author pill and the caption fade in.
Touch has no hover, so there they stay shown.

When any image carries a description, the caption bar gains an **ALT** button.
Opening it behaves like a small dialog: focus moves to the close control, Escape
or the close button returns focus to the trigger, and the covered strip is
`inert` while it is up.

---

## Video

The rule is absolute and enforced in CI: **videos never autoplay.**
`just guard-autoplay` greps `web/src` for `autoplay` and for any `.play(` outside
`VideoPlayer.svelte`, and fails the build on either. It is an accessibility
stance, not a preference. It is a filesystem grep rather than a git grep, so new
unsnapshotted files in this jj repo are seen too.

```
VideoCard at rest:  poster + play button. No <video> element exists.
       │ click
       ▼
  player.claim(id)  (synchronously, so this card never loses its own click)
       ▼
VideoPlayer mounts
       ├─ native HLS?  el.src = playlist
       ├─ else         dynamic import("hls.js"), attach
       ├─ el.play()    ← inside the user's click gesture chain, so sound is allowed
       ├─ IntersectionObserver: scrolled out of view ⇒ pause. No off-screen audio.
       └─ player.activeId != id ⇒ pause, and the card collapses back to its poster
```

`hls.js` is imported dynamically, so the wall's initial bundle does not carry a
video library. A teardown while that import is in flight sets a `cancelled` flag,
so nothing is constructed on a detached element.

The player reports buffering (`waiting` / `playing`) rather than showing a silent
black box, and renders a failure state instead: "this stream has ended" for a
live brick, "this video would not play here" otherwise. That is the last line of
defence for a live brick already laid on the wall, since the wall never moves.

Caption tracks render whenever mortar supplies them, and `crossorigin` is set
only alongside tracks (browsers refuse cross-origin track files on a non-CORS
media element). No upstream carries caption data yet, so the list is empty on
every brick today.

---

## Sensitive media

`Sensitive` covers a brick's media behind a reveal when a `!warn` blur rides on
it. The hidden tier never reaches the wall at all (see
[04](04-sources-and-moderation.md)), so anything that gets here can always be
revealed.

The choice is per brick and forgotten on reload, by design: no storage, no
lingering "show everything" switch.

---

## Wall states

`FeedGrid` is the single component that decides what the wall is doing.

| Condition | What the reader sees |
|---|---|
| `initialLoad` | A 12-card skeleton grid at the same column count as the real wall |
| `error` and no items | An error panel: sealed, handle-not-found, or unreachable, each with the right recovery |
| `done` and no items | "this wall has no bricks yet", with a handle box. An empty wall is a site, not a dead end |
| items present | The wall, plus a sentinel, plus a tail |
| `error` with items | A retry button: "more bricks did not arrive" |
| `stalled` | "the wall paused", with a "try for more" button. Scrolling retries too |
| `warming` or loading or pumping | A 4-card skeleton tail |
| `done` | "that is every brick. the wall is finished." |

Both recoverable errors drop the reader into a handle box, because a wall you
cannot see is still a door to another one. The typo keeps its text to fix; the
sealed wall clears it, because there is nothing to correct, only somewhere else
to go. Both focus and select the input on arrival.

### The scroll pump

```
IntersectionObserver on a sentinel, rootMargin 1200px
        │ fires
        ▼
    pump()
      while !done and the sentinel is still within reach:
         loadMore(); tick()
         grew?      ▸ reset the stall counter, continue
         error?     ▸ break
         no growth? ▸ back off 400ms × stalls; after 3, set `stalled` and break
```

**The pump pulls rather than waiting to be told, and that is the bug it exists to
fix.** `IntersectionObserver` fires on a *change* of state, and a page that comes
back short (mortar serves what it has rather than make you wait for a full one)
does not grow the wall enough to push the sentinel back out of the margin. It
stays intersecting, no second event arrives, and the wall stops for good with a
cursor still in its hand.

The observer effect also reads `feed.warming`, so it re-runs when the wall
freezes. A fresh observer fires its initial callback immediately, so a short first
screen starts paging without needing a later intersection change to nudge it.

Giving up quietly stranded the reader with no ending and no error, so a backed-off
pump says the wall paused instead, and the next pump (a scroll, or the button)
picks it back up. No countdowns, no spinners: unhurried.

### Freezing the reflow

While the first screen is reflowing, the first sign the reader wants to engage
freezes it: the wall must stop moving the instant they reach for it.

| Signal | Why |
|---|---|
| `wheel`, `touchmove`, `scroll` (passive, once) | Pointer engagement |
| Arrow keys, PageUp/Down, Home, End, Space | A keyboard user has no wheel to reach with |
| `focusin` anywhere inside the wall | A switch user's way of engaging |
| `prefers-reduced-motion: reduce` | Freeze before it moves at all: those readers never see the auto-reflow |

---

## Accessibility behaviours

Conformance target and colour work are in
[09-design-system.md](09-design-system.md); these are the behavioural pieces that
live in components.

- **A skip link** to `#wall`, visible on focus.
- **One polite live region** for the whole wall, narrating laying, batch sizes, a
  stall, and the end, so a screen-reader user hears async state instead of
  watching bricks appear in silence. The count it measures against resets while
  warming, so reflow churn never reads as new bricks.
- **`aria-busy`** on the wall during `initialLoad`.
- **One `h1` at a time.** The wall's is `sr-only`, and steps aside when an error
  panel raises the failure as the page's heading.
- **Custom controls stay keyboard-honest.** `LayoutPicker` is native radios under
  a sliding thumb (arrow keys move the selection); `ClientPicker` is a listbox
  with roving arrow-key focus, Home, End, Escape and click-away, because a native
  `<select>` cannot render the client icons; `SwitchWall` is a dialog with
  Escape, click-away, and a focus-out that closes when focus leaves the whole
  switcher.
- **Touch targets are at least 44px** (`min-h-11`) on every control, including
  the ones that shrink to icons on mobile.
- **Motion is gated.** Hover transforms are `motion-safe:` at every call site;
  the brick entrance becomes a crossfade under reduced motion; the layout thumb
  and the filmstrip both stop animating.

---

## Implementation layout

```
web/src/lib/
  columns.ts                 colsForWidth, the one column-count source
  components/
    Bento.svelte             dense CSS grid, feature spans, glaze filler
    Masonry.svelte           shortest-column placement by transform
    FeedGrid.svelte          wall states, the pump, freeze-on-engage, live region
    SkeletonGrid.svelte      loading columns measured the same way
    BrickShell.svelte        shared card chrome and accent
    AuthorChip.svelte        avatar, display name, handle
    Sensitive.svelte         the !warn reveal
    VideoPlayer.svelte       the ONE sanctioned .play()
    LandingWall.svelte       the inert demo wall behind the handle form
    HandleForm.svelte        the front door
    SwitchWall.svelte        the owner's face as a wall switcher
    LayoutPicker.svelte      bento / masonry / glaze
    ClientPicker.svelte      which atmosphere client links open in
    Icon.svelte              every lucide path, inline
    ClientIcon.svelte        per-client marks
    cards/{Post,Blog,Video,Glaze,Skeleton}Card.svelte
```

The landing page shows the product rather than describing it: a real (demo) wall
laid behind the handle form, `aria-hidden`, `inert`, `pointer-events-none`, and
masked so it recedes toward the form. If the feed cannot load, it simply never
appears and the form still works.

---

## Assumptions and open questions

**Assumptions**

- `ResizeObserver`, `IntersectionObserver` and `scrollend` are available;
  `scrollend` has a debounced fallback, the other two do not.
- HLS playback is available either natively (Safari) or through `hls.js`.
- Images from the AppView CDN and from PDS blob endpoints are hotlinkable and
  serve permissive CORS.

**Decisions**

- *One column-count function.* **`colsForWidth`, measured on the container.**
  Viewport-based breakpoints made the skeleton resolve into a different column
  count than the wall it was standing in for.
- *Two columns on a phone.* **Never one.** A single column is the doomscroll feed
  mason positioned against; a phone should still see a wall.
- *DOM order is feed order.* **Always, whatever the packing does.** Dense grid
  and transform placement both reorder visually; tab and screen-reader order must
  not follow.
- *Masonry places with transforms.* **One keyed list, absolutely positioned.**
  Per-column blocks destroy and recreate nodes on a re-place, losing in-card
  state and focus.
- *Kept columns, cleared only on a re-place.* **A brick stays in its column.** A
  late image load otherwise reshuffles the whole wall under the reader.
- *The pump pulls.* **A loop, not a callback.** `IntersectionObserver` fires on
  change, and a short page leaves the sentinel intersecting with no second event
  coming.
- *A stall is stated, not hidden.* **"The wall paused", with a button.** Silence
  read as a wall that had given up.
- *No autoplay, enforced by grep.* **One sanctioned `.play()`, click-gated.** A
  convention nobody can check is a convention that erodes.
- *Blur choice is per brick and not stored.* **No lingering global reveal.** A
  persisted "show everything" is a setting nobody remembers turning on.

**Open questions**

- *Bento row-track measurement.* Each brick's span is recomputed by a
  `ResizeObserver` per card. On a very long wall that is one observer per brick;
  it has not been profiled past a few hundred.
- *Glaze filler on a sparse wall.* The muted panel is drawn around the grid
  regardless of how full it is, so a wall with three images shows a lot of grout.
  Open: does it want a minimum brick count?
- *Live region verbosity.* Every pagination batch is announced. On a fast
  connection during a long scroll that is a lot of announcements; whether it
  wants throttling is unmeasured.
