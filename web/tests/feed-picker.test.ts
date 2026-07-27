import { expect, test, type Locator, type Page } from "@playwright/test";

// The feed picker, driven in a real browser against the real static build.
//
// THIS IS THE ONLY LANE IN THE REPO THAT RENDERS `FeedPicker` OR `FeedCard` AT
// ALL, and a green `just check` says nothing whatsoever about either. tsc cannot
// parse `.svelte`, so not one component file enters the typecheck program; all
// three vitest suites behind this change (`state/feeds.svelte.ts`,
// `lib/feedref.ts`, and the rest) are `.ts` running in node with no DOM. The
// picker's decisions live in those modules precisely because they can be
// checked there; everything that lives in the component body instead is only
// ever true if a case below is green: the screen opening at all from either
// front door, focus going in and coming back, `inert` on the page behind it,
// Escape and the back gesture, every row of the states table, and a pasted
// value that will not parse being said in place.
//
// So: do not read a passing `just check` as coverage of the picker, and do not
// delete a case here because "the types cover it". Nothing else can see any of
// this. `just test-e2e` is the lane; CI is where it always runs.
//
// Everything below is offline. The public AppView is stubbed with
// `page.route` (browsing and search ride
// `app.bsky.unspecced.getPopularFeedGenerators`, which carries no stability
// promise, so a case that really called it would be a case that starts failing
// on somebody else's deploy), and the two cases that need no listing at all use
// the recents list and the browse-unavailable state, neither of which touches
// the network by design.

/** One feed generator as the AppView reports it. */
function generator(overrides: Record<string, unknown> = {}) {
	return {
		uri: "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot",
		displayName: "What's Hot",
		description: "the busiest bricks on the network, in one wall",
		avatar: "https://cdn.test/whats-hot.jpg",
		likeCount: 7,
		creator: { handle: "alice.test" },
		...overrides,
	};
}

/** A second generator, published with no likes at all: the tally is hidden at
 *  zero, like every tally on the wall, and only a card that HAS one to show
 *  proves the difference. */
const QUIET = generator({
	uri: "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/quiet",
	displayName: "Quiet Kiln",
	likeCount: 0,
	description: "a slow wall",
});

/** What one stored `mason:feeds` entry looks like. Planted before the app loads
 *  so the recents row is populated without opening a feed first. */
const REMEMBERED = {
	uri: "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/remembered",
	name: "Remembered",
	avatar: null,
	creator: "bob.test",
	description: "a feed this reader has laid before",
	likeCount: 3,
};

interface AppView {
	/** The resting state: the popular list, with no query. */
	popular?: unknown[];
	/** The same endpoint with a `query`. */
	search?: unknown[];
	/** `app.bsky.feed.getActorFeeds`. */
	creator?: unknown[];
	/** How long the AppView takes to answer, so the loading state can be seen. */
	delayMs?: number;
	/** The AppView is unreachable: every request to it fails. */
	down?: boolean;
}

/** Stub the public AppView for this page. Fulfilled cross-origin responses are
 *  still subject to CORS, so the allow-origin header is not decoration: without
 *  it the browser drops every answer and every case below reads as an AppView
 *  that would not answer. */
async function appView(page: Page, answers: AppView): Promise<void> {
	await page.route(/public\.api\.bsky\.app/, async (route) => {
		if (answers.down) {
			await route.abort();
			return;
		}
		if (answers.delayMs) {
			await new Promise((resolve) => setTimeout(resolve, answers.delayMs));
		}
		const url = new URL(route.request().url());
		const feeds = url.pathname.endsWith("getActorFeeds")
			? (answers.creator ?? [])
			: url.searchParams.has("query")
				? (answers.search ?? [])
				: (answers.popular ?? []);
		await route.fulfill({
			json: { feeds },
			headers: { "access-control-allow-origin": "*" },
		});
	});
}

/** The landing page, with the second front door on it. Nothing here waits for
 *  the service worker: the picker is chrome over the landing page and owes the
 *  wall behind it nothing. */
async function landing(page: Page): Promise<void> {
	await page.goto("/");
	await expect(trigger(page)).toBeVisible();
	// a marker on the window object: a full navigation would build a new one and
	// take it with it, which is how "the picker never left the page" is asserted
	await page.evaluate(() => {
		(window as unknown as Record<string, boolean>).masonNeverLeft = true;
	});
}

/** Whether this is still the page the picker opened over. */
async function neverLeft(page: Page): Promise<boolean> {
	return await page.evaluate(
		() => (window as unknown as Record<string, boolean>).masonNeverLeft === true,
	);
}

/** The landing page's way into the picker. */
function trigger(page: Page): Locator {
	return page.getByRole("button", { name: "or pick a feed to lay" });
}

/** The picker's panel, found by what it IS rather than by a class or a test id:
 *  a modal dialog is the accessibility contract this screen signed up to. */
function panel(page: Page): Locator {
	return page.locator('[role="dialog"]');
}

/** The one input, which serves search, by-creator and paste. */
function query(page: Page): Locator {
	return panel(page).getByRole("textbox");
}

/** Open the picker from the landing page and wait for it to be up.
 *
 *  The click is retried rather than fired once. `page.goto` resolves on load,
 *  which is before the client bundle has hydrated the button, and a click that
 *  lands on static markup does nothing at all; there is no marker on the
 *  document to wait for instead. Clicking twice is not a hazard: `openPicker()`
 *  in the rune declines a second push while the picker is already up, which is
 *  the same guard that stops a reader from stacking two entries. */
async function openPicker(page: Page): Promise<void> {
	await expect(async () => {
		await trigger(page).click();
		await expect(panel(page)).toBeVisible({ timeout: 1_000 });
	}).toPass({ timeout: 20_000 });
}

/** Ask the picker something, the way a reader does. */
async function ask(page: Page, what: string): Promise<void> {
	await query(page).fill(what);
	await query(page).press("Enter");
}

/** Ask an element to take focus and report whether it did. `inert` cannot be
 *  read from outside any other way: an inert element keeps its attributes, its
 *  tabindex and its size, it simply refuses focus. */
async function takesFocus(page: Page, selector: string): Promise<boolean> {
	return await page.evaluate((target: string) => {
		const element = document.querySelector(target);
		if (!(element instanceof HTMLElement)) throw new Error(`nothing focusable at ${target}`);
		element.focus();
		return document.activeElement === element;
	}, selector);
}

/** The demo wall, laid, which is where the picker's second entry point lives.
 *  On a static host /api/feed answers at all only because the wasm service
 *  worker intercepted it. */
async function laidWall(page: Page): Promise<void> {
	await page.goto("/?actor=demo");
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});
	await expect(page.locator("#wall article").first()).toBeVisible({ timeout: 30_000 });
}

// The whole of the second front door in one case: a screen, opened over the
// landing page, listing feeds it asked the network for. The URL staying put is
// what makes it a screen rather than a route, which is why it is history state
// and not a `?picker=` parameter: the address bar keeps showing whatever is
// behind the picker, so a link copied from here is still a link to the wall.
test("the landing page's other front door opens the picker over it", async ({ page }) => {
	await appView(page, { popular: [generator(), QUIET] });
	await landing(page);
	const before = page.url();

	await openPicker(page);

	// a real modal dialog, named by the heading a sighted reader sees
	await expect(panel(page)).toHaveAttribute("aria-modal", "true");
	await expect(panel(page).getByRole("heading", { name: "pick a feed" })).toBeVisible();
	// focus went into the input on its own, which is what the picker is for
	await expect(query(page)).toBeFocused();

	// the results are a list, so a screen reader is told how many there are
	await expect(panel(page).getByRole("list", { name: "2 feeds" })).toBeVisible();

	// one card, carrying what the spec says a card carries
	const card = panel(page).getByRole("link", { name: "What's Hot, by @alice.test" });
	await expect(card).toBeVisible();
	await expect(card).toHaveAttribute(
		"href",
		"/?feed=at%3A%2F%2Fdid%3Aplc%3Az72i7hdynmk6r22z27h6tvur%2Fapp.bsky.feed.generator%2Fwhats-hot",
	);
	await expect(card.locator("img")).toHaveAttribute("src", "https://cdn.test/whats-hot.jpg");
	await expect(card.getByText("by @alice.test")).toBeVisible();
	await expect(card.getByText("the busiest bricks on the network, in one wall")).toBeVisible();
	await expect(card.locator('[aria-label="7 likes"]')).toBeVisible();

	// and the feed with no likes shows no tally at all, like every zero on the
	// wall: a fresh feed showing a 0 reads as a verdict on it
	const quiet = panel(page).getByRole("link", { name: "Quiet Kiln, by @alice.test" });
	await expect(quiet).toBeVisible();
	await expect(quiet.locator('[aria-label$="likes"]')).toHaveCount(0);

	expect(page.url()).toBe(before);
	expect(await neverLeft(page)).toBe(true);
});

// Escape shuts it, and focus goes back to the control that opened it rather
// than to the top of the document, so a keyboard reader lands where they were.
test("escape shuts the picker and hands focus back to the trigger", async ({ page }) => {
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	await page.keyboard.press("Escape");

	await expect(panel(page)).toHaveCount(0);
	await expect(trigger(page)).toBeFocused();
});

// The picker is a history entry, which is the whole reason the back gesture has
// to close it on a phone. What it must NOT do is leave mason: the landing page
// is still the page it was afterwards.
test("the back gesture shuts the picker and leaves the landing page behind it", async ({
	page,
}) => {
	await appView(page, { popular: [generator()] });
	await landing(page);
	const before = page.url();
	await openPicker(page);

	await page.goBack();

	await expect(panel(page)).toHaveCount(0);
	await expect(trigger(page)).toBeVisible();
	expect(page.url()).toBe(before);
	expect(await neverLeft(page)).toBe(true);
});

// The fifth row of the states table, and the only one with a wrong answer that
// costs something: a value that reads as a link and will not parse is said at
// the input, with the picker still up and the value still in the box. Laying
// the wall and letting mortar reject it would answer the same question one page
// later, with the picker gone and the value gone with it.
//
// The three cases are the three ways it goes wrong in a clipboard: the right
// host and the wrong record, a lookalike host, and something that is not a
// reference at all.
test("a pasted value that is not a feed says so in place, and nothing navigates", async ({
	page,
}) => {
	await appView(page, { popular: [generator()] });
	await landing(page);
	const before = page.url();
	await openPicker(page);

	for (const pasted of [
		"https://bsky.app/profile/alice.test/post/3k2a",
		"https://evilbsky.app/profile/alice.test/feed/whats-hot",
		"javascript:alert(1)",
	]) {
		await ask(page, pasted);

		// said in place, and tied to the input rather than floating beside it
		const complaint = panel(page).getByRole("alert");
		await expect(complaint).toBeVisible();
		await expect(query(page)).toHaveAttribute("aria-invalid", "true");
		const describedBy = await query(page).getAttribute("aria-describedby");
		expect(describedBy, `the complaint about ${pasted} names the input's description`).toBe(
			await complaint.getAttribute("id"),
		);

		// nothing navigated, the picker is still up, and the value is still there
		// to be fixed rather than retyped
		expect(page.url()).toBe(before);
		expect(await neverLeft(page)).toBe(true);
		await expect(panel(page)).toBeVisible();
		await expect(query(page)).toHaveValue(pasted);
	}

	// and editing the value takes the complaint away, rather than leaving it
	// standing over a box that no longer says what it complained about
	await query(page).fill("");
	await expect(panel(page).getByRole("alert")).toHaveCount(0);
});

// `inert` on the layout's content wrapper is what traps focus in the picker,
// and it is asserted from the outside, by asking the page behind for focus and
// being refused. Both halves matter: a wrapper-shaped mistake leaves the handle
// box tabbable behind an open picker, and a picker mounted INSIDE the wrapper it
// dims makes itself unfocusable and invisible to assistive tech, because inert
// covers every descendant.
test("the page behind an open picker refuses focus, and the picker takes it", async ({ page }) => {
	await appView(page, { popular: [generator()] });
	await landing(page);

	// the control: with no picker up, the handle box is an ordinary focusable
	// input, so the refusal below is inert and not a selector that matches nothing
	expect(await takesFocus(page, "#handle")).toBe(true);

	await openPicker(page);

	expect(await takesFocus(page, "#handle")).toBe(false);
	expect(await takesFocus(page, '[role="dialog"] input')).toBe(true);
	expect(
		await page.evaluate(() =>
			document.querySelector('[role="dialog"]')?.contains(document.activeElement),
		),
	).toBe(true);
});

// Rows two and three of the states table. They are one case because they are
// one input: what the reader typed is what decides which question was asked, so
// "no feeds by that name" and "that person has not made any feeds" have to be
// told apart by the picker rather than by the reader.
test("a search with nothing behind it says so, and a creator with none offers their wall", async ({
	page,
}) => {
	await appView(page, { popular: [generator()], search: [], creator: [] });
	await landing(page);
	await openPicker(page);

	await ask(page, "nothing at all like a feed name");
	await expect(panel(page).getByText("no feeds by that name", { exact: false })).toBeVisible();

	// a handle is the bridge between mason's two front doors, so the empty answer
	// offers the other one rather than a dead end
	await ask(page, "@Alice.Test");
	await expect(panel(page).getByText("that person has not made any feeds")).toBeVisible();
	await expect(panel(page).getByRole("link", { name: "lay @alice.test's wall instead" })).toHaveAttribute(
		"href",
		"/?actor=alice.test",
	);
});

// Row four, and the reason the picker is not a search box: browsing rides
// `app.bsky.unspecced.getPopularFeedGenerators`, which carries no stability
// promise at all. When it goes quiet the picker says so quietly and keeps the
// two load-bearing paths, neither of which touches the network: the recents
// list, and the paste box.
test("with the appview unreachable the picker still offers recents and the paste box", async ({
	page,
}) => {
	await page.addInitScript((entry) => {
		localStorage.setItem("mason:feeds", JSON.stringify([entry]));
	}, REMEMBERED);
	await appView(page, { down: true });
	await landing(page);
	await openPicker(page);

	await expect(panel(page).getByText("browsing is quiet right now", { exact: false })).toBeVisible();

	// the recents row, drawn from storage with no request behind it
	await expect(panel(page).getByRole("list", { name: "1 recent feed" })).toBeVisible();
	await expect(panel(page).getByRole("link", { name: "Remembered, by @bob.test" })).toBeVisible();

	// and the paste box still answers
	await ask(page, "https://bsky.app/profile/alice.test/post/3k2a");
	await expect(panel(page).getByRole("alert")).toBeVisible();
});

// Row one. Skeletons rather than a spinner or an empty list, because "nothing
// found" and "nothing yet" read identically and mean opposite things.
test("a page in flight is six skeletons at the picker's own column count", async ({ page }) => {
	await appView(page, { popular: [generator()], delayMs: 3_000 });
	await landing(page);
	await openPicker(page);

	await expect(panel(page).locator(".animate-pulse")).toHaveCount(6);
	// said as well as drawn: the skeletons themselves are aria-hidden
	await expect(panel(page).locator('[aria-live="polite"]')).toHaveText("looking for feeds");

	// and they are replaced by the answer rather than sitting under it
	await expect(panel(page).getByRole("link", { name: "What's Hot, by @alice.test" })).toBeVisible({
		timeout: 10_000,
	});
	await expect(panel(page).locator(".animate-pulse")).toHaveCount(0);
});

// The second entry point, on a laid wall, where the switcher is already the
// switch-walls affordance. Escape hands focus back to the switcher's own
// button, not to the control inside the panel that opened the picker: that one
// unmounts with the panel, and focus handed to a detached element goes to the
// top of the document.
test("the switcher on a laid wall is the picker's other way in", async ({ page }) => {
	await appView(page, { popular: [generator()] });
	await laidWall(page);

	const switcher = page.getByRole("button", { name: /^Switch wall/ });
	await switcher.click();
	await page.getByRole("button", { name: "or pick a feed to lay" }).click();

	await expect(panel(page)).toBeVisible();
	await expect(query(page)).toBeFocused();
	// the wall behind it is frozen, exactly as it is behind the reader
	expect(await takesFocus(page, 'input[name="layout"]')).toBe(false);

	await page.keyboard.press("Escape");
	await expect(panel(page)).toHaveCount(0);
	await expect(switcher).toBeFocused();
});

// Every control is at least 44px, which is a code rule in this repo rather than
// a review note. Read off the rendered boxes and not off class names: a
// `min-h-11` that something else overrides is still a 30px tap target, and only
// a rendered page can tell the difference.
test("every control in the picker is a 44px target", async ({ page }) => {
	// under reduced motion, so nothing is measured mid-animation: the panel's
	// entrance ends at scale(1) and every box read while it is still arriving is
	// read at 0.99 of the size it will be
	await page.emulateMedia({ reducedMotion: "reduce" });
	await appView(page, { popular: [generator(), QUIET] });
	await landing(page);
	await openPicker(page);
	// the cards have to be on the page before they can be measured
	await expect(panel(page).getByRole("list", { name: "2 feeds" })).toBeVisible();

	const controls = panel(page).locator("a, button, input");
	const count = await controls.count();
	// the close control, the box, the submit and two cards at least: a selector
	// that matched nothing would pass the loop below without checking anything
	expect(count).toBeGreaterThanOrEqual(5);
	for (let i = 0; i < count; i++) {
		const control = controls.nth(i);
		const box = await control.boundingBox();
		const named = (await control.getAttribute("aria-label")) ?? (await control.innerText());
		expect(box?.height ?? 0, `"${named}" is a 44px target`).toBeGreaterThanOrEqual(44);
	}
});

// Choosing a feed lays it and remembers it, which is what fills the recents row
// for next time. The write is the picker's only lasting effect on this browser,
// and it is asserted through `mason:feeds` itself rather than through the row it
// draws, because the row is what a second open would show and this is what it
// would be showing.
test("choosing a feed lays that feed and remembers it", async ({ page }) => {
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	await panel(page).getByRole("link", { name: "What's Hot, by @alice.test" }).click();

	await expect(page).toHaveURL(/\?feed=at%3A%2F%2Fdid%3Aplc%3A/);
	await expect(panel(page)).toHaveCount(0);
	const stored = await page.evaluate(() => localStorage.getItem("mason:feeds"));
	expect(stored).toContain("app.bsky.feed.generator/whats-hot");
});
