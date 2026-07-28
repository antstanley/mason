# mason

## 0.8.0

### Minor Changes

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - the header bar has room in it again, and a settings screen behind a cog.

  on a phone the layout picker was three segments laid side by side, and it was
  wide enough that two of its own touch targets had been shaved under 44px to fit
  a fourth control beside it. it is a dropdown there now, in the same language as
  the client picker, and every control on the bar is back to a 44px target. from
  sm up it is the slider it always was.

  the bar reads left to right: layout, the wall switcher, refresh, settings. the
  switcher's panel now lists the five feeds you opened most recently, as real
  links, so going back to one is a tap rather than a trip through the picker.

  which client a post opens in has moved to settings, because it is a choice you
  make once and the bar is for what changes while you read. settings is a screen
  held in history state like the reader and the picker: the address bar keeps
  naming the wall behind it and the back gesture closes it.

  two more clients to open posts in: twinkl and witchsky. twinkl spells its
  profile routes differently from everyone else, so mason rewrites the path rather
  than only swapping the host, which is the difference between a link that opens
  and one that 404s.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a brick now opens where it lies. a plain click on a card lifts it into a reader
  over the wall rather than sending you to another app: the same brick mason was
  already holding, at reading width, with the trip to the source demoted to one
  control inside it. the back gesture closes it, so does escape, the close button
  and the scrim, and the wall behind stays dimmed, inert and exactly where you
  left it. a modified click still goes straight out, because every card is still a
  real link.

  the sensitive-media reveal now follows the brick instead of the card, so a brick
  uncovered on the wall is still uncovered one click later in the reader. it is
  still forgotten on reload.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a bluesky feed generator is now a wall mason can lay, and the engine treats it
  as somebody else's algorithm rather than as more bricks to rank. `?feed=` pages
  the generator itself, one appview call per page, and lays what comes back in the
  order it came back: no snapshot, no cohort, no extension waves, no grout and no
  mixer. the wall ends exactly when the feed does. glaze still works over it, and
  asks for the generator's full page so an image wall is a wall of images rather
  than four of them.

  a feed reference is parsed rather than forwarded, in all three spellings people
  actually paste, and one that names no feed says so in its own words instead of
  asking you about your handle.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a link brings its own picture to the wall.

  when a post links somewhere and attached no image of its own, the wall now lays
  the linked page's `og:image` instead of a line of grey text, with the host, the
  headline and the description over the foot of it. an image the poster attached
  still wins: what somebody chose to show beats what a page happened to advertise.

  no new traffic and nothing new on the wire. the AppView already resolves the
  Open Graph tags when a link card is made, and mason has been receiving the
  picture alongside the title and description all along without drawing it.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - mason has a second front door. beside the handle box, and on the switcher of any
  laid wall, there is now a feed picker: a screen for finding a bluesky feed and
  laying it as a mason wall. it browses what the network ranks, remembers the last
  twelve feeds you opened, and takes one box for all three ways of naming a feed.
  what you type decides which question is asked: a name searches, a handle lists
  the feeds that person made, and a pasted feed link or at:// uri lays it. a paste
  that mason cannot lay says so at the box, and nothing navigates.

  it opens over whatever you were looking at rather than taking you off it, so the
  back gesture, escape and the close button all put you back where you were, and
  the address bar keeps showing the wall behind. feeds carrying the hidden
  moderation tier are never listed, and when the appview will not answer the
  picker says so quietly and keeps your recents and the paste box, which need it
  for nothing.

- [#76](https://github.com/antstanley/mason/pull/76) [`07c423a`](https://github.com/antstanley/mason/commit/07c423a80874b9c0dbba643f40547622ff1b3849) Thanks [@antstanley](https://github.com/antstanley)! - the wall pulls on a desktop too. keep scrolling up when it is already at the top
  and it opens the same way it does under a thumb, with the same line to cross and
  the same wall staying open while it lays itself. stop scrolling and it lays: a
  wheel has no letting go, so stopping is how you commit, and the pill says so.

  a flick that merely lands at the top does not refresh anything. the gesture has
  to start from rest, which is the one thing a momentum tail cannot do, and one
  stray notch of a mouse wheel moves the wall a little and costs nothing.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a feed wall now says whose feed it is: the switcher in the header carries the
  generator's own avatar and name, read from the appview the way a wall owner's
  face already is, and falls back to the feed's rkey rather than an empty button
  when the appview has nothing to say.

  a feed link that names no feed also stops asking you to check your handle. it
  gets its own panel, reading "no such feed", and an empty feed wall says "this
  feed has no bricks yet" instead of calling it a wall.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a laid wall is no longer final. one control in the header lays it again in
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

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - the reader's footer says where it is taking you, on one line.

  "open the post" has moved up onto the line the timestamp and the counts already
  had, over on the right, and it now names the client it opens: "open in Bluesky",
  or whichever one is set in settings. it reads the name off the finished link
  rather than off the setting, so a link that is not a bluesky post and therefore
  never gets rewritten does not claim it is opening somewhere it is not.

  the brick counter between "previous brick" and "next brick" is gone. it read as
  progress through the wall and was not: the wall keeps growing while the reader
  is open, so "4 of 22" quietly became "4 of 31" without the reader having moved.
  stepping still announces where it landed to a screen reader, which is a
  different job and the one that number was actually doing.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - a bluesky feed is a wall: `/?feed=<at-uri>` lays a feed generator with the same
  header, the same three views and the same chrome a graph wall gets. the client's
  routing surface is now both parameters, and `feed` wins when a link carries both.

- [#75](https://github.com/antstanley/mason/pull/75) [`baaedf7`](https://github.com/antstanley/mason/commit/baaedf71f3d66ffea75cd8dcbfc2fd6267cd9e69) Thanks [@antstanley](https://github.com/antstanley)! - the wall lays itself again when you pull it. drag the top of the wall down on a
  touchscreen and let go: it is the same refresh the button in the header asks for,
  reached the way a thumb reaches for it. the wall follows your finger, stiffens
  past the point where letting go would lay it, and says which side of that line
  you are on.

  let go and the wall stays open a crack while it lays itself, then closes: the
  gap shutting is how you know it is done. a drag that goes up, or sideways, or
  stops short is a scroll and costs nothing.
  the browser's own overscroll refresh is turned off on the way, because that one
  reloads the page: it would throw away the laid wall, its arrangement, where you
  were in it and anything playing, to fetch what mason already re-reads in place.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - the drawings match the wall again. the refresh change spec is folded into the
  canonical pages and moved onto the merged shelf, so nothing is pending: the feed
  entry point prints the six arguments it actually takes, the snapshot's `refresh`
  sits with the identity it belongs to rather than in the guarded state table, and
  the one sentence about which caches a refresh steps over reads the same on both
  pages that carry it.

  two lane descriptions caught up with the tests behind them: the browser lane is
  five specs now, not one smoke. and the feed-page cache key admits the limit it
  has carried since glaze started asking generators for a hundred posts at a time,
  which is the difference between an image wall and a quarter of one.

- [#75](https://github.com/antstanley/mason/pull/75) [`baaedf7`](https://github.com/antstanley/mason/commit/baaedf71f3d66ffea75cd8dcbfc2fd6267cd9e69) Thanks [@antstanley](https://github.com/antstanley)! - the wall switcher leads with feeds. open it and the door to the feed picker is
  the first thing in it, the only filled control on it, and where the panel puts
  your cursor; the feeds you opened recently sit right under it. the handle box is
  still there, below a rule and quieter, because a handle is still how you reach a
  person's wall.

  it is a better guess about why the panel was opened: you have one follow graph
  and thousands of feeds, and on a phone the old panel opened a keyboard over the
  list you were reaching for. the landing page is unchanged, where your own handle
  is the question being asked.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - the brick reader now has something to read: every image at its own aspect rather than the first one, the post's whole text with its line breaks, the external embed's whole description, a blog's cover at full width with every tag and a "read at" control, and a video's poster with a play button that claims the one player slot for itself, so the card behind the scrim goes quiet. left and right arrows step along the laid wall, and two controls do the same, stopping at both ends.

  a blog that used the same tag twice also stops taking the wall down with it. tags arrive from a document record exactly as it wrote them, repeats included, and both tag lists keyed on the tag itself, so a repeat inside a card's first four laid zero bricks at all.

### Patch Changes

- [#76](https://github.com/antstanley/mason/pull/76) [`07c423a`](https://github.com/antstanley/mason/commit/07c423a80874b9c0dbba643f40547622ff1b3849) Thanks [@antstanley](https://github.com/antstanley)! - the control bar stays with you on a desktop. it used to scroll away with the
  wall, so switching walls or laying this one again meant scrolling back to the
  top of an endless scroll first. it now sits at the top of the window, veiling
  the bricks that pass under it, and it is unchanged on a phone, where it has
  always been at the bottom under your thumb.

- [#74](https://github.com/antstanley/mason/pull/74) [`427668e`](https://github.com/antstanley/mason/commit/427668e7ecd76e564027b3136e1b166db7a67854) Thanks [@antstanley](https://github.com/antstanley)! - a stale build can no longer answer for the tree.

  the browser specs drive the real static site, and the preview server that hosts
  them only serves what is already in `web/build/`. it never compiles. running one
  spec straight after a source edit therefore reported on the build before it:
  green, fast, and about code that no longer existed.

  the run now refuses to start when the build is older than anything it was made
  from, and names the files. this is developer tooling and changes nothing about
  the wall itself.

- [#71](https://github.com/antstanley/mason/pull/71) [`509c6b0`](https://github.com/antstanley/mason/commit/509c6b06504049dee0ac6402b9f8e28377866953) Thanks [@antstanley](https://github.com/antstanley)! - the feed picker no longer offers feeds it cannot lay. a feed whose ranking is
  about who is asking, your mutuals, your mentions, the people you follow who post
  rarely, has no viewer to be about when nobody is logged in, and tapping its card
  gave "the wall wouldn't load" rather than a wall. those cards are gone, from
  browse, from search, from a creator's own list, and from recents.

  feeds are denied by name, because the same viewer-shaped feed ships from several
  publishers and a list of addresses would hide one copy and leave the rest. where
  a name is one a working feed could also carry it is pinned to its publisher
  instead: skyfeed's "Discover" is gone and bluesky's, which is the discover
  everybody means, is not.

- [#75](https://github.com/antstanley/mason/pull/75) [`baaedf7`](https://github.com/antstanley/mason/commit/baaedf71f3d66ffea75cd8dcbfc2fd6267cd9e69) Thanks [@antstanley](https://github.com/antstanley)! - the feed picker only offers feeds that actually lay a wall. every one of the top
  fifty popular feeds was asked for a page logged out, and the eleven that answer
  with an error or a handful of posts are no longer listed: a card that opens onto
  three bricks reads as broken whether the feed is gated or just quiet.

  it cuts the other way too. two feeds mason was hiding turned out to work fine
  logged out, and they are back: the rule was keyed to their names, and the names
  belong to several publishers. now anything that could plausibly work is measured
  per publisher rather than assumed, and `pnpm feeds:audit` re-derives the whole
  list against the live directory.

- [#73](https://github.com/antstanley/mason/pull/73) [`f263791`](https://github.com/antstanley/mason/commit/f263791ae1ba30c133faa3a166e061f7c7cc32ac) Thanks [@antstanley](https://github.com/antstanley)! - the session keeps the last twelve walls, not every wall you ever opened.

  mason remembers a laid wall so stepping back to it returns the same arrangement
  instead of rolling a new one. it remembered every wall, for as long as the tab
  stayed open, and an entry is a whole wall: every brick, its text, its image
  urls, and a record of every id it has already laid.

  that was fine when a wall was somewhere you arrived. the feed picker made
  hopping between them a one tap habit, and nothing ever took an entry away. the
  last twelve distinct walls are kept now, which is the length of the picker's
  recents row, so every wall that row can offer you in one tap is one that comes
  straight back. least recently used is the first out, so returning to a wall
  keeps it and the one you glanced at once is the one that goes. stepping back and
  forth between two walls costs two entries however long it goes on.

## 0.7.0

### Minor Changes

- [#57](https://github.com/antstanley/mason/pull/57) [`fd4700a`](https://github.com/antstanley/mason/commit/fd4700a630949206f21450b72ee213fa425631fc) Thanks [@antstanley](https://github.com/antstanley)! - the wall extends itself: when the scroll runs the pool low, mortar fans out to the next hundred authors it has never asked, in waves, until the whole follow graph is spent. endless scroll now quarries the entire graph instead of ending at the first cohort, and the wall only says it is done when there is genuinely nobody left to ask.

### Patch Changes

- [#55](https://github.com/antstanley/mason/pull/55) [`dd676fb`](https://github.com/antstanley/mason/commit/dd676fbf898c164747cddbf7b6807618d61f557f) Thanks [@antstanley](https://github.com/antstanley)! - the wall now fits in a pocket: the bottom-bar controls grew visible labels and thumb-sized targets and hold a single line on any phone, and the client picker says where posts open instead of hiding behind a glyph. empty walls explain themselves ("this wall has no bricks yet") and a paused feed offers "try for more" instead of silence. bricks with zero likes and reposts stop bragging about it, card titles breathe a little more, column widths no longer animate on reflow, and the glaze caption toggle closes as readily as it opens.

## 0.6.4

### Patch Changes

- [#44](https://github.com/antstanley/mason/pull/44) [`68f6fbd`](https://github.com/antstanley/mason/commit/68f6fbdaaaf18a216f6fc463bb399e87a29f33d5) Thanks [@antstanley](https://github.com/antstanley)! - scope the WCAG 2.2 AA claim honestly (video captions excepted pending upstream data) and reserve a caption-track path: VideoBrick grows an optional captions list and the player renders track elements when it is populated

- [#43](https://github.com/antstanley/mason/pull/43) [`38fd727`](https://github.com/antstanley/mason/commit/38fd727f2b403f5cf7ad64c1e6315cde1c8490e7) Thanks [@antstanley](https://github.com/antstanley)! - a deploy no longer hard-reloads an active tab: the reload waits until the tab is hidden or the next navigation, so the wall, scroll position, and playing video survive

- [#47](https://github.com/antstanley/mason/pull/47) [`d5ab5a9`](https://github.com/antstanley/mason/commit/d5ab5a982d5abf4fe95f3c0bbb642971d028f6fa) Thanks [@antstanley](https://github.com/antstanley)! - persist only the caches that changed, each under its own key, and never during preview polls

- [#48](https://github.com/antstanley/mason/pull/48) [`3237320`](https://github.com/antstanley/mason/commit/323732065c6d502e0205d1f75c0a7153e1571e72) Thanks [@antstanley](https://github.com/antstanley)! - masonry lays one keyed list in feed order: column changes keep every brick's node, state and focus, re-lays are a single pass, and tab order follows the feed

## 0.6.3

### Patch Changes

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the glaze alt-text overlay now works with a keyboard alone: opening it moves
  focus onto the close control, escape or the close button hands focus back to
  the alt trigger, the trigger reports its state with aria-expanded and
  aria-controls, and the covered picture and its paging controls go inert so tab
  and pointer stay on the panel.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the glaze filmstrip is friendlier to keyboard and screen-reader users:
  tabbing to the prev/next arrows reveals them instead of moving focus onto
  invisible buttons, and a polite live region announces "image n of m" as the
  strip pages.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - keyboard users on the glaze wall now see a brick's caption, author, and alt
  button: focusing into a card reveals them the same way hovering does, and the
  concealed alt button leaves the tab order until the card is revealed, so focus
  no longer lands on an invisible control.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: switching between bento and masonry no longer throws away your loaded wall.
  Only picking (or leaving) glaze re-fetches, since glaze is a different feed; the
  two grid-only layouts now just relay the same bricks instead of wiping the wall
  and refetching from the top.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - darken the glaze brick's floating controls so their text clears the 4.5:1
  contrast bar over any image: the paging arrows, the image counter, and the
  touch reveal button sit on a heavier ink scrim, and the caption bar is now
  near-opaque like the alt panel.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: a valid handle no longer gets told it does not exist. mason now only shows
  the handle-not-found message when the feed engine actually reports the actor is
  missing, rather than on any 404. In local mode a request that slips past the
  service worker onto the static host used to 404 and wrongly accuse a real
  handle; that case now reads as a plain feed-unavailable hiccup.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - tidy the wall's headings for screen readers: a failed wall now shows exactly one
  h1 (the failure itself, with the sr-only wall title stepping aside) instead of
  two, and every brick, not just blogs, carries a consistent accessible name on
  its article rather than a lone heading on some cards.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - brick links are scrubbed before they reach the wall: a blog or stream record
  that smuggles a `javascript:`, `data:`, or other non-http(s) url in its link
  field now lands without a link at all instead of arming the card, and the client
  picker refuses to rewrite anything but a real http(s) address.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: the wall no longer hangs on an endless skeleton when the service worker
  fails to register. Waiting for the worker to be ready is now bounded by the same
  short timeout as everything else, so a registration that never settles falls
  through to a normal request instead of leaving you staring at a loading wall
  forever.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the switch-wall button's aria-label dropped its em dash for a comma, so screen
  readers announce "switch wall, currently viewing @handle" instead of reading the
  dash aloud.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the switch-wall panel is now a proper modal: it marks itself aria-modal and
  closes when focus tabs out of it, so keyboard users no longer walk into the
  dimmed wall behind the open switcher.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - server mode now checks where a PDS pointer actually leads before it knocks: a
  DID document can no longer steer mortar at loopback, private, link-local, or
  cloud-metadata addresses, and only https endpoints are followed. hostile bricks
  stay outside the wall.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: scrolling a playing clip off the wall before it finishes loading no longer
  leaves a phantom playing in the background. If a video card is torn down while
  the player library is still loading, the player now bows out instead of building
  itself on a detached brick and quietly fetching segments (and audio) with nothing
  left to stop it.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the wall now narrates itself to screen readers: a single polite live region
  reports laying bricks, how many fresh bricks landed, when more bricks did not
  arrive, and when that is every brick, the pagination-failure tail is a status
  region, and the wall is marked aria-busy while the first screen loads.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: going back to a wall you already scrolled now returns it exactly as you left
  it. mason keeps each wall you visit for the length of the session, so browser
  back/forward rehydrates the same bricks in the same order (and your scroll lands
  where it should) instead of rolling a fresh arrangement that dropped you back at
  a single screen.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - fix: switching walls mid-load no longer bleeds the old wall into the new one.
  When you jump to a different wall while the first one is still fetching, a late
  response from the old wall could shove its bricks into the new wall, overwrite
  the cursor, or wrongly mark pagination finished (a wall stuck at one screen).
  Both the load-more and freeze steps now bail out when the wall has moved on, and
  resetting a wall clears the loading flag so the fresh wall starts clean.

- [#27](https://github.com/antstanley/mason/pull/27) [`856271e`](https://github.com/antstanley/mason/commit/856271ee572594122dae19c3cb4e450d7d7cde76) Thanks [@antstanley](https://github.com/antstanley)! - the warming reflow now stops for everyone: navigation keys and focus landing on
  the wall freeze it just like a scroll or a swipe does, and a reader who asks for
  reduced motion never sees the auto-reflow at all because the wall freezes before
  it starts moving.

## 0.6.2

### Patch Changes

- [#24](https://github.com/antstanley/mason/pull/24) [`17284a8`](https://github.com/antstanley/mason/commit/17284a8204d46c235a85d31d11b74bff96cfa89b) Thanks [@antstanley](https://github.com/antstanley)! - perf: a cold wall paints sooner. Five changes to the opening of a wall, none of
  which move a brick once it is laid:

  - one `getProfile` now resolves the handle AND reads the owner's logged-out
    opt-out, folding the two sequential AppView calls that gated every cold load
    into one round trip.
  - the first wall waits for a single follow-graph page (100 follows, already more
    than the cohort samples) instead of three, so the fan-out starts two round
    trips sooner. The rest of the graph is still chased in the background.
  - the first page's wait-for-a-better-mix deadline is now anchored to when the
    snapshot was created, so the first-paint wait counts against it rather than
    stacking on top of it: the opening wait is bounded, not doubled.
  - the landing page warms the engine while you are still at the form: a
    remembered handle warms that wall's caches, and with no handle the demo wall
    at least compiles the wasm off the critical path.
  - the roughly-first-screen bricks load their images eagerly and at high fetch
    priority; the rest of the wall stays lazy.

- [#23](https://github.com/antstanley/mason/pull/23) [`287f40c`](https://github.com/antstanley/mason/commit/287f40c63af83f3f0aade963d4a7552d3cf8e931) Thanks [@antstanley](https://github.com/antstanley)! - feat: glaze cards get a reveal control on touch. Where there is no hover to lift
  the author pill and caption, a small transparent double-chevron button now taps
  them up (and back down); it rides on the author pill's line, sitting at the
  bottom-right at rest and rising with the pill when the caption lifts. On hover
  devices nothing changes; the pill and caption still reveal on hover.

- [#25](https://github.com/antstanley/mason/pull/25) [`e9399c7`](https://github.com/antstanley/mason/commit/e9399c7bea9655ae6095273f8618b666dc4abef8) Thanks [@antstanley](https://github.com/antstanley)! - feat: the wall reflows as it fills, then freezes when you reach for it. Instead
  of waiting for a full first page and dropping it in one block, a cold wall now
  paints the moment the first bricks arrive and re-mixes the first screen as blogs,
  videos and live streams land behind the posts. The instant you scroll (or the
  wall settles, or a few seconds pass) the arrangement locks and normal scrolling
  takes over; from there a brick never moves. The mixer is pure, so each reflow is
  a real improvement rather than a reshuffle, and pages after the first are laid
  once and immutable exactly as before. Server mode and the demo wall keep working
  unchanged.

- [#21](https://github.com/antstanley/mason/pull/21) [`82ea0d0`](https://github.com/antstanley/mason/commit/82ea0d0340ad704dfa72a8e3bce9ff903fc2ebc4) Thanks [@antstanley](https://github.com/antstanley)! - fix: the menu bar stays on one line now that Glaze is a third layout, and the
  layout slider's thumb hugs each option. The segments size to their own content
  and the thumb measures the selected label and matches its width and position, so
  a short label like Glaze no longer leaves dead space inside the white highlight.
  On mobile the slider is icon-only and the client picker drops to just its icon
  (sized to match the layout icons; its label and chevron return at the sm
  breakpoint), keeping the bar to one row.

- [#26](https://github.com/antstanley/mason/pull/26) [`f1b3255`](https://github.com/antstanley/mason/commit/f1b3255f4e22a09ae9dda372f7ddbd6c0f82036b) Thanks [@antstanley](https://github.com/antstanley)! - perf: the browser engine is 103 KB smaller. The wasm build talked to the network
  through reqwest, which on wasm is only a thin wrapper over the browser's own
  fetch, but it dragged the `url` crate's IDNA/ICU Unicode tables in with it, none
  of which mason needs (every request URL is a plain ASCII atproto endpoint). The
  browser build now uses gloo-net instead, a direct fetch wrapper with no such
  tail. reqwest stays on the native server unchanged. The shared rate limiter and
  429/5xx retry loop are untouched; only the one-shot GET underneath is split by
  target. The shipped wasm drops from 389 KB to 286 KB gzipped (270 KB off the
  raw binary), so a cold start downloads and compiles less on the very path that
  gates first paint.

## 0.6.1

### Patch Changes

- [#19](https://github.com/antstanley/mason/pull/19) [`591ce4c`](https://github.com/antstanley/mason/commit/591ce4cb1c2c7964ec1f3722660c970bc592641b) Thanks [@antstanley](https://github.com/antstanley)! - fix: the wasm service worker survives a deploy. Each deploy deletes the previous
  build's hashed assets, so a worker that installed before a deploy would 404
  fetching its old wasm engine and then brick every `/api/feed` for the life of
  that session (the rejected init was memoised): a 500 that only hit visitors who
  had loaded the app before. The worker now precaches its own wasm and loads the
  engine from that cache, so it keeps serving until it is itself replaced; a failed
  init is never memoised. The client also revalidates the worker script on load
  (`updateViaCache: 'none'` + `update()`) and reloads once when a new engine takes
  control, so a deploy is picked up promptly instead of leaving a stale worker.

## 0.6.0

### Minor Changes

- [#17](https://github.com/antstanley/mason/pull/17) [`b2776d5`](https://github.com/antstanley/mason/commit/b2776d59a25b22fc2f6cde20175c958a00172eb9) Thanks [@antstanley](https://github.com/antstanley)! - glaze: an image wall. a new menu toggle (off by default) flips the wall to
  nothing but Bluesky posts that brought an image, fetched from the AppView and
  ranked by the same grout the full wall uses, with moderation and `!warn` blur
  intact. bento and masonry both lay it. glaze bricks lead with the picture and a
  clear strip carrying the poster; hover one and the post's text scrolls up over
  the image (a slim footer on touch, where there is no hover).

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
