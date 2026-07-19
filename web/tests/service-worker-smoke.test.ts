import { expect, test } from "@playwright/test";

// End to end through the real engine: the page loads, the service worker takes
// control, /api/feed round-trips through the wasm mortar, and bricks render.
// Actor `demo` is the offline fixture wall compiled into the wasm, so this
// needs no network beyond the static site itself.
test("the demo wall round-trips /api/feed through the wasm service worker", async ({ page }) => {
	await page.goto("/?actor=demo");

	// interception only applies once the SW controls the page
	await page.waitForFunction(() => navigator.serviceWorker.controller != null, undefined, {
		timeout: 30_000,
	});

	// a raw round-trip: on a static host this path only answers if the service
	// worker intercepted it and the wasm engine laid a page
	const roundTrip = await page.evaluate(async () => {
		const res = await fetch("/api/feed?actor=demo");
		return { status: res.status, body: (await res.json()) as { items?: unknown[] } };
	});
	expect(roundTrip.status).toBe(200);
	expect(Array.isArray(roundTrip.body.items)).toBe(true);
	expect(roundTrip.body.items!.length).toBeGreaterThan(0);

	// and the app itself renders those bricks on the wall (warm can take up to
	// the 8s ceiling before the first screen commits)
	await expect(page.locator("#wall article").first()).toBeVisible({ timeout: 30_000 });
});
