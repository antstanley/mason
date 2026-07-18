import { fetchFeed, FeedError } from "$lib/api";
import type { Brick } from "$lib/types";

/** How long to keep reflowing before freezing anyway, even if the wall says it
 *  is still warming. A wall that never settles must still become scrollable. */
const WARM_CEILING_MS = 8000;
/** Gap between preview polls. Short enough that the reflow feels live, long
 *  enough that the wall is not re-mixed on every single brick that lands. */
const POLL_MS = 350;

const sleep = (ms: number) => new Promise((resume) => setTimeout(resume, ms));

class FeedState {
  items = $state<Brick[]>([]);
  cursor = $state<string | null>(null);
  loading = $state(false);
  initialLoad = $state(true);
  /** The first screen is still being reflowed from a growing pool; a scroll (or
   *  the wall settling) freezes it and hands over to normal pagination. */
  warming = $state(false);
  done = $state(false);
  error = $state<string | null>(null);

  #actor = "";
  #mode: string | undefined;
  #seen = new Set<string>();
  // bumped on every reset/freeze so a superseded preview loop bows out
  #generation = 0;

  reset(actor: string, mode?: string) {
    this.#actor = actor;
    this.#mode = mode;
    this.items = [];
    this.cursor = null;
    this.done = false;
    this.error = null;
    this.initialLoad = true;
    this.warming = true;
    this.#seen.clear();
    const generation = ++this.#generation;
    void this.#warm(generation);
  }

  /** Poll the wall for its current best first screen and reflow it in place,
   *  until it settles, the reader scrolls (see `freeze`), or the ceiling hits. */
  async #warm(generation: number) {
    const until = Date.now() + WARM_CEILING_MS;
    try {
      // the poll is inherently sequential: each request, the reflow it drives,
      // and the pause before the next depend on the one before
      while (generation === this.#generation && this.warming) {
        // oxlint-disable-next-line no-await-in-loop
        const page = await fetchFeed(this.#actor, this.cursor, this.#mode, "preview");
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
    if (!this.warming || generation !== this.#generation) return;
    // supersede the preview loop; from here the wall never moves
    this.#generation++;
    this.warming = false;
    this.loading = true;
    this.error = null;
    try {
      const page = await fetchFeed(this.#actor, this.cursor, this.#mode, "freeze");
      this.#replace(page.items);
      this.cursor = page.cursor;
      if (!page.cursor) this.done = true;
    } catch (e) {
      this.#fail(previewError ?? e);
    } finally {
      this.loading = false;
      this.initialLoad = false;
    }
  }

  async loadMore() {
    // while warming the reflow owns the wall; pagination waits for the freeze
    if (this.loading || this.done || this.warming || !this.#actor) return;
    this.loading = true;
    this.error = null;
    try {
      const page = await fetchFeed(this.#actor, this.cursor, this.#mode);
      // belt-and-braces dedupe across pages
      const fresh = page.items.filter((b) => !this.#seen.has(b.id));
      for (const b of fresh) this.#seen.add(b.id);
      this.items.push(...fresh);
      this.cursor = page.cursor;
      if (!page.cursor) this.done = true;
    } catch (e) {
      this.#fail(e);
    } finally {
      this.loading = false;
      this.initialLoad = false;
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
    if (e instanceof FeedError && e.code === "login_required") {
      // the owner asked to be seen only by signed-in visitors; mason is a
      // logged-out reader, so this wall stays sealed
      this.error = "login-required";
    } else if (e instanceof FeedError && e.status === 404) {
      this.error = "handle-not-found";
    } else {
      this.error = "feed-unavailable";
    }
  }
}

export const feed = new FeedState();
