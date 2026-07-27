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

test.describe("the switcher lists recent feeds", () => {
  test.beforeEach(async ({ page }) => {
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

  test("keeps the door to the picker on screen beneath them", async ({ page }) => {
    await page.getByRole("button", { name: /Switch wall/ }).click();
    const panel = page.getByRole("dialog", { name: "Switch wall" });
    const door = panel.getByRole("button", { name: /pick a feed/i });

    const box = await door.boundingBox();
    expect(box).not.toBeNull();
    expect(box?.y ?? -1).toBeGreaterThanOrEqual(0);
  });
});
