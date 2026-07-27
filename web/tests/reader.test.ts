import { expect, test, type Locator, type Page } from "@playwright/test";

// The brick reader, driven in a real browser against the real static build.
//
// THIS IS THE ONLY LANE IN THE REPO THAT RENDERS `BrickReader` AT ALL, and a
// green `just check` says nothing whatsoever about it. tsc cannot parse
// `.svelte`, so not one component file enters the typecheck program; both
// vitest suites are `.ts` running in node with no DOM. The rune behind the
// reader (`state/reader.svelte.ts`) is typechecked and unit tested, but every
// claim that lives in the component body instead is only ever true if a case
// below is green: the dialog opening on a click at all, focus going in and
// coming back, the scroll lock, `inert` on the wall behind it, the reduced
// motion alternative, arrow stepping, and the shared reveal.
//
// So: do not read a passing `just check` as coverage of the reader, and do not
// delete a case here because "the types cover it". Nothing else can see any of
// this. `just test-e2e` is the lane; CI is where it always runs.
//
// Everything below is offline. The wall is actor `demo`, whose bricks are
// fixtures compiled into the wasm engine, laid by the service worker that
// intercepts /api/feed on a static host. Exactly one of those fixture bricks
// carries a `!warn` blur, which is what gives the reveal something to reveal.

/** Open the demo wall and wait for the service worker to take control. On a
 *  static host /api/feed answers at all only because it intercepted. */
async function laidWall(page: Page): Promise<void> {
	await page.goto("/?actor=demo");
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});
	// warm can take up to the 8s ceiling before the first screen commits
	await expect(page.locator("#wall article").first()).toBeVisible({ timeout: 30_000 });
}

/** Every brick laid on the wall right now. */
function cards(page: Page): Locator {
	return page.locator("#wall article");
}

/** The reader's panel, found by what it IS rather than by a class or a test id:
 *  a modal dialog is the accessibility contract this change signed up to. */
function panel(page: Page): Locator {
	return page.locator('[role="dialog"]');
}

/** The scrim behind the panel. Both it and the close control carry the same
 *  accessible name, so they are told apart by the one structural difference
 *  between them: the scrim is deliberately not tabbable. */
function scrim(page: Page): Locator {
	return page.locator('button[aria-label="Close the reader"][tabindex="-1"]');
}

/** The close control inside the panel. */
function closeControl(page: Page): Locator {
	return panel(page).getByRole("button", { name: "Close the reader" });
}

/** The live region that says where in the wall the reader has landed. It is the
 *  reader's own account of which brick it is showing, so a step is asserted
 *  against it rather than against a count of anything. */
function readout(page: Page): Locator {
	return panel(page).locator('p[aria-live="polite"]');
}

/** Where the reader says it is, parsed. Throws rather than returning a fallback:
 *  a readout that stopped saying this is a regression, not a default. */
async function position(page: Page): Promise<{ at: number; total: number }> {
	const said = (await readout(page).innerText()).trim();
	const parsed = /^brick (\d+) of (\d+)$/.exec(said);
	if (!parsed?.[1] || !parsed[2]) throw new Error(`the reader's readout says "${said}"`);
	return { at: Number(parsed[1]), total: Number(parsed[2]) };
}

/** A post card with nothing covering it, and its own post text. The covered
 *  fixture brick is skipped on purpose: its cover puts a "sensitive media"
 *  paragraph ahead of the post's, and this reads the first paragraph as the
 *  brick's text. */
async function plainPostCard(page: Page): Promise<{ card: Locator; text: string }> {
	const card = page
		.locator('#wall article[aria-label^="post by"]')
		.filter({ hasNot: page.getByRole("button", { name: "Show sensitive media" }) })
		.first();
	const text = (await card.locator("p").first().innerText()).trim();
	// a fixture post text is a whole sentence; an empty read here would make
	// every "the reader carries this text" assertion below pass vacuously
	expect(text.length).toBeGreaterThan(20);
	return { card, text };
}

/** Switch the wall to glaze, through the picker, which is also the only way a
 *  reader can: the image wall is a layout preference and an algorithm, never a
 *  URL parameter the page reads.
 *
 *  It waits for a control only `GlazeCard` renders rather than for a count,
 *  because the count is the same on both walls and the covered brick is first
 *  on both: a race here would test the masonry card again and report it as
 *  glaze coverage. */
async function glazeWall(page: Page): Promise<void> {
	await page.getByRole("group", { name: "Wall layout" }).getByText("Glaze", { exact: true }).click();
	// attached rather than visible: the pill's reveal button is display:none
	// where hover works, which is every desktop browser this runs in
	await expect(
		page.locator('#wall article button[aria-label="Show post details"]').first(),
	).toBeAttached({ timeout: 30_000 });
}

/** The one covered brick on the demo wall, addressed by its position rather
 *  than by its cover, so the same card can still be found after the reveal has
 *  taken that cover away. */
async function coveredCard(page: Page): Promise<Locator> {
	const at = await cards(page).evaluateAll((articles) =>
		articles.findIndex(
			(article) => article.querySelector('button[aria-label="Show sensitive media"]') !== null,
		),
	);
	if (at < 0) {
		throw new Error(
			"no covered brick on the demo wall: fixtures.rs gives brick 0 a !warn blur, and the shared reveal is unobservable without it",
		);
	}
	return cards(page).nth(at);
}

/** The play control on a card or in the reader, named by what it offers rather
 *  than by a class: a live brick says "Watch live" and a clip says "Play video".
 *  A prefix rather than the whole name, because a card names the video it would
 *  play ("Play video: <title>") and the reader, which shows that title beside
 *  the control, does not. Nothing plays until one of these is pressed. */
function playControl(scope: Locator): Locator {
	return scope.getByRole("button", { name: /^(Play video|Watch live)/ });
}

/** The first video brick on the demo wall, addressed by its position for the
 *  same reason the covered card is: pressing play takes the control away, and
 *  a locator that found the card BY that control would then find a different
 *  card, which is exactly the confusion the case below has to avoid. */
async function videoCard(page: Page): Promise<Locator> {
	const at = await cards(page).evaluateAll((articles) =>
		articles.findIndex(
			(article) =>
				article.querySelector(
					'button[aria-label^="Play video"], button[aria-label^="Watch live"]',
				) !== null,
		),
	);
	if (at < 0) throw new Error("no video brick on the demo wall");
	return cards(page).nth(at);
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

/** What `document.documentElement` is scrolling like right now. The reader locks
 *  it while it is up and must hand back whatever it found. */
async function rootOverflow(page: Page): Promise<string> {
	return await page.evaluate(() => document.documentElement.style.overflow);
}

/** How an element is actually animating, read off the computed style and the
 *  running animation rather than off its class list. The reduced-motion
 *  alternative is a media block in app.css overriding `.animate-reader-in`, so
 *  the class is present in BOTH states and proves nothing about either. */
async function motion(
	page: Page,
	selector: string,
): Promise<{ name: string; duration: string; frames: number; moves: boolean }> {
	return await page.evaluate((target: string) => {
		const element = document.querySelector(target);
		if (!element) throw new Error(`nothing at ${target}`);
		const style = getComputedStyle(element);
		const frames = element.getAnimations().flatMap((animation) => {
			const effect = animation.effect;
			// only a KeyframeEffect has keyframes to read; anything else has no
			// answer to "does the panel move" and is dropped rather than guessed at
			return effect instanceof KeyframeEffect ? effect.getKeyframes() : [];
		});
		return {
			name: style.animationName,
			duration: style.animationDuration,
			// carried out so a case can refuse to pass on an empty list: "no frame
			// carries a transform" is also true of no frames at all
			frames: frames.length,
			moves: frames.some(
				(frame) => typeof frame.transform === "string" && frame.transform !== "none",
			),
		};
	}, selector);
}

// The whole of the change in one case: a plain left click on a card renders that
// brick over the wall it came from, and the wall is still the page it was. The
// marker is what makes "did not navigate" observable: a full navigation would
// build a new window and take it with it, and a SvelteKit client-side one would
// too (the reader uses shallow routing precisely so neither happens). The URL
// staying put is the other half, and is why the reader is history state rather
// than a `?brick=` parameter it could never honour on someone else's wall.
test("a plain click reads a brick in place, over the wall it came from", async ({ page }) => {
	await laidWall(page);
	const { card, text } = await plainPostCard(page);

	const before = page.url();
	await page.evaluate(() => {
		(window as unknown as Record<string, boolean>).masonNeverLeft = true;
	});

	await card.locator("a").first().click({ position: { x: 10, y: 10 } });

	await expect(panel(page)).toBeVisible();
	// a real modal dialog, named by the brick rather than by nothing: the panel
	// is the page's one landmark while it is up
	await expect(panel(page)).toHaveAttribute("aria-modal", "true");
	await expect(panel(page)).toHaveAttribute("aria-label", /\S/);
	// exact, so this is the WHOLE text and not a clamp of it: the card is a
	// summary by design and the reader is the same brick with nothing left out
	await expect(panel(page).getByText(text, { exact: true })).toBeVisible();
	expect(page.url()).toBe(before);
	expect(
		await page.evaluate(() => (window as unknown as Record<string, boolean>).masonNeverLeft),
	).toBe(true);
});

// Escape shuts it, and focus goes back to the card that opened it rather than
// to the top of the document, so a keyboard reader lands where they were on the
// wall. The anchor is the element `activate` was handed as the click's
// currentTarget, which is what the rune remembers.
test("escape shuts the reader and hands focus back to the card", async ({ page }) => {
	await laidWall(page);
	const { card } = await plainPostCard(page);
	const anchor = card.locator("a").first();

	await anchor.click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();

	await page.keyboard.press("Escape");
	await expect(panel(page)).toHaveCount(0);
	await expect(anchor).toBeFocused();
});

// The reader is a history entry, which is the whole reason the back gesture has
// to close it on a phone. What it must NOT do is leave the wall: the same
// bricks are still laid afterwards, with no skeleton tail, which is what a wall
// that was re-fetched from scratch would show instead.
test("the back gesture shuts the reader and leaves the wall laid", async ({ page }) => {
	await laidWall(page);
	const laid = await cards(page).count();
	const { card } = await plainPostCard(page);

	await page.evaluate(() => {
		(window as unknown as Record<string, boolean>).masonNeverLeft = true;
	});
	await card.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();

	await page.goBack();

	await expect(panel(page)).toHaveCount(0);
	await expect(cards(page)).toHaveCount(laid);
	// SkeletonCard's root is the only `.animate-pulse` on a laid wall: the live
	// badge's dot is `motion-safe:animate-pulse`, a different class token
	await expect(page.locator("#wall .animate-pulse")).toHaveCount(0);
	expect(
		await page.evaluate(() => (window as unknown as Record<string, boolean>).masonNeverLeft),
	).toBe(true);
});

// `inert` on the layout's content wrapper is what traps focus in the reader,
// and it is asserted from the outside, by asking the wall for focus and being
// refused. Both halves matter: the header's own controls sit OUTSIDE the wall
// but inside that wrapper, so a wrapper-shaped mistake (marking the wall
// instead) leaves the pickers tabbable behind an open reader, and a
// reader-shaped one (mounting the panel inside the wrapper it dims) makes the
// reader itself unfocusable and invisible to assistive tech.
test("the wall behind an open reader refuses focus, and the reader takes it", async ({ page }) => {
	await laidWall(page);
	const { card } = await plainPostCard(page);

	// the control: with no reader up, the picker is an ordinary focusable radio,
	// so the refusal below is inert and not a selector that matches nothing
	expect(await takesFocus(page, 'input[name="layout"]')).toBe(true);

	await card.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();

	// focus moved in on its own, before anything below asks anything to take it
	expect(
		await page.evaluate(() =>
			document.querySelector('[role="dialog"]')?.contains(document.activeElement),
		),
	).toBe(true);

	expect(await takesFocus(page, 'input[name="layout"]')).toBe(false);
	expect(await takesFocus(page, '[role="dialog"] button[aria-label="Close the reader"]')).toBe(
		true,
	);
});

// The wall must not scroll behind the reader, and must scroll again however the
// reader was shut. All four routes, because the lock is released by a teardown
// hung on the reader's state rather than by any one control: the back gesture
// never calls close() at all, and a route that skipped the teardown would leave
// the whole page unscrollable with nothing on screen to explain it.
test("the wall is locked while the reader is up, and unlocked by every way out", async ({
	page,
}) => {
	await laidWall(page);
	const { card } = await plainPostCard(page);
	const anchor = card.locator("a").first();
	// whatever the page was scrolling like before any of this, which is what the
	// reader has to hand back rather than a hardcoded ''
	const before = await rootOverflow(page);

	/** Open the reader, check the lock is on, shut it the given way, check the
	 *  lock is off again. */
	async function through(route: string, shut: () => Promise<void>) {
		await anchor.click({ position: { x: 10, y: 10 } });
		await expect(panel(page)).toBeVisible();
		expect(await rootOverflow(page), `locked while the reader is up (${route})`).toBe("hidden");

		await shut();
		await expect(panel(page)).toHaveCount(0);
		expect(await rootOverflow(page), `unlocked again after ${route}`).toBe(before);
	}

	await through("escape", () => page.keyboard.press("Escape"));
	await through("the close control", () => closeControl(page).click());
	// a corner of the scrim, well clear of the centred panel: the gutter around
	// the panel is the scrim rather than a dead zone, and tapping it dismisses
	await through("a click on the scrim", () => scrim(page).click({ position: { x: 5, y: 5 } }));
	await through("the back gesture", async () => {
		await page.goBack();
	});
});

// The motion row from the design system, under both media states, read off the
// COMPUTED style and the running animation rather than off the class list: the
// reduced-motion alternative is an override inside a media block in app.css, so
// `.animate-reader-in` is on the panel either way and proves nothing about what
// it resolves to.
test("under full motion the panel rises as it arrives", async ({ page }) => {
	// set explicitly rather than inherited from the harness, so this case tests
	// the state it names whatever the runner's default is
	await page.emulateMedia({ reducedMotion: "no-preference" });
	await laidWall(page);
	const { card } = await plainPostCard(page);

	await card.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();

	const rise = await motion(page, '[role="dialog"]');
	// the reader's own keyframes, and not the crossfade every other entrance uses
	expect(rise.name).toBe("reader-in");
	expect(Number.parseFloat(rise.duration)).toBeGreaterThan(0);
	expect(rise.frames).toBeGreaterThan(0);
	// it moves: translateY(8px) scale(0.99) to nothing
	expect(rise.moves).toBe(true);

	const fade = await motion(page, 'button[aria-label="Close the reader"][tabindex="-1"]');
	// the scrim never moves in either state; it only ever fades
	expect(fade.name).toBe("brick-fade");
	expect(fade.frames).toBeGreaterThan(0);
	expect(fade.moves).toBe(false);
});

test("under reduced motion the panel does not move, and only the scrim fades", async ({ page }) => {
	await page.emulateMedia({ reducedMotion: "reduce" });
	await laidWall(page);
	const { card } = await plainPostCard(page);

	await card.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();

	const arrival = await motion(page, '[role="dialog"]');
	// the shared crossfade, at the row's own 0.15s: the panel arrives without
	// rising and without scaling
	expect(arrival.name).toBe("brick-fade");
	expect(arrival.duration).toBe("0.15s");
	expect(arrival.frames).toBeGreaterThan(0);
	expect(arrival.moves).toBe(false);

	// the scrim still fades, which is the whole of the reduced-motion arrival
	const fade = await motion(page, 'button[aria-label="Close the reader"][tabindex="-1"]');
	expect(fade.name).toBe("brick-fade");
	expect(fade.duration).toBe("0.15s");
	expect(fade.frames).toBeGreaterThan(0);
	expect(fade.moves).toBe(false);
});

// The arrows step along the laid wall, and stop where it stops. Stepping never
// paginates: a page pulled in from inside the reader would grow a wall the
// reader cannot see, so the brick count is asserted unchanged at the end.
test("the arrows step along the wall and stop at the last laid brick", async ({ page }) => {
	await laidWall(page);
	const laid = await cards(page).count();
	const { card, text } = await plainPostCard(page);

	await card.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();
	await expect(panel(page).getByText(text, { exact: true })).toBeVisible();
	const opened = await position(page);

	await page.keyboard.press("ArrowRight");
	// a different brick, said two ways: the reader's own account of where it is,
	// and the brick it opened on no longer being on screen
	await expect(readout(page)).toHaveText(`brick ${opened.at + 1} of ${opened.total}`);
	await expect(panel(page).getByText(text, { exact: true })).toHaveCount(0);

	// walk to the far end of the laid wall
	for (let step = opened.at + 1; step < opened.total; step++) {
		await page.keyboard.press("ArrowRight");
	}
	await expect(readout(page)).toHaveText(`brick ${opened.total} of ${opened.total}`);
	await expect(panel(page).getByRole("button", { name: "next brick" })).toBeDisabled();

	// and one step past the end is not a step
	await page.keyboard.press("ArrowRight");
	await expect(readout(page)).toHaveText(`brick ${opened.total} of ${opened.total}`);
	await expect(panel(page)).toBeVisible();
	await expect(cards(page)).toHaveCount(laid);
});

// The reveal choice follows the brick rather than the component that took it:
// uncovering media on the wall and finding it covered again one click later
// reads as a bug. The set is keyed by brick id and shared by the card and the
// reader, and this is the only lane that can watch it cross between them.
test("a brick revealed on the wall is still revealed in the reader", async ({ page }) => {
	await laidWall(page);
	const covered = await coveredCard(page);
	// covered: the media is inside the blurred, aria-hidden half of Sensitive
	await expect(covered.locator('[aria-hidden="true"] img')).toHaveCount(1);

	await covered.getByRole("button", { name: "Show sensitive media" }).click();
	await expect(covered.getByRole("button", { name: "Show sensitive media" })).toHaveCount(0);
	await expect(covered.locator('[aria-hidden="true"] img')).toHaveCount(0);

	// clicking the card's text rather than its media, which is where the reveal
	// control used to be
	await covered.locator("a").first().click({ position: { x: 10, y: 10 } });
	await expect(panel(page)).toBeVisible();
	await expect(panel(page).getByRole("button", { name: "Show sensitive media" })).toHaveCount(0);
	await expect(panel(page).locator('[aria-hidden="true"] img')).toHaveCount(0);
});

// "show anyway" is a reveal and nothing else. On two of the four cards that
// button is a DESCENDANT of the anchor the reader intercepts, so without the
// propagation stop the click opens the reader, and without the default stop it
// opens the post in a new tab.
//
// This is deliberately separate from the case above, and must stay separate:
// "still revealed when the reader opens on it" is satisfied by the WRONG
// behaviour too, since a reveal that also opened the reader would show an
// uncovered brick in it either way. Only an absent dialog tells the two apart.
test("show anyway reveals the media and nothing else", async ({ page }) => {
	await laidWall(page);
	const covered = await coveredCard(page);
	const before = page.url();
	const tabs = page.context().pages().length;

	await covered.getByRole("button", { name: "Show sensitive media" }).click();

	// revealed
	await expect(covered.getByRole("button", { name: "Show sensitive media" })).toHaveCount(0);
	await expect(covered.locator('[aria-hidden="true"] img')).toHaveCount(0);
	// and nothing else: no reader, no navigation, and no tab opened by the
	// anchor this button sits inside
	await expect(panel(page)).toHaveCount(0);
	expect(page.url()).toBe(before);
	expect(page.context().pages().length).toBe(tabs);
});

// The third activation point, on the one wall where no card has a card-wide
// link at all: a glaze card hands BrickShell no href, so its per-image anchors
// are the whole way in. Intercepting BrickShell alone would leave this entire
// wall unreadable in place, and nothing but this lane could tell.
//
// It carries the second half of the reveal rule too, at its OTHER in-anchor
// site: GlazeCard's single branch wraps `Sensitive` in the anchor it
// intercepts, exactly as BrickShell does on a post card.
test("a glaze card's image anchor reads the brick in place", async ({ page }) => {
	await laidWall(page);
	await glazeWall(page);
	const covered = await coveredCard(page);
	const before = page.url();

	await covered.getByRole("button", { name: "Show sensitive media" }).click();
	await expect(covered.getByRole("button", { name: "Show sensitive media" })).toHaveCount(0);
	await expect(panel(page)).toHaveCount(0);
	expect(page.url()).toBe(before);

	// and the image itself, which is the only thing a glaze card is
	await covered.locator("a").first().click();
	await expect(panel(page)).toBeVisible();
	expect(page.url()).toBe(before);
});

// One video plays at a time, network-wide, and the reader has to claim the
// player under an id of its OWN (`reader:<brick.id>`) rather than under the
// brick's. A card tears its player down when `player.activeId` stops matching
// its own brick id, so a reader claiming that same id would leave the card
// mounted and playing behind the scrim: two elements and two audio streams,
// with only one of them on screen.
//
// It has to be the SAME brick on both sides to catch that. Opening the reader
// on a different video would tear the first card down under either id, so the
// wrong claim would pass. Entirely offline: the fixture playlist is remote and
// never loads, but mounting the player is what is being watched, not playing it.
test("the reader claims the player under its own id, so the card behind lets go", async ({
	page,
}) => {
	await laidWall(page);
	const card = await videoCard(page);

	// the card plays in place, exactly as it did before the reader existed
	await playControl(card).click();
	await expect(card.locator("video")).toHaveCount(1);

	// the video card's ONLY anchor is its watch-at-source link, so this is the
	// one plain-click route into the reader that card has
	await card.locator("a").first().click();
	await expect(panel(page)).toBeVisible();
	// nothing has claimed anything yet, so the card is still mounted behind
	await expect(page.locator("#wall video")).toHaveCount(1);

	await playControl(panel(page)).click();
	await expect(panel(page).locator("video")).toHaveCount(1);
	// and the card lost the claim, through the teardown path that already existed
	await expect(page.locator("#wall video")).toHaveCount(0);
});

// The anchor is kept rather than swapped for a button so the browser's own
// affordances survive, and only an unmodified primary click is taken. A
// modified one still goes to the source, which is how a lot of people move
// content between apps. ControlOrMeta rather than Meta: the reader declines
// both, and CI is linux, where the "open in a new tab" chord is control.
test("a modified click still goes to the source, and opens no reader", async ({ page }) => {
	await laidWall(page);
	const { card } = await plainPostCard(page);
	const anchor = card.locator("a").first();

	// still a real outbound link, whichever client the reader picked: the anchor
	// was kept rather than swapped for a button precisely so this survives
	expect(await anchor.getAttribute("href")).toMatch(/^https?:\/\//);

	// a context event rather than a popup one: the anchor carries
	// rel="noopener", so the tab it opens has no opener to be a popup of
	const opened = page.context().waitForEvent("page");
	await anchor.click({ position: { x: 10, y: 10 }, modifiers: ["ControlOrMeta"] });
	const tab = await opened;
	await tab.close();

	// the tab is already the proof the click was not intercepted (an intercepted
	// one calls preventDefault, and then no tab opens at all), so this is the
	// same claim said the other way round
	await expect(panel(page)).toHaveCount(0);
});
