// Every decision the pull gesture makes: which drags become pulls and which are
// handed back to the browser, how far the band stretches, when it arms, and
// which releases lay the wall again. `PullToRefresh.svelte` is a .svelte file
// and no lane in this repo typechecks or renders one, which is exactly why the
// decisions live in a rune module and are pinned here.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { PULL_LAYING, PULL_MAX, PULL_SLOP, PULL_THRESHOLD, PullState } from "./pull.svelte";

/** A gesture, played through a fresh state: where the finger went down, then
 *  every place it moved to. Returns the state so a case can read what it did. */
function drag(points: Array<[number, number]>, ready = true): PullState {
  const pull = new PullState();
  const [first, ...rest] = points;
  pull.start({ x: first?.[0] ?? 0, y: first?.[1] ?? 0 }, ready);
  for (const [x, y] of rest) pull.move({ x, y });
  return pull;
}

/** Straight down from (0, 0) by `travel` px, in one move. */
function pullDown(travel: number): PullState {
  return drag([
    [0, 0],
    [0, travel],
  ]);
}

describe("PullState", () => {
  it("rests at zero, unarmed, and not pulling", () => {
    const pull = new PullState();
    expect(pull.distance).toBe(0);
    expect(pull.pulling).toBe(false);
    expect(pull.armed).toBe(false);
  });

  it("ignores the whole gesture when the caller says it is not ready", () => {
    // the wall is scrolled, or busy, or something is open over it: the caller
    // owns that question and answers it once, at touch down
    const pull = drag(
      [
        [0, 0],
        [0, 200],
      ],
      false,
    );
    expect(pull.pulling).toBe(false);
    expect(pull.distance).toBe(0);
    expect(pull.release()).toBe(false);
  });

  it("does not read a direction out of a finger that has barely moved", () => {
    const pull = drag([
      [0, 0],
      [0, PULL_SLOP - 1],
    ]);
    expect(pull.pulling).toBe(false);
    expect(pull.distance).toBe(0);
  });

  it("follows the finger 1:1 up to the threshold, so arming is honest", () => {
    expect(pullDown(PULL_SLOP).distance).toBe(PULL_SLOP);
    expect(pullDown(40).distance).toBe(40);
    expect(pullDown(PULL_THRESHOLD).distance).toBe(PULL_THRESHOLD);
  });

  it("stiffens past the threshold and never reaches the far end", () => {
    const far = pullDown(PULL_THRESHOLD + 40);
    // stretched, but by less than the finger travelled
    expect(far.distance).toBeGreaterThan(PULL_THRESHOLD);
    expect(far.distance).toBeLessThan(PULL_THRESHOLD + 40);
    // and a pull of any size stays inside the band
    expect(pullDown(2000).distance).toBeLessThan(PULL_MAX);
    expect(pullDown(2000).distance).toBeGreaterThan(PULL_MAX - 10);
  });

  it("arms at the threshold and not a pixel before", () => {
    expect(pullDown(PULL_THRESHOLD - 1).armed).toBe(false);
    expect(pullDown(PULL_THRESHOLD).armed).toBe(true);
    expect(pullDown(PULL_THRESHOLD + 100).armed).toBe(true);
  });

  it("claims the move once it is a pull, and only then", () => {
    const pull = new PullState();
    pull.start({ x: 0, y: 0 }, true);
    // under the slop: undecided, so the browser keeps the gesture
    expect(pull.move({ x: 0, y: PULL_SLOP - 1 })).toBe(false);
    // decided: from here the finger belongs to the wall
    expect(pull.move({ x: 0, y: 40 })).toBe(true);
    expect(pull.move({ x: 0, y: 90 })).toBe(true);
  });

  it("hands an upward drag back, so a scroll is never a pull", () => {
    const pull = drag([
      [0, 0],
      [0, -40],
      [0, 200],
    ]);
    // and it stays handed back for the rest of the gesture: a drag that wandered
    // back down through its own origin must not become a pull halfway
    expect(pull.pulling).toBe(false);
    expect(pull.distance).toBe(0);
    expect(pull.release()).toBe(false);
  });

  it("hands a sideways drag back, so a swipe is never a pull", () => {
    const pull = drag([
      [0, 0],
      [60, 20],
      [60, 200],
    ]);
    expect(pull.pulling).toBe(false);
    expect(pull.distance).toBe(0);
  });

  it("keeps a mostly-down drag that wanders a little across", () => {
    const pull = drag([
      [0, 0],
      [10, 80],
    ]);
    expect(pull.pulling).toBe(true);
    expect(pull.armed).toBe(true);
  });

  it("unwinds with a finger that eases back up, and lays nothing", () => {
    const pull = new PullState();
    pull.start({ x: 0, y: 0 }, true);
    pull.move({ x: 0, y: 100 });
    expect(pull.armed).toBe(true);
    // thought better of: the wall follows them all the way home
    pull.move({ x: 0, y: 30 });
    expect(pull.distance).toBe(30);
    pull.move({ x: 0, y: -20 });
    expect(pull.distance).toBe(0);
    expect(pull.release()).toBe(false);
  });

  it("lays the wall again on a release past the threshold, once", () => {
    const pull = pullDown(PULL_THRESHOLD + 20);
    expect(pull.release()).toBe(true);
    // and the gesture is over: a second release cannot re-fire it, which is
    // what keeps one gesture to one hundred-author fan-out
    expect(pull.release()).toBe(false);
    expect(pull.distance).toBe(0);
    expect(pull.pulling).toBe(false);
  });

  it("lays nothing on a release short of the threshold", () => {
    const pull = pullDown(PULL_THRESHOLD - 1);
    expect(pull.release()).toBe(false);
    expect(pull.distance).toBe(0);
  });

  it("lays nothing when the gesture is cancelled, however far it got", () => {
    const pull = pullDown(PULL_MAX);
    expect(pull.armed).toBe(true);
    // a second finger, a touchcancel, a screen arriving over the wall
    pull.cancel();
    expect(pull.distance).toBe(0);
    expect(pull.pulling).toBe(false);
    expect(pull.release()).toBe(false);
  });

  it("starts each gesture from where the finger went down, not from the wall", () => {
    // the second gesture starts 500px down the screen; distance is travel from
    // the finger's own origin, never the coordinate it happens to be at
    const pull = new PullState();
    pull.start({ x: 0, y: 500 }, true);
    pull.move({ x: 0, y: 540 });
    expect(pull.distance).toBe(40);
  });

  it("puts the wall where the finger is while a finger is on it", () => {
    const pull = pullDown(40);
    expect(pull.offset).toBe(40);
    expect(pull.offset).toBe(pull.distance);
  });

  it("rests the wall on the shelf while the refresh it asked for runs", () => {
    const pull = pullDown(PULL_THRESHOLD + 20);
    expect(pull.release()).toBe(true);
    // the gesture is over and the wall is home, until the trigger says the
    // refresh took
    expect(pull.offset).toBe(0);
    pull.laying = true;
    expect(pull.offset).toBe(PULL_LAYING);
    // and the warm ending is what lets it down: the gap closing IS the signal
    pull.laying = false;
    expect(pull.offset).toBe(0);
  });

  it("keeps the wall on its shelf when a finger lands mid-refresh", () => {
    // a reader touching a brick while their own refresh runs. `start` resets the
    // gesture, and the shelf is not part of the gesture: dropping the wall here
    // would yank it out from under the tap.
    const pull = new PullState();
    pull.laying = true;
    pull.start({ x: 0, y: 200 }, false);
    expect(pull.laying).toBe(true);
    expect(pull.offset).toBe(PULL_LAYING);
    pull.cancel();
    expect(pull.laying).toBe(true);
    expect(pull.offset).toBe(PULL_LAYING);
  });

  it("gives the finger the wall even if a shelf is still up", () => {
    const pull = new PullState();
    pull.laying = true;
    pull.start({ x: 0, y: 0 }, true);
    pull.move({ x: 0, y: 30 });
    // pulling beats laying: one of them is a hand on the wall
    expect(pull.offset).toBe(30);
  });

  it("names no DOM global", () => {
    // The gesture arrives as touch events and this module reads none of them:
    // the caller hands over two numbers and one boolean. That is what keeps
    // these cases running for real in node rather than against a fake DOM, and
    // it is a claim about the source, so it is checked against the source.
    const source = readFileSync(new URL("./pull.svelte.ts", import.meta.url), "utf8");
    expect(
      source.match(
        /\b(?:window|document|navigator|history|location|localStorage|sessionStorage|scrollTo|scrollBy|scrollIntoView|requestAnimationFrame|TouchEvent|HTMLElement)\b/,
      ),
    ).toBeNull();
  });
});
