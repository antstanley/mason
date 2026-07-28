import { expect, test, type Locator, type Page } from "@playwright/test";

// The pull gesture, driven in a real browser against the real static build, on
// a touch device at the width it exists for.
//
// THIS IS THE ONLY LANE THAT RENDERS `PullToRefresh` OR MOVES THE WALL AT ALL.
// The gesture's decisions (which drags are pulls, the band, the threshold, which
// releases lay the wall) are pinned in `src/lib/state/pull.test.ts`, which runs
// in node against two numbers and a boolean. Everything the component claims is
// only ever true if a case below is green: that the listeners are attached at
// all, that a touch event reaches them, that the wall actually moves, that the
// indicator says which side of the threshold the reader is on, and that letting
// go ends in exactly one refresh.
//
// Everything here is offline. The wall is actor `demo`, whose bricks are
// fixtures compiled into the wasm engine, laid by the service worker that
// intercepts /api/feed on a static host. As in refresh.test.ts, the demo wall
// IGNORES the `refresh` flag in the engine, so what these cases cover is the
// CLIENT half: the gesture, the trigger, and one flagged request going out.
//
// The gesture is dispatched rather than performed: playwright's touchscreen can
// tap and nothing else, so each case builds the same touchstart/touchmove/
// touchend sequence a finger produces. The listeners under test read only
// `touches`, `changedTouches` and `cancelable`, all of which a constructed event
// carries honestly.

// A phone, with a touchscreen. Both halves matter: `hasTouch` is what makes the
// browser dispatch (and construct) touch events at all, and 375px is where a
// reader's thumb is the only way to reach the top of the wall.
test.use({ viewport: { width: 375, height: 812 }, hasTouch: true });

/** How far a pull has to travel to arm, mirroring PULL_THRESHOLD. Spelled here
 *  rather than imported: this spec runs against the BUILT site, where that
 *  module is bundled and minified, so a value read from the source would be a
 *  claim about a file this lane never loads. A drift between the two shows up
 *  as the short pull arming or the long one not, which is what these cases are. */
const THRESHOLD = 72;

/** How far the wall stays open while the refresh a pull asked for runs,
 *  mirroring PULL_LAYING. Spelled here for the same reason THRESHOLD is. */
const LAYING = 54;

/** Open the demo wall, wait for the service worker to take control, and wait
 *  for the first screen to COMMIT: a pull refuses while the wall is warming, so
 *  a settled wall is the only one this gesture can act on. */
async function laidWall(page: Page): Promise<void> {
	await page.goto("/?actor=demo");
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});
	await expect(cards(page).first()).toBeVisible({ timeout: 30_000 });
	await expect(control(page)).toBeEnabled({ timeout: 30_000 });
}

/** Every brick laid on the wall right now. */
function cards(page: Page): Locator {
	return page.locator("#wall article");
}

/** The header control. Not the subject here, but it is the wall's readiness
 *  signal: it is disabled for exactly as long as a wall is being laid. */
function control(page: Page): Locator {
	return page.getByRole("button", { name: "lay this wall again" });
}

/** The pull indicator's words, or null when it is not on screen. It is
 *  `aria-hidden` (the wall's one live region already narrates the warm), so it
 *  is found by its class rather than by a role. */
async function indicator(page: Page): Promise<string | null> {
	return await page.evaluate(() => {
		const pill = document.querySelector("[aria-hidden='true'].fixed.inset-x-0.z-30");
		return pill?.textContent?.trim() ?? null;
	});
}

/** How far down the wall is currently pulled, in px, read off the real
 *  computed transform rather than off any state the app holds. This is where
 *  the wall IS, mid-transition included, so a case that wants where it is going
 *  polls this rather than sampling it once. */
async function wallOffset(page: Page): Promise<number> {
	return await page.evaluate(() => {
		const wall = document.querySelector("#wall");
		if (!wall) throw new Error("no #wall on the page");
		const matrix = new DOMMatrixReadOnly(getComputedStyle(wall).transform);
		return matrix.m42;
	});
}

// Where the wall has been TOLD to go is read off the inline style svelte writes,
// rather than off the computed transform above, and `letGo` is where that
// happens: the snap onto the shelf and the glide home are CSS transitions, so
// the computed value lags both by their duration, while the inline style is
// correct in the same flush as the release that caused it.

/** Count what the CLIENT asks `/api/feed` for, by wrapping `fetch` before the
 *  app loads. In local mode the service worker answers this path out of wasm, so
 *  what is worth counting is what the feed state DECIDED to ask for. */
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

/** Put a finger on the wall and drag it down `travel` px, in the steps a real
 *  one produces, WITHOUT letting go. Returns with the gesture still live, so a
 *  case can read the wall and the indicator mid-pull.
 *
 *  Each step yields a frame, because svelte writes the DOM in a microtask and
 *  the wall is only readable once it has. */
async function pullDown(page: Page, travel: number, from = 120): Promise<void> {
	await page.evaluate(
		async ({ travel, from }) => {
			const target = document.querySelector("#wall") ?? document.body;
			const frame = () => new Promise((resume) => requestAnimationFrame(resume));
			const at = (type: string, y: number) => {
				const touch = new Touch({ identifier: 1, target, clientX: 60, clientY: y });
				target.dispatchEvent(
					new TouchEvent(type, {
						touches: type === "touchend" ? [] : [touch],
						targetTouches: type === "touchend" ? [] : [touch],
						changedTouches: [touch],
						bubbles: true,
						cancelable: true,
					}),
				);
			};
			at("touchstart", from);
			await frame();
			// four steps, so the gesture passes through the slop, through the
			// undecided window and out the other side exactly as a finger does
			for (const step of [0.25, 0.5, 0.75, 1]) {
				at("touchmove", from + travel * step);
				// the steps ARE sequential: each move is read against the one before
				// oxlint-disable-next-line no-await-in-loop
				await frame();
			}
		},
		{ travel, from },
	);
}

/** What the release did, sampled in the same turn it happened. */
interface Released {
	/** Where the wall was told to go: the shelf, or straight home. */
	target: number;
	/** What the indicator said, or null when it had already gone. */
	pill: string | null;
}

/** Let go, wherever the finger is, and read what the release did before
 *  yielding to anything that could undo it.
 *
 *  THE READ IS SYNCHRONOUS, AND ON THIS WALL THAT IS THE ONLY MECHANISM THAT CAN
 *  SEE THE SHELF. The demo wall's bricks are compiled into the wasm, so its
 *  refresh is two service-worker round trips over fixtures and settles in
 *  MILLISECONDS: measured here, the wall is back off the shelf before the next
 *  animation frame. A sample taken after a frame cannot tell a shelf that never
 *  happened from one already let down, and a case that cannot tell those apart
 *  is worse than no case at all. So this drains the microtask queue and nothing
 *  else: svelte writes the DOM in a microtask, while a fetch, a service-worker
 *  response and a timer are all tasks, and none of them can land inside this
 *  function. On a live wall the shelf lasts seconds; that is the same state,
 *  read where it is deterministic. */
async function letGo(page: Page): Promise<Released> {
	return await page.evaluate(async () => {
		const target = document.querySelector("#wall") ?? document.body;
		const touch = new Touch({ identifier: 1, target, clientX: 60, clientY: 0 });
		target.dispatchEvent(
			new TouchEvent("touchend", {
				touches: [],
				targetTouches: [],
				changedTouches: [touch],
				bubbles: true,
				cancelable: true,
			}),
		);
		// twice, so a flush that schedules another flush is drained too
		await Promise.resolve();
		await Promise.resolve();
		const style = document.querySelector("#wall")?.getAttribute("style") ?? "";
		const pill = document.querySelector("[aria-hidden='true'].fixed.inset-x-0.z-30");
		return {
			target: Number(/translateY\((-?[\d.]+)px\)/.exec(style)?.[1] ?? Number.NaN),
			pill: pill?.textContent?.trim() ?? null,
		};
	});
}

// The whole gesture, in one case: the wall follows the finger, the indicator
// says which side of the threshold it is on, and letting go past it lays the
// wall again exactly once, without taking the bricks away.
test("a pull past the threshold lays the wall again, once", async ({ page }) => {
	await countFeedRequests(page);
	await laidWall(page);
	const before = (await feedRequests(page)).length;
	const laid = await cards(page).count();
	expect(laid).toBeGreaterThan(0);

	await pullDown(page, THRESHOLD + 60);

	// the wall really moved, and by less than the finger did: past the threshold
	// the band stiffens
	const pulled = await wallOffset(page);
	expect(pulled).toBeGreaterThanOrEqual(THRESHOLD);
	expect(pulled).toBeLessThan(THRESHOLD + 60);
	expect(await indicator(page)).toBe("let go to lay again");

	const released = await letGo(page);

	// THE SHELF: the wall does not go home on release, it settles onto a shelf
	// and stays there for as long as the refresh it asked for runs, so the
	// indicator has a gap to sit in rather than a card to sit on. See `letGo` for
	// why this is read where it is.
	expect(released.target).toBe(LAYING);
	expect(released.pill).toBe("laying bricks");

	// the bricks never left: a refresh reflows the outgoing wall, it never
	// collapses to the twelve-card initial grid
	await expect(cards(page).first()).toBeVisible();
	await expect(control(page)).toBeEnabled({ timeout: 30_000 });
	await expect(cards(page).first()).toBeVisible();

	// and the warm ending let the wall down off the shelf, which is the signal
	// the reader actually watches. Polled rather than sampled, because this one
	// is about where the wall ENDS UP: the glide home is a transition, so the
	// computed transform equals its destination only once it has run.
	await expect.poll(async () => Math.round(await wallOffset(page)), { timeout: 5000 }).toBe(0);
	expect(await indicator(page)).toBeNull();

	// one gesture, one refresh: the flagged cursorless preview and the commit
	// that rides its cursor. This is the same budget the button spends, because
	// it is the same call.
	const asked = (await feedRequests(page)).slice(before);
	expect(asked).toHaveLength(2);
	expect(asked.filter((url) => url.includes("refresh=1"))).toHaveLength(1);
});

// The threshold is the whole of the gesture's rate limit, and a pull that stops
// short of it has to cost nothing: a thumb resettling on the glass at the top of
// a wall must not be a hundred-author fan-out.
test("a pull short of the threshold moves the wall and lays nothing", async ({ page }) => {
	await countFeedRequests(page);
	await laidWall(page);
	const before = (await feedRequests(page)).length;

	await pullDown(page, THRESHOLD - 24);

	// it followed the finger 1:1 under the threshold, so what the reader sees is
	// how far they have to go
	expect(Math.round(await wallOffset(page))).toBe(THRESHOLD - 24);
	expect(await indicator(page)).toBe("pull to lay again");

	const released = await letGo(page);

	// straight home, with no stop on the shelf: the shelf belongs to a refresh,
	// and a pull that stopped short did not start one
	expect(released.target).toBe(0);
	expect(released.pill).toBeNull();
	// the wall snaps back and nothing was asked for
	await expect
		.poll(async () => Math.round(await wallOffset(page)), { timeout: 5000 })
		.toBe(0);
	expect(await feedRequests(page)).toHaveLength(before);
	expect(await indicator(page)).toBeNull();
});

// A drag upward at the top of the wall is a scroll, and it has to stay one: the
// gesture decides once, in the first few pixels, and hands the finger back for
// the rest of the drag even if it wanders back down through where it started.
test("an upward drag is a scroll, and never becomes a pull", async ({ page }) => {
	await countFeedRequests(page);
	await laidWall(page);
	const before = (await feedRequests(page)).length;

	await page.evaluate(async () => {
		const target = document.querySelector("#wall") ?? document.body;
		const frame = () => new Promise((resume) => requestAnimationFrame(resume));
		const at = (type: string, y: number) => {
			const touch = new Touch({ identifier: 1, target, clientX: 60, clientY: y });
			target.dispatchEvent(
				new TouchEvent(type, {
					touches: type === "touchend" ? [] : [touch],
					targetTouches: type === "touchend" ? [] : [touch],
					changedTouches: [touch],
					bubbles: true,
					cancelable: true,
				}),
			);
		};
		at("touchstart", 300);
		await frame();
		at("touchmove", 260); // up: this is a scroll
		await frame();
		at("touchmove", 460); // and back down through the origin, well past the threshold
		await frame();
		at("touchend", 460);
		await frame();
	});

	expect(Math.round(await wallOffset(page))).toBe(0);
	expect(await indicator(page)).toBeNull();
	expect(await feedRequests(page)).toHaveLength(before);
});

// A window listener hears every touch on the page, so "at the top of the wall"
// is not enough on its own: the switcher's panel sits over the wall, its scrim
// covers the rest, and neither is the wall. The switcher is the case that proves
// the rule, because its openness is local component state that nothing outside
// the component can read, so the `blocked` prop the three full-screen screens
// use cannot see it. What can see it is where the touch landed.
test("a drag on the open switcher panel is not a pull of the wall behind it", async ({ page }) => {
	await countFeedRequests(page);
	await laidWall(page);
	const before = (await feedRequests(page)).length;

	await page.getByRole("button", { name: /^Switch wall/ }).click();
	await expect(page.getByRole("dialog", { name: "Switch wall" })).toBeVisible();

	// the same gesture, started inside the panel rather than on the wall
	await page.evaluate(async () => {
		const target = document.querySelector('[role="dialog"]');
		if (!target) throw new Error("no switcher panel to drag");
		const frame = () => new Promise((resume) => requestAnimationFrame(resume));
		const at = (type: string, y: number) => {
			const touch = new Touch({ identifier: 1, target, clientX: 60, clientY: y });
			target.dispatchEvent(
				new TouchEvent(type, {
					touches: type === "touchend" ? [] : [touch],
					targetTouches: type === "touchend" ? [] : [touch],
					changedTouches: [touch],
					bubbles: true,
					cancelable: true,
				}),
			);
		};
		at("touchstart", 300);
		await frame();
		for (const step of [40, 90, 140, 190]) {
			at("touchmove", 300 + step);
			// oxlint-disable-next-line no-await-in-loop
			await frame();
		}
		at("touchend", 490);
		await frame();
	});

	// the wall never moved and nothing was asked for
	expect(Math.round(await wallOffset(page))).toBe(0);
	expect(await indicator(page)).toBeNull();
	expect(await feedRequests(page)).toHaveLength(before);
	// and the panel is still up: the gesture was not ours to take
	await expect(page.getByRole("dialog", { name: "Switch wall" })).toBeVisible();
});

// The gesture is only a refresh where there is a wall to lay again. `PullToRefresh`
// is mounted on the same pair the header is (`?actor=` or `?feed=`), so on the
// front door there is nothing listening at all.
//
// The front door is NOT request-free, and that is the point of counting them
// here rather than asserting none: `LandingWall` lays a real demo wall behind
// the handle form, and `HandleForm` warms one. What a pull there must not add is
// a THIRD ask, and above all not a flagged one.
test("the front door has nothing to pull", async ({ page }) => {
	await countFeedRequests(page);
	await page.goto("/");
	await expect(page.getByRole("button", { name: "lay bricks" })).toBeVisible({ timeout: 30_000 });
	// wait for that wall to be laid before counting: both of the front door's
	// asks are issued at mount, so a count taken while the form is up but the
	// bricks are not is a count of a page still loading
	await expect(page.locator("article").first()).toBeVisible({ timeout: 30_000 });
	const before = (await feedRequests(page)).length;

	await pullDown(page, THRESHOLD + 60);
	await letGo(page);

	expect(await indicator(page)).toBeNull();
	const asked = await feedRequests(page);
	expect(asked).toHaveLength(before);
	expect(asked.filter((url) => url.includes("refresh=1"))).toHaveLength(0);
});

// The desktop half of the same gesture. A wheel is the input a reader without a
// touchscreen has, and the whole thing is here rather than in the touch block
// because the two differ in exactly two places a browser can see: there is no
// down and no up, so a pull starts from rest and commits when the wheel stops.
//
// hasTouch is off and the viewport is a desktop one, so these run the path a
// mouse and a trackpad take. `page.mouse.wheel` dispatches a real wheel event,
// which is what makes this worth having: the deltas, their sign and their
// timestamps are the browser's rather than a fixture's.
test.describe("the wheel pull, on a desktop", () => {
	test.use({ viewport: { width: 1280, height: 800 }, hasTouch: false });

	/** Watch the wall's own inline transform, keeping the furthest it ever got and
	 *  the words the pill said there.
	 *
	 *  Sampled on every write rather than read afterwards, and that is not
	 *  belt-and-braces: a wheel pull commits 140ms after the last event, so an
	 *  `expect` that arrives late reads a wall that has already been let down and
	 *  cannot tell that from a pull that never happened. */
	async function watchPull(page: Page): Promise<void> {
		await page.evaluate(() => {
			const wall = document.querySelector("#wall");
			if (!wall) throw new Error("no #wall to watch");
			const seen = { offset: 0, pill: "" };
			(window as unknown as { masonPullWatch: typeof seen }).masonPullWatch = seen;
			const sample = () => {
				const style = wall.getAttribute("style") ?? "";
				const at = Number(/translateY\((-?[\d.]+)px\)/.exec(style)?.[1] ?? 0);
				if (at > seen.offset) {
					seen.offset = at;
					seen.pill =
						document
							.querySelector("[aria-hidden='true'].fixed.inset-x-0.z-30")
							?.textContent?.trim() ?? "";
				}
			};
			new MutationObserver(sample).observe(wall, { attributes: true, attributeFilter: ["style"] });
		});
	}

	/** The furthest the wall was pulled, and what the pill said there. */
	async function watched(page: Page): Promise<{ offset: number; pill: string }> {
		return await page.evaluate(
			() => (window as unknown as { masonPullWatch: { offset: number; pill: string } }).masonPullWatch,
		);
	}

	/** Keep scrolling up at the top of the wall, `notches` times, AT A TRACKPAD'S
	 *  CADENCE: 16ms apart, in one evaluate.
	 *
	 *  Dispatched rather than driven through `page.mouse.wheel`, and the reason is
	 *  the gesture's own clock. Every mouse.wheel call is a round trip to the
	 *  browser, which puts tens to hundreds of milliseconds between events; the
	 *  pull commits 140ms after the wheel stops and starts afresh only after 200ms
	 *  of quiet, so a driven sequence lands on the wrong side of both and reads as
	 *  a reader who pushed once, stopped, and pushed again. A real trackpad reports
	 *  every ~8-16ms. The timing is the thing under test here, so the timing is
	 *  what this reproduces; `page.mouse.wheel` is still used below, where what
	 *  matters is that the page really scrolls. */
	async function keepPullingUp(page: Page, notches: number, delta = -40): Promise<void> {
		await page.evaluate(
			async ({ notches, delta }) => {
				const target = document.querySelector("#wall") ?? document.body;
				for (let i = 0; i < notches; i++) {
					target.dispatchEvent(
						new WheelEvent("wheel", { deltaY: delta, deltaMode: 0, bubbles: true, cancelable: true }),
					);
					// oxlint-disable-next-line no-await-in-loop
					await new Promise((resume) => setTimeout(resume, 16));
				}
			},
			{ notches, delta },
		);
	}

	test("keeping the wheel scrolling up at the top lays the wall again", async ({ page }) => {
		await countFeedRequests(page);
		await laidWall(page);
		const before = (await feedRequests(page)).length;
		const laid = await cards(page).count();
		expect(laid).toBeGreaterThan(0);
		await watchPull(page);

		// 8 x 40px of wheel, scaled to 0.4, is 128px of pull: comfortably past the
		// 72px threshold and into the band
		await keepPullingUp(page, 8);

		// the wall moved past the threshold, and the pill said so there. "stop"
		// rather than "let go", because there is nothing held to let go of
		const pulled = await watched(page);
		expect(pulled.offset).toBeGreaterThanOrEqual(THRESHOLD);
		expect(pulled.pill).toBe("stop to lay again");

		// STOPPING IS THE RELEASE: nothing else happens here. The wall is waited
		// for by the request it sends rather than by the control's disabled
		// window, which on this wall is about ten milliseconds wide: its bricks are
		// compiled into the wasm, so a poll would miss it and report a control that
		// behaved perfectly as broken. refresh.test.ts pins that window where it
		// can be seen, synchronously, at the button.
		await expect
			.poll(async () => (await feedRequests(page)).filter((url) => url.includes("refresh=1")).length, {
				timeout: 30_000,
			})
			.toBe(1);
		await expect(control(page)).toBeEnabled({ timeout: 30_000 });

		// the bricks never left, and one gesture is one refresh
		await expect(cards(page).first()).toBeVisible();
		await expect.poll(async () => Math.round(await wallOffset(page)), { timeout: 5000 }).toBe(0);
		const asked = (await feedRequests(page)).slice(before);
		expect(asked).toHaveLength(2);
	});

	test("a single notch moves the wall a little and lays nothing", async ({ page }) => {
		await countFeedRequests(page);
		await laidWall(page);
		const before = (await feedRequests(page)).length;
		await watchPull(page);

		await keepPullingUp(page, 1);

		// it answered, because the wall is at its top and the reader pushed up,
		// and it never reached the line
		const pulled = await watched(page);
		expect(pulled.offset).toBeGreaterThan(0);
		expect(pulled.offset).toBeLessThan(THRESHOLD);
		expect(pulled.pill).toBe("pull to lay again");

		// and stopping there costs nothing: the threshold is the whole rate limit
		await expect.poll(async () => Math.round(await wallOffset(page)), { timeout: 5000 }).toBe(0);
		expect(await indicator(page)).toBeNull();
		expect(await feedRequests(page)).toHaveLength(before);
	});

	test("a wheel scrolling down the wall never pulls it", async ({ page }) => {
		await countFeedRequests(page);
		await laidWall(page);
		const before = (await feedRequests(page)).length;

		await page.mouse.move(640, 400);
		await page.mouse.wheel(0, 300);
		await page.mouse.wheel(0, 300);

		expect(await indicator(page)).toBeNull();
		expect(Math.round(await wallOffset(page))).toBe(0);
		// scrolled the wall, which is what a wheel going down is for
		expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
		// and pagination is allowed to have fetched, so this asserts the one thing
		// a scroll must never do rather than that nothing happened at all
		const asked = (await feedRequests(page)).slice(before);
		expect(asked.filter((url) => url.includes("refresh=1"))).toHaveLength(0);
	});

	test("the bar sticks to the top of a desktop wall", async ({ page }) => {
		await laidWall(page);
		const bar = page.locator("header");
		const atRest = await bar.boundingBox();
		expect(atRest?.y ?? -1).toBeGreaterThanOrEqual(0);

		// scroll well down the wall, past several screens
		await page.evaluate(() => window.scrollTo(0, 2000));
		await expect.poll(async () => page.evaluate(() => window.scrollY)).toBeGreaterThan(500);

		const stuck = await bar.boundingBox();
		// still at the top of the viewport, and still the same bar
		expect(stuck?.y ?? -1).toBeGreaterThanOrEqual(0);
		expect(stuck?.y ?? -1).toBeLessThanOrEqual(1);
		await expect(page.getByRole("button", { name: "lay this wall again" })).toBeVisible();
	});
});
