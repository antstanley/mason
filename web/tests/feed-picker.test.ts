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

/** Three stored entries, in the order the picker draws them.
 *
 *  THREE and not one, which is the whole point of them. `REMEMBERED` above is a
 *  row of length one, and one is precisely the length at which a recents row
 *  cannot go wrong: taking a card remembers its feed, remembering moves that
 *  feed to the head of the same list, and moving the only card to the front of a
 *  list of one changes nothing. Every reorder bug this row can have is invisible
 *  there, which is how one lived through three reviews.
 *
 *  Distinct dids as well as distinct rkeys, so a landed URL names exactly one of
 *  the three and can never be read as a near miss. */
const ALPHA = {
	uri: "at://did:plc:aaaaaaaaaaaaaaaaaaaaaaaa/app.bsky.feed.generator/alpha",
	name: "Alpha",
	avatar: null,
	creator: "alpha.test",
	description: "the first wall this reader laid",
	likeCount: 1,
};
const BRAVO = {
	uri: "at://did:plc:bbbbbbbbbbbbbbbbbbbbbbbb/app.bsky.feed.generator/bravo",
	name: "Bravo",
	avatar: null,
	creator: "bravo.test",
	description: "the second wall this reader laid",
	likeCount: 2,
};
const CHARLIE = {
	uri: "at://did:plc:cccccccccccccccccccccccc/app.bsky.feed.generator/charlie",
	name: "Charlie",
	avatar: null,
	creator: "charlie.test",
	description: "the third wall this reader laid",
	likeCount: 3,
};
const RECENTS = [ALPHA, BRAVO, CHARLIE];

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

/** The recents row, found by what a screen reader is told it holds rather than
 *  by position: the results row below it is a list of cards too, and a bare
 *  `getByRole("link")` would walk straight into it. */
function recentsRow(page: Page): Locator {
	return panel(page).getByRole("list", { name: `${RECENTS.length} recent feeds` });
}

/** Plant a recents row before the app loads.
 *
 *  `page.addInitScript` and NOT `context.addInitScript`, which is load-bearing
 *  for the middle-click case below rather than a style choice: a context script
 *  runs in EVERY page of the context, including the background tab that click
 *  opens, and that tab is same-origin, so it shares this localStorage. It would
 *  re-plant this fixture over the write the click had just made, and the storage
 *  assertion would be reading the seed back to itself. */
async function plantRecents(page: Page): Promise<void> {
	await page.addInitScript((entries) => {
		localStorage.setItem("mason:feeds", JSON.stringify(entries));
	}, RECENTS);
}

/** Whichever feed the address bar is showing, decoded. Read through `URL` and
 *  not matched as a pattern, so the assertion is the whole uri and cannot pass
 *  on a prefix two of these three fixtures share. */
function laidFeed(url: string): string | null {
	return new URL(url).searchParams.get("feed");
}

/** The head of `mason:feeds`, which is the feed most recently opened. */
async function rememberedFirst(page: Page): Promise<string | undefined> {
	const stored = await page.evaluate(() => localStorage.getItem("mason:feeds"));
	return (JSON.parse(stored ?? "[]") as { uri: string }[])[0]?.uri;
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
	// "pick a feed to lay" here, without the "or": on the panel it is the leading
	// action rather than the alternative to one, and it is where the panel puts
	// focus. The landing page's door keeps its "or", where the handle box IS the
	// question being asked
	await page.getByRole("button", { name: "pick a feed to lay" }).click();

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

// The same write, from the button that dispatches no click at all. Chromium
// fires auxclick alone for the middle button, so a card listening for `click`
// only remembered every feed this reader laid here and no feed they sent to a
// background tab, which is the one activation the card's own comment singles
// out. `navbar.test.ts` pins the switcher panel's copy of this link; both go
// through `feeds.rememberFromLink`, and these two cases are the only lanes that
// can see either of them wired up at all.
test("middle-clicking a feed card sends it to a tab and still remembers it", async ({
	context,
	page,
}) => {
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	const opened = context.waitForEvent("page");
	await panel(page).getByRole("link", { name: "What's Hot, by @alice.test" }).click({
		button: "middle",
	});
	const tab = await opened;
	// `commit` rather than a laid wall: what that tab does next is not this
	// case's business, and it is closed before it can ask anybody anything
	await tab.waitForURL(/\?feed=at%3A%2F%2Fdid%3Aplc%3A/, { waitUntil: "commit" });
	await tab.close();

	const stored = await page.evaluate(() => localStorage.getItem("mason:feeds"));
	expect(stored).toContain("app.bsky.feed.generator/whats-hot");

	// nothing navigated here, so the picker is still up over the page it opened
	// over: the reader sent one feed elsewhere and is free to pick another
	await expect(panel(page)).toBeVisible();
	expect(await neverLeft(page)).toBe(true);
});

// The recents row, taken at a position that is not the first, which is the only
// way to see the thing these two cases are here for.
//
// Taking a card remembers its feed, and remembering moves that feed to the head
// of the very list the card is drawn from: the list reorders under the click
// that started it. Svelte flushes in a microtask BETWEEN one event listener and
// the next, so a row keyed by position hands the anchor being clicked to
// whichever feed has just slid into that slot, `href` and all, before the
// browser resolves the navigation. Every card but the first laid the feed one
// place above it, in silence: the header, the wall and `mason:feeds` all agreed
// on a feed the reader never chose, and tapping again worked, because by then
// the feed they wanted was at the front. It read as flakiness.
//
// Keying by `feed.uri` MOVES the anchor instead of rebinding it, which is what
// the switcher panel's copy of this list always did (`navbar.test.ts`), and that
// panel being right is what proved the key rather than the click path was the
// cause. The picker's results row stays keyed by position on purpose and must:
// see the comment in `FeedPicker.svelte`.
//
// Nothing else in the repo can see any of this. The row is a `.svelte` body, so
// tsc never reads it; the vitest suites drive `feeds.svelte.ts` in node with no
// DOM, where there is no anchor and no navigation to be wrong about.
test("taking the third recent card lays the third feed, not the one above it", async ({ page }) => {
	await plantRecents(page);
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	// the row is the three that were planted, in the order they were planted:
	// "the third card is Charlie" is the premise everything below rests on
	const cards = recentsRow(page).getByRole("link");
	await expect(cards).toHaveCount(RECENTS.length);
	for (const [at, feed] of RECENTS.entries()) {
		await expect(cards.nth(at)).toHaveAttribute("aria-label", `${feed.name}, by @${feed.creator}`);
	}
	// and the third one's href is right BEFORE it is touched, so the assertion
	// after the click is about the navigation rather than about the markup
	const third = cards.nth(2);
	await expect(third).toHaveAttribute("href", `/?feed=${encodeURIComponent(CHARLIE.uri)}`);

	await third.click();

	await expect.poll(() => laidFeed(page.url())).toBe(CHARLIE.uri);
	// and the reorder really happened, so this is not green because `remember`
	// quietly stopped firing: the list DID move under the click and the anchor
	// went with it rather than being rebound
	expect(await rememberedFirst(page)).toBe(CHARLIE.uri);
});

// The same defect through the button that dispatches no `click` at all, where
// the wrong wall lands in a background tab the reader is not even looking at.
// The second card rather than the third, so the two cases do not share a
// position: under the old key this one opened Alpha.
test("middle-clicking the second recent card sends that same feed to the tab", async ({
	context,
	page,
}) => {
	await plantRecents(page);
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	const second = recentsRow(page).getByRole("link").nth(1);
	await expect(second).toHaveAttribute("aria-label", `${BRAVO.name}, by @${BRAVO.creator}`);

	const opened = context.waitForEvent("page");
	await second.click({ button: "middle" });
	const tab = await opened;
	// `commit` rather than a laid wall: the URL the browser committed to is the
	// whole of what this case is about, and waiting on the wasm worker in a
	// second page would be waiting for nothing
	await tab.waitForURL(/\?feed=at%3A%2F%2F/, { waitUntil: "commit" });
	const landed = laidFeed(tab.url());
	await tab.close();
	expect(landed).toBe(BRAVO.uri);

	// the list moved under the click here too, and this page did not: the picker
	// is still up over the page it opened over, with the row now led by Bravo
	expect(await rememberedFirst(page)).toBe(BRAVO.uri);
	await expect(panel(page)).toBeVisible();
	expect(await neverLeft(page)).toBe(true);
});

// "more feeds", and the flag behind it. NOTHING else in the repo touches this
// control: the picker's paging decision lives in a component body, where tsc
// cannot see it, and the vitest cases drive the rune, which has no `atEnd` on
// it at all. That gap is why the bug below lived here.
//
// The picker is mounted once in +layout.svelte and never unmounts, so its own
// `$state` outlives the dialog. Every case here therefore closes and reopens,
// which is the gesture the flag has to survive.
test("the end of one list is forgotten by the time the picker opens again", async ({ page }) => {
	// popular has another page waiting; the search answer is the end of its own
	// list. The distinction is the whole case: one control, two questions.
	await page.route(/public\.api\.bsky\.app/, async (route) => {
		const url = new URL(route.request().url());
		const searching = url.searchParams.has("query");
		await route.fulfill({
			json: searching ? { feeds: [QUIET] } : { feeds: [generator()], cursor: "page-2" },
			headers: { "access-control-allow-origin": "*" },
		});
	});
	await landing(page);
	await openPicker(page);

	const more = panel(page).getByRole("button", { name: "more feeds" });
	// popular carries a cursor, so there is more to ask for
	await expect(more).toBeVisible();

	// a search whose answer is the end of the list: asking for more adds nothing,
	// which IS the end, and the control takes itself away
	await ask(page, "quiet");
	await expect(panel(page).getByRole("link", { name: /Quiet Kiln/ })).toBeVisible();
	await more.click();
	await expect(more).toBeHidden();

	// close, reopen. The results are popular's again, cursor and all, so the
	// control belongs back on the screen: it was hidden by an answer to a question
	// nobody is asking any more.
	await page.getByRole("button", { name: "Close the feed picker" }).last().click();
	await expect(panel(page)).toHaveCount(0);
	await openPicker(page);

	await expect(panel(page).getByRole("link", { name: /What's Hot/ })).toBeVisible();
	await expect(more).toBeVisible();
});

test("a list with no more pages still hides the control while it is the question", async ({
	page,
}) => {
	// the other half, so the case above cannot pass by the flag never being set:
	// a cursorless popular list hides the control on the first ask and keeps it
	// hidden while that list is what the picker is showing.
	await appView(page, { popular: [generator()] });
	await landing(page);
	await openPicker(page);

	const more = panel(page).getByRole("button", { name: "more feeds" });
	await more.click();
	await expect(more).toBeHidden();
	// still hidden a moment later: nothing reopened, so nothing changed the question
	await expect(panel(page).getByRole("link", { name: /What's Hot/ })).toBeVisible();
	await expect(more).toBeHidden();
});
