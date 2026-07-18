import { fetchFeed, FeedError } from "$lib/api";
import type { Brick } from "$lib/types";

class FeedState {
  items = $state<Brick[]>([]);
  cursor = $state<string | null>(null);
  loading = $state(false);
  initialLoad = $state(true);
  done = $state(false);
  error = $state<string | null>(null);

  #actor = "";
  #seen = new Set<string>();

  reset(actor: string) {
    this.#actor = actor;
    this.items = [];
    this.cursor = null;
    this.done = false;
    this.error = null;
    this.initialLoad = true;
    this.#seen.clear();
    void this.loadMore();
  }

  async loadMore() {
    if (this.loading || this.done || !this.#actor) return;
    this.loading = true;
    this.error = null;
    try {
      const page = await fetchFeed(this.#actor, this.cursor);
      // belt-and-braces dedupe across pages
      const fresh = page.items.filter((b) => !this.#seen.has(b.id));
      for (const b of fresh) this.#seen.add(b.id);
      this.items.push(...fresh);
      this.cursor = page.cursor;
      if (!page.cursor) this.done = true;
    } catch (e) {
      if (e instanceof FeedError && e.code === "login_required") {
        // the owner asked to be seen only by signed-in visitors; mason is a
        // logged-out reader, so this wall stays sealed
        this.error = "login-required";
      } else if (e instanceof FeedError && e.status === 404) {
        this.error = "handle-not-found";
      } else {
        this.error = "feed-unavailable";
      }
    } finally {
      this.loading = false;
      this.initialLoad = false;
    }
  }
}

export const feed = new FeedState();
