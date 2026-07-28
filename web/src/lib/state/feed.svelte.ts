import { fetchFeed, FeedError, type FeedTarget } from "$lib/api";
import type { Brick, FeedMode, MortarErrorCode } from "$lib/types";

/** How long to keep reflowing before freezing anyway, even if the wall says it
 *  is still warming. A wall that never settles must still become scrollable. */
const WARM_CEILING_MS = 8000;
/** Gap between preview polls. Short enough that the reflow feels live, long
 *  enough that the wall is not re-mixed on every single brick that lands. */
const POLL_MS = 350;

const sleep = (ms: number) => new Promise((resume) => setTimeout(resume, ms));

/** A laid wall kept for the length of the session, so back/forward returns the
 *  same arrangement (and scroll) instead of rolling a new seed. */
interface Snapshot {
  items: Brick[];
  cursor: string | null;
  done: boolean;
  seen: Set<string>;
}

/** Exported for the unit tests, which build throwaway instances; the app only
 *  ever uses the `feed` singleton below. */
export class FeedState {
  items = $state<Brick[]>([]);
  cursor = $state<string | null>(null);
  loading = $state(false);
  initialLoad = $state(true);
  /** The first screen is still being reflowed from a growing pool; a scroll (or
   *  the wall settling) freezes it and hands over to normal pagination. */
  warming = $state(false);
  done = $state(false);
  error = $state<string | null>(null);

  /** Which wall is laid, or null before the first reset. Null rather than an
   *  empty-actor sentinel: "no wall yet" is a state a two-shape target cannot
   *  spell, and every read of it is a guard that has to hold anyway. */
  #target: FeedTarget | null = null;
  #mode: FeedMode | undefined;
  #seen = new Set<string>();
  // bumped on every reset/freeze so a superseded preview loop bows out
  #generation = 0;
  // per target+mode, the last laid wall this session (for back/forward)
  #cache = new Map<string, Snapshot>();
  /** The reader asked for this wall on purpose and no cursorless request has
   *  been ADOPTED since. Read by the two requests that can be cursorless
   *  (`#warm`'s poll and `freeze`'s commit) and spent when an answer lands,
   *  never when a request is issued: a request whose result is thrown away
   *  would otherwise spend the refresh and the wall that finally commits would
   *  be the unrefreshed one. Spent on a THROW too, in `#warm`'s catch, which is
   *  a fourth disarm point the spec's three do not name and the one that keeps
   *  one tap to one fan-out on the error path. */
  #refreshPending = false;
  /** A refresh's own cursorless request is in flight. Holds `freeze` off until
   *  it settles, so one tap can only ever produce one cursorless request. See
   *  the guard in `freeze` for why holding is the only shape that works. */
  #refreshInFlight = false;

  /** The session cache key for one wall.
   *
   *  Built from the target's KIND and its value, never from the target object:
   *  an object interpolated into a template literal stringifies to
   *  "[object Object]", which would collapse every wall in the session onto one
   *  entry. The kind is carried because the two sources share one namespace of
   *  strings, so a feed reference spelled like a handle cannot rehydrate that
   *  handle's graph wall. The separator is the unit separator, which no handle,
   *  AT-URI or mode contains, and which is what mortar keys its own feed pages
   *  with. */
  #key(target: FeedTarget, mode?: FeedMode) {
    const source = "feed" in target ? `feed\u{1f}${target.feed}` : `actor\u{1f}${target.actor}`;
    return `${source}\u{1f}${mode ?? ""}`;
  }

  /** Remember the current committed wall so returning to it rehydrates instead
   *  of rolling a fresh seed. Only ever called for a settled (non-warming) wall. */
  #save() {
    if (!this.#target) return;
    this.#cache.set(this.#key(this.#target, this.#mode), {
      items: this.items.slice(),
      cursor: this.cursor,
      done: this.done,
      seen: new Set(this.#seen),
    });
  }

  reset(target: FeedTarget, mode?: FeedMode) {
    this.#target = target;
    this.#mode = mode;
    this.error = null;
    this.loading = false;
    // a new wall is not the refreshed one: neither half of a refresh may
    // survive into it, whether the refresh settled or was superseded mid-flight
    this.#refreshPending = false;
    this.#refreshInFlight = false;
    const cached = this.#cache.get(this.#key(target, mode));
    if (cached) {
      // returning to a wall already laid this session (back/forward): rehydrate
      // it exactly, keeping the seed, the arrangement and the scroll, rather
      // than rolling a new snapshot that lands the reader on a skeleton.
      this.#generation++; // bow out any in-flight preview/freeze/pagination
      this.items = cached.items.slice();
      this.cursor = cached.cursor;
      this.done = cached.done;
      this.#seen = new Set(cached.seen);
      this.warming = false;
      this.initialLoad = false;
      return;
    }
    this.items = [];
    this.cursor = null;
    this.done = false;
    this.initialLoad = true;
    this.warming = true;
    this.#seen = new Set();
    const generation = ++this.#generation;
    void this.#warm(generation, target);
  }

  /** Lay this wall again, now. The reader asked, so the new wall re-reads the
   *  fast content caches upstream rather than reshuffling the same bricks.
   *
   *  Refuses while loading, while warming, or with no wall at all, and refuses
   *  without side effects: the control is the whole rate limit, so a double tap
   *  must not become two hundred-author fan-outs. Guarded in the same shape as
   *  `loadMore`, minus `done`: a wall that ran out of bricks is exactly the one
   *  worth asking again. */
  refresh() {
    const target = this.#target;
    if (this.loading || this.warming || !target) return;
    // BEFORE the generation bump, so a reset that lands mid-refresh (a back
    // gesture, a re-navigation to the same wall) lays the wall again instead of
    // handing back the arrangement the reader just asked to replace
    this.#cache.delete(this.#key(target, this.#mode));
    this.cursor = null;
    this.done = false;
    this.error = null;
    this.warming = true;
    // `items` are LEFT ON THE WALL and `initialLoad` is untouched: the outgoing
    // wall reflows into the new one through the same #replace the warm loop
    // already uses, rather than collapsing to the twelve-card skeleton grid,
    // which mid-session reads as something breaking rather than as a refresh
    this.#refreshPending = true;
    this.#refreshInFlight = true;
    const generation = ++this.#generation;
    void this.#warm(generation, target);
  }

  /** Poll the wall for its current best first screen and reflow it in place,
   *  until it settles, the reader scrolls (see `freeze`), or the ceiling hits.
   *
   *  The target is passed in rather than read back off the instance: this loop
   *  outlives its own reset, and a target read after a switch would send the
   *  new wall's name with the old wall's cursor for the one poll it takes the
   *  generation check to notice. */
  async #warm(generation: number, target: FeedTarget) {
    const until = Date.now() + WARM_CEILING_MS;
    try {
      // the poll is inherently sequential: each request, the reflow it drives,
      // and the pause before the next depend on the one before
      while (generation === this.#generation && this.warming) {
        // the flag rides on a cursorless request only, which is the only shape
        // mortar honours it on: a cursor means a later page, and a refresh is
        // always a first page
        const flagged = this.#refreshPending && this.cursor === null;
        // oxlint-disable-next-line no-await-in-loop
        const page = await fetchFeed(target, this.cursor, this.#mode, "preview", flagged);
        if (generation !== this.#generation) return; // a newer wall took over
        if (flagged) {
          // adopted, so the refresh is spent on the answer actually being kept,
          // and the commit held behind the marker is free to go
          this.#refreshPending = false;
          this.#refreshInFlight = false;
        }
        // the preview cursor carries the seed, so the next poll and the freeze
        // land on this same warming snapshot instead of rolling a new one
        this.cursor = page.cursor;
        this.#replace(page.items);
        if (this.items.length > 0) this.initialLoad = false;
        if (!page.warming || Date.now() > until) {
          // oxlint-disable-next-line no-await-in-loop
          await this.freeze(generation);
          return;
        }
        // oxlint-disable-next-line no-await-in-loop
        await sleep(POLL_MS);
      }
    } catch (e) {
      if (generation !== this.#generation) return; // a newer wall took over
      if (this.#refreshInFlight) {
        // the refresh's own request is the first thing this loop awaits, so a
        // throw with the marker still set IS that request settling. Release the
        // marker BEFORE asking for the commit: `freeze` is held while it is
        // set, so a refresh whose preview threw would otherwise leave the wall
        // warming forever behind its own guard.
        this.#refreshInFlight = false;
        // and spend the flag with it. This is a FOURTH disarm point, and the
        // three the spec names (a cursorless response is ADOPTED, `freeze`'s
        // `finally`, the next `reset`) reach none of this path in time: the
        // commit below is cursorless, because `refresh` nulled the cursor, so
        // an unspent flag would ride it and one tap would have issued two
        // flagged cursorless requests, sequentially. That is two fresh seeds,
        // two snapshots and two hundred-author fan-outs, which is the exact
        // thing the marker exists to prevent, so a flagged request that settles
        // WITHOUT being adopted has to spend the flag as surely as an adopted
        // one does. Do not tidy this back to three.
        // The price is that a refresh whose first request never answered
        // commits an unrefreshed wall. That is the cheaper half: the control is
        // live again the moment this freeze settles, so the reader can ask
        // again, which costs one more tap rather than a second fan-out nobody
        // asked for.
        this.#refreshPending = false;
      }
      // a preview failed; commit what a real request gives us, which also
      // surfaces a real error (a sealed wall, a bad handle) properly
      await this.freeze(generation, e);
    }
  }

  /** Lock the reflow: commit the first screen and switch to pagination. Called
   *  when the wall settles, the ceiling hits, or the reader scrolls. */
  async freeze(generation = this.#generation, previewError?: unknown) {
    // captured before the await, and the guard below covers the wall-less case
    // this can be called in: the grid freezes on the reader's first engagement,
    // which can land before any wall has been asked for
    const target = this.#target;
    // during warming only an in-flight freeze sets loading, so the loading
    // guard makes a second engagement while the freeze fetch runs a no-op
    if (!target || !this.warming || this.loading || generation !== this.#generation) return;
    // A refresh's own cursorless request is still in flight, so this commit is
    // HELD rather than sent, with no side effect at all: not the generation
    // bump, not `loading`. Deferring is the whole mechanism and merely
    // stripping this request's flag would be worse than doing nothing. A
    // cursorless request rolls its own fresh seed, builds a second snapshot and
    // fills it from the untouched five-minute author-feed cache, so it clears
    // the twelve-author first-paint gate off cache hits alone while the
    // refreshing fill is still working through a hundred rate-limited AppView
    // calls: the unflagged one wins, and what it commits is the wall from
    // BEFORE the refresh. Flagging both is not the answer either, because that
    // is two seeds, two snapshots and two hundred-author fan-outs from one tap.
    // Nothing is lost by holding: `#warm` adopts the preview's cursor, which
    // carries the refreshing snapshot's seed, and freezes from there itself
    // when the wall settles or the ceiling hits. This is not a rare race, it is
    // the ordinary path under `prefers-reduced-motion: reduce`, where the grid
    // freezes the instant `warming` flips true with no scroll event at all.
    // A held commit is dropped rather than queued, so a reader who engaged
    // during a refresh waits for the refreshing snapshot (or the 8s ceiling)
    // instead of committing the moment the marker clears. That is the trade the
    // hold makes on purpose: what they would commit early is a wall built from
    // a different seed.
    if (this.#refreshInFlight) return;
    // supersede the preview loop; from here the wall never moves (the loop
    // rechecks the generation after every poll, so no late preview lands)
    const gen = ++this.#generation;
    this.loading = true;
    this.error = null;
    // same rule as the poll's: cursorless only. A backstop rather than a live
    // path today: every way a refresh's own request can settle now spends the
    // flag (adopted in the loop above, thrown in its catch), and the flag is
    // never armed without the marker, which holds this commit off, so a refresh
    // cannot reach here still armed. Kept as the read the spec asks for, so a
    // future path that releases the marker without spending the flag flags the
    // request that COMMITS rather than laying an unrefreshed wall.
    const flagged = this.#refreshPending && this.cursor === null;
    try {
      const page = await fetchFeed(target, this.cursor, this.#mode, "freeze", flagged);
      if (this.#generation !== gen) return; // a newer wall took over
      this.#replace(page.items);
      this.cursor = page.cursor;
      if (!page.cursor) this.done = true;
      this.#save();
    } catch (e) {
      if (this.#generation !== gen) return; // a newer wall took over
      this.#fail(previewError ?? e);
    } finally {
      if (this.#generation === gen) {
        // the wall has settled, adopted or failed, so a refresh that reached
        // this commit is over: disarm here rather than at issue time and the
        // flag can never leak into a later wall
        this.#refreshPending = false;
        // warming flips off HERE, in the same synchronous continuation the
        // committed order (or the error) lands, so the wall sees a single
        // update that both ends warming and carries the final arrangement,
        // and re-places exactly that update instead of a bare flag flip
        this.warming = false;
        this.loading = false;
        this.initialLoad = false;
      }
    }
  }

  async loadMore() {
    const target = this.#target;
    // while warming the reflow owns the wall; pagination waits for the freeze
    if (this.loading || this.done || this.warming || !target) return;
    const gen = this.#generation;
    this.loading = true;
    this.error = null;
    try {
      const page = await fetchFeed(target, this.cursor, this.#mode);
      if (this.#generation !== gen) return; // a newer wall took over
      // belt-and-braces dedupe across pages
      const fresh = page.items.filter((b) => !this.#seen.has(b.id));
      for (const b of fresh) this.#seen.add(b.id);
      this.items.push(...fresh);
      this.cursor = page.cursor;
      if (!page.cursor) this.done = true;
      this.#save();
    } catch (e) {
      if (this.#generation !== gen) return; // a newer wall took over
      this.#fail(e);
    } finally {
      if (this.#generation === gen) {
        this.loading = false;
        this.initialLoad = false;
      }
    }
  }

  /** Replace the whole first screen with a new arrangement, deduped. The grid
   *  keys bricks by id, so shared bricks reorder in place and only genuinely new
   *  ones animate in. `#seen` is rebuilt so pagination after the freeze dedupes
   *  against exactly what is on the wall. */
  #replace(items: Brick[]) {
    const seen = new Set<string>();
    const fresh: Brick[] = [];
    for (const b of items) {
      if (!seen.has(b.id)) {
        seen.add(b.id);
        fresh.push(b);
      }
    }
    this.items = fresh;
    this.#seen = seen;
  }

  #fail(e: unknown) {
    // the compared literals are typed MortarErrorCode, so a code renamed in
    // mortar (and the regenerated contract fixture) fails typechecking here
    if (e instanceof FeedError && e.code === ("login_required" satisfies MortarErrorCode)) {
      // the owner asked to be seen only by signed-in visitors; mason is a
      // logged-out reader, so this wall stays sealed
      this.error = "login-required";
    } else if (e instanceof FeedError && e.code === ("actor_not_found" satisfies MortarErrorCode)) {
      // only mortar's own actor-not-found envelope means the handle is bad. In
      // local mode a request that escapes the service worker hits the static
      // host and 404s with a non-JSON error doc (code "unknown"), which must not
      // be mistaken for a missing handle.
      this.error = "handle-not-found";
    } else if (e instanceof FeedError && e.code === ("feed_not_found" satisfies MortarErrorCode)) {
      // a reference naming no generator mortar can page: unknown, withdrawn, or
      // mistyped. It carries its own code precisely so it cannot land in the
      // branch above, which hands back a handle box to fix a handle the reader
      // never typed.
      this.error = "feed-not-found";
    } else {
      this.error = "feed-unavailable";
    }
  }
}

export const feed = new FeedState();
