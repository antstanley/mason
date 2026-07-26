// Every decision the brick reader makes: which clicks it takes, the freeze that
// has to land before the history push, where a step lands on a wall it locates
// by id, and which of the two ways it shuts. The dialog that renders all this is
// a `.svelte` file and no lane in this repo typechecks one, which is exactly why
// the decisions live in a rune module and are pinned here.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { pushState, replaceState } from "$app/navigation";
import { fetchFeed } from "$lib/api";
import { feed } from "./feed.svelte";
import { reader, ReaderState } from "./reader.svelte";
import type { Brick } from "$lib/types";

vi.mock("$app/navigation", () => ({ pushState: vi.fn(), replaceState: vi.fn() }));

// `page.state` is the reader's open/shut signal and the real one needs a live
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

// Not about this module, about its graph: the reader imports the `feed`
// singleton for `freeze()` and `items`, feed.svelte.ts imports $lib/api, and
// api.ts imports $app/environment, which has nothing to answer in vitest's node
// environment. Mocking it here also makes "the reader never fetches" assertable.
vi.mock("$lib/api", () => {
  class MockFeedError extends Error {}
  return { fetchFeed: vi.fn(), FeedError: MockFeedError };
});

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

/** The shape `activate` reads off a click, with `preventDefault` spied. */
interface TestClick {
  metaKey: boolean;
  ctrlKey: boolean;
  shiftKey: boolean;
  altKey: boolean;
  button: number;
  currentTarget: EventTarget | null;
  preventDefault: () => void;
}

/** A plain primary-button click, with whichever field a case wants flipped. */
function click(overrides: Partial<TestClick> = {}): TestClick {
  return {
    metaKey: false,
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    button: 0,
    currentTarget: null,
    preventDefault: vi.fn(),
    ...overrides,
  };
}

/** The laid wall every case steps along, named so a case can hold an end of it
 *  without indexing (`noUncheckedIndexedAccess` makes every index optional). */
const first = brick("a");
const middle = brick("b");
const last = brick("c");
const wall = [first, middle, last];

const back = vi.fn();

beforeEach(() => {
  vi.mocked(pushState).mockReset();
  vi.mocked(replaceState).mockReset();
  vi.mocked(fetchFeed).mockReset();
  back.mockReset();
  // `history` is a browser global with nothing behind it in node; close()'s
  // own-entry branch is the only thing that reaches for it
  vi.stubGlobal("history", { back });
  // every open freezes the wall; the shared singleton was never reset so the
  // real freeze would be a no-op anyway, and the spy keeps it that way whatever
  // an earlier case left behind. The ordering case swaps in its own recorder.
  vi.spyOn(feed, "freeze").mockImplementation(() => Promise.resolve());
  feed.items = wall.slice();
  delete pageState.brick;
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("activation", () => {
  it.each([
    ["cmd-click", { metaKey: true }],
    ["ctrl-click", { ctrlKey: true }],
    ["shift-click", { shiftKey: true }],
    ["alt-click", { altKey: true }],
    ["a middle click", { button: 1 }],
  ])("declines %s and leaves the browser to it", (_name, modifier) => {
    const opened = new ReaderState();
    const open = vi.spyOn(opened, "open");
    const event = click(modifier);

    expect(opened.activate(event, first)).toBe(false);
    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(open).not.toHaveBeenCalled();
    expect(pushState).not.toHaveBeenCalled();
    expect(opened.brick).toBeNull();
  });

  it("takes a plain left click, preventing the default exactly once", () => {
    const opened = new ReaderState();
    const target = brick("a");
    const event = click();

    expect(opened.activate(event, target)).toBe(true);
    expect(event.preventDefault).toHaveBeenCalledTimes(1);
    expect(pushState).toHaveBeenCalledExactlyOnceWith("", { brick: "a" });
    expect(opened.brick).toBe(target);
  });

  it("remembers the anchor the click came from and gives focus back to it", () => {
    const opened = new ReaderState();
    const focus = vi.fn();
    // the reader only ever calls focus() on the opener, so a bare object stands
    // in for the anchor; node has no HTMLElement to build a real one from
    const anchor = { focus } as unknown as EventTarget;

    opened.activate(click({ currentTarget: anchor }), brick("a"));
    opened.returnFocus();
    expect(focus).toHaveBeenCalledTimes(1);
  });

  it("survives a click with no focusable target behind it", () => {
    const opened = new ReaderState();
    opened.activate(click({ currentTarget: null }), brick("a"));
    expect(() => opened.returnFocus()).not.toThrow();
  });
});

describe("opening", () => {
  it("freezes the wall before it pushes the history entry", () => {
    const order: string[] = [];
    vi.spyOn(feed, "freeze").mockImplementation(() => {
      order.push("freeze");
      return Promise.resolve();
    });
    vi.mocked(pushState).mockImplementation(() => {
      order.push("push");
    });

    new ReaderState().open(brick("a"));
    // a warming wall reorders between preview polls, so the arrangement is
    // committed before the reader starts locating a brick inside it
    expect(order).toEqual(["freeze", "push"]);
  });

  it("is up only while page.state says so, and keeps its brick after that", () => {
    const opened = new ReaderState();
    expect(opened.isOpen).toBe(false);

    opened.open(brick("a"));
    pageState.brick = "a"; // the router's own update, once the push lands
    expect(opened.isOpen).toBe(true);

    delete pageState.brick; // the back gesture, which never calls close()
    expect(opened.isOpen).toBe(false);
    expect(opened.brick?.id).toBe("a"); // still held, so a closing dialog can render
  });
});

describe("stepping", () => {
  it("locates the open brick by id and steps to its neighbours", () => {
    const opened = new ReaderState();
    opened.open(middle);
    expect(opened.index).toBe(1);
    expect(opened.canPrev).toBe(true);
    expect(opened.canNext).toBe(true);

    opened.next();
    expect(opened.brick?.id).toBe("c");
    expect(opened.index).toBe(2);
    // a step replaces the entry rather than adding one, so a single back
    // gesture still closes the reader instead of walking it back brick by brick
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", { brick: "c" });
    expect(pushState).toHaveBeenCalledTimes(1); // the open, and nothing since

    opened.prev();
    expect(opened.brick?.id).toBe("b");
  });

  it("stops at the last laid brick rather than paginating", () => {
    const loadMore = vi.spyOn(feed, "loadMore");
    const opened = new ReaderState();
    opened.open(last);
    expect(opened.canNext).toBe(false);

    opened.next();
    expect(opened.brick?.id).toBe("c"); // unmoved
    expect(replaceState).not.toHaveBeenCalled();
    expect(loadMore).not.toHaveBeenCalled();
    expect(fetchFeed).not.toHaveBeenCalled();
  });

  it("stops at the first laid brick", () => {
    const loadMore = vi.spyOn(feed, "loadMore");
    const opened = new ReaderState();
    opened.open(first);
    expect(opened.canPrev).toBe(false);

    opened.prev();
    expect(opened.brick?.id).toBe("a"); // unmoved
    expect(replaceState).not.toHaveBeenCalled();
    expect(loadMore).not.toHaveBeenCalled();
    expect(fetchFeed).not.toHaveBeenCalled();
  });

  it("steps nowhere at all once its brick has left the wall", () => {
    const loadMore = vi.spyOn(feed, "loadMore");
    const opened = new ReaderState();
    opened.open(brick("gone")); // never laid, so findIndex misses it
    expect(opened.index).toBe(-1);
    expect(opened.canPrev).toBe(false);
    expect(opened.canNext).toBe(false);

    opened.next();
    opened.prev();
    expect(opened.brick?.id).toBe("gone"); // unmoved, and not silently reindexed
    expect(replaceState).not.toHaveBeenCalled();
    expect(loadMore).not.toHaveBeenCalled();
    expect(fetchFeed).not.toHaveBeenCalled();
  });
});

describe("closing", () => {
  it("pops the entry it pushed", () => {
    const opened = new ReaderState();
    opened.open(brick("a"));
    opened.close();
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).not.toHaveBeenCalled();
  });

  it("replaces the state when the entry is not its own", () => {
    // nothing was pushed, so going back would leave the wall rather than close
    // the reader; the brick comes off the current entry instead
    new ReaderState().close();
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
    expect(back).not.toHaveBeenCalled();
  });

  it("spends its entry once, whichever way it went", () => {
    const opened = new ReaderState();
    opened.open(brick("a"));
    opened.close();
    opened.close(); // a second escape, or a scrim click racing the back gesture
    expect(back).toHaveBeenCalledTimes(1);
    expect(replaceState).toHaveBeenCalledExactlyOnceWith("", {});
  });
});

describe("the singleton", () => {
  it("is one shut reader for the whole app", () => {
    expect(reader).toBeInstanceOf(ReaderState);
    expect(reader.brick).toBeNull();
    expect(reader.index).toBe(-1);
    expect(reader.isOpen).toBe(false);
  });
});
