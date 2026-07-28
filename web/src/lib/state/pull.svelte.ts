/** The pull gesture: drag the top of the wall down and let go to lay it again.
 *
 *  A touch reader's instinct at the top of a wall is to drag it, and until now
 *  the wall had nothing there. The gesture is the second trigger for the one
 *  refresh the header button already asks for, never a second kind of refresh:
 *  what it ends in is `feed.refresh()`, the same call with the same rate limit.
 *
 *  Everything with a wrong answer lives HERE rather than in the component that
 *  listens, because nothing in this repo typechecks or unit tests a `.svelte`
 *  body: which drags are pulls, how far the band stretches, when it arms, and
 *  whether letting go lays the wall. `PullToRefresh.svelte` is left with the
 *  listeners, the indicator, and no decisions.
 *
 *  It names no DOM global and takes no events. The caller reads a touch's
 *  coordinates and answers "may this gesture start", which keeps this module
 *  runnable for real in vitest's node environment (see pull.test.ts, which
 *  greps for exactly that) and keeps the one question this module cannot answer
 *  (is the wall at the top, is it busy, is something over it) at the one place
 *  that can see it. */

/** How far the wall follows the finger 1:1, and the point at which letting go
 *  lays it again.
 *
 *  72px rather than the ~110 a browser's own overscroll refresh uses: this
 *  gesture can only start at the very top of the wall, where a downward drag is
 *  not a scroll and cannot be mistaken for one, so it does not need the margin a
 *  browser needs to tell the two apart. Far enough that a thumb resettling on
 *  the glass is not a hundred-author fan-out, short enough to be one motion. */
export const PULL_THRESHOLD = 72;

/** The furthest the wall ever travels, however hard the pull. Past the
 *  threshold the band stiffens and approaches this without reaching it, so
 *  there is no point where the wall stops answering the finger, and no clamp to
 *  hit and sit against. */
export const PULL_MAX = 120;

/** How far a finger has to travel before this decides what the gesture IS. Under
 *  it, a touch is a tap that has not let go yet, and reading a direction out of
 *  three pixels of noise would claim gestures that were never pulls. */
export const PULL_SLOP = 8;

/** The shelf: how far the wall stays open while the pull's own refresh runs.
 *
 *  A released pull used to snap the wall home at once and leave the indicator
 *  floating over the bricks, and a recording on a real phone is what settled it:
 *  for the whole warm, three seconds and more on a live wall, the pill sat on top
 *  of the first card. Holding the wall open gives the pill the gap it is drawn
 *  in, and it makes the wall closing the signal that the refresh finished, which
 *  is the thing every reader of a phone already knows this gesture by.
 *
 *  54px is the pill (38px) with the same 8px of air above it as below it, and
 *  nothing more: this is a shelf, not a second pull, and a wall held further open
 *  than its own affordance needs reads as a wall that has lost its place. The
 *  air is the point rather than a rounding: at 52 the pill cleared the bricks by
 *  two pixels, which is a gap in the arithmetic and none on the screen.
 *  `PullToRefresh` spends it, and its `PILL` and `PILL_GAP` are the two numbers
 *  this one is made of. */
export const PULL_LAYING = 54;

/** How much of a wheel's delta becomes pull.
 *
 *  A wheel is not a finger and cannot be treated like one. One notch of a mouse
 *  wheel is 100px in chrome, so at 1:1 a single stray notch at the top of the
 *  wall would arm the gesture and a second would lay a hundred-author fan-out
 *  nobody asked for. At 0.4 it takes two deliberate notches, and a trackpad,
 *  which sends many small deltas rather than few large ones, gets a pull that
 *  answers continuously the way the touch one does. */
export const PULL_WHEEL_SCALE = 0.4;

/** How long the wheel must have been quiet before a wheel gesture may START a
 *  pull, in milliseconds.
 *
 *  THIS IS THE WHOLE DEFENCE AGAINST MOMENTUM, and without it the desktop
 *  gesture is unusable. A trackpad flick that reaches the top of the wall keeps
 *  delivering wheel events after the fingers have left the glass, all of them
 *  pointing up, all of them at `scrollY === 0`: exactly the shape of a pull, and
 *  nobody asked for it. What a momentum tail never has is a gap before it,
 *  because it is the continuation of the scroll that just arrived. A reader who
 *  means to pull has one: they reach the top, the wall stops, and then they push
 *  again. So the pull starts from REST, and momentum is excluded by the one
 *  property it structurally cannot have.
 *
 *  200ms is comfortably longer than the gap between events inside one flick
 *  (~8-16ms) and shorter than the pause a reader takes before pushing again. */
export const PULL_REST_MS = 200;

/** How long the wheel must be quiet before an armed wheel pull lays the wall,
 *  in milliseconds.
 *
 *  A wheel has no equivalent of letting go, so stopping is the release: push
 *  past the threshold, stop pushing, and the wall lays. Short enough that it
 *  follows the reader's hand rather than making them wait, long enough that the
 *  gaps inside one gesture do not fire it early. It is a real difference from
 *  the touch path and the only one: a finger says when it is done, a wheel is
 *  only ever observed to have stopped. */
export const PULL_SETTLE_MS = 140;

/** Where a finger is. Structural and flat, so the caller hands over two numbers
 *  read off a touch rather than the event itself, which is a DOM type this
 *  module deliberately cannot name. */
export interface PullPoint {
  x: number;
  y: number;
}

/** What is pulling the wall, or null between gestures. Read by the indicator,
 *  which has one word to change: a finger lets go and a wheel stops. */
export type PullBy = "touch" | "wheel" | null;

/** The rubber band.
 *
 *  1:1 up to the threshold, so arming is honest: the wall has moved exactly as
 *  far as the finger when it arms, and the reader's own hand is the gauge. Past
 *  it the band stretches asymptotically towards `PULL_MAX`, which is what makes
 *  the far end feel like a limit rather than a wall. */
function band(travel: number): number {
  if (travel <= PULL_THRESHOLD) return travel;
  const stretch = PULL_MAX - PULL_THRESHOLD;
  const over = travel - PULL_THRESHOLD;
  return PULL_THRESHOLD + (stretch * over) / (over + stretch);
}

/** Exported for the unit tests, which build throwaway instances; the app only
 *  ever uses the `pull` singleton below. */
export class PullState {
  /** How far down the wall is pulled, in px, after the band. Zero at rest, and
   *  the component both moves the wall by it and fades the indicator in with
   *  it, so one number is the whole of what the gesture looks like. */
  distance = $state(0);

  /** The wall is being pulled right now, by a finger or by a wheel. Read by the
   *  component to leave the snap-back transition off while the reader is driving
   *  (a transition mid-gesture is lag) and to promote the wall to its own layer
   *  only for as long as it is moving. */
  pulling = $state(false);

  /** Far enough that letting go lays the wall again. The indicator says so
   *  before the reader commits, because a gesture with a threshold has to tell
   *  you which side of it you are on. */
  armed = $derived(this.distance >= PULL_THRESHOLD);

  /** The wall is being laid because of a pull, and stays open on the shelf until
   *  it is not.
   *
   *  Set by the trigger rather than derived here, because the answer is
   *  `feed.warming` and this module imports no feed: it would drag `$lib/api`
   *  and `$app/environment` into a graph that currently runs in node with
   *  nothing mocked. The trigger reads it back off the wall right after asking,
   *  so this is "the refresh took" rather than "a refresh was asked for", and a
   *  refusal cannot leave the wall propped open with nothing behind it. */
  laying = $state(false);

  /** How far down the wall sits, in px: what the finger is doing, or the shelf
   *  it rests on while the refresh runs, or home.
   *
   *  One number for the whole gesture, so the wall and the indicator cannot
   *  disagree about where the top of the wall is. `pulling` wins over `laying`
   *  because a finger beats a leftover, though the two cannot overlap today: a
   *  gesture may not start while the wall is busy. */
  offset = $derived(this.pulling ? this.distance : this.laying ? PULL_LAYING : 0);

  /** Where the finger went down. */
  #origin: PullPoint = { x: 0, y: 0 };

  /** This gesture is still a candidate. False once it is abandoned (the finger
   *  went up, or sideways, or a second one landed), and only the next
   *  `start` can set it again: a gesture that has been read as a scroll must
   *  stay a scroll for as long as the finger is down, or a drag that wandered
   *  back up through its own origin would turn into a pull halfway. */
  #tracking = false;

  /** What is pulling the wall, or null between gestures. It decides which
   *  release is allowed to lay the wall: a finger says when it is done and a
   *  wheel is only ever observed to have stopped, so `settle` must not fire for
   *  a finger that paused mid-drag, and `release` has nothing to do with a
   *  wheel. Two inputs, two releases, and neither may answer for the other. */
  by = $state<PullBy>(null);

  /** The wheel pull's raw travel before the band, in px. Accumulated rather than
   *  measured, because a wheel reports movement and not position: there is no
   *  origin to subtract, only a running total to add to and unwind. */
  #travel = 0;

  /** When the last wheel event arrived, in the caller's own clock. The gap
   *  before the next one is what tells a deliberate pull from a momentum tail;
   *  see `PULL_REST_MS`. */
  #lastWheel = Number.NEGATIVE_INFINITY;

  /** A finger went down.
   *
   *  `ready` is everything this module cannot see, answered by the caller in one
   *  boolean: the wall is at its top, it is not already being laid, nothing is
   *  open over it, and this is the only finger on the glass. It is read once,
   *  here, rather than on every move: a wall that starts warming mid-gesture
   *  still lets the reader finish the pull they were making, and the refresh it
   *  ends in refuses on its own if it has to. */
  start(point: PullPoint, ready: boolean): void {
    this.#reset();
    if (!ready) return;
    this.by = "touch";
    this.#origin = point;
    this.#tracking = true;
  }

  /** The finger moved. Returns whether this gesture belongs to the pull, which
   *  is the caller's cue to take the move away from the browser: while the wall
   *  is being pulled, nothing else may scroll on that finger. */
  move(point: PullPoint): boolean {
    if (!this.#tracking) return false;
    const down = point.y - this.#origin.y;
    const across = point.x - this.#origin.x;

    if (!this.pulling) {
      // undecided: too little travel to read a direction out of
      if (Math.max(Math.abs(across), Math.abs(down)) < PULL_SLOP) return false;
      // decided, once and for the whole gesture. A pull is downward and more
      // down than across; anything else is a scroll or a swipe, and this hands
      // the finger back rather than fighting it for the rest of the drag.
      if (down < PULL_SLOP || Math.abs(across) >= down) {
        this.#tracking = false;
        return false;
      }
      this.pulling = true;
    }

    // clamped at zero rather than abandoned: a reader who pulls too far and
    // eases back up is unwinding the gesture, and the wall follows them all the
    // way home. Letting go there is a pull that was thought better of, which is
    // what the threshold is for.
    this.distance = band(Math.max(0, down));
    return true;
  }

  /** A wheel turned. Returns whether the pull claimed it.
   *
   *  The desktop gesture, and the same one: keep scrolling up when the wall is
   *  already at the top and it opens exactly as a finger opens it, with the same
   *  band, the same threshold and the same shelf. Only the ends differ, because
   *  a wheel has neither a down nor an up.
   *
   *  `deltaY` in PIXELS and negative for up, which is the wheel's own sign
   *  convention; the caller normalises a line-mode or page-mode wheel before it
   *  gets here, since that conversion needs a viewport and this module has none.
   *  `at` is the event's own timestamp, passed in for the same reason: a clock
   *  is a global, and this module does without one.
   *
   *  `ready` is read only when a gesture STARTS, exactly as it is for touch, so
   *  a wall that starts warming under a pull still lets the reader finish it. */
  wheel(deltaY: number, at: number, ready: boolean): boolean {
    // Recorded before any decision below, and for EVERY wheel event rather than
    // only the ones that pull: a scroll down the wall is what makes the next
    // scroll up not-from-rest, and that is precisely what stops a flick to the
    // top from turning into a pull on its own momentum.
    const rested = at - this.#lastWheel > PULL_REST_MS;
    this.#lastWheel = at;

    if (this.by === "wheel" && this.pulling) {
      // up adds, down unwinds. Clamped at zero and abandoned there rather than
      // held, because a reader who has scrolled the wall back down is scrolling,
      // and the rest of that gesture is not ours.
      this.#travel = Math.max(0, this.#travel - deltaY * PULL_WHEEL_SCALE);
      if (this.#travel === 0) {
        this.#reset();
        return false;
      }
      this.distance = band(this.#travel);
      return true;
    }

    // A new gesture: upward, allowed, and from rest.
    if (!ready || deltaY >= 0 || !rested) return false;
    this.by = "wheel";
    this.pulling = true;
    this.#travel = -deltaY * PULL_WHEEL_SCALE;
    this.distance = band(this.#travel);
    return true;
  }

  /** The finger lifted. Returns whether the wall should be laid again.
   *
   *  Reports rather than acts: `feed.refresh()` and closing an open reader are
   *  the trigger's job, the same way they are the button's, so this module
   *  imports neither and cannot start a fan-out by being poked in a test. */
  release(): boolean {
    const lay = this.by === "touch" && this.pulling && this.armed;
    this.#reset();
    return lay;
  }

  /** The wheel has gone quiet. Returns whether the wall should be laid again.
   *
   *  Stopping IS the release on a wheel, which is the one place the two inputs
   *  are not the same gesture. It answers only for a wheel pull: a finger that
   *  rests mid-drag has not let go of anything, and laying the wall under a
   *  motionless thumb would be a refresh nobody asked for. */
  settle(): boolean {
    const lay = this.by === "wheel" && this.pulling && this.armed;
    this.#reset();
    return lay;
  }

  /** Somebody else took the gesture: a second finger, a `touchcancel`, a
   *  screen arriving over the wall. Never lays the wall, whatever the distance. */
  cancel(): void {
    this.#reset();
  }

  /** End the gesture, and ONLY the gesture. `laying` is deliberately untouched:
   *  it is the aftermath of a pull rather than part of one, and it outlives the
   *  finger by the length of a warm. `start` resets before it decides whether to
   *  track, so clearing it here would drop the shelf out from under the wall the
   *  moment the reader touched a brick while their own refresh was still
   *  running. */
  #reset(): void {
    this.#tracking = false;
    this.pulling = false;
    this.distance = 0;
    this.by = null;
    this.#travel = 0;
    // NOT #lastWheel. It outlives the gesture on purpose: it is a record of when
    // the wheel last turned, not of what the wall was doing, and clearing it
    // here would hand a momentum tail its rest and let the flick that ended a
    // gesture start the next one.
  }
}

export const pull = new PullState();
