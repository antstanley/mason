// Which popular feeds can mason actually lay a wall from, logged out?
//
// `pnpm feeds:audit` (from web/). It pages the AppView's popular list to fifty,
// asks each feed for a logged-out `getFeed`, and prints which ones clear the
// bar: more than five posts. The output is the source of `UNLAYABLE_FEEDS` in
// src/lib/state/feeds.svelte.ts, which is a static list precisely so the picker
// costs one request rather than fifty-one to draw.
//
// It also re-tests every entry already on that list, found by searching for its
// name, because a denylist nobody re-derives only ever grows: an entry that
// starts working again is a feed mason hides for no reason. That half is what
// caught skygaze.io's "For You", which answers nineteen posts and was being
// hidden by a name-only rule meant for somebody else's.
//
// Node, not vitest: this talks to the live network, so it is a tool rather than
// a test. No lane runs it, and none should.

const APPVIEW = "https://public.api.bsky.app";
const UA = "mason-feed-audit (+https://github.com/antstanley/mason)";

/** More than this many posts, logged out, or the feed is not worth a card. A
 *  wall of three bricks reads as broken whether the feed is gated or just
 *  quiet, and the reader cannot tell the difference either. */
const BAR = 5;
/** Asked for per feed. Comfortably over the bar, so "more than five" is a
 *  question this can answer in one request. */
const LIMIT = 20;
/** Attempts per feed. A 502 can be a bad minute rather than a broken feed, and
 *  hiding a working feed is the more expensive mistake. */
const TRIES = 2;
/** How many popular feeds to test. */
const TOP = 50;

/** Every name currently denied in feeds.svelte.ts, with the publisher it is
 *  pinned to or null for a name-only rule. Kept here rather than imported: this
 *  is a .mjs run by node, the list lives in a `.svelte.ts` that only vite can
 *  load, and a stale copy here shows up as a row saying LAYS that the real list
 *  does not have. */
const DENIED = [
  ["popular with friends", null],
  ["mutuals", null],
  ["quiet posters", null],
  ["mentions", null],
  ["for you", "spacecowboy17.bsky.social"],
  ["for you", "flicknow.xyz"],
  ["only posts", "mackuba.eu"],
  ["only posts", "dogcuddlelover.bsky.social"],
  ["onlyposts", "skyfeed.xyz"],
  ["the 'gram", "why.bsky.world"],
  ["discover", "skyfeed.xyz"],
  ["latest from follows", "why.bsky.world"],
  ["teams", "retr0.id"],
  ["best of follows", "bsky.app"],
  ["media", "jcsalterego.bsky.social"],
  ["trans+queer shitposters", "jaz.sh"],
];

const sleep = (ms) => new Promise((resume) => setTimeout(resume, ms));

async function xrpc(method, params) {
  const url = new URL(`/xrpc/${method}`, APPVIEW);
  for (const [key, value] of Object.entries(params)) url.searchParams.set(key, String(value));
  try {
    const response = await fetch(url, { headers: { "user-agent": UA } });
    const text = await response.text();
    let body = null;
    try {
      body = JSON.parse(text);
    } catch {
      body = null;
    }
    return { status: response.status, body };
  } catch (error) {
    return { status: 0, body: { error: "NetworkError", message: String(error) } };
  }
}

/** Ask one feed for a page, up to TRIES times, and keep the best answer. Best
 *  and not last: the question is whether this feed CAN lay a wall. */
async function probe(uri) {
  // starts null rather than at a zeroed row, so a feed that fails every attempt
  // reports the status and error it actually gave. A placeholder here printed
  // `0 NotTried` over the 401s and 502s that are the whole finding.
  let best = null;
  for (let attempt = 0; attempt < TRIES; attempt++) {
    const got = await xrpc("app.bsky.feed.getFeed", { feed: uri, limit: LIMIT });
    const here = {
      status: got.status,
      posts: Array.isArray(got.body?.feed) ? got.body.feed.length : 0,
      error: got.body?.error ?? "",
    };
    if (!best || here.posts > best.posts || (best.status !== 200 && here.status === 200)) {
      best = here;
    }
    if (best.posts > BAR) break;
    await sleep(400);
  }
  return { ...best, lays: best.status === 200 && best.posts > BAR };
}

function row(lays, status, posts, error, name, creator) {
  const mark = lays ? "lays " : "HIDE ";
  const said = `${String(status).padEnd(4)} posts=${String(posts).padEnd(3)}`;
  return `  ${mark} ${said} ${(error || "-").padEnd(22)} ${name}  @${creator}`;
}

// The popular list, paged to TOP. One call answers about forty, so the cursor is
// what makes "the top fifty" true rather than approximately true.
const seen = new Map();
let cursor;
while (seen.size < TOP) {
  const params = cursor ? { limit: 50, cursor } : { limit: 50 };
  const page = await xrpc("app.bsky.unspecced.getPopularFeedGenerators", params);
  if (page.status !== 200) {
    console.error(`popular list failed: ${page.status} ${page.body?.error ?? ""}`);
    process.exit(1);
  }
  const feeds = page.body.feeds ?? [];
  for (const feed of feeds) if (!seen.has(feed.uri)) seen.set(feed.uri, feed);
  cursor = page.body.cursor;
  if (!cursor || feeds.length === 0) break;
}
const popular = [...seen.values()].slice(0, TOP);

console.log(`the top ${popular.length} popular feeds, logged out, bar is >${BAR} posts:\n`);
const hide = [];
for (const feed of popular) {
  const got = await probe(feed.uri);
  const name = feed.displayName ?? "";
  const creator = feed.creator?.handle ?? "";
  if (!got.lays) hide.push({ name, creator, ...got });
  console.log(row(got.lays, got.status, got.posts, got.error, name, creator));
  await sleep(150);
}

console.log(`\n${popular.length - hide.length} lay a wall, ${hide.length} do not.\n`);
console.log("the ones to deny, as UNLAYABLE_FEEDS entries:\n");
for (const feed of hide) {
  console.log(`  { name: ${JSON.stringify(feed.name.toLowerCase())}, creator: ${JSON.stringify(feed.creator)} },`);
}

console.log("\nentries already denied, re-tested (a `lays` here is a feed being hidden for nothing):\n");
for (const [name, creator] of DENIED) {
  const found = await xrpc("app.bsky.unspecced.getPopularFeedGenerators", { limit: 25, query: name });
  const matches = (found.body?.feeds ?? []).filter(
    (feed) =>
      (feed.displayName ?? "").trim().toLowerCase() === name &&
      (creator === null || (feed.creator?.handle ?? "") === creator),
  );
  if (matches.length === 0) {
    console.log(`  ?     no publisher of "${name}" in the directory right now`);
    continue;
  }
  for (const feed of matches.slice(0, 4)) {
    const got = await probe(feed.uri);
    console.log(
      row(got.lays, got.status, got.posts, got.error, feed.displayName, feed.creator?.handle ?? ""),
    );
    await sleep(150);
  }
}
