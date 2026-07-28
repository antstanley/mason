// FeedState is the TypeScript half of the feed protocol: the warming poll
// loop, the freeze handshake, pagination dedupe and the per-session snapshot
// cache. These tests pin its transitions against a mocked fetchFeed; the wire
// format itself is pinned on the Rust side.
import { readFileSync } from "node:fs";
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
/** Every request that asked mortar to re-read the fast content caches. */
const flagged = () => mockFetchFeed.mock.calls.filter((c) => c[4] === true);

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
    // and none of them asks for a refresh: only the control does that
    expect(mockFetchFeed.mock.calls).toEqual([
      [{ actor: "alice" }, null, undefined, "preview", false],
      [{ actor: "alice" }, "c1", undefined, "preview", false],
      [{ actor: "alice" }, "c2", undefined, "freeze", false],
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
    expect(mockFetchFeed).toHaveBeenLastCalledWith(
      { actor: "alice" },
      "c1",
      undefined,
      "freeze",
      false,
    );

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

/** A settled alice wall: items ["alice-1"], cursor "alice-c". What a refresh
 *  has to replace, and what it must keep on screen while it does. */
async function laidWall() {
  const feed = new FeedState();
  settleImmediately();
  feed.reset({ actor: "alice" });
  await vi.advanceTimersByTimeAsync(0);
  return feed;
}

describe("refresh", () => {
  it("leaves the outgoing wall up, with no skeleton grid, while the new one warms", async () => {
    const feed = await laidWall();
    const preview = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed.mockImplementation(() => preview.promise);

    feed.refresh();
    expect(feed.warming).toBe(true);
    // the twelve-card grid is initialLoad only, and a refresh never sets it:
    // skeletons mid-session read as something breaking, not as a refresh
    expect(feed.initialLoad).toBe(false);
    expect(ids(feed)).toEqual(["alice-1"]); // the bricks stay on the wall
    expect(feed.cursor).toBeNull(); // a refresh is always a first page
    expect(feed.done).toBe(false);
    expect(mockFetchFeed.mock.calls).toEqual([
      [{ actor: "alice" }, null, undefined, "preview", true],
    ]);

    // and when the first arrangement lands, the old wall reflows into it
    preview.resolve(page(["fresh-1"], "fresh-p", false));
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["fresh-1"]);
    expect(feed.warming).toBe(false);
  });

  it("carries the flag on exactly one request across a preview and its freeze", async () => {
    const feed = await laidWall();
    mockFetchFeed.mockClear();

    feed.refresh();
    await vi.advanceTimersByTimeAsync(0); // the preview settles and #warm freezes
    expect(feed.warming).toBe(false);
    expect(flagged()).toHaveLength(1);
    expect(mockFetchFeed.mock.calls).toEqual([
      // the cursorless poll carries it
      [{ actor: "alice" }, null, undefined, "preview", true],
      // and the commit rides the preview's cursor, which carries the refreshing
      // snapshot's seed, so it lands on that snapshot without a flag of its own
      [{ actor: "alice" }, "alice-p", undefined, "freeze", false],
    ]);
  });

  it("still commits, and flags nothing twice, when the refresh preview fails", async () => {
    const feed = await laidWall();
    mockFetchFeed.mockReset();
    mockFetchFeed
      .mockRejectedValueOnce(new Error("preview blip"))
      .mockResolvedValueOnce(page(["fresh-1"], "fresh-c"));

    feed.refresh();
    await vi.advanceTimersByTimeAsync(0);
    // the marker is released when the flagged request settles, thrown or not,
    // or the wall would sit warming forever behind its own guard
    expect(feed.warming).toBe(false);
    expect(feed.error).toBeNull();
    expect(ids(feed)).toEqual(["fresh-1"]); // and the wall still commits

    // the flag is spent by that same settling, thrown or not. It is a fourth
    // disarm point beside the three the spec names (adopt, freeze's finally,
    // the next reset), and it is the one that closes the error path: the commit
    // below is cursorless, because refresh nulled the cursor, so an unspent
    // flag would ride it and one tap would have sent two flagged cursorless
    // requests, which is two seeds, two snapshots and two hundred-author
    // fan-outs. Asserted over the whole call list rather than by counting,
    // because WHICH request carries it is the half that matters.
    expect(mockFetchFeed.mock.calls).toEqual([
      [{ actor: "alice" }, null, undefined, "preview", true],
      [{ actor: "alice" }, null, undefined, "freeze", false],
    ]);
    expect(flagged()).toEqual([[{ actor: "alice" }, null, undefined, "preview", true]]);
  });

  it("sends one flagged request when the reader engages and the refresh preview then fails", async () => {
    const feed = await laidWall();
    const preview = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed
      .mockImplementationOnce(() => preview.promise)
      .mockResolvedValueOnce(page(["fresh-1"], "fresh-c"));

    feed.refresh();
    void feed.freeze(); // the reader engages while the flagged request is out
    expect(mockFetchFeed).toHaveBeenCalledTimes(1); // held by the marker

    preview.reject(new Error("preview blip")); // and then it never answers
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(false); // the wall still leaves the reflow
    expect(ids(feed)).toEqual(["fresh-1"]);

    // the commit that follows a thrown preview is cursorless, so this is the
    // shape where two flagged requests would go out under one tap, one after
    // the other. Over the whole call list, only the preview carries the flag.
    expect(flagged()).toEqual([[{ actor: "alice" }, null, undefined, "preview", true]]);
    expect(mockFetchFeed.mock.calls).toEqual([
      [{ actor: "alice" }, null, undefined, "preview", true],
      [{ actor: "alice" }, null, undefined, "freeze", false],
    ]);
  });

  it("holds a freeze that beats the refresh preview, and commits on the refreshed wall", async () => {
    const feed = await laidWall();
    const preview = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed
      .mockImplementationOnce(() => preview.promise)
      .mockResolvedValueOnce(page(["fresh-2", "fresh-1"], "fresh-c"));

    feed.refresh();
    // the reader engages while the flagged request is still in flight. Under
    // prefers-reduced-motion the grid does exactly this the instant warming
    // flips true, with no scroll event at all, so this is a default path.
    void feed.freeze();
    expect(mockFetchFeed).toHaveBeenCalledTimes(1); // held, not sent
    expect(feed.loading).toBe(false); // and held with no side effect at all
    expect(feed.warming).toBe(true);

    preview.resolve(page(["fresh-1"], "fresh-p", false));
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(false);
    expect(ids(feed)).toEqual(["fresh-2", "fresh-1"]);

    // one tap, at most one fan-out
    expect(flagged().length).toBeLessThanOrEqual(1);
    const committed = mockFetchFeed.mock.calls.filter((c) => c[3] === "freeze");
    expect(committed).toHaveLength(1);
    // and the half that matters: the request that COMMITS is on the refreshed
    // snapshot, by carrying the flag or by riding the flagged preview's cursor.
    // The count alone is green on the broken shape, where the freeze goes out
    // cursorless AND unflagged: it satisfies "exactly one flagged call", clears
    // its first-paint bar off the untouched author-feed cache while the
    // refreshing fill is still fanning out, and commits the pre-refresh wall.
    expect(committed[0]?.[4] === true || committed[0]?.[1] != null).toBe(true);
  });

  it("cannot leak the flag into the next wall", async () => {
    const feed = await laidWall();
    feed.refresh();
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(false);
    mockFetchFeed.mockClear();

    feed.reset({ actor: "bob" }); // a different wall, laid the ordinary way
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["bob-1"]);
    expect(flagged()).toHaveLength(0);
  });

  it("drops the session entry, so coming back finds the refreshed wall", async () => {
    const feed = await laidWall();
    mockFetchFeed.mockReset();
    mockFetchFeed
      .mockResolvedValueOnce(page(["fresh-1"], "fresh-p", false)) // the refresh preview
      .mockResolvedValueOnce(page(["fresh-2", "fresh-1"], "fresh-c")); // and its commit
    feed.refresh();
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["fresh-2", "fresh-1"]);

    settleImmediately();
    feed.reset({ actor: "bob" }); // away
    await vi.advanceTimersByTimeAsync(0);
    const calls = mockFetchFeed.mock.calls.length;

    feed.reset({ actor: "alice" }); // and back
    expect(mockFetchFeed).toHaveBeenCalledTimes(calls); // still a rehydration
    expect(ids(feed)).toEqual(["fresh-2", "fresh-1"]); // of the REFRESHED wall
    expect(feed.cursor).toBe("fresh-c");
  });

  it("does not resurrect the outgoing wall when a reset lands mid-refresh", async () => {
    const feed = await laidWall();
    const stale = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed
      .mockImplementationOnce(() => stale.promise) // the refresh, still in flight
      .mockResolvedValueOnce(page(["fresh-1"], "fresh-p", false)) // the reset's preview
      .mockResolvedValueOnce(page(["fresh-2"], "fresh-c")); // and its commit

    feed.refresh();
    // the entry went before the generation bump, so a back gesture landing
    // mid-refresh lays the wall again instead of handing back the arrangement
    // the reader just asked to replace
    feed.reset({ actor: "alice" });
    expect(feed.initialLoad).toBe(true); // laid afresh, not rehydrated
    expect(ids(feed)).toEqual([]);
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["fresh-2"]);

    // and the superseded refresh lands last, touching nothing
    stale.resolve(page(["stale-1"], "stale-p", true));
    await vi.advanceTimersByTimeAsync(5000);
    expect(ids(feed)).toEqual(["fresh-2"]);
  });

  it("refuses a second refresh while the first is still warming", async () => {
    const feed = await laidWall();
    const preview = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed.mockImplementation(() => preview.promise);

    feed.refresh();
    expect(mockFetchFeed).toHaveBeenCalledTimes(1);
    feed.refresh(); // the double tap
    feed.refresh();
    expect(mockFetchFeed).toHaveBeenCalledTimes(1); // one fan-out, not three
    expect(flagged()).toHaveLength(1);

    preview.resolve(page(["fresh-1"], "fresh-p", false));
    await vi.advanceTimersByTimeAsync(0);
    expect(feed.warming).toBe(false);
  });

  it("refuses with no wall at all, and while a page is in flight", async () => {
    const cold = new FeedState();
    cold.refresh(); // nothing has ever been reset on it
    expect(mockFetchFeed).not.toHaveBeenCalled();

    const feed = await laidWall();
    const next = deferred<FeedResponse>();
    mockFetchFeed.mockReset();
    mockFetchFeed.mockImplementation(() => next.promise);
    void feed.loadMore();
    expect(feed.loading).toBe(true);

    feed.refresh(); // pagination owns the wall until its page lands
    expect(mockFetchFeed).toHaveBeenCalledTimes(1);
    expect(feed.warming).toBe(false); // refused without a single side effect
    expect(feed.cursor).toBe("alice-c");

    next.resolve(page(["alice-2"], null));
    await vi.advanceTimersByTimeAsync(0);
    expect(ids(feed)).toEqual(["alice-1", "alice-2"]);
  });

  it("names no DOM global and imports no reader", () => {
    // Two greps, because one is not enough. Closing an open reader is right on
    // a refresh, but the call belongs at the trigger: reader.svelte.ts already
    // imports this module, so the reverse import is a cycle, it would drag
    // $app/navigation and $app/state into this file's graph (which is what the
    // api mock at the top exists to prevent), and reader.close() reaches
    // history.back() one module away, where no DOM grep over feed.svelte.ts
    // can see it.
    const source = readFileSync(new URL("./feed.svelte.ts", import.meta.url), "utf8");
    expect(source).not.toMatch(/from\s+["'][^"']*reader\.svelte/);
    expect(
      source.match(
        /\b(?:window|document|navigator|history|location|localStorage|sessionStorage|scrollTo|scrollBy|scrollIntoView|requestAnimationFrame|HTMLElement)\b/,
      ),
    ).toBeNull();
  });
});
