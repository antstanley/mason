// The scheme guard, on its own. It used to exist only inside `clientUrl`, which
// meant the only way to vet a link was to also rewrite it, and the reader's
// external embed took that rewrite along with the guard it actually wanted.
import { describe, expect, it } from "vitest";
import { httpUrl } from "./url";

describe("httpUrl", () => {
  it.each([
    ["a plain https link", "https://example.com/story"],
    ["an http one", "http://example.com/story"],
    ["a link card's address, query and all", "https://example.com/a/b?utm=1#top"],
    ["a bsky.app link, which it has no opinion about", "https://bsky.app/starter-pack/alice/3abc"],
  ])("hands back %s untouched", (_name, url) => {
    // the same string, byte for byte, and not `new URL(url).toString()`: an
    // embed prints its own address under the headline, so the href and the
    // words have to agree
    expect(httpUrl(url)).toBe(url);
  });

  it.each([
    ["javascript", "javascript:alert(1)"],
    ["data", "data:text/html,<script>alert(1)</script>"],
    ["vbscript", "vbscript:msgbox"],
    ["file", "file:///etc/passwd"],
    ["mailto", "mailto:alice@example.com"],
  ])("drops a %s: url, which must never reach an href", (_name, url) => {
    expect(httpUrl(url)).toBe("");
  });

  it.each([
    ["an empty field", ""],
    ["prose", "not a url"],
    ["a scheme-relative address", "//bsky.app/profile/alice"],
    ["a bare host", "example.com/story"],
  ])("drops %s, which is not a url at all", (_name, url) => {
    expect(httpUrl(url)).toBe("");
  });

  it("is not fooled by leading whitespace around a bad scheme", () => {
    // the URL parser strips leading control characters and whitespace before it
    // reads the scheme, so " javascript:..." parses as javascript: and has to
    // be dropped on the protocol rather than on how the string looks
    expect(httpUrl(" \n\tjavascript:alert(1)")).toBe("");
  });

  it("keeps a padded http url exactly as it arrived", () => {
    // it parses, so it is a link, and the raw string is what the card shows
    const padded = "  https://example.com/story";
    expect(httpUrl(padded)).toBe(padded);
  });
});
