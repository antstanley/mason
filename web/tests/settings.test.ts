// The settings screen, the cog that opens it, and the client picker that moved
// into it.
//
// PLAYWRIGHT IS THE ONLY LANE THAT CAN SEE ANY OF THIS. TypeScript 7 cannot parse
// a `.svelte` file, so zero component files enter the tsc program, and every
// vitest suite is `.ts` and imports no component. `settings.svelte.ts` has its
// own unit tests for the history rules; what is here is the shell those rules
// drive, and a green `just check` says nothing about it.
//
// Offline throughout: the demo wall is fixtures compiled into the wasm engine.
import { expect, test } from "@playwright/test";

const WALL = "/?actor=demo";

/** The cog. Exact, because "Close settings" contains "settings" too. */
const cog = (page: import("@playwright/test").Page) =>
  page.getByRole("button", { name: "Settings", exact: true });

const panel = (page: import("@playwright/test").Page) =>
  page.getByRole("dialog", { name: "settings" });

test.describe("the settings screen", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(WALL);
    await page.locator("#wall article").first().waitFor();
  });

  test("the cog is the right-most control on the bar", async ({ page }) => {
    await expect(cog(page)).toBeVisible();

    // a 44px target, the rule every control on this bar keeps
    const box = await cog(page).boundingBox();
    expect(box).not.toBeNull();
    expect(Math.round(box?.width ?? 0)).toBeGreaterThanOrEqual(44);
    expect(Math.round(box?.height ?? 0)).toBeGreaterThanOrEqual(44);

    // right-most: nothing in the header starts further right
    const lefts = await page
      .locator("header button:visible, header a:visible")
      .evaluateAll((els) => els.map((el) => el.getBoundingClientRect().left));
    expect(Math.max(...lefts)).toBeCloseTo(box?.x ?? -1, 0);
  });

  test("the client picker is off the bar and inside settings", async ({ page }) => {
    // it moved because it is not a choice anybody makes while reading
    await expect(page.getByRole("button", { name: /Open posts in/ })).toHaveCount(0);

    await cog(page).click();

    await expect(panel(page)).toBeVisible();
    await expect(page.getByRole("button", { name: /Open posts in/ })).toBeVisible();
  });

  test("the client listbox still works inside the dialog", async ({ page }) => {
    await cog(page).click();
    await page.getByRole("button", { name: /Open posts in/ }).click();
    await expect(page.getByRole("listbox", { name: "Open posts in" })).toBeVisible();

    const options = page.getByRole("listbox", { name: "Open posts in" }).getByRole("option");
    await options.nth(1).click();

    await expect(page.getByRole("listbox", { name: "Open posts in" })).toHaveCount(0);
    // the choice is a local preference and it is kept
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("mason:client")))
      .not.toBeNull();
  });

  test("the combobox is wide, and opening it neither moves nor scrolls the panel", async ({
    page,
  }) => {
    await cog(page).click();
    const dialog = panel(page);
    await expect(dialog).toBeVisible();

    const trigger = dialog.getByRole("button", { name: /Open posts in/ });
    const width = (await trigger.boundingBox())?.width ?? 0;
    expect(Math.round(width)).toBeGreaterThanOrEqual(200);

    // let the entrance animation land before measuring: the panel enters from
    // scale(0.99), so a box read mid-animation is 444px wide where the settled
    // one is 448, and the difference reads as a shift that is not there
    const settled = () =>
      dialog.evaluate((el) => Promise.all(el.getAnimations().map((a) => a.finished)).then(() => {}));
    await settled();

    const box = async () => {
      const b = await dialog.boundingBox();
      return [Math.round(b?.x ?? 0), Math.round(b?.y ?? 0), Math.round(b?.width ?? 0), Math.round(b?.height ?? 0)];
    };
    const before = await box();

    await trigger.click();
    const listbox = page.getByRole("listbox", { name: "Open posts in" });
    await expect(listbox).toBeVisible();

    // the panel must not move, resize, or grow a scrollbar. A scroll container
    // clips and scrolls its absolutely positioned descendants, so making this
    // panel one trapped the popover inside it and put a scrollbar in the modal.
    expect(await box()).toEqual(before);
    await expect
      .poll(() =>
        page.evaluate(() => {
          const d = document.querySelector('[role="dialog"]');
          return d ? getComputedStyle(d).overflowY : "";
        }),
      )
      .toBe("visible");

    // and the list itself is wide and wholly on screen rather than clipped
    const list = await listbox.boundingBox();
    expect(Math.round(list?.width ?? 0)).toBeGreaterThanOrEqual(200);
    expect(list?.y ?? -1).toBeGreaterThanOrEqual(0);
  });

  test("every client mason knows is listed", async ({ page }) => {
    await cog(page).click();
    await panel(page).getByRole("button", { name: /Open posts in/ }).click();
    const options = page.getByRole("listbox", { name: "Open posts in" }).getByRole("option");
    await expect(options).toHaveCount(5);
    await expect(options).toContainText([
      "Bluesky",
      "Mu Social",
      "Blacksky",
      "Twinkl",
      "Witchsky",
    ]);
  });

  test("it holds the wall inert behind it, and mounts outside what it dims", async ({ page }) => {
    await cog(page).click();
    await expect(panel(page)).toBeVisible();

    // the pair that matters: inert covers descendants, so a screen nested in the
    // wrapper it dims would open unfocusable and invisible to assistive tech
    const shape = await page.evaluate(() => {
      const wrapper = document.querySelector("div.mx-auto.min-h-screen");
      const dialog = document.querySelector('[role="dialog"]');
      return {
        inert: wrapper?.hasAttribute("inert") ?? false,
        outside: !!dialog && !!wrapper && !wrapper.contains(dialog),
      };
    });
    expect(shape).toEqual({ inert: true, outside: true });

    // and the page behind does not scroll
    await expect
      .poll(() => page.evaluate(() => document.documentElement.style.overflow))
      .toBe("hidden");
  });

  test("the address bar never mentions it, and every way out restores the wall", async ({
    page,
  }) => {
    const wall = page.url();

    for (const shut of [
      async () => page.keyboard.press("Escape"),
      async () => page.goBack(),
      async () => page.getByRole("button", { name: "Close settings" }).last().click(),
    ]) {
      await cog(page).click();
      await expect(panel(page)).toBeVisible();
      // a screen is not a wall: the URL keeps naming the wall behind it
      expect(page.url()).toBe(wall);

      await shut();

      await expect(panel(page)).toHaveCount(0);
      expect(page.url()).toBe(wall);
      // the scroll lock is released whichever way it was shut, including the back
      // gesture, which never calls closeSettings at all
      await expect
        .poll(() => page.evaluate(() => document.documentElement.style.overflow))
        .not.toBe("hidden");
      // and focus goes back to the cog that opened it
      await expect(cog(page)).toBeFocused();
    }
  });

  test("the cog cannot be reached while the reader is up, so one overlay is all there is", async ({
    page,
  }) => {
    // The mutual-exclusion rule itself is unit-tested (settings.svelte.ts pushes
    // page state carrying only its own key, which drops the reader's), and it is
    // a guarantee for a future trigger rather than a live path: the bar lives
    // inside the wrapper every overlay makes inert, so the cog is not reachable
    // while one is open. That is the reachable truth, so it is what this asserts.
    // RefreshWall carries the same note for the same reason.
    await page.locator("#wall article a").first().click();
    await expect(page.getByRole("dialog").first()).toBeVisible();

    // inert is inherited, so asking the cog to take focus is the honest test:
    // an inert subtree refuses it
    const reachable = await page.evaluate(() => {
      const el = document.querySelector<HTMLElement>('[aria-label="Settings"]');
      el?.focus();
      return document.activeElement === el;
    });
    expect(reachable).toBe(false);
    await expect(page.locator('[role="dialog"]')).toHaveCount(1);
  });
});

test.describe("the settings screen on a phone", () => {
  test.use({ viewport: { width: 375, height: 780 } });

  test("the bar still fits, with the cog on it", async ({ page }) => {
    await page.goto(WALL);
    await page.locator("#wall article").first().waitFor();

    await expect(cog(page)).toBeVisible();
    const doc = await page.evaluate(() => ({
      scroll: document.documentElement.scrollWidth,
      client: document.documentElement.clientWidth,
    }));
    expect(doc.scroll).toBeLessThanOrEqual(doc.client + 1);
  });

  test("opens and closes the same way it does on a desktop", async ({ page }) => {
    await page.goto(WALL);
    await page.locator("#wall article").first().waitFor();

    await cog(page).click();
    await expect(panel(page)).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(panel(page)).toHaveCount(0);
    await expect(cog(page)).toBeFocused();
  });
});
