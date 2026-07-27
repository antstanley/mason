// Which question one input is asking, and the one answer that navigates
// nowhere. `FeedPicker.svelte` renders every branch below and no lane in this
// repo typechecks or runs a component body, which is exactly why the decision
// lives in `feedref.ts` rather than in it: here it has a compiler and these
// cases, there it would have neither.
//
// The `feed` cases are the client half of a parser mortar owns
// (server/crates/mortar-core/src/sources/feedref.rs). They are paired
// deliberately: the accepted spellings are the ones that module accepts, and the
// rejected ones are the ones it rejects, because a value refused here is a feed
// the reader cannot open at all while a value refused there is only an error
// panel.
import { describe, expect, it } from "vitest";
import { askedFor } from "./feedref";

const WHATS_HOT = "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot";

describe("an empty box", () => {
  // An emptied input is the resting state and not a search that found nothing:
  // telling somebody who typed nothing that there are no feeds by that name is
  // an answer to a question they did not ask.
  it("asks for what the network ranks rather than for nothing", () => {
    expect(askedFor("")).toEqual({ kind: "browse" });
    expect(askedFor("   ")).toEqual({ kind: "browse" });
    expect(askedFor("\n\t ")).toEqual({ kind: "browse" });
  });
});

describe("a pasted reference", () => {
  // The spelling mason ultimately queries with, and the one in the address bar
  // when somebody is looking at a feed. Both are what people actually have in
  // their clipboard.
  it("is the at-uri and the bsky.app link, in either authority spelling", () => {
    expect(askedFor(WHATS_HOT)).toEqual({ kind: "feed", ref: WHATS_HOT });

    const web = "at://did:web:feeds.example.com/app.bsky.feed.generator/3k2a.b_c~d";
    expect(askedFor(web)).toEqual({ kind: "feed", ref: web });

    const byHandle = "at://alice.bsky.social/app.bsky.feed.generator/whats-hot";
    expect(askedFor(byHandle)).toEqual({ kind: "feed", ref: byHandle });

    const link = "https://bsky.app/profile/alice.bsky.social/feed/whats-hot";
    expect(askedFor(link)).toEqual({ kind: "feed", ref: link });

    const byDid = "https://bsky.app/profile/did:plc:z72i7hdynmk6r22z27h6tvur/feed/whats-hot";
    expect(askedFor(byDid)).toEqual({ kind: "feed", ref: byDid });
  });

  // A clipboard carries the space and the newline around what was copied, and a
  // reference that came back `unparseable` for a trailing newline would be a
  // picker refusing a feed it can lay.
  it("survives the whitespace a clipboard puts around it", () => {
    expect(askedFor(`  ${WHATS_HOT}\n`)).toEqual({ kind: "feed", ref: WHATS_HOT });
  });

  // The collection is the whole check on an AT-URI, and `feed` is the whole
  // check on a bsky.app path. A post, a list or a profile points at something
  // mortar cannot page, so the picker says so in place instead of laying a wall
  // that fails.
  it("is not a post, a list, a profile or a lists link", () => {
    expect(askedFor("at://did:plc:aa/app.bsky.feed.post/3k2a")).toEqual({ kind: "unparseable" });
    expect(askedFor("at://did:plc:aa/app.bsky.graph.list/3k2a")).toEqual({ kind: "unparseable" });
    expect(askedFor("at://did:plc:aa/app.bsky.feed.generatorx/3k2a")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("https://bsky.app/profile/alice.test/post/3k2a")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("https://bsky.app/profile/alice.test/lists/3k2a")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("https://bsky.app/profile/alice.test")).toEqual({ kind: "unparseable" });
  });

  // The rule is the exact origin, not a suffix: a host that merely ends in
  // bsky.app, or one that continues past it, is somebody else's host, and a
  // scheme-relative URL names no origin at all.
  it("is not a lookalike host or a scheme-relative path", () => {
    expect(askedFor("https://evilbsky.app/profile/alice.test/feed/x")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("https://bsky.app.example.com/profile/alice.test/feed/x")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("http://bsky.app/profile/alice.test/feed/x")).toEqual({ kind: "unparseable" });
    expect(askedFor("//bsky.app/profile/alice.test/feed/x")).toEqual({ kind: "unparseable" });
  });

  // Three segments exactly. A query string or a deeper path would otherwise ride
  // along into the `?feed=` value, and an empty rkey names no record at all.
  it("is not a reference with anything extra on the end", () => {
    expect(askedFor(`${WHATS_HOT}?utm_source=x`)).toEqual({ kind: "unparseable" });
    expect(askedFor(`${WHATS_HOT}/`)).toEqual({ kind: "unparseable" });
    expect(askedFor(`${WHATS_HOT}#frag`)).toEqual({ kind: "unparseable" });
    expect(askedFor("at://did:plc:aa/app.bsky.feed.generator/")).toEqual({ kind: "unparseable" });
    expect(askedFor("at://did:plc:aa/app.bsky.feed.generator")).toEqual({ kind: "unparseable" });
    expect(askedFor("https://bsky.app/profile//feed/whats-hot")).toEqual({ kind: "unparseable" });
  });

  // An authority is a DID mason can resolve or a handle, and nothing else. An
  // unfamiliar method carries colons, so it fails the handle set too and cannot
  // slip through as one.
  it("is not an authority mason cannot resolve", () => {
    expect(askedFor("at://did:evil:aa/app.bsky.feed.generator/3k2a")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("at://did:plc:/app.bsky.feed.generator/3k2a")).toEqual({ kind: "unparseable" });
  });

  // A scheme is what tells a paste from a phrase, so anything carrying one is
  // answered as a reference and refused as one. Nothing here reaches a search,
  // and nothing here navigates.
  it("is never a javascript: or data: value dressed as one", () => {
    expect(askedFor("javascript:alert(1)")).toEqual({ kind: "unparseable" });
    expect(askedFor("javascript://bsky.app/profile/alice.test/feed/x")).toEqual({
      kind: "unparseable",
    });
    expect(askedFor("data:text/html,<script>")).toEqual({ kind: "unparseable" });
  });

  // atproto bounds every part a reference is built from, so a string past the
  // cap is not a reference: it is length aimed at whatever the value ends up
  // interpolated into.
  it("is not a string longer than any real reference", () => {
    const over = `at://did:plc:aa/app.bsky.feed.generator/${"a".repeat(1024)}`;
    expect(askedFor(over)).toEqual({ kind: "unparseable" });
  });
});

describe("a typed handle", () => {
  // The bridge between mason's two front doors: a handle in the picker means
  // "show me the feeds this person made" rather than "show me their wall".
  it("asks for that person's feeds, normalized as the handle box does", () => {
    expect(askedFor("alice.bsky.social")).toEqual({ kind: "creator", handle: "alice.bsky.social" });
    expect(askedFor("@Alice.Bsky.Social  ")).toEqual({
      kind: "creator",
      handle: "alice.bsky.social",
    });
    expect(askedFor("feeds.example.co.uk")).toEqual({
      kind: "creator",
      handle: "feeds.example.co.uk",
    });
  });
});

describe("anything else", () => {
  // A phrase is a search, including the phrases that look almost like the other
  // two: a bare word has no dot, a sentence has whitespace, and a numeric last
  // label is not a handle.
  it("is a search for feeds by name", () => {
    expect(askedFor("science")).toEqual({ kind: "search", term: "science" });
    expect(askedFor("  book club  ")).toEqual({ kind: "search", term: "book club" });
    expect(askedFor("cat pics 2026")).toEqual({ kind: "search", term: "cat pics 2026" });
    expect(askedFor("2026: a retrospective")).toEqual({
      kind: "search",
      term: "2026: a retrospective",
    });
    expect(askedFor("192.168.0.1")).toEqual({ kind: "search", term: "192.168.0.1" });
  });
});
