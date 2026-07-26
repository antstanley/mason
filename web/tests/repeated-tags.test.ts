import { expect, test, type Locator, type Page } from "@playwright/test";
import type { BlogBrick, FeedResponse } from "$lib/types";

// A blog can carry the same tag twice, and for a while that took the wall down.
//
// Nothing dedupes a document's tags on the way to the client: mortar's
// standardsite source passes `tags: doc.tags` straight through from the record.
// Both tag loops used to key on the tag itself, and Svelte answers a repeated
// key by THROWING each_key_duplicate as it renders. On the card that throw
// happened mid-wall and left zero bricks on screen; in the reader it happened
// inside the click, so the reader simply never opened.
//
// This is the tier that can see it. `just check` never renders a component:
// tsc cannot parse .svelte, and vitest runs in node with no DOM, so playwright
// is the only lane in the repo that mounts one at all.
//
// The wall here is this file's own, since the demo fixtures carry no repeated
// tag. The service worker is what answers /api/feed on a static host, so it is
// blocked and the route is fulfilled from here instead.
test.use({ serviceWorkers: "block" });

const author = {
	did: "did:plc:tagprobe",
	handle: "waller.test",
	displayName: "A Waller",
	avatar: null,
};

/** One blog, tagged the way a `site.standard.document` record is allowed to be:
 *  "mortar" repeats, and it repeats INSIDE the first four, which is the four
 *  the card slices to. One fixture therefore covers both loops. */
const repeated: BlogBrick = {
	kind: "blog",
	id: "at://did:plc:tagprobe/site.standard.document/repeat",
	url: "https://example.com/blog/repeated-tags",
	author,
	title: "one brick, tagged twice",
	description: "the same tag twice over, which is a repeated key and not a repeated word",
	coverImage: null,
	publication: { name: "The Daily Brick", url: "https://example.com", icon: null },
	tags: ["mortar", "brick", "mortar", "grout", "kiln"],
	publishedAt: "2026-07-01T09:00:00Z",
};

const feed: FeedResponse = { items: [repeated], cursor: null };

/** The tag chips inside a card or inside the reader, found by what they read
 *  as rather than by their classes. */
function tagChips(scope: Locator): Locator {
	return scope.locator("span").filter({ hasText: /^#\S/ });
}

/** Serve the one-brick wall and open it, collecting anything the page throws.
 *  A duplicate key surfaces as a pageerror, both from the initial render and
 *  from the click, so an empty list is part of what is being asserted. */
async function layTheWall(page: Page): Promise<string[]> {
	const thrown: string[] = [];
	page.on("pageerror", (error) => thrown.push(error.message));
	await page.route("**/api/feed*", (route) =>
		route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify(feed),
		}),
	);
	await page.goto("/?actor=waller.test");
	return thrown;
}

test("a blog whose first four tags repeat still lays a brick", async ({ page }) => {
	const thrown = await layTheWall(page);

	// the whole wall, not just this brick: the throw was mid-render, so before
	// the fix this count was 0 and every other brick went with it
	const card = page.locator("#wall article");
	await expect(card).toHaveCount(1, { timeout: 30_000 });
	// the card's own slice: four of the five, with the repeat among them
	await expect(tagChips(card)).toHaveCount(4);
	expect(thrown).toEqual([]);
});

test("and the reader opens on it, with every tag rather than four", async ({ page }) => {
	const thrown = await layTheWall(page);

	const card = page.locator("#wall article");
	await expect(card).toHaveCount(1, { timeout: 30_000 });
	await card.locator("a").first().click();

	const dialog = page.locator('[role="dialog"]');
	await expect(dialog).toBeVisible();
	// all five, repeat included: the reader's whole job on this field is to show
	// what the card had to drop
	await expect(tagChips(dialog)).toHaveCount(5);
	expect(thrown).toEqual([]);
});
