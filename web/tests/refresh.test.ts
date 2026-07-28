import { expect, test, type Locator, type Page } from "@playwright/test";

// The refresh control, driven in a real browser against the real static build,
// at the width it has to survive.
//
// THIS IS THE ONLY LANE IN THE REPO THAT RENDERS `RefreshWall` AT ALL, and a
// green `just check` says nothing whatsoever about it. tsc cannot parse
// `.svelte`, so not one component file enters the typecheck program, and the
// vitest suite behind this change (`state/feed.svelte.ts`) is a `.ts` running in
// node with no DOM. `refresh()` itself is covered there, thoroughly. What is
// only ever true if a case below is green is everything the component body
// claims: that the control is on the bar at all at 375px, that it carries an
// accessible name, that `disabled` is a real attribute and not a look, that the
// outgoing wall stays up while the new one warms, and that a double tap is one
// refresh. `just test-e2e` is the lane; CI's e2e job is the enforcement point.
//
// Everything below is offline. The wall is actor `demo`, whose bricks are
// fixtures compiled into the wasm engine, laid by the service worker that
// intercepts /api/feed on a static host.
//
// WHAT THIS LANE DOES NOT COVER, honestly: the demo wall IGNORES `refresh` in
// the engine. Its bricks are compiled in, so there is no cache to step over and
// no newer answer to step onto, and `handle_feed` returns from the demo arm
// before the flag reaches anything. So these cases are about the CLIENT half of
// a refresh: the control, the disabled window, and what the wall looks like
// while it re-lays. The re-read itself (the flag reaching the two fast content
// caches, and never reaching an extension wave) is `cargo nextest`'s, in
// mortar-core.
//
// AND IT DOES NOT ASSERT THE READER CLOSE. `RefreshWall` calls `reader.close()`
// ahead of `feed.refresh()`, and no click here can reach that line: an open
// reader makes the layout's content wrapper `inert`, this control sits inside
// that wrapper, and it is the only trigger a refresh has. The call is the
// guarantee for the next trigger, not a live path, so a case here could only
// fake the state it claims to test.

// 375px, which is the constraint the header bar is written against: it never
// wraps on mobile, so a control that does not earn its width pushes another one
// off the screen rather than dropping to a second line. Every case runs at it,
// because the disabled window and the reflow are the same at any width and the
// width is the thing that is only true here.
test.use({ viewport: { width: 375, height: 812 } });

/** Open the demo wall, wait for the service worker to take control, and wait
 *  for the first screen to COMMIT. On a static host /api/feed answers at all
 *  only because the worker intercepted; the control is disabled until the wall
 *  settles, so waiting on it is waiting for a wall that can be refreshed. */
async function laidWall(page: Page): Promise<void> {
	await page.goto("/?actor=demo");
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});
	// warm can take up to the 8s ceiling before the first screen commits
	await expect(cards(page).first()).toBeVisible({ timeout: 30_000 });
	await expect(control(page)).toBeEnabled({ timeout: 30_000 });
}

/** Every brick laid on the wall right now. */
function cards(page: Page): Locator {
	return page.locator("#wall article");
}

/** The refresh control, found by its accessible name rather than by a class or
 *  a test id: the name is the whole of what a screen-reader reader gets, since
 *  the label is `sr-only` and the icon is `aria-hidden`. */
function control(page: Page): Locator {
	return page.getByRole("button", { name: "lay this wall again" });
}

/** Count what the CLIENT asks `/api/feed` for, by wrapping `fetch` before the
 *  app loads.
 *
 *  Not Playwright's own request events, and not `page.route`: in local mode the
 *  service worker answers this path out of wasm, so what is worth counting is
 *  what the feed state DECIDED to ask for. That is exactly the claim "a second
 *  tap starts nothing" makes. */
async function countFeedRequests(page: Page): Promise<void> {
	await page.addInitScript(() => {
		const asked: string[] = [];
		(window as unknown as { masonFeedRequests: string[] }).masonFeedRequests = asked;
		const original = window.fetch;
		window.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
			const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
			if (url.includes("/api/feed")) asked.push(url);
			return original.call(window, input, init);
		};
	});
}

/** Every `/api/feed` the client has asked for so far, in order. */
async function feedRequests(page: Page): Promise<string[]> {
	return await page.evaluate(
		() => (window as unknown as { masonFeedRequests: string[] }).masonFeedRequests,
	);
}

/** What the wall looked like across a refresh, sampled on every DOM change
 *  rather than polled: the states this is about last two service-worker round
 *  trips, and an `expect` that arrives afterwards cannot tell "it never
 *  happened" from "it already finished". */
interface WallWatch {
	/** The fewest bricks the wall ever had. Zero is the `initialLoad` state: the
	 *  twelve-card grid renders INSTEAD of the wall, so a wall that kept its
	 *  bricks is a wall that never collapsed to skeletons. */
	minCards: number;
	/** The most skeleton cards on the wall at once. Four is the warming tail and
	 *  is expected; twelve is the initial grid, which a refresh must never lay. */
	maxSkeletons: number;
}

/** Start watching the wall. Installed after it has settled, so nothing from the
 *  first load is in the sample. */
async function watchWall(page: Page): Promise<void> {
	await page.evaluate(() => {
		const wall = document.querySelector("#wall");
		if (!wall) throw new Error("no #wall to watch");
		// SkeletonCard's root is the only `.animate-pulse` on a laid wall: the
		// live badge's dot is `motion-safe:animate-pulse`, a different token
		const sample = () => {
			const watch = (window as unknown as { masonWallWatch: WallWatch }).masonWallWatch;
			watch.minCards = Math.min(watch.minCards, wall.querySelectorAll("article").length);
			watch.maxSkeletons = Math.max(watch.maxSkeletons, wall.querySelectorAll(".animate-pulse").length);
		};
		(window as unknown as { masonWallWatch: WallWatch }).masonWallWatch = {
			minCards: wall.querySelectorAll("article").length,
			maxSkeletons: wall.querySelectorAll(".animate-pulse").length,
		};
		new MutationObserver(sample).observe(wall, { childList: true, subtree: true });
	});
}

/** What the watcher saw. */
async function watched(page: Page): Promise<WallWatch> {
	return await page.evaluate(
		() => (window as unknown as { masonWallWatch: WallWatch }).masonWallWatch,
	);
}

// The control exists, says what it is, and fits. At 375px the bar carries the
// layout picker, this, the client picker and the switcher on one line that never
// wraps, so "it fits" is a real claim about this control and not decoration: the
// row overflowing is how the switcher walks off the right-hand edge.
test("the control is on the bar at 375px, named, and a 44px target", async ({ page }) => {
	await laidWall(page);
	const button = control(page);

	await expect(button).toBeVisible();
	// a real <button>, so the disabled state below is the platform's
	await expect(button).toHaveJSProperty("tagName", "BUTTON");
	await expect(button).toHaveAttribute("type", "button");

	const box = await button.boundingBox();
	// the 44px touch target, from min-h-11. Rounded off, because a fractional
	// layout at this width can land on 43.99
	expect(Math.round(box?.height ?? 0)).toBeGreaterThanOrEqual(44);

	// nothing on the page scrolls sideways: the row is `flex-nowrap`, so a
	// control that did not earn its width would push the bar past the viewport
	const overflow = await page.evaluate(() => ({
		document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
		row: (() => {
			const bar = document.querySelector("header > div");
			if (!bar) throw new Error("no control row in the header");
			return bar.scrollWidth - bar.clientWidth;
		})(),
	}));
	expect(overflow.document).toBeLessThanOrEqual(0);
	expect(overflow.row).toBeLessThanOrEqual(0);
});

// The whole point of the control, and the thing a refresh must never look like:
// the outgoing wall stays on screen and reflows into the new one. Skeletons
// mid-session read as something breaking, so the twelve-card grid is
// `initialLoad` only and a refresh never sets it. The four-card tail underneath
// IS expected, and is the wall saying more is on its way.
test("a refresh lays the wall again underneath the reader, without taking it away", async ({
	page,
}) => {
	await laidWall(page);
	const laid = await cards(page).count();
	expect(laid).toBeGreaterThan(0);
	await watchWall(page);

	await control(page).click();

	// the bricks never left, said twice: at the moment of the click, and over
	// every DOM change until the wall settles again
	await expect(cards(page).first()).toBeVisible();
	await expect(control(page)).toBeEnabled({ timeout: 30_000 });
	await expect(cards(page).first()).toBeVisible();

	const watch = await watched(page);
	// zero bricks is the initialLoad state, which renders the twelve-card grid
	// INSTEAD of the wall
	expect(watch.minCards).toBeGreaterThan(0);
	// the four-card warming tail, expected and required: without it the wall
	// would just sit there looking finished while it re-lays. Four rather than
	// "some", because twelve is the grid this must never be, and the two are the
	// same element with a different count
	expect(watch.maxSkeletons).toBe(4);
	// and a settled wall has no skeletons left on it at all
	await expect(page.locator("#wall .animate-pulse")).toHaveCount(0);
});

// `disabled` IS the rate limit: one refresh costs a hundred-author AppView
// fan-out and there is no server-side throttle by design, so a double tap must
// not become two.
//
// THE READ IS SYNCHRONOUS, AND THAT IS THE ONLY MECHANISM THAT CAN SEE IT. The
// disabled window here is roughly two service-worker round trips (the demo wall
// answers a preview already settled, so the client freezes on the first poll),
// which is short enough that a plain `await expect(button).toBeDisabled()` can
// arrive after it has closed and fail on a control that behaved perfectly.
// Delaying `/api/feed` with `context.route` to widen the window is not
// available: the service worker answers that path out of wasm over compiled-in
// fixtures, so there is no network request to delay. What is left is to never
// yield: one `page.evaluate` that clicks the control and reads its `disabled`
// property in the same function, draining only the microtask queue in between,
// which cannot deliver a service-worker response.
//
// A flaky assertion on the one control that is a rate limit would be worse than
// no assertion at all, because it gets deleted rather than fixed.
test("the control disables itself for the refresh it started, and a second tap starts nothing", async ({
	page,
}) => {
	await countFeedRequests(page);
	await laidWall(page);
	const before = (await feedRequests(page)).length;

	const seen = await page.evaluate(async () => {
		const button = [...document.querySelectorAll("button")].find(
			(candidate) => candidate.textContent?.trim() === "lay this wall again",
		);
		if (!button) throw new Error("no control named 'lay this wall again' on the header bar");

		const enabledBefore = !button.disabled;
		button.click();
		// Svelte writes the DOM in a microtask, so the attribute is not on the
		// element the instant the handler returns. Drain the microtask queue and
		// NOTHING else: a fetch, a service-worker response and a timer are all
		// tasks, so the window under test cannot close inside this function.
		// Twice, so a flush that schedules another flush is drained too.
		await Promise.resolve();
		await Promise.resolve();
		const disabledDuring = button.disabled;
		// the double tap, on a control that is now disabled. A styled-off look
		// would take this click; a real `disabled` attribute swallows it.
		button.click();
		return { enabledBefore, disabledDuring, stillDisabled: button.disabled };
	});

	expect(seen.enabledBefore).toBe(true);
	expect(seen.disabledDuring).toBe(true);
	expect(seen.stillDisabled).toBe(true);

	// and it comes back on its own when the wall settles, which is what makes it
	// a rate limit rather than a one-shot
	await expect(control(page)).toBeEnabled({ timeout: 30_000 });

	// one tap, one refresh: the cursorless preview and the commit that rides its
	// cursor. Two taps would be four, and the second pair would carry a second
	// `refresh=1`, which is a second hundred-author fan-out.
	const asked = (await feedRequests(page)).slice(before);
	expect(asked).toHaveLength(2);
	expect(asked.filter((url) => url.includes("refresh=1"))).toHaveLength(1);
});
