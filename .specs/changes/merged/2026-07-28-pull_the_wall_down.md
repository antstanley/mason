# Change: Pull the wall down to lay it again

**Status:** Merged · **Date:** 2026-07-28 · **Merged:** 2026-07-28 · **Owner:** Ant Stanley · **Target:** `web/`

The refresh control shipped as a button in the header. On a phone, the gesture a
reader actually makes at the top of a wall is to drag it down, and the wall had
nothing there. This change adds the pull as a second trigger for the one refresh
that already exists: the same `feed.refresh()`, the same one-shot flag, the same
rate limit, reached the way a thumb reaches for it.

Proposed and shipped together, so it goes straight to Merged. It closes the
*Pull to refresh* open question that has sat at the foot of
[`08-wall-and-bricks.md`](../../08-wall-and-bricks.md) since the refresh landed,
and it answers that question's three requirements in order: a gesture threshold,
a rubber band, and a reduced-motion path.

---

## Motivation

A refresh nobody can find is a refresh nobody uses. The control is icon-only on a
phone (the bar never wraps at 375px, so it had to be), it sits in a bottom bar
with three other controls, and the reader whose instinct it serves is already
holding the top of the wall with their thumb.

There is a second reason, and it is the stronger one: **the browser's own pull to
refresh is worse than nothing here.** Chrome on Android turns an overscroll at
the top into a full page reload, which drops the laid wall, its scroll position,
the seed behind its arrangement and any playing video, then warms a new wall from
a cold service worker. That is the exact restart the refresh control was built to
avoid, offered by the platform on the exact gesture this change is about. Adding
the pull means the gesture can be claimed, and the reload turned off, in one
move.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/07-web-client.md`](../../07-web-client.md) | A `state/pull.svelte.ts` row in the rune-singleton table; `refresh()` gains a second trigger; the reader-close obligation is now held at two triggers, live at one of them; the testing table names the new spec |
| [`.specs/08-wall-and-bricks.md`](../../08-wall-and-bricks.md) | A **Pulling** section under Refreshing: the band, what counts as a pull, the wall's own movement, the suppressed browser gesture, and the indicator. The open question is replaced by the two it leaves behind |

No engine page changes. The pull produces exactly the request a button press
produces, so `02`, `04`, `05` and `06` describe it already and the wire is
untouched.

---

## What shipped

The prose landed on the pages above; this section is the decision record behind
it, and the reasoning that is not worth repeating in a canonical page.

### The gesture is touch only

No wheel path, no trackpad path. A wheel has no equivalent of *letting go*, which
is the whole commitment step, and a desktop reader is never more than a glance
from the button. This is now an open question rather than a closed door.

### The threshold is 72px, and the wall follows the finger to get there

1:1 travel up to the threshold makes arming honest: the reader's own hand is the
gauge, and the wall has moved exactly as far as the finger when the indicator
changes what it says. Past it the band stretches asymptotically toward 120px, so
there is no point at which the wall stops answering and nothing to hit and sit
against.

72 rather than the ~110 a browser uses for the same gesture, because a browser
has to tell a pull from a scroll at the top of *any* document. This gesture only
ever starts at the top of a wall, where a downward drag is not a scroll, so it
does not need that margin.

### The wall moves, not a spinner over it

The gesture says the reader has hold of the wall. `main#wall` is what translates,
and deliberately not the layout's wrapper: on a phone that wrapper holds the
fixed control bar, and a transform on it would drag the bar down the screen along
with the bricks.

### A released pull rests the wall on a shelf, and this was corrected in review

The first cut sent the wall home on release and left the indicator floating over
the bricks. A screen recording on a real iPhone is what settled it: for the whole
warm, over three seconds on a live wall, the pill sat on top of the first card,
because a wall at home has no gap to draw an affordance in.

So a release now settles the wall 54px open and holds it there until the warm
ends. The gap closing is the signal that the refresh finished, which is both
cheaper than a word and the thing every reader of a phone already knows this
gesture by.

The offset became one number in the rune module (`pull.offset`: the finger, then
the shelf, then zero) rather than two rules in two files, so the wall and the
indicator cannot disagree about where the top of the wall is. `laying` is set
from `feed.warming` right after `refresh()` returns, so it says "the refresh
took"; and it is deliberately not cleared by a gesture reset, because the shelf
outlives the finger and a reader touching a brick mid-refresh must not have the
wall drop out from under the tap.

### The reduced-motion path is the snap back, not the pull

A pull is direct manipulation: the wall goes where the finger goes, which is not
an animation and is not gated. What is gated is the glide home, which is
`motion-safe:transition-transform`, so a reader who asked for less motion gets
the wall back at once instead of watching it travel. This is the same rule the
brick entrance and the hover transforms already follow.

### The indicator outlives the gesture, on purpose

It stays up in the shelf's gap saying "laying bricks" until the warm ends.
Without that the gesture lands on nothing: the four-card skeleton tail that says
more is coming sits at the bottom of the wall, off screen from the top where the
reader is standing.

It is `aria-hidden` and adds no live region. The wall has exactly one polite
region, it already says "laying bricks" while warming, and this is a touch
affordance narrating a touch gesture. The button remains the named,
platform-disabled way to ask.

### Two implementation notes that are load-bearing

- **The listener is attached with `addEventListener`, not a svelte event
  attribute.** A touch listener on `window` is passive by default in every
  browser that matters, a passive listener's `preventDefault` is ignored, and
  svelte's event attributes cannot pass listener options. Without it the document
  scrolls under a wall that is already moving and the two add up.
  `preventDefault` is called only on a claimed move and only when the event is
  cancelable, because iOS hands over uncancelable moves once a scroll is under
  way; there the pull rides the platform's rubber band rather than replacing it.
- **An effect must not read the state it writes.** The first cut of
  `PullToRefresh` cleared its own indicator flag with an effect that both read
  and wrote it. Svelte aborts the flush over that cycle, and an aborted flush
  leaves the DOM holding the last thing written to it: the symptom was the wall
  staying pulled down after the finger had gone, with the wall's own state
  already back at zero. The effect now reads `feed.warming` only, and the flag is
  set from `feed.warming` immediately after `refresh()` returns, which is also
  what keeps it from sticking when a refresh refuses.

---

## Testing

| Lane | What it pins |
|---|---|
| `web/src/lib/state/pull.test.ts` (vitest, node) | Every decision: the slop, the 1:1 travel, the band's ceiling, arming, the direction rule and that it holds for the whole gesture, unwinding, release, cancel. Plus the source grep that keeps the module free of DOM globals |
| `web/tests/pull-refresh.test.ts` (playwright, chromium, `hasTouch`) | The half only a browser can answer: that the listeners exist, that the wall really moves, that the indicator says which side of the threshold the reader is on, that a release settles onto the shelf and the warm ending lets it down, that letting go ends in exactly one `refresh=1`, that a short pull and an upward drag cost nothing, and that the front door has nothing to pull |

The playwright spec dispatches the touch sequence rather than performing it,
because playwright's touchscreen can tap and nothing else.

Two reads, and which one a case uses is about what it is claiming. Where the
wall ENDS UP is polled off the computed transform, since both glides are CSS
transitions and a computed value equals its destination only once the transition
has run. Where the wall was TOLD to go is read off the inline style in the same
turn as the release, draining microtasks and nothing else, and on the demo wall
that is the only mechanism that can see the shelf at all: its bricks are compiled
into the wasm, so its refresh settles in milliseconds and the wall is off the
shelf before the next animation frame. A case that samples a frame later cannot
tell a shelf that never happened from one already let down.

---

## Assumptions and open questions

- *A trackpad pull.* Touch only today. Whether a wheel-driven overscroll should
  arm the same refresh is open, and turns on whether a wheel can express letting
  go.
- *Pulling up at the end of a wall.* The symmetric gesture would ask for the next
  cohort, which the scroll pump already does invisibly. Open, and probably not
  worth a gesture.
- *`overscroll-behavior-y: contain` is set on `html` for the whole app*, not only
  on a wall. Nothing else in mason wants the browser's reload gesture either, so
  this is deliberate, but it is a wider blast radius than the feature and worth
  naming.
