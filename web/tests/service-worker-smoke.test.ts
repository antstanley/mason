import { expect, test, type Page } from "@playwright/test";
import type { FeedResponse } from "$lib/types";

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

/** The header's layout picker, which is the whole chrome's canary: it lives
 *  inside the one `{#if}` in +layout.svelte that gates the skip link, both
 *  pickers, the wall switcher and the mobile bottom padding together. */
function layoutPicker(page: Page) {
	return page.getByRole("group", { name: "Wall layout" });
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

// A wall opened as /?feed= is a wall, so it gets the chrome a graph wall gets.
// `nonsense` is a feed reference that cannot parse, which is what keeps this
// case entirely offline: mortar answers 400 before a socket is opened, and the
// wall behind the chrome is the error panel. The chrome is the point. The
// header, both pickers, the skip link and the mobile bottom padding sit behind
// ONE `{#if}` in +layout.svelte, no lane typechecks a .svelte file, and a route
// widened without widening that condition renders a feed wall with no controls
// at all and no way back.
test("a feed wall renders the chrome a graph wall gets", async ({ page }) => {
	await underServiceWorker(page);

	await page.goto("/?feed=nonsense");
	await expect(layoutPicker(page)).toBeVisible();
	await expect(page.locator("#wall")).toBeVisible();
	await expect(page.getByRole("heading", { name: /wouldn't load/ })).toBeVisible();
	// the failure is the page's single h1 here, and the sr-only wall title steps
	// aside for it, so what must never appear is the handle-shaped one
	await expect(page.locator("#wall h1")).toHaveCount(1);
	await expect(page.locator("#wall h1")).not.toContainText("wall on mason");

	// with both parameters in the URL the feed is the wall laid, which is
	// mortar's own precedence: read the other way round, the demo actor would
	// have laid bricks here.
	await page.goto("/?actor=demo&feed=nonsense");
	await expect(layoutPicker(page)).toBeVisible();
	await expect(page.getByRole("heading", { name: /wouldn't load/ })).toBeVisible();
	await expect(page.locator("#wall article")).toHaveCount(0);
});

// The one case chromium cannot reach through the real engine: a feed wall that
// LAYS. mortar answers a feed target from the AppView and from nowhere else, so
// there is no offline fixture behind `?feed=` the way `demo` is one behind
// `?actor=`. This block therefore blocks the worker and answers /api/feed from
// the test itself, which is the only offline way to put a laid feed wall in
// front of a browser.
//
// It is worth the exception because of what only a laid wall renders: the
// page's single sr-only <h1>, which steps aside whenever the wall failed (the
// error panel raises its own). Widening the route without widening that heading
// leaves a screen reader hearing "@'s wall on mason" on the one landmark
// heading the page has, and no other lane in this repo can see it: tsc does not
// parse .svelte at all.
test.describe("a laid feed wall", () => {
	test.use({ serviceWorkers: "block" });

	const FEED_URI = "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot";

	/** One page of a feed generator, as mortar would answer it. Typed against
	 *  the wire mirror, so a brick shape that drifted fails the typecheck rather
	 *  than rendering nothing and failing this test obscurely. */
	const onePage: FeedResponse = {
		items: [
			{
				kind: "post",
				id: "at://did:plc:brick/app.bsky.feed.post/one",
				url: "https://bsky.app/profile/brick.test/post/one",
				author: {
					did: "did:plc:brick",
					handle: "brick.test",
					displayName: "a brick",
					avatar: null,
				},
				text: "one brick, laid from a feed",
				createdAt: "2026-07-26T00:00:00Z",
				likeCount: 0,
				repostCount: 0,
				images: [],
				external: null,
			},
		],
		cursor: null,
		warming: false,
	};

	test("lays bricks under a heading of its own", async ({ page }) => {
		// with no worker to intercept it, /api/feed is an ordinary network
		// request and the test answers it. Both the preview and the freeze land
		// here, and each is preceded by api.ts's 2 second wait for a controller
		// that is never coming, which is why this case is slower than the rest.
		await page.route("**/api/feed*", (route) => route.fulfill({ json: onePage }));

		await page.goto(`/?feed=${encodeURIComponent(FEED_URI)}`);

		await expect(page.locator("#wall article").first()).toBeVisible({ timeout: 30_000 });
		await expect(layoutPicker(page)).toBeVisible();

		// read out of the DOM, never assumed
		const heading = page.locator("#wall h1");
		await expect(heading).toHaveCount(1);
		await expect(heading).toHaveText("a bluesky feed, laid on mason");
	});
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

// `feed_page` is six optional strings in a row, so a transposed pair
// typechecks on both sides and still lays the wrong wall. The tsc project over
// the worker counts the arguments; it cannot read their order, and this is the
// only lane that can.
//
// Entirely offline on the demo wall, and each of the three assertions below
// fails on a different adjacent transposition:
//   - every brick is a post          the `mode` slot (glaze filters the fixture
//                                    pool, which is 70/15/15 post/blog/video)
//   - warming is false               the `intent` slot (only a demo preview
//                                    sets it), and equally the `refresh` slot
//                                    beside it: transposed, `intent` reads "1"
//                                    and parses as a normal committed page,
//                                    which reports no warming at all
//   - the cursor comes back verbatim the `cursor` slot (the demo preview
//                                    re-encodes the INCOMING offset, so a
//                                    cursor read from the wrong slot decodes to
//                                    nothing and returns as offset 0), and the
//                                    same intent/refresh swap, whose committed
//                                    page hands back the NEXT screen instead
// The actor/feed pair needs no assertion of its own: transposed, `feed` would
// be "demo", which parses as no feed at all, and the first fetch would be a 400.
//
// The demo wall ignores `refresh` itself (its bricks are fixtures compiled into
// the wasm, and there is nothing behind them to re-read), so what the parameter
// proves here is the BINDING and not the re-read: the engine's own tests own
// what a refresh does once it arrives.
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
		`/api/feed?actor=demo&mode=glaze&intent=preview&refresh=1&cursor=${encodeURIComponent(cursor)}`,
	);
	expect(preview.status).toBe(200);

	const kinds = (preview.body.items ?? []).map((item) => item.kind);
	expect(kinds.length).toBeGreaterThan(0);
	// filtered rather than `.every`, so a failure names what came back
	expect(kinds.filter((kind) => kind !== "post")).toEqual([]);
	expect(preview.body.warming).toBe(false);
	expect(preview.body.cursor).toBe(cursor);
});
