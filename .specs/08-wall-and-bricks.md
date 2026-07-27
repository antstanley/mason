# 08 - The Wall and Its Bricks

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

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
6. Open any brick in place, at full size, without leaving the wall.

---

## Layouts

One control (`LayoutPicker`) offers three views, and all three work on either
wall source. `glaze` is also an algorithm: picking it sets `mode=glaze`, which
on a graph wall re-fetches an images-only wall and on a feed wall filters the
feed's own posts to those carrying an image.

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
- **Every outbound anchor stays a real link, and the anchor that stands for the
  brick opens the reader on a plain click.** Anchors keep `target="_blank"` and
  `rel="noopener noreferrer"`, and the Bluesky ones keep their `clientUrl`
  rewrite, so middle-click, cmd-click and "copy link address" all still reach
  the source. A blog anchor points at the publication directly and always did:
  `clientUrl` rewrites `bsky.app` hostnames and nothing else, so a blog card has
  never had one to keep. A plain unmodified left click is intercepted
  (`preventDefault`) and opens the brick reader. Keeping the anchor rather than
  swapping it for a button is what preserves the browser's own affordances.

  `GlazeCard`'s existing rule that a tap opens the post while a drag scrolls
  the strip still holds; the tap now opens the reader instead.

  **Which anchor stands for the brick differs by card, and there are three of
  them.** Only post and blog cards hand `BrickShell` an `href`, so only they
  have a card-wide link: a video card's one anchor is its watch-at-source link,
  and a glaze card's are per image. All three call the same
  `reader.activate(event, brick)`, so the modifier-key rule lives in one place.
  Intercepting `BrickShell` alone would leave the entire glaze wall and every
  video brick with no way into the reader.

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
brick in the reader while a drag scrolls the strip and the arrows never trip
navigation. At rest the card is just the picture; on hover an author pill and
the caption fade in. Touch has no hover, so there they stay shown.

When any image carries a description, the caption bar gains an **ALT** button.
Opening it behaves like a small dialog: focus moves to the close control, Escape
or the close button returns focus to the trigger, and the covered strip is
`inert` while it is up.

---

## The brick reader

`BrickReader` renders one brick at reading width over a dimmed wall. It is
mounted once in `+layout.svelte`, so the header dims with everything else, and
it renders nothing at all unless the reader is showing a brick: `page.state.brick`
naming one the `reader` rune is still holding (see
[07-web-client.md](07-web-client.md)).

It shows what the card had to leave out. Nothing here is fetched: the reader is
the same `Brick` the card was given.

It shows that brick and nothing around it. No replies, no thread context, no
parent post: the reader is one brick, larger. That boundary is what keeps it a
rendering change rather than a new surface, and it is stated here because a
reader-shaped overlay is exactly where somebody will assume a conversation
lives.

| Kind | What the reader adds over the card |
|---|---|
| Post | Every image, each at its own aspect ratio, rather than the first one; the full text unclamped; the external embed with its whole description; the timestamp |
| Post (from glaze) | The same reader. A glaze brick is a post brick, so its filmstrip becomes a stack and every alt text is readable |
| Video | The poster with a play button that mounts `VideoPlayer` exactly as the card does; title, activity or viewer count, runtime, author |
| Blog | The cover at full width, the publication chip, the whole description, every tag rather than four, the published date, and "read at <publication>" as the primary control |

A blog's article body is still not rendered, and the reader is where that
limit is most visible: the content union of `site.standard.document` is
platform-specific and mason never parses it (see
[00-overview.md](00-overview.md)). The reader answers that by making the
outbound link the primary action on a blog rather than a footnote.

### Dialog behaviour

The reader is a modal dialog in the same language as `SwitchWall`:
`role="dialog"` with `aria-modal="true"`, labelled by the brick's own heading
or its author line.

- Focus moves into the reader on open and returns to the card that opened it
  on close, so a keyboard reader lands back where they were on the wall.
- Escape closes. So does the close control, a click on the scrim, and the back
  gesture (the reader is a history entry).
- The layout's content wrapper is `inert` while the reader is up, which is what
  traps focus. The reader is mounted as that wrapper's **sibling**, not inside
  it, because `inert` applies to an element and all of its descendants: a reader
  nested in the wrapper it dims would be unfocusable and invisible to assistive
  tech the moment it opened.
- `document.documentElement` gets `overflow: hidden`, which stops the wall
  scrolling behind the reader. It does not stop the pump: the sentinel's
  observer stays connected, so bricks may still be appended behind an open
  reader. That is harmless, because an append never moves a laid brick and the
  reader locates its own brick by id.
- Left and right arrows step to the previous and next brick on the wall, and
  two visible controls do the same. Stepping stops at the ends of the laid
  wall; the reader never triggers pagination. A step swaps the whole panel, so
  it keeps focus inside the reader and announces the brick's new position
  politely.
- One video plays at a time, network-wide, unchanged, but the reader must claim
  the player under **its own id** (`reader:<brick.id>`). A card collapses back
  to its poster only when `player.activeId` stops matching its own brick id, so
  a reader claiming the same id would leave the card mounted and playing behind
  the scrim, two elements and two audio streams. Claiming a distinct id makes
  the card a loser of the claim and tears it down through the path that already
  exists. `VideoPlayer` remains the only sanctioned `.play(`, and
  `just guard-autoplay` still holds over the reader.

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

The choice is per brick and keyed by brick id in the `revealed` session set, so
a brick uncovered on the card is still uncovered in the reader and on a
re-place. It is forgotten on reload, by design: the set lives in a rune, never
in storage, so there is no lingering "show everything" switch.

On the post card and on the glaze card's single and grid branches the reveal
button sits *inside* the card's own anchor, so it stops the click twice:
`stopPropagation` keeps it away from the reader's activation handler on that
anchor, and `preventDefault` keeps the anchor's `href` from opening the source,
which propagation alone cannot reach because the browser gates that navigation
on `defaultPrevented` rather than on the listener chain. Together they keep
"show anyway" a reveal and only a reveal.

---

## Wall states

`FeedGrid` is the single component that decides what the wall is doing.

| Condition | What the reader sees |
|---|---|
| `initialLoad` | A 12-card skeleton grid at the same column count as the real wall |
| `error` and no items | An error panel: sealed, handle-not-found, no-such-feed, or unreachable, each with the right recovery |
| `done` and no items | "this wall has no bricks yet", with a handle box. An empty wall is a site, not a dead end |
| items present | The wall, plus a sentinel, plus a tail |
| `error` with items | A retry button: "more bricks did not arrive" |
| `stalled` | "the wall paused", with a "try for more" button. Scrolling retries too |
| `warming` or loading or pumping | A 4-card skeleton tail |
| `warming` with items already laid | The wall the reader was reading, reflowing into the refreshed one, with the usual four-card skeleton tail beneath it. Never the twelve-card initial grid, which is `initialLoad` only and a refresh does not set it |
| `done` | "that is every brick. the wall is finished." |

Both recoverable errors drop the reader into a handle box, because a wall you
cannot see is still a door to another one. The typo keeps its text to fix; the
sealed wall clears it, because there is nothing to correct, only somewhere else
to go. Both focus and select the input on arrival.

A feed wall changes the chrome, not the wall. `SwitchWall` shows the feed
generator's avatar and name where it would show the owner's face, and the empty
state reads "this feed has no bricks yet". Every other state (the skeletons, the
stall, the ending, the three error panels) is unchanged, and all three views are
offered.

The fourth panel is the one a feed wall adds, and it deliberately offers neither
handle box nor retry: a bad feed link is not a mistyped handle, and a retry
cannot repair a feed that is not there. The way on is the header's switcher,
which on a feed wall is also the door to the feed picker, plus the demo link
every panel carries.

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

### Refreshing

`RefreshWall` sits in the header beside the layout and client pickers: one
control, no confirmation, no count.

**It does not scroll.** The wall re-lays under the reader, wherever they are.

This is a product choice and not a mechanical necessity, and it is worth being
precise about which, because the mechanics nearly went the other way. A
programmatic scroll is indistinguishable from a reader reaching for the wall:
`window.scrollTo` moves the position synchronously but queues its `scroll`
event, and that event is delivered *after* the microtask that re-runs the
freeze effect and arms its listeners. Before the in-flight marker existed, a
refresh that scrolled therefore froze itself on its own scroll in either call
order, and committed the pre-refresh wall. The marker removes that: a freeze
during a refresh is deferred until the flagged preview lands, whoever triggered
it, so a scroll is now safe.

What is left is the reason the control still does not scroll: the outgoing wall
stays on screen and reflows into the new one, and that is a thing the reader has
to be looking at to see. Jumping them to the top is not wrong, it just throws
the reflow away. The alternative is cheap now, and it is an open question below
rather than a closed door.

Under `prefers-reduced-motion: reduce` there is no scroll in it at all, and a
refresh is still exactly one request out: `FeedGrid` freezes the instant
`warming` flips true, with no listener attached and no scroll event to wait for,
so a reduced-motion refresh is a cursorless preview carrying the flag plus a
freeze that the in-flight marker holds until that preview's cursor has been
adopted. One cursorless request, one refreshed fan-out, and one reflow when the
preview lands.

The control closes an open reader before it asks for the wall, because a refresh
replaces `feed.items` wholesale and the reader locates its open brick in that
list by id. Today no click can reach that line, since an open reader makes the
content wrapper `inert` and this control sits inside it, so the call is the
guarantee held for the next trigger rather than a live path.

It is disabled while the feed is loading or warming, and that is the whole rate
limit: one refresh can be in flight, so a double tap cannot start two
hundred-author fan-outs. The wall keeps its single polite live region, which
already says "laying bricks" while warming; a refresh is a warm, so it needs no
new announcement and `RefreshWall` adds no region of its own. The announcement
of *new* bricks is suppressed for free, because the region's count resets while
warming so a reflow's churn never reads as new bricks.

---

## The feed picker

A handle assumes its own answer: it asks the reader who they already want to
read. The feed picker is mason's other front door, and it is a screen rather
than a field, standing beside the handle box as a peer.

`FeedPicker` is reached two ways: from the landing page, where it sits under the
handle box, and from `SwitchWall` on a laid wall, which is already the
switch-walls affordance. Either way it opens over the current content and is held
in history state (`pushState('', { picker: 'feeds' })`), so the back gesture
returns the reader to where they were instead of out of mason.

Five ways to reach a feed, because they answer five different states of knowing
what you want:

| Section | Source | Shown when |
|---|---|---|
| Recent | `mason:feeds` in `localStorage`, most recent first, capped at 12 | The reader has opened a feed before |
| Search | `app.bsky.unspecced.getPopularFeedGenerators?query=<q>` | The reader types a search term |
| By creator | `app.bsky.feed.getActorFeeds?actor=<handle>` | The input parses as a handle |
| Popular | The same popular endpoint with no query, paged by its cursor | Always, as the resting state |
| Paste | The value is handed to mortar as `?feed=`, which parses it | The input is a `bsky.app` feed link or an AT-URI |

One input serves search, creator and paste: what the reader typed decides which
question is being asked. That is the same input the handle box takes, and **by
creator is the bridge between the two front doors**: a handle here means "show me
the feeds this person made" rather than "show me their wall".
`getActorFeeds` accepts a bare handle, so it costs no resolution hop.

Each result is a card carrying the feed's avatar, its display name, its
creator's handle, its description clamped to two lines, and its like count
(hidden at zero, like every tally on the wall). Activating one opens
`/?feed=<uri>` and writes it to `mason:feeds`.

### The picker reads the AppView directly, and that is chrome rather than moderation

The picker is a directory listing, so it asks the public AppView from the browser
exactly as `profile` already does for a wall owner's avatar. Content moderation
stays where it belongs: every brick a chosen feed yields is filtered by mortar
when the wall is laid.

The picker does owe one check of its own, because a feed generator's own record
carries labels. A feed whose view or whose creator carries the hidden tier is not
listed, so mason never advertises a feed it would then refuse to lay properly.
That puts the hidden-label list on both sides of the wire, which is normally the
shape of a bug, so it is pinned rather than copied: the contract fixture carries
`vocab.hiddenLabels` as object keys and `contract-check.ts` asserts the
TypeScript list equals it, the same mechanism that already keeps the error codes
and the video sources in step (see [06-wire-contract.md](06-wire-contract.md)).

### States

| Condition | What the reader sees |
|---|---|
| Loading | Six card skeletons at the picker's own column count |
| A search with no results | "no feeds by that name", and the paste hint |
| A handle with no feeds | "that person has not made any feeds", with a link to lay their wall instead |
| The AppView unreachable | Recents and the paste box, plus a quiet line saying browsing is unavailable. Paste is the load-bearing path and always works |
| A pasted value that will not parse | The input says so in place; nothing navigates |

The picker is a dialog in the same language as mason's other overlays:
`role="dialog"`, `aria-modal="true"`, focus into the input on open, Escape and
back to close, `inert` behind it, and the results as a list so a screen reader is
told how many there are. It mounts as a sibling of the layout's content wrapper,
beside the brick reader and outside the subtree it dims, and it shares that
wrapper's `inert` condition rather than adding a second one: the wrapper is
inert while **either** overlay is open. Opening one closes the other, because
the picker is a landing surface and the reader is a wall surface, and neither
has anything to say over the top of the other. Touch targets stay at 44px and
the cards' hover lift is `motion-safe:`, like a brick's.

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
  switcher. `RefreshWall` is a plain `<button>` with an accessible name and a
  disabled state that is real rather than styled, so a screen-reader user is
  told the wall is already refreshing instead of pressing a control that
  silently does nothing.
- **Touch targets are at least 44px** (`min-h-11`) on every control, including
  the ones that shrink to icons on mobile.
- **The reader is a real dialog.** `role="dialog"`, `aria-modal="true"`, an
  accessible name from the brick, focus in on open and back to the opening card
  on close, Escape and click-away, and `inert` on everything behind it (with the
  reader mounted outside the inert subtree). Left and right arrows step along
  the wall from inside it, and they cannot collide with the wall's own
  navigation-key freeze handler because the two key sets are disjoint: that
  handler matches the vertical set (`ArrowDown`, `ArrowUp`, `PageDown`,
  `PageUp`, `Home`, `End`, space) and never the horizontal one. The disjointness
  is what carries this, not the freeze: `feed.freeze()` is async and does not
  clear `warming` until its fetch resolves, so the wall's listeners are still
  attached while the reader mounts.
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
    BrickReader.svelte       one brick at reading width, over the wall
    VideoPlayer.svelte       the ONE sanctioned .play()
    LandingWall.svelte       the inert demo wall behind the handle form
    FeedPicker.svelte        the second front door: browse, search, paste
    FeedCard.svelte          one feed generator as a result
    HandleForm.svelte        the front door
    SwitchWall.svelte        the owner's face as a wall switcher
    LayoutPicker.svelte      bento / masonry / glaze
    RefreshWall.svelte       lay this wall again, now
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
- *Whether a refresh should return the reader to the top.* It does not, and this
  is a live choice rather than the constraint it started as: the in-flight
  marker defers any freeze during a refresh, including one a programmatic scroll
  would trigger, so scrolling is no longer self-defeating. The cost is only that
  a reader taken to the top does not watch the reflow they asked for. Open: a
  reader deep in a long wall probably wants the top, and a reader who tapped
  refresh to see what is new probably does not.
- *Pull to refresh.* A touch reader's instinct is to drag down, and the wall has
  nothing there. It competes with the browser's own overscroll refresh and needs
  a gesture threshold, a rubber band and a reduced-motion path. Open.
