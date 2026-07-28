// A link's Open Graph picture, and which picture wins when a post has both.
//
// PLAYWRIGHT IS THE ONLY LANE THAT CAN SEE THIS. TypeScript 7 cannot parse a
// `.svelte` file, so zero component files enter the tsc program, and every
// vitest suite is `.ts` and imports no component.
//
// The demo wall cannot exercise it: `fixtures.rs` sets `external: None` on every
// post it compiles in, so there is no link with a picture anywhere on it. These
// cases block the service worker and answer `/api/feed` themselves, which is the
// only way to put a crafted brick on the wall. Still offline: the images are
// inline data URIs and nothing leaves the machine.
import { expect, test } from "@playwright/test";
import type { FeedResponse } from "../src/lib/types";

/** A picture with no bytes on the wire. */
const svg = (bg: string, text: string) =>
  `data:image/svg+xml;base64,${Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="628"><rect width="1200" height="628" fill="${bg}"/><text x="600" y="330" font-size="90" fill="#fff" text-anchor="middle" font-family="sans-serif">${text}</text></svg>`,
  ).toString("base64")}`;

const OG = svg("#3b82f6", "OG");
const OWN = svg("#16a34a", "OWN");

const author = { did: "did:plc:a", handle: "alice.test", displayName: "Alice", avatar: null };
const external = {
  uri: "https://www.example.com/some/very/long/path?utm=1",
  title: "A headline from the linked page",
  description: "The og:description the page advertises, which runs on a while before it is clamped.",
  thumb: OG,
};

const post = (id: string, text: string, over: Record<string, unknown>) => ({
  kind: "post" as const,
  id,
  url: `https://bsky.app/profile/alice.test/post/${id}`,
  author,
  text,
  createdAt: "1970-01-01T00:00:00.000Z",
  likeCount: 0,
  repostCount: 0,
  images: [],
  external: null,
  ...over,
});

/** A brick carrying a link mortar would never have let out of `sources/`.
 *
 *  `external_embed` in sources/bluesky.rs drops the WHOLE embed when its uri is
 *  not http(s), so this shape reaches the client only from a mortar the SPA was
 *  not built with, which is a real state: in server mode the SPA calls a native
 *  binary that deploys on its own clock. The client's own vetting is the second
 *  line, and this is the only lane in the repo that can see it. No picture, so
 *  the one-preview count below still holds. */
const unvettable = {
  ...external,
  uri: "javascript:document.title='PWNED'",
  thumb: null,
};

/** A link card whose address happens to be a bsky.app one, and a route no
 *  client but bsky.app serves.
 *
 *  `external.uri` is a stranger's link, not a url mason built, so it is vetted
 *  and never rewritten: only `/profile/` and `/post/<rkey>` were ever checked
 *  against the other clients. No picture, so the one-preview count below still
 *  holds. */
const strangersBskyLink = {
  ...external,
  uri: "https://bsky.app/starter-pack/alice.test/3abc",
  title: "A starter pack somebody linked to",
  thumb: null,
};

const WALL = {
  items: [
    post("1", "a link with a picture on the other end", { external }),
    post("2", "its own picture wins", {
      images: [{ src: OWN, alt: "the post's own picture", aspectRatio: { width: 1200, height: 628 } }],
      external,
    }),
    post("3", "a link with no picture", { external: { ...external, thumb: null } }),
    post("4", "a link nobody should be handed", { external: unvettable }),
    post("5", "a link card pointing back at bluesky", { external: strangersBskyLink }),
  ],
  cursor: null,
  warming: false,
} as unknown as FeedResponse;

// the wasm worker answers /api/feed from compiled-in fixtures, so it has to be
// out of the way before a crafted wall can be served
test.use({ serviceWorkers: "block" });

test.beforeEach(async ({ context, page }) => {
  await context.route("**/api/feed*", (route) =>
    route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(WALL) }),
  );
  await page.goto("/?actor=demo");
  await page.locator("#wall article").first().waitFor();
});

test("a link's picture stands in when the post attached none", async ({ page }) => {
  const brick = page.locator("#wall article").nth(0);
  const preview = brick.locator("figure");
  await expect(preview).toBeVisible();

  // the overlay carries all three, over the foot of the picture
  const caption = preview.locator("figcaption");
  await expect(caption).toBeVisible();
  // the host rather than the whole address, and `www.` stripped. Read from the
  // DOM rather than as rendered text, which the uppercase style would fold.
  await expect
    .poll(() => caption.locator("p").first().evaluate((el) => el.textContent?.trim()))
    .toBe("example.com");
  await expect(caption).toContainText("A headline from the linked page");
  await expect(caption).toContainText("og:description");

  // and the overlay really is over the picture, not stacked under it
  const image = await preview.locator("img").boundingBox();
  const overlay = await caption.boundingBox();
  expect(overlay?.y ?? 0).toBeGreaterThan(image?.y ?? 0);
  expect((overlay?.y ?? 0) + (overlay?.height ?? 0)).toBeLessThanOrEqual(
    (image?.y ?? 0) + (image?.height ?? 0) + 1,
  );

  // the text block below would say the same thing twice
  await expect(brick.locator("div.rounded-xl p.truncate")).toHaveCount(0);
});

test("a picture the post attached wins over the link's", async ({ page }) => {
  const brick = page.locator("#wall article").nth(1);

  // what somebody chose to show beats what a page happened to advertise
  await expect(brick.locator("figure")).toHaveCount(0);
  await expect(brick.locator("img").first()).toHaveAttribute("alt", "the post's own picture");
  // and the link is still named, in the text block it always had
  await expect(brick.locator("div.rounded-xl p.truncate")).toHaveCount(1);
});

test("a link with no picture falls back to the words", async ({ page }) => {
  const brick = page.locator("#wall article").nth(2);

  await expect(brick.locator("figure")).toHaveCount(0);
  await expect(brick.locator("div.rounded-xl p.truncate")).toHaveText(
    "A headline from the linked page",
  );
});

test("exactly one brick on this wall shows a link preview", async ({ page }) => {
  // the guard against the precedence rule inverting: if the attached image ever
  // stopped winning, this would be two
  await expect(page.locator("#wall figure")).toHaveCount(1);
});

test("a link that is not http(s) reaches no href, on the wall or in the reader", async ({
  page,
}) => {
  const brick = page.locator("#wall article").nth(3);
  // the card still says what the link said; only the way to it is gone
  await expect(brick.locator("div.rounded-xl p.truncate")).toHaveText(
    "A headline from the linked page",
  );
  await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);

  // the reader is where external.uri reaches an anchor at all, so it is the
  // place this has to hold
  await brick.locator("a").first().click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText("A headline from the linked page");
  await expect(page.locator('a[href^="javascript:"]')).toHaveCount(0);

  // an anchor with no href is not a link: the accessibility tree reports none,
  // it takes no click and no Tab. href="" would still be a link, and one that
  // reopens mason, which is why the empty answer drops the attribute instead.
  await expect(
    dialog.getByRole("link", { name: /A headline from the linked page/ }),
  ).toHaveCount(0);
  await expect(dialog.locator('a[href=""]')).toHaveCount(0);

  // and the post's own way out is untouched, so this is the embed being vetted
  // rather than the reader losing its links
  await expect(dialog.getByRole("link", { name: "open in Bluesky" })).toBeVisible();
});

// Every case above runs on the default client, which is bsky.app, and on that
// setting `clientUrl` rewrites nothing at all: that is exactly why the reader
// reusing it on a link card went unnoticed. This describe is the one that sets
// a different client, which is what the setting is for.
test.describe("a link card carries a stranger's address, whatever client is set", () => {
  test.beforeEach(async ({ page }) => {
    // planted before the app boots, which is the same shape settings writes and
    // the state module reads on construction
    await page.addInitScript(() => localStorage.setItem("mason:client", "twinkl.social"));
    await page.goto("/?actor=demo");
    await page.locator("#wall article").first().waitFor();
  });

  test("a bsky.app link card is not rewritten, and its href is the address it shows", async ({
    page,
  }) => {
    const brick = page.locator("#wall article").nth(4);
    await brick.locator("a").first().click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // the setting really is live, which is what makes this the embed being
    // exempted rather than the client picker being ignored: the post's OWN way
    // out is rewritten, host and profile spelling both
    const out = dialog.getByRole("link", { name: "open in Twinkl" });
    await expect(out).toBeVisible();
    await expect(out).toHaveAttribute("href", /^https:\/\/twinkl\.social\/@/);

    // and the link card goes where the link card says it goes
    const embed = dialog.getByRole("link", { name: /A starter pack somebody linked to/ });
    await expect(embed).toHaveAttribute(
      "href",
      "https://bsky.app/starter-pack/alice.test/3abc",
    );
    // the address under the headline is the same string as the href. A control
    // is named after where it lands, and /starter-pack/ is a route twinkl does
    // not serve, so the rewrite would have been a promise the link cannot keep
    await expect(embed).toContainText("https://bsky.app/starter-pack/alice.test/3abc");
    await expect(page.locator('a[href^="https://twinkl.social/starter-pack/"]')).toHaveCount(0);
  });
});

test("the reader shows the link's picture under the post's own", async ({ page }) => {
  // the card shows one or the other because it is small; the reader has room for
  // both, so a post that attached a picture AND linked somewhere shows each
  await page.locator("#wall article").nth(1).locator("a").first().click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();

  await expect(dialog.locator("figcaption").filter({ hasText: "example.com" })).toBeVisible();
  await expect(dialog.locator("img[alt=\"the post's own picture\"]")).toBeVisible();
});
