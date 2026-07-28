// The header bar's two shapes, and the recents the switcher lists.
//
// PLAYWRIGHT IS THE ONLY LANE THAT CAN SEE ANY OF THIS. TypeScript 7 cannot parse
// a `.svelte` file, so zero component files enter the tsc program, and every
// vitest suite is `.ts` and imports no component. The layout picker below `sm` is
// a different control from the one above it, and nothing but a browser at a
// stated width can tell which one shipped.
//
// Offline throughout: the demo wall is fixtures compiled into the wasm engine.
import { expect, test } from "@playwright/test";

const WALL = "/?actor=demo";

/** Five recents, most recent first, planted before the app boots. */
const RECENTS = JSON.stringify(
  ["Science", "Art", "Blacksky", "Books", "Cats", "Sixth", "Seventh"].map((name) => ({
    uri: `at://did:plc:x/app.bsky.feed.generator/${name.toLowerCase()}`,
    name,
    avatar: null,
    creator: "alice.test",
    description: "",
    likeCount: 0,
  })),
);

test.describe("the layout picker is a dropdown on a phone", () => {
  test.use({ viewport: { width: 375, height: 780 } });

  test.beforeEach(async ({ page }) => {
    await page.goto(WALL);
    await page.locator("#wall article").first().waitFor();
  });

  test("the slider is not rendered, and the trigger is", async ({ page }) => {
    // both shapes are in the markup with the other display:none, which takes it
    // out of the tab order and the accessibility tree alike
    await expect(page.getByRole("group", { name: "Wall layout" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: /Wall layout, currently/ })).toBeVisible();
  });

  test("every control on the bar is a 44px target and the row does not overflow", async ({
    page,
  }) => {
    // this is what the dropdown bought: three segments side by side had pushed
    // two of their own targets under 44px to make room for a fourth control
    const boxes = await page
      .locator("header button:visible")
      .evaluateAll((els) => els.map((el) => Math.round(el.getBoundingClientRect().height)));
    expect(boxes.length).toBeGreaterThanOrEqual(4);
    expect(Math.min(...boxes)).toBeGreaterThanOrEqual(44);

    const doc = await page.evaluate(() => ({
      scroll: document.documentElement.scrollWidth,
      client: document.documentElement.clientWidth,
    }));
    expect(doc.scroll).toBeLessThanOrEqual(doc.client + 1);
  });

  test("choosing a layout closes the popover, renames the trigger and is kept", async ({ page }) => {
    const trigger = page.getByRole("button", { name: /Wall layout, currently/ });
    await trigger.click();

    const listbox = page.getByRole("listbox", { name: "Wall layout" });
    await expect(listbox).toBeVisible();
    await expect(listbox.getByRole("option")).toHaveCount(3);
    // focus lands on an option, so a keyboard reader is already in the list
    await expect(listbox.getByRole("option", { selected: true })).toBeFocused();

    await listbox.getByRole("option", { name: /Masonry/ }).click();

    await expect(listbox).toHaveCount(0);
    await expect(page.getByRole("button", { name: /currently Masonry/ })).toBeVisible();
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("mason:layout")))
      .toBe("masonry");
  });

  test("escape shuts the popover and hands focus back to the trigger", async ({ page }) => {
    const trigger = page.getByRole("button", { name: /Wall layout, currently/ });
    await trigger.click();
    await expect(page.getByRole("listbox", { name: "Wall layout" })).toBeVisible();

    await page.keyboard.press("Escape");

    await expect(page.getByRole("listbox", { name: "Wall layout" })).toHaveCount(0);
    await expect(trigger).toBeFocused();
  });
});

test("the bar reads left to right: layout, switcher, refresh, settings", async ({ page }) => {
  await page.goto(WALL);
  await page.locator("#wall article").first().waitFor();

  // the order is the point, so it is read off the rendered geometry rather than
  // off the markup: a control moved in the source but floated elsewhere by CSS
  // would still pass a source-order check
  const order = await page.evaluate(() => {
    const header = document.querySelector("header");
    const found: [number, string][] = [];
    for (const el of header?.querySelectorAll("fieldset,[aria-haspopup],button") ?? []) {
      const box = el.getBoundingClientRect();
      if (box.width === 0) continue;
      if (el.tagName === "BUTTON" && el.closest("[aria-haspopup]") && !el.getAttribute("aria-haspopup"))
        continue;
      const name =
        el.querySelector("legend")?.textContent ??
        el.getAttribute("aria-label") ??
        el.textContent?.trim().slice(0, 20) ??
        "";
      found.push([Math.round(box.left), name]);
    }
    return found.sort((a, b) => a[0] - b[0]).map(([, name]) => name);
  });

  expect(order).toHaveLength(4);
  expect(order[0]).toMatch(/Wall layout/);
  expect(order[1]).toMatch(/Switch wall/);
  expect(order[2]).toMatch(/lay this wall again/i);
  expect(order[3]).toBe("Settings");
});

test("the layout picker is still a slider on a desktop", async ({ page }) => {
  await page.goto(WALL);
  await page.locator("#wall article").first().waitFor();

  await expect(page.getByRole("group", { name: "Wall layout" })).toBeVisible();
  await expect(page.getByRole("button", { name: /Wall layout, currently/ })).toHaveCount(0);
});

/** A marker on the window object. A full page load builds a new one and takes
 *  it with it, which is how "the router took this click and the page never
 *  reloaded" is asserted; `feed-picker.test.ts` marks its page the same way. */
async function markPage(page: import("@playwright/test").Page): Promise<void> {
  await page.evaluate(() => {
    (window as unknown as Record<string, boolean>).masonNeverLeft = true;
  });
}

/** Whether this is still the page the switcher was opened over. */
async function neverLeft(page: import("@playwright/test").Page): Promise<boolean> {
  return await page.evaluate(
    () => (window as unknown as Record<string, boolean>).masonNeverLeft === true,
  );
}

/** The panel's scrim: the half that dims the wall and swallows every click.
 *  Told apart from the panel's other controls by its name, and asserted
 *  separately from the dialog because it is the half a reader feels. */
function scrim(page: import("@playwright/test").Page) {
  return page.getByRole("button", { name: "Close switch panel" });
}

test.describe("the switcher lists recent feeds", () => {
  test.beforeEach(async ({ context, page }) => {
    // whatever leaves the wasm worker when a feed wall is asked for goes
    // nowhere: these cases are about the panel, not about the wall behind it,
    // and none of them reads a brick
    await context.route(/public\.api\.bsky\.app/, (route) =>
      route.fulfill({ json: { feed: [] }, headers: { "access-control-allow-origin": "*" } }),
    );
    await page.addInitScript((value) => localStorage.setItem("mason:feeds", value), RECENTS);
    await page.goto(WALL);
    await page.locator("#wall article").first().waitFor();
  });

  test("shows the five most recent, in order, as real links", async ({ page }) => {
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    await expect(panel).toBeVisible();

    const links = panel.locator('a[href^="/?feed="]');
    // five and not the twelve the picker keeps: this panel opens upward from a
    // fixed bar, and every row it adds pushes the "pick a feed" door off screen
    await expect(links).toHaveCount(5);
    await expect(links).toHaveText(["Science", "Art", "Blacksky", "Books", "Cats"]);

    // a real href, so a middle click or a cmd click opens a wall in a new tab
    await expect(links.first()).toHaveAttribute(
      "href",
      "/?feed=at%3A%2F%2Fdid%3Aplc%3Ax%2Fapp.bsky.feed.generator%2Fscience",
    );
  });

  test("the panel stays on screen, wherever the switcher sits on the bar", async ({ page }) => {
    // a regression this suite did not have when it was needed: the panel used to
    // hang off the trigger's right edge, which only worked while the switcher was
    // the right-most control. Reordering the bar put a 320px panel off the left
    // edge of a 375px screen, clipping its own input.
    await page.setViewportSize({ width: 375, height: 780 });
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const dialog = page.getByRole("dialog", { name: "Switch wall" });
    await expect(dialog).toBeVisible();

    const box = await dialog.boundingBox();
    expect(box).not.toBeNull();
    expect(box?.x ?? -1).toBeGreaterThanOrEqual(0);
    expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(375 + 1);
    expect(box?.y ?? -1).toBeGreaterThanOrEqual(0);

    // and nothing inside it is clipped either
    const label = await dialog.locator('label[for="switch-handle"]').boundingBox();
    expect(label?.x ?? -1).toBeGreaterThanOrEqual(0);
  });

  // The case above asserts the links' count, order and href and never clicks
  // one, which is exactly how this survived: the panel's openness is local
  // component state, and a client-side navigation does not touch it. Taking a
  // recent feed left the dialog AND its full-viewport scrim mounted over the
  // wall that had just laid, dimming it and swallowing every click, with a
  // screen-reader reader still inside a dialog called "Switch wall" over a wall
  // that had already switched.
  test("taking a recent feed lays it and takes the panel down with it", async ({ page }) => {
    await markPage(page);
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    await expect(panel).toBeVisible();

    await panel.locator('a[href^="/?feed="]').first().click();

    await page.waitForURL(/\?feed=at%3A%2F%2F/);
    await expect(page.getByRole("dialog", { name: "Switch wall" })).toHaveCount(0);
    await expect(scrim(page)).toHaveCount(0);
    // and it is still a real link the router took, not a page the browser
    // reloaded: closing the panel unmounts the anchor mid-click, so this is the
    // half that would break if the close raced the navigation
    expect(await neverLeft(page)).toBe(true);
  });

  // The other half of the same rule, and the reason the close is not
  // unconditional: a modified click opens the feed somewhere else and leaves
  // this wall exactly where it was. ControlOrMeta rather than Meta, because CI
  // is linux, where the new-tab chord is control.
  test("a modified click opens the feed elsewhere and leaves the switcher up", async ({ page }) => {
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    // the second row, so the recents write below is a reorder rather than a
    // no-op: "Science" is already at the front
    const link = panel.locator('a[href^="/?feed="]').nth(1);
    await expect(link).toHaveText("Art");

    const opened = page.context().waitForEvent("page");
    await link.click({ modifiers: ["ControlOrMeta"] });
    const tab = await opened;
    await tab.close();

    // the wall behind has not moved, so neither has the panel: closing here
    // would shut the switcher on somebody who just sent a feed to a background
    // tab and is reaching for a second
    await expect(panel).toBeVisible();
    expect(page.url()).toContain("actor=demo");
    // and the feed is remembered either way, because a feed opened in a new tab
    // is still a feed this reader opened
    const stored = await page.evaluate(() => localStorage.getItem("mason:feeds"));
    expect((JSON.parse(stored ?? "[]") as { name: string }[])[0]?.name).toBe("Art");
  });

  // The activation neither of the cases above can reach, and the one the
  // recents write missed for exactly that reason: a browser dispatches NO click
  // event for a non-primary button, so a middle click arrives as auxclick and
  // nothing else. ControlOrMeta above is still a PRIMARY click, so it dispatches
  // one and always did remember the feed; this reader opened wall after wall in
  // background tabs and built no recents list at all.
  test("a middle click sends the feed to a background tab and still remembers it", async ({
    page,
  }) => {
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    // the second row again, so the write is a reorder rather than a no-op
    const link = panel.locator('a[href^="/?feed="]').nth(1);
    await expect(link).toHaveText("Art");

    const opened = page.context().waitForEvent("page");
    await link.click({ button: "middle" });
    const tab = await opened;
    // the tab really is the feed, not a copy of this wall. `commit` because the
    // wall behind that URL is not what this case is about, and waiting for it to
    // lay would be waiting on the wasm worker in a second page for nothing
    await tab.waitForURL(/feed=at%3A%2F%2Fdid%3Aplc%3Ax%2Fapp\.bsky\.feed\.generator%2Fart/, {
      waitUntil: "commit",
    });
    await tab.close();

    // the load-bearing half: `mason:feeds` was written from a click that never
    // was one
    const stored = await page.evaluate(() => localStorage.getItem("mason:feeds"));
    expect((JSON.parse(stored ?? "[]") as { name: string }[])[0]?.name).toBe("Art");

    // and the panel is still up, for the same reason a modified click leaves it
    // up: the wall behind it has not moved
    await expect(panel).toBeVisible();
    expect(page.url()).toContain("actor=demo");
  });

  test("keeps the door to the picker on screen beneath them", async ({ page }) => {
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    const door = panel.getByRole("button", { name: /pick a feed/i });

    const box = await door.boundingBox();
    expect(box).not.toBeNull();
    expect(box?.y ?? -1).toBeGreaterThanOrEqual(0);
  });
});

// The panel's other link, which had the same defect for the same reason and
// gets the same treatment. It renders on every wall but the demo one, so the
// wall behind this case is somebody else's; nothing is asked of it, and the
// header renders as soon as the URL names a wall. The service worker is out of
// the way and /api/feed answered here, so the case stays offline and instant.
test.describe("the panel's way to the demo wall", () => {
  test.use({ serviceWorkers: "block" });

  test.beforeEach(async ({ context, page }) => {
    await context.route("**/api/feed*", (route) =>
      route.fulfill({ json: { items: [], cursor: null, warming: false } }),
    );
    await context.route(/public\.api\.bsky\.app/, (route) =>
      route.fulfill({ json: {}, headers: { "access-control-allow-origin": "*" } }),
    );
    await page.goto("/?actor=alice.test");
    await expect(page.getByRole("button", { name: /Switch wall/ })).toBeVisible();
  });

  test("wandering to the demo wall takes the panel down with it", async ({ page }) => {
    await markPage(page);
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    await expect(panel).toBeVisible();

    await panel.getByRole("link", { name: /demo wall/ }).click();

    await page.waitForURL(/actor=demo/);
    await expect(page.getByRole("dialog", { name: "Switch wall" })).toHaveCount(0);
    await expect(scrim(page)).toHaveCount(0);
    expect(await neverLeft(page)).toBe(true);
  });
});
