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
        // oxlint-disable-next-line no-await-in-loop
        const page = await fetchFeed(target, this.cursor, this.#mode, "preview");
        if (generation !== this.#generation) return; // a newer wall took over
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
      // a preview failed; commit what a real request gives us, which also
      // surfaces a real error (a sealed wall, a bad handle) properly
      if (generation === this.#generation) await this.freeze(generation, e);
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
    // supersede the preview loop; from here the wall never moves (the loop
    // rechecks the generation after every poll, so no late preview lands)
    const gen = ++this.#generation;
    this.loading = true;
    this.error = null;
    try {
      const page = await fetchFeed(target, this.cursor, this.#mode, "freeze");
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
