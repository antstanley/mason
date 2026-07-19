// fetchFeed's two edges: error classification off the wire (FE-3) and the
// service-worker control race that must never hang a page (FE-4).
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fetchFeed, FeedError } from "$lib/api";

// mutable so each test picks its side of the browser check; api.ts reads the
// live binding on every call
const env = { browser: false };
vi.mock("$app/environment", () => ({
  get browser() {
    return env.browser;
  },
}));

const okBody = JSON.stringify({ items: [], cursor: null });

beforeEach(() => {
  env.browser = false;
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe("error classification (FE-3)", () => {
  it("surfaces mortar's error envelope as a coded FeedError", async () => {
    vi.stubGlobal(
      "fetch",
      vi
        .fn()
        .mockResolvedValue(
          new Response(JSON.stringify({ error: "actor_not_found" }), { status: 404 }),
        ),
    );
    const failure = fetchFeed("nobody.test");
    await expect(failure).rejects.toBeInstanceOf(FeedError);
    await expect(failure).rejects.toMatchObject({ code: "actor_not_found", status: 404 });
  });

  it("codes a non-JSON error body as unknown, not as a missing handle", async () => {
    // local mode: a request that escapes the SW 404s on the static host with
    // an HTML error doc; that must never read as actor_not_found
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(new Response("<html>not found</html>", { status: 404 })),
    );
    await expect(fetchFeed("demo")).rejects.toMatchObject({ code: "unknown", status: 404 });
  });

  it("passes actor, cursor, mode and intent on the wire", async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(okBody, { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    await expect(fetchFeed("demo", "cur1", "glaze", "preview")).resolves.toEqual({
      items: [],
      cursor: null,
    });
    expect(fetchMock).toHaveBeenCalledWith(
      "/api/feed?actor=demo&cursor=cur1&mode=glaze&intent=preview",
    );
  });
});

describe("service-worker control race (FE-4)", () => {
  it("gives up waiting for control after the timeout instead of hanging", async () => {
    env.browser = true;
    vi.useFakeTimers();
    const fetchMock = vi.fn().mockResolvedValue(new Response(okBody, { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal("navigator", {
      serviceWorker: {
        controller: null,
        ready: new Promise(() => {}), // a rejected register leaves this pending forever
        addEventListener: () => {}, // controllerchange never fires
      },
    });

    const pending = fetchFeed("demo");
    await vi.advanceTimersByTimeAsync(1999);
    expect(fetchMock).not.toHaveBeenCalled(); // still hoping for control

    await vi.advanceTimersByTimeAsync(1); // the 2s ceiling hits
    await expect(pending).resolves.toEqual({ items: [], cursor: null });
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it("fetches immediately when the service worker already controls the page", async () => {
    env.browser = true;
    vi.useFakeTimers();
    const fetchMock = vi.fn().mockResolvedValue(new Response(okBody, { status: 200 }));
    vi.stubGlobal("fetch", fetchMock);
    vi.stubGlobal("navigator", {
      serviceWorker: {
        controller: {},
        ready: new Promise(() => {}),
        addEventListener: () => {},
      },
    });

    // no timer advance at all: the controlled page must not wait
    await expect(fetchFeed("demo")).resolves.toEqual({ items: [], cursor: null });
    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});
