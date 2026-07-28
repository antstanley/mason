// The header's read of a feed generator's own identity: what it shows before
// the AppView answers, what it shows when the AppView never answers, and what
// it does with an answer that arrives for a feed the reader has already left.
// `SwitchWall` renders all of this and no lane in this repo renders a
// component, which is exactly why the decisions live here rather than in it.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { APPVIEW } from "$lib/appview";
import { FeedInfoState } from "./feedinfo.svelte";

// the module reads `browser` to decide whether to fetch at all, and vitest's
// node environment answers false to the real one, which would make every case
// below assert the pre-network state
vi.mock("$app/environment", () => ({ browser: true }));

const WHATS_HOT = "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot";

/** One `app.bsky.feed.getFeedGenerator` answer, shaped as the module reads it. */
function generator(view: Record<string, unknown>) {
  return { ok: true, json: () => Promise.resolve({ view }) };
}

/** An answer whose landing this file decides rather than the microtask queue.
 *  `fetch` is handed a promise that stays pending until `land()` is called,
 *  which is the only way to make a SUPERSEDED request answer after the request
 *  that superseded it: two already-resolved mocks have identical `.then` chains
 *  and drain in issue order, so the stale one always lands first. */
function heldAnswer(view: Record<string, unknown>) {
  let land!: () => void;
  const answer = new Promise<unknown>((resolve) => {
    land = () => resolve(generator(view));
  });
  return { answer, land };
}

/** An unknown or withdrawn generator: the AppView 400s or 404s, and `res.ok`
 *  is the whole of what the module inspects. */
const miss = { ok: false, json: () => Promise.resolve({ error: "UnknownFeed" }) };

const fetchMock = vi.fn<(url: string) => Promise<unknown>>();

/** Drain the fetch promise chain. A macrotask, because the module hangs three
 *  `.then`s off the request and a fixed number of microtask awaits here would
 *  pin the count of a private implementation detail. */
const settle = () => new Promise((resume) => setTimeout(resume, 0));

beforeEach(() => {
  fetchMock.mockReset();
  vi.stubGlobal("fetch", fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("naming a feed", () => {
  it("shows the reference's rkey before the AppView has answered", () => {
    const info = new FeedInfoState();
    fetchMock.mockReturnValue(new Promise(() => {})); // in flight, forever
    info.load(WHATS_HOT);
    expect(info.name).toBe("whats-hot");
    expect(info.avatar).toBeNull();
    expect(info.creator).toBeNull();
  });

  it("takes the generator's display name, face and creator once it answers", async () => {
    const info = new FeedInfoState();
    fetchMock.mockResolvedValue(
      generator({
        displayName: "What's Hot",
        avatar: "https://cdn.test/whats-hot.jpg",
        creator: { handle: "bsky.app" },
      }),
    );
    info.load(WHATS_HOT);
    await settle();
    expect(info.name).toBe("What's Hot");
    expect(info.avatar).toBe("https://cdn.test/whats-hot.jpg");
    expect(info.creator).toBe("bsky.app");
  });

  it("asks the AppView with the reference percent-encoded", async () => {
    const info = new FeedInfoState();
    fetchMock.mockResolvedValue(generator({ displayName: "What's Hot" }));
    info.load(WHATS_HOT);
    await settle();
    // the base comes from the one module that owns it rather than being spelled
    // again here; what this pins is the endpoint and the encoding, and a
    // reference full of colons and slashes going into a query unescaped is the
    // failure it exists for
    expect(fetchMock).toHaveBeenCalledWith(
      `${APPVIEW}/xrpc/app.bsky.feed.getFeedGenerator?feed=at%3A%2F%2Fdid%3Aplc%3Az72i7hdynmk6r22z27h6tvur%2Fapp.bsky.feed.generator%2Fwhats-hot`,
    );
  });

  it("names a bsky.app feed link by its rkey too", () => {
    const info = new FeedInfoState();
    fetchMock.mockReturnValue(new Promise(() => {}));
    info.load("https://bsky.app/profile/bsky.app/feed/whats-hot");
    expect(info.name).toBe("whats-hot");
  });

  it("keeps the rkey for a generator published without a display name", async () => {
    const info = new FeedInfoState();
    fetchMock.mockResolvedValue(generator({ avatar: "https://cdn.test/f.jpg" }));
    info.load(WHATS_HOT);
    await settle();
    expect(info.name).toBe("whats-hot"); // never the empty string
    expect(info.avatar).toBe("https://cdn.test/f.jpg");
  });

  it("asks once per reference, however often the header re-renders", () => {
    const info = new FeedInfoState();
    fetchMock.mockResolvedValue(generator({ displayName: "What's Hot" }));
    info.load(WHATS_HOT);
    info.load(WHATS_HOT);
    info.load(WHATS_HOT);
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});

describe("a miss never blocks the wall", () => {
  it("leaves the rkey and no face when the feed is unknown to the AppView", async () => {
    const info = new FeedInfoState();
    fetchMock.mockResolvedValue(miss);
    info.load(WHATS_HOT);
    await settle();
    // the wall itself is mortar's answer, not this one: a header that could not
    // name the feed still names it something, and nothing here throws
    expect(info.name).toBe("whats-hot");
    expect(info.avatar).toBeNull();
    expect(info.creator).toBeNull();
  });

  it("leaves the rkey when the AppView is unreachable", async () => {
    const info = new FeedInfoState();
    fetchMock.mockRejectedValue(new TypeError("network down"));
    info.load(WHATS_HOT);
    await settle();
    expect(info.name).toBe("whats-hot");
    expect(info.avatar).toBeNull();
  });

  it("drops an answer that lands after the reader moved to another feed", async () => {
    const info = new FeedInfoState();
    // The first feed's answer is HELD, so it lands after the second's. That is
    // the ordering the case is named after and the only one in which the guard
    // does any work: with both answers resolved up front they drain in issue
    // order, the stale one lands first, and the fresh one overwrites it whether
    // or not the guard is there.
    const held = heldAnswer({
      displayName: "What's Hot",
      avatar: "https://cdn.test/hot.jpg",
      creator: { handle: "bsky.app" },
    });
    // and the second feed brings a face and a creator of its own, so a stale
    // write cannot hide behind the fresh handler's unconditional assignment: a
    // fixture with no avatar clears that field either way, which is how this
    // case used to pass by accident.
    fetchMock.mockReturnValueOnce(held.answer).mockResolvedValueOnce(
      generator({
        displayName: "Quiet Posters",
        avatar: "https://cdn.test/quiet.jpg",
        creator: { handle: "quiet.test" },
      }),
    );

    info.load(WHATS_HOT);
    info.load("at://did:plc:other/app.bsky.feed.generator/quiet"); // switched mid-flight
    await settle();
    expect(info.name).toBe("Quiet Posters"); // the feed the reader is on now

    held.land();
    await settle();

    // all three fields, because the header wears all three: the button's name,
    // its face, and the creator its aria-label names the feed by
    expect(info.name).toBe("Quiet Posters");
    expect(info.avatar).toBe("https://cdn.test/quiet.jpg");
    expect(info.creator).toBe("quiet.test");
  });
});
