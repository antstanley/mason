// FeedState is the TypeScript half of the feed protocol: the warming poll
// loop, the freeze handshake, pagination dedupe and the per-session snapshot
// cache. These tests pin its transitions against a mocked fetchFeed; the wire
// format itself is pinned on the Rust side.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fetchFeed, FeedError, type FeedTarget } from "$lib/api";
import { FeedState } from "./feed.svelte";
import type { Brick, FeedResponse } from "$lib/types";

vi.mock("$lib/api", () => {
  // mirrors the real FeedError shape without importing the real module,
  // which would drag in $app/environment
  class MockFeedError extends Error {
    constructor(
      public code: string,
      public status: number,
    ) {
      super(`feed error: ${code} (${status})`);
    }
  }
  return { fetchFeed: vi.fn(), FeedError: MockFeedError };
});

const mockFetchFeed = vi.mocked(fetchFeed);

function brick(id: string): Brick {
  return {
    kind: "post",
    id,
    url: `https://example.test/${id}`,
    author: { did: "did:plc:x", handle: "x.test", displayName: null, avatar: null },
    text: id,
    createdAt: "2026-01-01T00:00:00Z",
    likeCount: 0,
    repostCount: 0,
    images: [],
    external: null,
  };
}

function page(ids: string[], cursor: string | null, warming?: boolean): FeedResponse {
  const res: FeedResponse = { items: ids.map(brick), cursor };
  if (warming !== undefined) res.warming = warming;
  return res;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const ids = (feed: FeedState) => feed.items.map((b) => b.id);
const intents = () => mockFetchFeed.mock.calls.map((c) => c[3]);

/** How a mocked wall names its bricks. A feed target is marked as one, so a
 *  test can tell the two sources apart when they are spelled identically, which
 *  is exactly the collision the session cache key has to survive. */
const label = (target: FeedTarget) => ("feed" in target ? `feed-${target.feed}` : target.actor);

/** Wire the mock so any wall settles on its first preview and commits on the
 *  freeze: preview -> [<wall>-1] cursor `<wall>-p`, freeze -> same brick,
 *  cursor `<wall>-c`. */
function settleImmediately() {
  mockFetchFeed.mockImplementation((target, _cursor, _mode, intent) =>
    Promise.resolve(
      intent === "freeze"
        ? page([`${label(target)}-1`], `${label(target)}-c`)
        : page([`${label(target)}-1`], `${label(target)}-p`, false),
    ),
  );
}

beforeEach(() => {
  vi.useFakeTimers();
  mockFetchFeed.mockReset();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("warming", () => {
  it("polls previews, settles, then freezes exactly once reusing the preview cursor", async () => {
    const feed = new FeedState();
    mockFetchFeed
      .mockResolvedValueOnce(page(["a"], "c1", true)) // preview 1: still warming
      .mockResolvedValueOnce(page(["a", "b"], "c2", false)) // preview 2: settled
      .mockResolvedValueOnce(page(["b", "a"], "c3")); // freeze commits

    feed.reset({ actor: "alice" });
    expect(feed.warming).toBe(true);
    expect(feed.initialLoad).toBe(true);

    await vi.advanceTimersByTimeAsync(0); // preview 1 lands
    expect(feed.warming).toBe(true);
    expect(ids(feed)).toEqual(["a"]);
    expect(feed.initialLoad).toBe(false); // first bricks lift the skeleton

    await vi.advanceTimersByTimeAsync(350); // poll gap, preview 2, freeze
    expect(feed.warming).toBe(false);
    expect(feed.loading).toBe(false);
    expect(ids(feed)).toEqual(["b", "a"]); // freeze's arrangement wins
    expect(feed.cursor).toBe("c3");
    // each request carries the cursor of the one before, so poll and freeze
    // stay on the same warming snapshot (same seed) instead of re-rolling
    expect(mockFetchFeed.mock.calls).toEqual([
      [{ actor: "alice" }, null, undefined, "preview"],
      [{ actor: "alice" }, "c1", undefined, "preview"],
      [{ actor: "alice" }, "c2", undefined, "freeze"],
    ]);

    // frozen means frozen: no poll ever fires again
    await vi.advanceTimersByTimeAsync(10_000);
    expect(mockFetchFeed).toHaveBeenCalledTimes(3);
  });

  it("freezes at the 8s ceiling even if the wall never settles", async () => {
    const feed = new FeedState();
    mockFetchFeed.mockImplementation((_target, _cursor, _mode, intent) =>
      Promise.resolve(intent === "freeze" ? page(["a", "b"], "cz") : page(["a"], "cp", true)),
    );

    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(7500);
    expect(feed.warming).toBe(true); // still under the ceiling, still polling

    await vi.advanceTimersByTimeAsync(2000); // crosses 8000ms
    expect(feed.warming).toBe(false);
    expect(feed.cursor).toBe("cz");
    expect(intents().filter((i) => i === "freeze")).toHaveLength(1);
    expect(intents().at(-1)).toBe("freeze");
    const polled = mockFetchFeed.mock.calls.length;

    // and the loop is dead after the forced freeze
    await vi.advanceTimersByTimeAsync(10_000);
    expect(mockFetchFeed).toHaveBeenCalledTimes(polled);
  });

  it("a scroll-freeze supersedes the poll loop mid-gap", async () => {
    const feed = new FeedState();
    mockFetchFeed.mockResolvedValueOnce(page(["a"], "c1", true)); // preview 1
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0); // preview 1 lands, loop sleeps 350ms
    expect(feed.warming).toBe(true);

    mockFetchFeed.mockResolvedValueOnce(page(["a", "b"], "c2")); // the freeze
    const frozen = feed.freeze(); // the reader scrolled
    // the poll loop is superseded at once (generation bump), but warming only
    // drops when the freeze settles, in the same tick as the committed order,
    // so the wall re-places exactly the update carrying the final arrangement
    expect(feed.loading).toBe(true);
    expect(feed.warming).toBe(true);
    void feed.freeze(); // a second engagement mid-fetch: no-op via the loading guard
    await frozen;
    expect(feed.warming).toBe(false);
    expect(ids(feed)).toEqual(["a", "b"]);
    expect(feed.cursor).toBe("c2");
    // the freeze committed the warming snapshot the preview was building
    expect(mockFetchFeed).toHaveBeenLastCalledWith({ actor: "alice" }, "c1", undefined, "freeze");

    // the sleeping poll wakes, sees a newer generation and bows out silently
    await vi.advanceTimersByTimeAsync(5000);
    expect(mockFetchFeed).toHaveBeenCalledTimes(2);
  });

  it("a second freeze is a no-op once the wall is frozen", async () => {
    const feed = new FeedState();
    settleImmediately();
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(false);
    const calls = mockFetchFeed.mock.calls.length;

    await feed.freeze();
    expect(mockFetchFeed).toHaveBeenCalledTimes(calls);
  });

  it("ignores a stale wall response after a switch (FE-1)", async () => {
    const feed = new FeedState();
    const alicePreview = deferred<FeedResponse>();
    mockFetchFeed.mockImplementation((target, _cursor, _mode, intent) => {
      if (label(target) === "alice") return alicePreview.promise;
      return Promise.resolve(
        intent === "freeze" ? page(["bob-1"], "bob-c") : page(["bob-1"], "bob-p", false),
      );
    });

    feed.reset({ actor: "alice" }); // preview in flight, never resolves yet
    feed.reset({ actor: "bob" }); // the reader switched walls
    await vi.advanceTimersByTimeAsync(0); // bob settles and freezes
    expect(ids(feed)).toEqual(["bob-1"]);
    expect(feed.cursor).toBe("bob-c");

    // alice's stale preview finally lands; it must not touch bob's wall
    alicePreview.resolve(page(["alice-1"], "alice-p", true));
    await vi.advanceTimersByTimeAsync(5000);
    expect(ids(feed)).toEqual(["bob-1"]);
    expect(feed.cursor).toBe("bob-c");
    // and the superseded warm loop never issued an alice freeze
    expect(mockFetchFeed.mock.calls.filter((c) => label(c[0]) === "alice")).toHaveLength(1);
  });

  it("a failed preview still commits the wall through the freeze", async () => {
    const feed = new FeedState();
    mockFetchFeed
      .mockRejectedValueOnce(new Error("preview blip"))
      .mockResolvedValueOnce(page(["a"], "c1"));

    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.error).toBeNull();
    expect(feed.warming).toBe(false);
    expect(ids(feed)).toEqual(["a"]);
  });
});

describe("error mapping (FE-3)", () => {
  it.each([
    ["login_required", "login-required"],
    ["actor_not_found", "handle-not-found"],
    // its own token, not the handle one: the panel a bad feed reference gets
    // must not ask the reader to check a handle they never typed
    ["feed_not_found", "feed-not-found"],
    ["rate_limited", "feed-unavailable"],
    ["unknown", "feed-unavailable"], // a static-host 404 is not a bad handle
  ])("maps a FeedError %s wall to the %s token", async (code, token) => {
    const feed = new FeedState();
    mockFetchFeed.mockRejectedValue(new FeedError(code, 400));
    // laid on the kind of wall the code can actually arrive on: mortar only
    // emits feed_not_found for a feed target. The mapping reads the code and
    // never the target, which is why one table covers both kinds of wall.
    feed.reset(
      code === "feed_not_found"
        ? { feed: "at://did:plc:x/app.bsky.feed.generator/gone" }
        : { actor: "alice" },
    );
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.error).toBe(token);
    expect(feed.warming).toBe(false);
    expect(feed.loading).toBe(false);
    expect(feed.initialLoad).toBe(false);
    expect(feed.items).toEqual([]);
  });

  it("maps a non-FeedError failure to feed-unavailable", async () => {
    const feed = new FeedState();
    mockFetchFeed.mockRejectedValue(new TypeError("network down"));
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.error).toBe("feed-unavailable");
  });

  it("keeps the laid wall when a later page fails", async () => {
    const feed = new FeedState();
    settleImmediately();
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);

    mockFetchFeed.mockRejectedValueOnce(new FeedError("actor_not_found", 404));
    await feed.loadMore();
    expect(feed.error).toBe("handle-not-found");
    expect(ids(feed)).toEqual(["alice-1"]); // the bricks already laid stay up
    expect(feed.loading).toBe(false);
  });
});

describe("pagination", () => {
  it("dedupes bricks across pages against everything on the wall", async () => {
    const feed = new FeedState();
    mockFetchFeed
      .mockResolvedValueOnce(page(["a", "b"], "c1", false)) // preview, settled
      .mockResolvedValueOnce(page(["a", "b"], "c2")); // freeze
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);

    mockFetchFeed.mockResolvedValueOnce(page(["b", "c"], null)); // b repeats
    await feed.loadMore();
    expect(ids(feed)).toEqual(["a", "b", "c"]);
    expect(feed.done).toBe(true); // a null cursor ends the wall
    // a committed page carries no intent at all
    expect(mockFetchFeed.mock.calls.at(-1)).toEqual([{ actor: "alice" }, "c2", undefined]);
  });

  it("does not paginate while the wall is still warming", async () => {
    const feed = new FeedState();
    mockFetchFeed.mockResolvedValue(page(["a"], "c1", true));
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(true);

    await feed.loadMore();
    expect(intents().every((i) => i === "preview")).toBe(true);
  });
});

describe("session cache (FE-9)", () => {
  it("rehydrates a revisited wall without refetching, seen set intact", async () => {
    const feed = new FeedState();
    settleImmediately();
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    feed.reset({ actor: "bob" });
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["bob-1"]);
    const calls = mockFetchFeed.mock.calls.length;

    feed.reset({ actor: "alice" }); // back/forward returns to alice
    expect(mockFetchFeed).toHaveBeenCalledTimes(calls); // no refetch at all
    expect(ids(feed)).toEqual(["alice-1"]);
    expect(feed.cursor).toBe("alice-c"); // the frozen cursor, not the preview's
    expect(feed.warming).toBe(false);
    expect(feed.initialLoad).toBe(false); // no skeleton on a rehydrated wall

    // the snapshot's seen set still dedupes the next page
    mockFetchFeed.mockResolvedValueOnce(page(["alice-1", "alice-2"], null));
    await feed.loadMore();
    expect(ids(feed)).toEqual(["alice-1", "alice-2"]);
  });

  it("caches per target+mode: the same actor in another mode warms afresh", async () => {
    const feed = new FeedState();
    settleImmediately();
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    const calls = mockFetchFeed.mock.calls.length;

    feed.reset({ actor: "alice" }, "glaze"); // same wall, images-only algorithm
    expect(feed.warming).toBe(true);
    await vi.advanceTimersByTimeAsync(0);
    expect(mockFetchFeed.mock.calls.length).toBeGreaterThan(calls);
    expect(mockFetchFeed.mock.calls.at(-1)?.[2]).toBe("glaze");
  });

  it("a graph wall and a feed wall never rehydrate into each other", async () => {
    const feed = new FeedState();
    settleImmediately();
    feed.reset({ actor: "alice" });
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["alice-1"]);
    const calls = mockFetchFeed.mock.calls.length;

    // deliberately the same string as the actor above. A key built from the
    // target object would stringify to "[object Object]" and a key built from
    // its value alone would match "alice": either way this reset would hand
    // back alice's graph wall instead of laying the feed.
    feed.reset({ feed: "alice" });
    expect(feed.warming).toBe(true); // a fresh wall, not a rehydration
    await vi.advanceTimersByTimeAsync(0);
    expect(mockFetchFeed.mock.calls.length).toBeGreaterThan(calls);
    expect(mockFetchFeed.mock.calls.at(-1)?.[0]).toEqual({ feed: "alice" });
    expect(ids(feed)).toEqual(["feed-alice-1"]);
    const laid = mockFetchFeed.mock.calls.length;

    // and both entries survive: each wall comes back as itself, from the cache
    feed.reset({ actor: "alice" });
    expect(mockFetchFeed).toHaveBeenCalledTimes(laid);
    expect(ids(feed)).toEqual(["alice-1"]);
    feed.reset({ feed: "alice" });
    expect(mockFetchFeed).toHaveBeenCalledTimes(laid);
    expect(ids(feed)).toEqual(["feed-alice-1"]);
  });
});
