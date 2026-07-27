// Everything the feed picker decides: the history entry that holds it open (and
// closes the reader by doing it), the recents list it keeps in `mason:feeds`,
// the three questions it asks the public AppView, the hidden tier it must not
// list, and what it does when the AppView will not answer at all.
//
// `FeedPicker.svelte` renders all of this and no lane in this repo typechecks or
// runs a component body, which is exactly why none of these decisions live in
// it. Every state in the picker's states table is reachable from here.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pushState, replaceState } from "$app/navigation";
import { APPVIEW } from "$lib/appview";
import {
  feeds,
  FeedsState,
  HIDDEN_LABELS,
  MAX_RECENT_FEEDS,
  type FeedListing,
} from "./feeds.svelte";

// Two things have to exist before the imports above are evaluated, so both are
// built in hoisted blocks rather than in beforeEach: vitest hoists imports above
// the file body, and two modules in this graph read a browser global the moment
// they are imported (this one builds its recents list from localStorage, and
// handle.svelte.ts reads `mason:handle` the same way).
//
// `browser` is the picker's own guard on every fetch, so it is mutable here: the
// real one answers false in node, which would make every case below assert the
// pre-network state.
const env = vi.hoisted(() => ({ browser: true }));
vi.mock("$app/environment", () => env);

/** The fake `mason:feeds` store, as a plain Map so a case can plant a value or
 *  read back what was written without going through the module. */
const stored = vi.hoisted(() => {
  const entries = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: (key: string) => entries.get(key) ?? null,
      setItem: (key: string, value: string) => void entries.set(key, value),
      removeItem: (key: string) => void entries.delete(key),
    },
  });
  return entries;
});

vi.mock("$app/navigation", () => ({ pushState: vi.fn(), replaceState: vi.fn() }));

// `page.state` is the picker's open/shut signal and the real one needs a live
// router. The getter defers the reference to call time, because a vi.mock
// factory is hoisted above this module's own initialization.
const pageState: App.PageState = {};
vi.mock("$app/state", () => ({
  page: {
    get state() {
      return pageState;
    },
  },
}));

const STORAGE_KEY = "mason:feeds";
const POPULAR = `${APPVIEW}/xrpc/app.bsky.unspecced.getPopularFeedGenerators`;
const ACTOR_FEEDS = `${APPVIEW}/xrpc/app.bsky.feed.getActorFeeds`;

const fetchMock = vi.fn<(url: string) => Promise<unknown>>();
const back = vi.fn();

/** One `generatorView` as the AppView reports it, with whichever field a case
 *  wants changed. Deliberately untyped: it stands in for somebody else's JSON,
 *  and half these cases are about a field being missing or wrong. */
function view(overrides: Record<string, unknown> = {}) {
  return {
    uri: "at://did:plc:one/app.bsky.feed.generator/one",
    displayName: "One",
    description: "the first feed",
    avatar: "https://cdn.test/one.jpg",
    likeCount: 7,
    creator: { handle: "alice.test" },
    ...overrides,
  };
}

/** One page of results, with an optional cursor for the next. */
function answer(views: unknown[], cursor?: string) {
  const body = cursor === undefined ? { feeds: views } : { feeds: views, cursor };
  return { ok: true, json: () => Promise.resolve(body) };
}

/** A stored recent, or a result the picker would hand back. */
function feedListing(uri: string, name = uri): FeedListing {
  return { uri, name, avatar: null, creator: "alice.test", description: "", likeCount: 0 };
}

beforeEach(() => {
  env.browser = true;
  stored.clear();
  fetchMock.mockReset();
  back.mockReset();
  vi.mocked(pushState).mockReset();
  vi.mocked(replaceState).mockReset();
  // `history` is a browser global with nothing behind it in node; closePicker's
  // own-entry branch is the only thing that reaches for it
  vi.stubGlobal("history", { back });
  vi.stubGlobal("fetch", fetchMock);
  delete pageState.picker;
  delete pageState.brick;
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("the picker is history, not a URL", () => {
  it("opens on its own key alone, which is what shuts the reader", () => {
    pageState.brick = "a"; // the reader is up over a wall
    const picker = new FeedsState();

    picker.openPicker();

    const pushed = vi.mocked(pushState).mock.calls[0]?.[1];
    expect(pushed).toEqual({ picker: "feeds" });
    // the load-bearing half: a push REPLACES page state rather than merging
    // into it, so carrying only the picker's key is what closes the reader. The
    // reader's half is the same shape and reader.test.ts pins it there.
    expect(pushed).not.toHaveProperty("brick");
    expect(pushState).toHaveBeenCalledExactlyOnceWith("", { picker: "feeds" });
  });

  it("is up only while page.state says so", () => {
    const picker = new FeedsState();
    expect(picker.isOpen).toBe(false);

    picker.openPicker();
    pageState.picker = "feeds"; // the router's own update, once the push lands
    expect(picker.isOpen).toBe(true);

    delete pageState.picker; // the back gesture, which never calls closePicker
    expect(picker.isOpen).toBe(false);
  });

  it("does not stack a second entry when it is already open", () => {
    const picker = new FeedsState();
    picker.openPicker();
    pageState.picker = "feeds";
    // the landing page's button and the wall's switch affordance both open the
    // one picker; two entries would need two back gestures to leave
    picker.openPicker();
    expect(pushState).toHaveBeenCalledTimes(1);
  });

  it("pops the entry it pushed", () => {
    const picker = new FeedsState();
    picker.openPicker();
    picker.closePicker();
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).not.toHaveBeenCalled();
  });

  it("replaces the state when the entry is not its own", () => {
    // nothing was pushed (a reload landed on the entry, say), so going back
    // would leave mason rather than close the picker
    new FeedsState().closePicker();
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
    expect(back).not.toHaveBeenCalled();
  });

  it("spends its entry once, whichever way it went", () => {
    const picker = new FeedsState();
    picker.openPicker();
    picker.closePicker();
    picker.closePicker(); // a second escape, or a scrim click racing the back
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
  });
});

describe("recents", () => {
  it("reads mason:feeds most recent first", () => {
    stored.set(STORAGE_KEY, JSON.stringify([feedListing("at://a"), feedListing("at://b")]));
    expect(new FeedsState().recent.map((f) => f.uri)).toEqual(["at://a", "at://b"]);
  });

  it("caps the list at MAX_RECENT_FEEDS", () => {
    const picker = new FeedsState();
    for (let i = 0; i < MAX_RECENT_FEEDS + 3; i++) picker.remember(feedListing(`at://${i}`));

    expect(picker.recent).toHaveLength(MAX_RECENT_FEEDS);
    // the newest is at the head and the oldest three fell off the end
    expect(picker.recent[0]?.uri).toBe(`at://${MAX_RECENT_FEEDS + 2}`);
    expect(picker.recent.map((f) => f.uri)).not.toContain("at://0");
  });

  it("moves a feed opened again to the front rather than listing it twice", () => {
    const picker = new FeedsState();
    picker.remember(feedListing("at://a"));
    picker.remember(feedListing("at://b"));
    picker.remember(feedListing("at://a", "still one feed"));

    expect(picker.recent.map((f) => f.uri)).toEqual(["at://a", "at://b"]);
    expect(picker.recent[0]?.name).toBe("still one feed"); // the newer record wins
  });

  it("writes the list back, so the next visit starts where this one left off", () => {
    const picker = new FeedsState();
    picker.remember(feedListing("at://a"));
    expect(stored.get(STORAGE_KEY)).toBe(JSON.stringify(picker.recent));
    expect(new FeedsState().recent.map((f) => f.uri)).toEqual(["at://a"]);
  });

  it("caps and dedupes what it reads, not just what it writes", () => {
    // a hand-edited list, or one written by a version that capped somewhere
    // else; the rule is the picker's, not the string's
    const many = Array.from({ length: MAX_RECENT_FEEDS + 5 }, (_, i) => feedListing(`at://${i}`));
    stored.set(STORAGE_KEY, JSON.stringify([...many, feedListing("at://0")]));

    const recent = new FeedsState().recent;
    expect(recent).toHaveLength(MAX_RECENT_FEEDS);
    expect(new Set(recent.map((f) => f.uri)).size).toBe(MAX_RECENT_FEEDS);
  });

  it("reads an unparseable mason:feeds as no recents at all", () => {
    stored.set(STORAGE_KEY, "{not json");
    expect(new FeedsState().recent).toEqual([]);
  });

  it("reads a mason:feeds that is not a list as no recents at all", () => {
    stored.set(STORAGE_KEY, JSON.stringify({ uri: "at://a" }));
    expect(new FeedsState().recent).toEqual([]);
  });

  it("drops a stored entry with nothing to open, and keeps the rest", () => {
    stored.set(
      STORAGE_KEY,
      JSON.stringify([{ name: "no uri" }, null, "a string", feedListing("at://a")]),
    );
    expect(new FeedsState().recent.map((f) => f.uri)).toEqual(["at://a"]);
  });

  it("fills in a stored entry whose fields are missing or the wrong type", () => {
    stored.set(
      STORAGE_KEY,
      JSON.stringify([{ uri: "at://did:plc:one/app.bsky.feed.generator/one", likeCount: "many" }]),
    );
    const recent = new FeedsState().recent[0];
    expect(recent?.name).toBe("one"); // the rkey, never an empty card
    expect(recent?.likeCount).toBe(0);
    expect(recent?.avatar).toBeNull();
    expect(recent?.creator).toBeNull();
    expect(recent?.description).toBe("");
  });
});

describe("the three questions", () => {
  it("asks the popular endpoint for the resting state", async () => {
    fetchMock.mockResolvedValue(answer([view()]));
    const picker = new FeedsState();

    await picker.browse();

    expect(fetchMock).toHaveBeenCalledExactlyOnceWith(`${POPULAR}?limit=30`);
    expect(picker.question).toBe("popular");
    expect(picker.results).toEqual([
      {
        uri: "at://did:plc:one/app.bsky.feed.generator/one",
        name: "One",
        avatar: "https://cdn.test/one.jpg",
        creator: "alice.test",
        description: "the first feed",
        likeCount: 7,
      },
    ]);
  });

  it("does not re-ask for the list it is already showing", async () => {
    fetchMock.mockResolvedValue(answer([view()]));
    const picker = new FeedsState();
    await picker.browse();
    await picker.browse(); // the picker reopened, or the screen re-rendered
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("pages the popular list by its own cursor", async () => {
    fetchMock
      .mockResolvedValueOnce(answer([view()], "page-2"))
      .mockResolvedValueOnce(answer([view({ uri: "at://two", displayName: "Two" })]));
    const picker = new FeedsState();

    await picker.browse();
    await picker.more();

    expect(fetchMock).toHaveBeenLastCalledWith(`${POPULAR}?limit=30&cursor=page-2`);
    // the second page joins the first rather than replacing it
    expect(picker.results.map((f) => f.name)).toEqual(["One", "Two"]);
    // no cursor came back with it, so that is the end of the list
    await picker.more();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("searches the same endpoint with the term, escaped", async () => {
    fetchMock.mockResolvedValue(answer([]));
    const picker = new FeedsState();

    await picker.search("  cats & dogs  ");

    // trimmed, and every value goes through URLSearchParams: a term carrying an
    // `&` must not be able to add a parameter of its own
    expect(fetchMock).toHaveBeenCalledExactlyOnceWith(`${POPULAR}?query=cats+%26+dogs&limit=30`);
    expect(picker.question).toBe("search");
    expect(picker.term).toBe("cats & dogs");
    // the empty state the picker shows for this is "no feeds by that name"
    expect(picker.results).toEqual([]);
    expect(picker.browseUnavailable).toBe(false);
  });

  it("goes back to the resting list when the box is emptied", async () => {
    fetchMock.mockResolvedValue(answer([view()]));
    const picker = new FeedsState();

    await picker.search("   ");

    // not a search that found nothing: nobody asked a question
    expect(picker.question).toBe("popular");
    expect(fetchMock).toHaveBeenCalledExactlyOnceWith(`${POPULAR}?limit=30`);
  });

  it("lists one person's feeds from a bare handle, with no resolution hop", async () => {
    fetchMock.mockResolvedValue(answer([]));
    const picker = new FeedsState();

    await picker.byCreator(" @Alice.Test ");

    // one request, and it is the listing itself: getActorFeeds takes a handle,
    // so the bridge between the two front doors costs no getProfile first
    expect(fetchMock).toHaveBeenCalledExactlyOnceWith(`${ACTOR_FEEDS}?actor=alice.test&limit=30`);
    expect(picker.question).toBe("creator");
    // the empty state here reads "that person has not made any feeds", which is
    // a different sentence to the search's, which is why `question` is kept
    expect(picker.results).toEqual([]);
  });

  it("goes back to the resting list when the handle is only an @", async () => {
    fetchMock.mockResolvedValue(answer([view()]));
    const picker = new FeedsState();
    await picker.byCreator("@");
    expect(picker.question).toBe("popular");
    expect(fetchMock).toHaveBeenCalledExactlyOnceWith(`${POPULAR}?limit=30`);
  });

  it("is loading while an answer is in flight, and not after it", async () => {
    // a promise this case lands by hand, so the assertion below can run while
    // the question is still in flight
    const inFlight: { land?: (page: unknown) => void } = {};
    fetchMock.mockReturnValue(
      new Promise((resolve) => {
        inFlight.land = resolve;
      }),
    );
    const picker = new FeedsState();

    const asked = picker.search("cats");
    // the picker shows skeletons here; "nothing found" and "nothing yet" read
    // identically and mean opposite things
    expect(picker.loading).toBe(true);
    inFlight.land?.(answer([view()]));
    await asked;
    expect(picker.loading).toBe(false);
  });

  it("drops an answer to a question the reader has moved on from", async () => {
    // the first question is held open on purpose, so its answer lands strictly
    // after the second's rather than in whatever order the mocks happen to
    // settle in; a race this test cannot control is a race it cannot pin
    const slow: { land?: (page: unknown) => void } = {};
    fetchMock
      .mockReturnValueOnce(
        new Promise((resolve) => {
          slow.land = resolve;
        }),
      )
      .mockResolvedValueOnce(answer([view({ uri: "at://fast", displayName: "Fast" })]));
    const picker = new FeedsState();

    const first = picker.search("slow");
    const second = picker.search("fast");
    await second;
    slow.land?.(answer([view({ uri: "at://slow", displayName: "Slow" })], "more"));
    await first;

    // the older answer must not be shown under the newer question's heading,
    // nor leave its cursor behind for `more()` to page a list nobody asked for
    expect(picker.results.map((f) => f.name)).toEqual(["Fast"]);
    expect(picker.term).toBe("fast");
    await picker.more();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it("never asks the AppView from a build with no browser", async () => {
    env.browser = false;
    const picker = new FeedsState();

    await picker.browse();
    await picker.search("cats");
    await picker.byCreator("alice.test");

    expect(fetchMock).not.toHaveBeenCalled();
  });
});

describe("the hidden tier", () => {
  // one case per label, driven off the module's own runtime list rather than a
  // retyped copy: `HiddenLabel` is erased and can generate nothing, and the
  // Record HIDDEN_LABELS is built from cannot be short a member without failing
  // to typecheck. A label added to mortar's HIDDEN_LABELS therefore fails here
  // before it can become a listing bug.
  const labels = Object.keys(HIDDEN_LABELS);

  it("has one case per label in the hidden tier", () => {
    expect(labels).toHaveLength(5);
  });

  it.each(labels)("does not list a feed labelled %s", async (label) => {
    fetchMock.mockResolvedValue(
      answer([view({ labels: [{ val: label }] }), view({ uri: "at://ok" })]),
    );
    const picker = new FeedsState();

    await picker.browse();

    // mason must not advertise a feed it would then refuse to lay
    expect(picker.results.map((f) => f.uri)).toEqual(["at://ok"]);
  });

  it.each(labels)("does not list a feed whose creator is labelled %s", async (label) => {
    fetchMock.mockResolvedValue(
      answer([
        view({ creator: { handle: "alice.test", labels: [{ val: label }] } }),
        view({ uri: "at://ok" }),
      ]),
    );
    const picker = new FeedsState();

    await picker.browse();

    expect(picker.results.map((f) => f.uri)).toEqual(["at://ok"]);
  });

  it("lists a feed carrying the warn tier, which mortar blurs rather than hides", async () => {
    fetchMock.mockResolvedValue(answer([view({ labels: [{ val: "!warn" }] })]));
    const picker = new FeedsState();
    await picker.browse();
    expect(picker.results).toHaveLength(1);
  });

  it("lists a feed whose labels are absent, empty or malformed", async () => {
    fetchMock.mockResolvedValue(
      answer([
        view({ labels: undefined, uri: "at://none" }),
        view({ labels: [], uri: "at://empty" }),
        view({ labels: [{}], uri: "at://malformed" }),
      ]),
    );
    const picker = new FeedsState();
    await picker.browse();
    expect(picker.results.map((f) => f.uri)).toEqual(["at://none", "at://empty", "at://malformed"]);
  });

  it("does not list a view with no uri, which names no feed to open", async () => {
    fetchMock.mockResolvedValue(answer([{ displayName: "nowhere" }, view()]));
    const picker = new FeedsState();
    await picker.browse();
    expect(picker.results.map((f) => f.name)).toEqual(["One"]);
  });

  it("names a feed published without a display name by its rkey", async () => {
    fetchMock.mockResolvedValue(answer([view({ displayName: undefined })]));
    const picker = new FeedsState();
    await picker.browse();
    expect(picker.results[0]?.name).toBe("one");
  });
});

describe("when the AppView will not answer", () => {
  it.each([
    ["is unreachable", () => fetchMock.mockRejectedValue(new TypeError("network down"))],
    [
      "answers 500",
      () => fetchMock.mockResolvedValue({ ok: false, json: () => Promise.resolve({}) }),
    ],
    [
      "answers something that is not JSON",
      () =>
        fetchMock.mockResolvedValue({ ok: true, json: () => Promise.reject(new Error("html")) }),
    ],
  ])("says browsing is unavailable when it %s", async (_name, arrange) => {
    stored.set(STORAGE_KEY, JSON.stringify([feedListing("at://a")]));
    arrange();
    const picker = new FeedsState();

    await expect(picker.browse()).resolves.toBeUndefined(); // never throws at the screen

    expect(picker.browseUnavailable).toBe(true);
    expect(picker.loading).toBe(false);
    expect(picker.results).toEqual([]);
    // the two load-bearing paths: recents still render and the paste box, which
    // never comes through here at all, still works
    expect(picker.recent.map((f) => f.uri)).toEqual(["at://a"]);
    picker.remember(feedListing("at://b"));
    expect(picker.recent.map((f) => f.uri)).toEqual(["at://b", "at://a"]);
  });

  it("does not page on from a failure", async () => {
    fetchMock
      .mockResolvedValueOnce(answer([view()], "page-2"))
      .mockRejectedValueOnce(new TypeError("network down"));
    const picker = new FeedsState();

    await picker.browse();
    await picker.more();
    expect(picker.browseUnavailable).toBe(true);

    await picker.more();
    expect(fetchMock).toHaveBeenCalledTimes(2); // the cursor went with the failure
  });

  it("clears the flag as soon as browsing works again", async () => {
    fetchMock.mockRejectedValueOnce(new TypeError("network down"));
    const picker = new FeedsState();
    await picker.browse();
    expect(picker.browseUnavailable).toBe(true);

    fetchMock.mockResolvedValue(answer([view()]));
    await picker.search("cats");
    expect(picker.browseUnavailable).toBe(false);
    expect(picker.results).toHaveLength(1);
  });
});

describe("the singleton", () => {
  it("is one shut picker for the whole app", () => {
    expect(feeds).toBeInstanceOf(FeedsState);
    expect(feeds.isOpen).toBe(false);
    expect(feeds.results).toEqual([]);
    expect(feeds.question).toBe("popular");
    expect(feeds.loading).toBe(false);
    expect(feeds.browseUnavailable).toBe(false);
  });
});
