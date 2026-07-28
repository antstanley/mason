# Change: Read a brick without leaving the wall

**Status:** Merged · **Date:** 2026-07-26 · **Merged:** 2026-07-27 · **Owner:** Ant Stanley · **Target:** Repo-wide

Every brick on the wall is a link out. A card is a summary by design (a post card
renders its first image and nothing more, a blog card clamps its description to
three lines, an external embed's title is truncated), and the only way to see the
whole thing is to leave mason for bsky.app or the reader's chosen client. This
change adds a **brick reader**: an in-place overlay that renders the brick mason
already holds at full size and reading width, with the outbound link demoted to
one control inside it. No new upstream read and no wire change: every brick on
the wall already carries its whole text, every image, its embed, its counts and
its author.

---

## Motivation

The wall is a summary surface and it should be. Bricks are small so many of them
share a screen, which is the whole point of a masonry wall. But the cost today is
that engaging with any one of them means a full navigation to another app: the
laid arrangement, the scroll position, the warm engine and any playing video all
go, and coming back is a fresh wall. A reader who wants to look properly at one
brick pays for it with the wall they were reading.

Nothing about that is a data limit. `FeedResponse` already carries a post's full
text, all of its images with their aspect ratios and alt text, its external embed
with the full description, and its tallies. The card shows a slice of that
because a card is small, not because that is all mason knows. Leaving the app to
read what the app is already holding is a routing accident, and closing it needs
no engine work at all.

---

## Affected spec pages

No wire change: `FeedResponse`, `Brick` and the cursor are untouched, so there is
no `Type changes` section below and no contract fixture to regenerate. One Rust
file is still edited (`fixtures.rs`, to give a demo brick a `!warn` blur), because
otherwise the shared-reveal half of this change has nothing that can observe it.

| Canonical page | Nature of change |
|---|---|
| [`.specs/07-web-client.md`](../../07-web-client.md) | A `reader` rune singleton, shallow-routing history state, and the freeze-on-open rule |
| [`.specs/08-wall-and-bricks.md`](../../08-wall-and-bricks.md) | Cards activate the reader; a new `BrickReader` section; the `Sensitive` reveal becomes shared; implementation layout |
| [`.specs/09-design-system.md`](../../09-design-system.md) | The reader's motion and its reduced-motion alternative |
| [`.specs/00-overview.md`](../../00-overview.md) | Goals gain reading in place; the blog-content non-goal is restated precisely |

---

## Proposed changes

Each block below is the prose as it should read once merged, so links *inside* a
quoted block are relative to the canonical page it lands on, not to this
directory. Links outside the blocks resolve from `.specs/changes/` as usual.

### `.specs/00-overview.md` → Goals (Add)

> 8. Let a reader read a brick without leaving the wall: an in-place reader
>    renders the brick's own content at full size, and the trip to the source
>    becomes a choice rather than the only option.

### `.specs/00-overview.md` → Non-goals (Modify)

Replace the **No blog content rendering** bullet with:

> - **No blog content rendering.** A blog brick is metadata plus a link out,
>   on the wall and in the reader alike. The `site.standard.document` content
>   union is platform-specific (Leaflet, pckt.blog, Offprint and WordPress all
>   differ) and is never parsed, so the reader shows a blog's metadata at full
>   size and makes the publication the primary destination.

### `.specs/07-web-client.md` → Responsibilities (Add)

> 5. Hold the open brick as history state, so a reader opens one in place and the
>    back gesture closes it.

### `.specs/07-web-client.md` → Reactive state (Modify)

Add two rows to the table:

> | `state/reader.svelte.ts` | `reader` | The brick being read in place; its position on the wall is derived by id | none (history state) |
> | `state/sensitive.svelte.ts` | `revealed` | Brick ids whose `!warn` media the reader uncovered | none (session set) |

### `.specs/07-web-client.md` → Reactive state → The reader is history, not a URL (Add)

> **The reader is history state, not a URL.** Opening a brick calls SvelteKit's
> `pushState('', { brick: id })`, so the address bar keeps showing `?actor=` and
> the back button (and the phone's back gesture) closes the reader instead of
> leaving the wall. `page.state.brick` is the single source of truth for whether
> the reader is open, declared as `App.PageState` in `app.d.ts`; `reader` holds
> the brick itself and derives its position on the wall from it when it needs to
> step.
>
> A URL parameter was the alternative and it cannot be honest here. A
> `?brick=<at-uri>` link would be shareable and would not work: the recipient's
> wall is built from a different seed, that brick is almost certainly not on it,
> and mason has no way to fetch one brick by id. History state promises exactly
> what it delivers, and leaves `?actor=` as the only thing a shared link carries.
>
> **Opening the reader freezes the wall.** A click is engagement, so
> `reader.open` calls `feed.freeze()` before anything else. That is not only
> consistency with the wheel, touch and focus signals: while the wall is warming
> the arrangement reorders between preview polls, so a position on the wall would
> go stale under an open reader.
>
> **The reader holds the brick, and locates it by id.** `reader` stores the
> `Brick` itself and derives its position with
> `feed.items.findIndex(b => b.id === id)` when it needs to step. Threading an
> index down from the wall was the alternative and it is worse in every direction:
> `FeedGrid`'s `brick` snippet takes `(item, priority)` and nothing else, so an
> index would mean editing that signature, both layout components' prop types,
> both render call sites, all four cards and `BrickShell`, none of which any lane
> typechecks. Locating by id is O(n) once per click on a list of hundreds, it
> keeps `FeedGrid` out of this change entirely, and a replaced or reordered
> `feed.items` returns -1 rather than silently pointing at the wrong brick.
>
> Page state does not survive a reload, so reloading with a reader open reopens
> the wall without it. That is the intended behaviour rather than a gap to work
> around: the reader is a view of a brick on a wall that is itself being rebuilt.

**Merged as:** everything above landed except the claim that `page.state.brick`
is *the single source of truth for whether the reader is open*, which the build
proved wrong. A history entry outlives the rune that pushed it (open, back,
reload, forward, and the entry comes back with its id while the rune is empty),
so what shipped is `ReaderState.showing`: page state naming a brick **and** the
rune holding that same brick, with `isOpen` as its boolean. Page state alone
would make the layout's wrapper `inert` under a reader rendering nothing, which
is the wall frozen under nothing at all. The canonical page carries the shipped
predicate instead of this clause.

### `.specs/07-web-client.md` → Decisions (Add)

> - *The reader is history state, not a URL.* **`pushState('', { brick })`.** The
>   back gesture has to close an overlay on a phone, and a `?brick=` link would
>   advertise a deep link mason cannot serve: a single brick is not fetchable, so
>   the recipient would get the wall and a dropped parameter.
> - *Opening the reader freezes the wall.* **A click is engagement.** The reader
>   locates its brick in `feed.items` by id, and a warming wall reorders between
>   polls.
> - *No index is threaded down to the cards.* **The reader locates the brick by
>   id.** An index would edit the snippet signature, both layouts, all four cards
>   and `BrickShell` to save a `findIndex` over a few hundred items, and every one
>   of those files is invisible to `tsc`.
> - *The reveal choice follows the brick.* **A session set of brick ids, shared
>   by the card and the reader.** Uncovering a brick on the wall and finding it
>   covered again one click later reads as a bug. It is still forgotten on
>   reload: the set lives in a rune, not in storage.

### `.specs/08-wall-and-bricks.md` → Responsibilities (Add)

> 6. Open any brick in place, at full size, without leaving the wall.

### `.specs/08-wall-and-bricks.md` → Cards (Modify)

Replace the **Outbound links** bullet with:

> - **Every outbound anchor stays a real link, and the anchor that stands for the
>   brick opens the reader on a plain click.** Anchors keep `target="_blank"` and
>   `rel="noopener noreferrer"`, and the Bluesky ones keep their `clientUrl`
>   rewrite, so middle-click, cmd-click and "copy link address" all still reach
>   the source. A blog anchor points at the publication directly and always did:
>   `clientUrl` rewrites `bsky.app` hostnames and nothing else, so a blog card has
>   never had one to keep. A plain
>   unmodified left click is intercepted (`preventDefault`) and opens the brick
>   reader. Keeping the anchor rather than swapping it for a button is what
>   preserves the browser's own affordances.
>
>   `GlazeCard`'s existing rule that a tap opens the post while a drag scrolls
>   the strip still holds; the tap now opens the reader instead.
>
>   **Which anchor stands for the brick differs by card, and there are three of
>   them.** Only post and blog cards hand `BrickShell` an `href`, so only they
>   have a card-wide link: a video card's one anchor is its watch-at-source link,
>   and a glaze card's are per image. All three call the same
>   `reader.activate(event, brick)`, so the modifier-key rule lives in one place.
>   Intercepting `BrickShell` alone would leave the entire glaze wall and every
>   video brick with no way into the reader.

### `.specs/08-wall-and-bricks.md` → The brick reader (Add, new section after Cards)

> ## The brick reader
>
> `BrickReader` renders one brick at reading width over a dimmed wall. It is
> mounted once in `+layout.svelte`, so the header dims with everything else, and
> it renders nothing at all unless `page.state.brick` is set.
>
> It shows what the card had to leave out. Nothing here is fetched: the reader is
> the same `Brick` the card was given.
>
> It shows that brick and nothing around it. No replies, no thread context, no
> parent post: the reader is one brick, larger. That boundary is what keeps it a
> rendering change rather than a new surface, and it is stated here because a
> reader-shaped overlay is exactly where somebody will assume a conversation
> lives.
>
> | Kind | What the reader adds over the card |
> |---|---|
> | Post | Every image, each at its own aspect ratio, rather than the first one; the full text unclamped; the external embed with its whole description; the timestamp |
> | Post (from glaze) | The same reader. A glaze brick is a post brick, so its filmstrip becomes a stack and every alt text is readable |
> | Video | The poster with a play button that mounts `VideoPlayer` exactly as the card does; title, activity or viewer count, runtime, author |
> | Blog | The cover at full width, the publication chip, the whole description, every tag rather than four, the published date, and "read at <publication>" as the primary control |
>
> A blog's article body is still not rendered, and the reader is where that
> limit is most visible: the content union of `site.standard.document` is
> platform-specific and mason never parses it (see
> [00-overview.md](00-overview.md)). The reader answers that by making the
> outbound link the primary action on a blog rather than a footnote.
>
> ### Dialog behaviour
>
> The reader is a modal dialog in the same language as `SwitchWall`:
> `role="dialog"` with `aria-modal="true"`, labelled by the brick's own heading
> or its author line.
>
> - Focus moves into the reader on open and returns to the card that opened it
>   on close, so a keyboard reader lands back where they were on the wall.
> - Escape closes. So does the close control, a click on the scrim, and the back
>   gesture (the reader is a history entry).
> - The layout's content wrapper is `inert` while the reader is up, which is what
>   traps focus. The reader is mounted as that wrapper's **sibling**, not inside
>   it, because `inert` applies to an element and all of its descendants: a reader
>   nested in the wrapper it dims would be unfocusable and invisible to assistive
>   tech the moment it opened.
> - `document.documentElement` gets `overflow: hidden`, which stops the wall
>   scrolling behind the reader. It does not stop the pump: the sentinel's
>   observer stays connected, so bricks may still be appended behind an open
>   reader. That is harmless, because an append never moves a laid brick and the
>   reader locates its own brick by id.
> - Left and right arrows step to the previous and next brick on the wall, and
>   two visible controls do the same. Stepping stops at the ends of the laid
>   wall; the reader never triggers pagination.
> - One video plays at a time, network-wide, unchanged, but the reader must claim
>   the player under **its own id** (`reader:<brick.id>`). A card collapses back
>   to its poster only when `player.activeId` stops matching its own brick id, so
>   a reader claiming the same id would leave the card mounted and playing behind
>   the scrim, two elements and two audio streams. Claiming a distinct id makes
>   the card a loser of the claim and tears it down through the path that already
>   exists. `VideoPlayer` remains the only sanctioned `.play(`, and
>   `just guard-autoplay` still holds over the reader.

**Merged as:** two edits inside this block. "Renders nothing at all unless
`page.state.brick` is set" became "unless the reader is showing a brick", for
the reason annotated on the Reactive state block above, and the stepping bullet
gained the two things a step actually does beyond moving: it keeps focus inside
the panel (a control that runs out of wall goes `disabled`, and one that only
exists on the old brick's kind unmounts, both of which drop focus on `<body>`
outside the dialog) and announces the new position politely, since the whole
panel swaps under a reader who cannot see it.

### `.specs/08-wall-and-bricks.md` → Sensitive media (Modify)

> `Sensitive` covers a brick's media behind a reveal when a `!warn` blur rides on
> it. The hidden tier never reaches the wall at all (see
> [04](04-sources-and-moderation.md)), so anything that gets here can always be
> revealed.
>
> The choice is per brick and keyed by brick id in the `revealed` session set, so
> a brick uncovered on the card is still uncovered in the reader and on a
> re-place. It is forgotten on reload, by design: the set lives in a rune, never
> in storage, so there is no lingering "show everything" switch.

**Merged as:** written, plus a paragraph this block did not carry and step 4 of
the implementation notes only half did. "Show anyway" sits inside the card's
anchor on the post card and on the glaze card's single and grid branches, and it
has to stop the click **twice**: `stopPropagation` for the reader's activation
handler, and `preventDefault` for the anchor's own `href`, which propagation
cannot reach because the browser gates that navigation on `defaultPrevented`
rather than on the listener chain. Measured while building, and it belongs on
the canonical page, since a reveal that navigates is the exact regression.

### `.specs/08-wall-and-bricks.md` → Accessibility behaviours (Add)

> - **The reader is a real dialog.** `role="dialog"`, `aria-modal="true"`, an
>   accessible name from the brick, focus in on open and back to the opening card
>   on close, Escape and click-away, and `inert` on everything behind it (with the
>   reader mounted outside the inert subtree). Left and right arrows step along
>   the wall from inside it, and they cannot collide with the wall's own
>   navigation-key freeze handler because the two key sets are disjoint: that
>   handler matches the vertical set (`ArrowDown`, `ArrowUp`, `PageDown`,
>   `PageUp`, `Home`, `End`, space) and never the horizontal one. The disjointness
>   is what carries this, not the freeze: `feed.freeze()` is async and does not
>   clear `warming` until its fetch resolves, so the wall's listeners are still
>   attached while the reader mounts.

### `.specs/08-wall-and-bricks.md` → Implementation layout (Modify)

Add to the `components/` list, after `Sensitive.svelte`:

> ```
>     BrickReader.svelte       one brick at reading width, over the wall
> ```

### `.specs/09-design-system.md` → Motion (Add)

Add a row to the motion table:

> | Brick reader open | Scrim fade 200ms `linear`; panel `0.24s cubic-bezier(0.16, 1, 0.3, 1)` from `translateY(8px) scale(0.99)` | Scrim fade only, `0.15s linear`; the panel does not move |

---

## Implementation notes

No wire change: nothing to regenerate in the contract fixture, and nothing in
`types.ts` follows. One Rust file is touched all the same, and step 3 says why.

```
1. web/src/app.d.ts:8
     `interface PageState` is COMMENTED OUT, so this creates it rather than
     extending it: uncomment and declare `brick?: string`. That is what types
     `page.state.brick` across the app.

2. web/src/lib/state/sensitive.svelte.ts        (new, ~12 lines)
     export const revealed = new SvelteSet<string>()   // svelte/reactivity
     Session-scoped; never persisted.

3. server/crates/mortar-core/src/fixtures.rs:186
     Give fixture brick `i == 0` a `blur: Some(Blur { label: "!warn".into() })`.
     Both arms are `blur: None` today: :148 is the VIDEO arm, :186 is the post
     arm, and the post arm is the one that matters.

     The index is not arbitrary and picking another one silently breaks the
     test. `:152`'s `_ =>` arm is shared by all 84 posts, so "one" needs a
     condition on `i`; `PostCard` renders `<Sensitive>` only inside `{#if img}`
     and `:153` gives images only to `i.is_multiple_of(3)`, so two thirds of the
     posts would render no reveal control at all. Brick 0 is a post
     (`0 % 20` misses the blog arms), carries an image, and is first on both the
     wall and the glaze wall, so Playwright reaches it without scrolling.

     Without this, no demo brick is ever covered, and since Playwright is the
     only lane that can see a component, the entire shared-reveal half of this
     change ships with nothing able to observe it. It does NOT touch the wire:
     contract.json is built from its own canonical instances in
     tests/contract.rs, not from fixtures.rs.

4. web/src/lib/components/Sensitive.svelte:17
     Replace the local `revealed = $state(false)` (at :17; :18 is </script>)
     with a lookup against the shared set. The component gains an `id` prop.
     FOUR call sites forward `brick.id`, not three: PostCard.svelte:19,
     VideoCard.svelte:52, and GlazeCard.svelte TWICE, at :138 (carousel branch)
     and :201 (single/grid branch). GlazeCard.svelte:69 has an UNRELATED local
     `revealed` for its touch pill: do not delete that one.
     Also give Sensitive's show-anyway button (:32) an event.stopPropagation():
     on PostCard and on GlazeCard's single/grid branch it is a DESCENDANT of the
     anchor step 6 intercepts, so without it a reveal opens the reader instead.

5. web/src/lib/state/reader.svelte.ts           (new, ~45 lines)
     open(brick):  feed.freeze(); pushState('', { brick: brick.id }); set a
                   private pushed flag
     close():      history.back() ONLY if that flag is set, else
                   replaceState('', {}); clear it either way
     activate(event, brick): the one modifier-key rule. Return false (no
                   interception) on metaKey/ctrlKey/shiftKey/altKey or
                   button !== 0; otherwise preventDefault, open, return true.
     next()/prev(): locate by id (feed.items.findIndex), step, replaceState.
                   -1 means the brick is gone from the wall: do not step.
     Holds the Brick as $state; `page.state.brick` decides open/shut.

6. Three activation points, all calling reader.activate:
     web/src/lib/components/BrickShell.svelte:32   the card-wide anchor. Only
       PostCard.svelte:17 and BlogCard.svelte:13 pass an href, so this covers
       post and blog only.
     web/src/lib/components/cards/VideoCard.svelte:147   the watch-at-source
       anchor. VideoCard.svelte:51 passes BrickShell NO href, so this is the
       card's ONLY anchor and it therefore stops being a plain-click route to
       the source: a plain click opens the reader, which carries the watch link
       itself. Every modified click still goes straight out, and the play button
       is untouched, so the poster still plays in place.
     web/src/lib/components/cards/GlazeCard.svelte:147 and :195   the image
       anchors, one per branch. GlazeCard.svelte:132 passes NO href either.
     The filmstrip arrows, the ALT panel and the touch pill are SIBLINGS of
     those anchors and need nothing; Sensitive's button is a descendant and is
     handled in step 4.

7. web/src/lib/components/BrickReader.svelte    (new)
     The kind switch mirrors the FeedGrid snippet at :204. Reuse AuthorChip,
     Sensitive, Icon and VideoPlayer; do NOT add a second .play( call site, or
     `just guard-autoplay` fails. Claim the player as `reader:${brick.id}`, NOT
     as brick.id, or the card behind keeps playing (VideoCard.svelte:47).

8. web/src/routes/+layout.svelte:110
     Mark the wrapper div inert when the reader is open, and mount
     <BrickReader /> AFTER that div's closing tag at :134. It must not go at
     :133, which is inside the wrapper: inert covers descendants, so the reader
     would open unfocusable and invisible.

9. web/tests/reader.test.ts                     (new Playwright spec)
     The demo wall is the fixture: click a card, assert the reader opens with the
     full text, Escape closes it, focus returns to the card, and the wall did not
     navigate. Add two more that nothing else can catch: cmd-click still opens a
     tab, and clicking "show anyway" on the step 3 fixture reveals the media and
     leaves [role=dialog] ABSENT. This lane is the ONLY one that can cover a
     component (tsc drops .svelte files and both vitest suites are .ts), so the
     reader has no typechecked coverage without it. See 07-web-client.md,
     Testing.

10. just check && just test-e2e
   pnpm changeset   (minor: a new surface)
```

### Four things to get wrong

Each of these compiles, passes `just check`, and is invisible to every lane in
the repo except a Playwright case somebody has to think to write.

1. **Mounting the reader inside the element it makes inert.** `inert` covers
   descendants, so the reader opens unfocusable and invisible. It goes after
   `+layout.svelte:134`, outside the wrapper. The wrapper is also the right
   element to mark, rather than the wall: on the wall, the header's pickers stay
   tabbable behind an open reader.
2. **Claiming the video player under the brick's own id.** A card tears down its
   player when `player.activeId` stops matching its brick id, so claiming the
   same id leaves the card playing behind the scrim. Claim `reader:<brick.id>`.
3. **Intercepting `BrickShell` only.** Two of the four cards pass it no `href`,
   so the glaze wall and every video brick would have no way into the reader.
4. **Forgetting that `Sensitive`'s button is inside the intercepted anchor** on
   two of the four cards. Without a `stopPropagation`, "show anyway" opens the
   reader instead of revealing the media.

What is *not* a hazard, despite looking like one: `FeedGrid`'s window-level
navigation-key handler. It is tempting to say `reader.open`'s freeze has already
torn it down, and that is wrong twice over, since `feed.freeze()` is async and
does not clear `warming` until its fetch resolves, and it returns immediately
when a freeze is already in flight. What actually holds is that its key set
(`FeedGrid.svelte:174`) is vertical navigation only, and the reader steps on left
and right, so the two never overlap however the freeze is progressing. Space is
in that set and does reach the handler from inside the reader, which is harmless
because a second freeze is a no-op.

Nor is the pump: the sentinel's observer stays connected, so bricks can still be
appended behind an open reader. An append never moves a laid brick and the reader
locates its own brick by id.

---

## Merge plan

1. Apply each `Proposed changes` block to its canonical page; bump that page's
   `**Date:**` to the merge date.
2. No schema change and no fixture change.
3. Flip this file's `**Status:**` to `Merged`, add `**Merged:** YYYY-MM-DD`, and
   move it to `.specs/changes/merged/`.
4. Update `.specs/README.md`: remove it from the pending list and add it to the
   merged table.

---

## Assumptions and open questions

**Assumptions**

- SvelteKit 2's shallow routing (`pushState` from `$app/navigation`) works in a
  fully client-rendered static build. It is client-side by construction, but
  nothing in mason uses it yet.
- `inert` is available. It is in every browser above mason's stated floor
  (Chrome 91+, Safari 15+, Firefox 147+), and unlike `ResizeObserver` there is
  no fallback: without it the wall behind the reader stays tabbable.
- A brick's own fields are enough to read it properly. True for posts and video;
  a blog is metadata by design.

**Decisions**

- *No new endpoint.* **The reader renders the brick already on the wall.** A
  reader that fetched anything would need a wire shape, an upstream read, a
  cache, an error state and a loading state, to show fields mason is already
  holding in memory.
- *Keep the anchor, intercept the click.* **`preventDefault` on an unmodified
  left click only.** Swapping the `<a>` for a `<button>` would cost middle-click,
  cmd-click and "copy link address", which is how a lot of people move content
  between apps.
- *A dialog in the house language.* **`role="dialog"` plus `inert`, like
  `SwitchWall`.** A native `<dialog>` with `showModal()` would bring its own
  focus trap and top layer, and its own quirks (scroll containment, the backdrop
  pseudo-element, iOS behaviour); the repo already has a working pattern.
- *One demo brick gains a `!warn` blur.* **The shared reveal needs something to
  reveal.** Every fixture brick is `blur: None`, and Playwright over the demo
  wall is the only lane that can see a component at all, so without this the
  reveal ships unobservable. It costs one Rust file and touches no wire: the
  contract fixture is built from its own canonical instances, not from
  `fixtures.rs`.
- *Arrow keys step, they do not paginate.* **Stepping stops at the last laid
  brick.** Loading a page from inside a reader would grow the wall the reader
  cannot see, and the pump is already the only thing that grows it.
- *Replies and thread context are out of scope.* **The selected brick's own
  content, and nothing else.** `app.bsky.feed.getPostThread` is readable
  unauthenticated, so a conversation is reachable; it is not cheap. It would make
  mason a two-endpoint app, which retires the claim
  [06-wire-contract.md](../../06-wire-contract.md) opens with; it would render the
  first content on the wall from outside the reader's follow graph, so the label
  tiers have to run in mortar rather than in the client, where the cohort filter
  cannot reach; and the offline demo wall would need a fixture thread compiled in
  or the installed app opens a reader that errors. Sized on 2026-07-26 and
  deferred deliberately, not overlooked. The reader as specified here is a
  rendering change with no wire surface, and that is the whole of its appeal.

**Open questions**

- *A shareable brick link.* The reader is deliberately not addressable. Making
  it so needs a single-brick lookup (`app.bsky.feed.getPosts` for a post, a
  `getRecord` for a blog or a stream), which is a new endpoint, a new wire shape
  and a new error path. Open: is "share this brick" worth that, given the source
  link already shares the post itself?
- *Blog bodies.* The reader makes the metadata-only blog brick feel thinnest.
  Open: is there a subset of the `site.standard.document` content union (plain
  text, or one platform's) worth parsing, or does the non-goal stand?
- *Reading a brick that leaves the wall.* Nothing removes a brick from
  `feed.items` today, so the open brick cannot vanish. If a future change ever
  prunes laid bricks, the reader's index needs to survive it.
