import { expect, test, type Page } from "@playwright/test";

/** What one /api/feed round trip made from the page looks like. Narrowed, not
 *  asserted: this file IS typechecked, so a shape claimed here is a shape the
 *  compiler holds the test to. */
interface FeedAnswer {
	status: number;
	body: {
		items?: { kind?: string }[];
		cursor?: string | null;
		warming?: boolean;
		error?: string;
	};
}

/** Open the app and wait for the service worker to take control. Interception
 *  only applies once it does, and on a static host /api/feed answers at all
 *  only because it intercepted. */
async function underServiceWorker(page: Page): Promise<void> {
	await page.goto("/?actor=demo");
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});
}

/** One raw round trip through the worker, from inside the page. */
async function apiFeed(page: Page, url: string): Promise<FeedAnswer> {
	return await page.evaluate(async (target: string) => {
		const res = await fetch(target);
		return { status: res.status, body: (await res.json()) as FeedAnswer["body"] };
	}, url);
}

// End to end through the real engine: the page loads, the service worker takes
// control, /api/feed round-trips through the wasm mortar, and bricks render.
// Actor `demo` is the offline fixture wall compiled into the wasm, so this
// needs no network beyond the static site itself.
test("the demo wall round-trips /api/feed through the wasm service worker", async ({ page }) => {
	await underServiceWorker(page);

	// a raw round-trip: on a static host this path only answers if the service
	// worker intercepted it and the wasm engine laid a page
	const roundTrip = await apiFeed(page, "/api/feed?actor=demo");
	expect(roundTrip.status).toBe(200);
	// narrowed, not asserted: this file IS typechecked, so the `items!` this
	// replaced was a live counterexample to the rule the guidelines now state
	const { items } = roundTrip.body;
	expect(Array.isArray(items)).toBe(true);
	expect(items?.length ?? 0).toBeGreaterThan(0);

	// and the app itself renders those bricks on the wall (warm can take up to
	// the 8s ceiling before the first screen commits)
	await expect(page.locator("#wall article").first()).toBeVisible({ timeout: 30_000 });
});

// The two ways a request can name no wall at all. Both are 400 `bad_request`,
// and both are answered entirely offline: the first never reaches mortar (the
// worker answers its own guard), and the second is rejected by the feed
// reference parser before a socket is opened, which is the whole reason a
// malformed `?feed=` is a parse rather than a fallback.
test("a request naming no wall answers 400 without touching the network", async ({ page }) => {
	await underServiceWorker(page);

	const noSource = await apiFeed(page, "/api/feed");
	expect(noSource.status).toBe(400);
	expect(noSource.body.error).toBe("bad_request");

	const notAFeed = await apiFeed(page, "/api/feed?feed=nonsense");
	expect(notAFeed.status).toBe(400);
	expect(notAFeed.body.error).toBe("bad_request");
});

// `feed_page` is five optional strings in a row, so a transposed pair
// typechecks on both sides and still lays the wrong wall. The tsc project over
// the worker counts the arguments; it cannot read their order, and this is the
// only lane that can.
//
// Entirely offline on the demo wall, and each of the three assertions below
// fails on a different adjacent transposition:
//   - every brick is a post          the `mode` slot (glaze filters the fixture
//                                    pool, which is 70/15/15 post/blog/video)
//   - warming is false               the `intent` slot (only a demo preview
//                                    sets it)
//   - the cursor comes back verbatim the `cursor` slot (the demo preview
//                                    re-encodes the INCOMING offset, so a
//                                    cursor read from the wrong slot decodes to
//                                    nothing and returns as offset 0)
// The actor/feed pair needs no assertion of its own: transposed, `feed` would
// be "demo", which parses as no feed at all, and the first fetch would be a 400.
test("the service worker binds every positional slot", async ({ page }) => {
	await underServiceWorker(page);

	const wall = await apiFeed(page, "/api/feed?actor=demo&mode=glaze");
	expect(wall.status).toBe(200);
	const cursor = wall.body.cursor;
	if (typeof cursor !== "string") {
		throw new Error("the glaze demo wall carries more than one page, so page one has a cursor");
	}

	const preview = await apiFeed(
		page,
		`/api/feed?actor=demo&mode=glaze&intent=preview&cursor=${encodeURIComponent(cursor)}`,
	);
	expect(preview.status).toBe(200);

	const kinds = (preview.body.items ?? []).map((item) => item.kind);
	expect(kinds.length).toBeGreaterThan(0);
	// filtered rather than `.every`, so a failure names what came back
	expect(kinds.filter((kind) => kind !== "post")).toEqual([]);
	expect(preview.body.warming).toBe(false);
	expect(preview.body.cursor).toBe(cursor);
});
