// `clientUrl`, which is the one thing standing between a brick's link and the
// app a reader actually uses. It had no tests at all until a client arrived that
// spells its routes differently, so these cover the rules it always had as well
// as the one it just gained.
import { describe, expect, it } from "vitest";
import { CLIENTS, clientName, clientUrl } from "./client.svelte";

const POST = "https://bsky.app/profile/alice.test/post/3l6oveex3ii2l";

describe("clientUrl", () => {
  it("leaves a link alone when the reader is on bluesky", () => {
    expect(clientUrl(POST, "bsky.app")).toBe(POST);
  });

  it("swaps the host for a client that mirrors bsky.app's routes", () => {
    expect(clientUrl(POST, "mu.social")).toBe(
      "https://mu.social/profile/alice.test/post/3l6oveex3ii2l",
    );
    expect(clientUrl(POST, "witchsky.app")).toBe(
      "https://witchsky.app/profile/alice.test/post/3l6oveex3ii2l",
    );
  });

  it("rewrites the path too for a client that does not", () => {
    // twinkl serves a profile at /@handle, so a host swap alone is a 404
    expect(clientUrl(POST, "twinkl.social")).toBe(
      "https://twinkl.social/@alice.test/post/3l6oveex3ii2l",
    );
  });

  it("rewrites a bare profile link, not only a post", () => {
    expect(clientUrl("https://bsky.app/profile/alice.test", "twinkl.social")).toBe(
      "https://twinkl.social/@alice.test",
    );
  });

  it("carries a did through as the handle segment", () => {
    const did = "did:plc:z72i7hdynmk6r22z27h6tvur";
    expect(clientUrl(`https://bsky.app/profile/${did}/post/abc`, "twinkl.social")).toBe(
      `https://twinkl.social/@${did}/post/abc`,
    );
  });

  it("passes a link that is not a bsky.app link straight through", () => {
    // a blog or a stream is not a Bluesky post and no client can show it
    for (const url of ["https://example.com/post/1", "https://stream.place/alice"]) {
      expect(clientUrl(url, "twinkl.social")).toBe(url);
      expect(clientUrl(url, "mu.social")).toBe(url);
    }
  });

  it("drops a scheme that must never reach an href", () => {
    // the negative space: only http(s) may be handed to an <a>
    for (const url of [
      "javascript:alert(1)",
      "data:text/html,<script>alert(1)</script>",
      "vbscript:msgbox",
      "file:///etc/passwd",
    ]) {
      expect(clientUrl(url, "bsky.app")).toBe("");
      expect(clientUrl(url, "twinkl.social")).toBe("");
    }
  });

  it("drops a string that is not a url at all", () => {
    for (const url of ["", "not a url", "//bsky.app/profile/alice"]) {
      expect(clientUrl(url, "twinkl.social")).toBe("");
    }
  });

  it("does not rewrite a lookalike host", () => {
    // evilbsky.app is not bsky.app, and neither is a subdomain of it
    for (const url of [
      "https://evilbsky.app/profile/alice.test/post/1",
      "https://bsky.app.example.com/profile/alice.test/post/1",
    ]) {
      expect(clientUrl(url, "twinkl.social")).toBe(url);
    }
  });

  it("has a spelling decided for every client, so a new one cannot ship unchecked", () => {
    // each id resolves to a rewrite rather than falling through to a default
    for (const c of CLIENTS) {
      const out = clientUrl(POST, c.host);
      expect(out).toMatch(/^https:\/\//);
      expect(out).toContain("3l6oveex3ii2l");
      // and none of them leaves the bsky.app host behind
      if (c.host !== "bsky.app") expect(new URL(out).hostname).toBe(c.host);
    }
  });
});

// The label the reader's "open in ..." control wears. It is read off the
// finished url rather than off the setting, and these are the cases where those
// two answers differ.
describe("clientName", () => {
  it("names the client a rewritten link is about to land in", () => {
    for (const c of CLIENTS) {
      expect(clientName(clientUrl(POST, c.host))).toBe(c.label);
    }
  });

  it("refuses to name a client for a link that is not going to one", () => {
    // clientUrl passes a non-bsky link through untouched whatever the setting
    // is, so a stream.place page opens at stream.place even on Twinkl. Naming
    // the setting here would promise somewhere the link never goes.
    const stream = "https://stream.place/alice.test/live";
    expect(clientUrl(stream, "twinkl.social")).toBe(stream);
    expect(clientName(stream)).toBeNull();
  });

  it("has nothing to say about a string that is not a url", () => {
    // what clientUrl returns when it rejects one, so this is the pair that
    // actually reaches the reader rather than a hypothetical
    for (const url of ["", "not a url", "javascript:alert(1)"]) {
      expect(clientName(url)).toBeNull();
    }
  });
});
