# Change: Read a brick without leaving the wall

**Status:** Proposed · **Date:** 2026-07-26 · **Owner:** Ant Stanley · **Target:** Repo-wide

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
no `Type changes` section below and no fixture to regenerate.

| Canonical page | Nature of change |
|---|---|
| [`.specs/07-web-client.md`](../07-web-client.md) | A `reader` rune singleton, shallow-routing history state, and the freeze-on-open rule |
| [`.specs/08-wall-and-bricks.md`](../08-wall-and-bricks.md) | Cards activate the reader; a new `BrickReader` section; the `Sensitive` reveal becomes shared; implementation layout |
| [`.specs/09-design-system.md`](../09-design-system.md) | The reader's motion and its reduced-motion alternative |
| [`.specs/00-overview.md`](../00-overview.md) | Goals gain reading in place; the blog-content non-goal is restated precisely |

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

> | `state/reader.svelte.ts` | `reader` | The brick being read in place, and its index on the wall | none (history state) |
> | `state/sensitive.svelte.ts` | `revealed` | Brick ids whose `!warn` media the reader uncovered | none (session set) |

### `.specs/07-web-client.md` → Reactive state → The reader is history, not a URL (Add)

> **The reader is history state, not a URL.** Opening a brick calls SvelteKit's
> `pushState('', { brick: id })`, so the address bar keeps showing `?actor=` and
> the back button (and the phone's back gesture) closes the reader instead of
> leaving the wall. `page.state.brick` is the single source of truth for whether
> the reader is open, declared as `App.PageState` in `app.d.ts`; `reader` holds
> the brick and its index so the reader can step along the wall without
> re-deriving either.
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
> the arrangement reorders between preview polls, so an index into `feed.items`
> would go stale under an open reader.

### `.specs/07-web-client.md` → Decisions (Add)

> - *The reader is history state, not a URL.* **`pushState('', { brick })`.** The
>   back gesture has to close an overlay on a phone, and a `?brick=` link would
>   advertise a deep link mason cannot serve: a single brick is not fetchable, so
>   the recipient would get the wall and a dropped parameter.
> - *Opening the reader freezes the wall.* **A click is engagement.** The reader
>   holds an index into `feed.items`, and a warming wall reorders between polls.
> - *The reveal choice follows the brick.* **A session set of brick ids, shared
>   by the card and the reader.** Uncovering a brick on the wall and finding it
>   covered again one click later reads as a bug. It is still forgotten on
>   reload: the set lives in a rune, not in storage.

### `.specs/08-wall-and-bricks.md` → Responsibilities (Add)

> 6. Open any brick in place, at full size, without leaving the wall.

### `.specs/08-wall-and-bricks.md` → Cards (Modify)

Replace the **Outbound links** bullet with:

> - **Outbound links stay real links, and a plain click opens the reader
>   instead.** Every card keeps its `<a href>` with `target="_blank"`,
>   `rel="noopener noreferrer"` and the `clientUrl` rewrite, so middle-click,
>   cmd-click and "copy link address" all still reach the source. A plain
>   unmodified left click is intercepted (`preventDefault`) and opens the brick
>   reader. Keeping the anchor rather than swapping it for a button is what
>   preserves the browser's own affordances, and it means the card is still
>   meaningful with JavaScript half-loaded.

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
>   traps focus and keeps the wall's own key handlers out of the way.
> - `document.documentElement` gets `overflow: hidden`, so the wall does not
>   scroll behind the reader and the pump cannot fire under it.
> - Left and right arrows step to the previous and next brick on the wall, and
>   two visible controls do the same. Stepping stops at the ends of the laid
>   wall; the reader never triggers pagination.
> - One video plays at a time, network-wide, unchanged: the reader claims the
>   player by brick id, so opening a reader over a playing card pauses the card.
>   `VideoPlayer` remains the only sanctioned `.play(`, and `just guard-autoplay`
>   still holds over the reader.

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

### `.specs/08-wall-and-bricks.md` → Accessibility behaviours (Add)

> - **The reader is a real dialog.** `role="dialog"`, `aria-modal="true"`, an
>   accessible name from the brick, focus in on open and back to the opening card
>   on close, Escape and click-away, and `inert` on everything behind it. Arrow
>   keys step along the wall from inside it; the wall's own arrow-key freeze
>   handler never sees them, because the wall is inert.

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

Client only. No Rust changes, no wasm rebuild needed to see it, and nothing to
regenerate in the contract fixture.

```
1. web/src/app.d.ts:8
     Uncomment `interface PageState` and declare `brick?: string`. This is what
     types `page.state.brick` across the app.

2. web/src/lib/state/sensitive.svelte.ts        (new, ~12 lines)
     export const revealed = new SvelteSet<string>()   // svelte/reactivity
     Session-scoped; never persisted.

3. web/src/lib/components/Sensitive.svelte:19
     Replace the local `revealed = $state(false)` with a lookup against the
     shared set. The component gains an `id` prop (the brick id); every call
     site in cards/ forwards `brick.id`.

4. web/src/lib/state/reader.svelte.ts           (new, ~40 lines)
     open(brick, index): feed.freeze(); then pushState('', { brick: brick.id })
     close(): history.back() when the reader owns the top entry, else
              replaceState('', {})
     next() / prev(): step the index within feed.items, replaceState the id
     Holds `brick` and `index` as $state; `page.state.brick` decides open/shut.

5. web/src/lib/components/BrickShell.svelte:32
     Add an onclick to the existing <a>. Return early (no preventDefault) when
     event.metaKey || ctrlKey || shiftKey || altKey || button !== 0, so every
     modified click keeps its browser behaviour. Otherwise preventDefault and
     call reader.open. BrickShell needs the brick and its index as props;
     FeedGrid's `brick` snippet (web/src/lib/components/FeedGrid.svelte:204)
     already has both and threads them through each card.

6. web/src/lib/components/BrickReader.svelte    (new)
     The kind switch mirrors the FeedGrid snippet at :204. Reuse AuthorChip,
     Sensitive, Icon and VideoPlayer; do NOT add a second .play( call site, or
     `just guard-autoplay` fails.

7. web/src/routes/+layout.svelte:110
     Mark the wrapper div inert when the reader is open, and mount
     <BrickReader /> after {@render children()} at :133.

8. web/tests/reader.test.ts                     (new Playwright spec)
     The demo wall is the fixture: click a card, assert the reader opens with the
     full text, Escape closes it, focus returns to the card, and the wall did not
     navigate. This lane is the ONLY one that can cover a component (tsc drops
     .svelte files and both vitest suites are .ts), so the reader has no
     typechecked coverage without it. See 07-web-client.md, Testing.

9. just check && just test-e2e
   pnpm changeset   (minor: a new surface)
```

### The one thing to get wrong

`inert` on the layout wrapper is what traps focus, and it must go on the wrapper
rather than on the wall, or the header's pickers stay tabbable behind the reader.
`FeedGrid`'s window-level `keydown` and `scroll` listeners
(`FeedGrid.svelte:189`) are on `window`, not on the wall, so `inert` does not
stop them: the `overflow: hidden` on the root element is what keeps the pump
from firing, and the arrow-key freeze handler is harmless because the wall is
already frozen by `reader.open`.

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
- *Arrow keys step, they do not paginate.* **Stepping stops at the last laid
  brick.** Loading a page from inside a reader would grow the wall the reader
  cannot see, and the pump is already the only thing that grows it.
- *Replies and thread context are out of scope.* **The selected brick's own
  content, and nothing else.** `app.bsky.feed.getPostThread` is readable
  unauthenticated, so a conversation is reachable; it is not cheap. It would make
  mason a two-endpoint app, which retires the claim
  [06-wire-contract.md](../06-wire-contract.md) opens with; it would render the
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
