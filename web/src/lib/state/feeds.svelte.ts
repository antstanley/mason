import { browser } from "$app/environment";
import { pushState, replaceState } from "$app/navigation";
import { page } from "$app/state";
import { APPVIEW } from "$lib/appview";
import { feedRkey } from "./feedinfo.svelte";
import { cleanHandle } from "./handle.svelte";
import type { HiddenLabel } from "$lib/types";

/** Everything the feed picker knows: the feeds this reader has opened before,
 *  the three questions it can ask the public AppView, and which feeds it must
 *  not list.
 *
 *  All of it lives in a rune module rather than in `FeedPicker.svelte` because
 *  this file is typechecked and unit tested and a `.svelte` body is neither.
 *  That includes the history push that opens the picker: half of a rule about
 *  two overlays, sitting in a file nothing in the repo can read, is half a
 *  rule. */

/** How many feeds `mason:feeds` remembers. Twelve is a screen of cards on the
 *  picker's widest column count, and the list is a convenience rather than a
 *  library: a reader who wants the thirteenth has the search box.
 *
 *  Exported so the cap has one spelling: a test asserting `12` in its own hand
 *  would pass a module that had quietly stopped capping at all. */
export const MAX_RECENT_FEEDS = 12;

/** Where the recents list lives, beside `mason:handle` (state/handle.svelte.ts).
 *  Local, because logged out there is no saved-feeds list on the network to
 *  read and mason has nowhere else to put one. */
const STORAGE_KEY = "mason:feeds";

/** How many results one question asks for. Both endpoints cap at 100 and
 *  default to 50; 30 is a couple of screens of cards, which is enough that the
 *  first page rarely needs a second and small enough that a slow AppView is not
 *  answering with fifty cards nobody scrolled to. */
const RESULTS_PER_PAGE = 30;

/** Browse and search are one endpoint, with and without a `query`.
 *
 *  `app.bsky.unspecced.*` carries no stability promise whatsoever, and the
 *  stable-looking `app.bsky.feed.searchFeedGenerators` answers 501 on the public
 *  AppView, so this is a known and accepted dependency rather than an oversight.
 *  When it goes, browsing goes quiet (`browseUnavailable` below) and recents and
 *  paste, which are the load-bearing paths, keep working. */
const POPULAR = `${APPVIEW}/xrpc/app.bsky.unspecced.getPopularFeedGenerators`;

/** One person's feeds, which is the bridge between mason's two front doors: a
 *  handle typed here means "show me the feeds this person made" rather than
 *  "show me their wall". It accepts a bare handle, so that bridge costs no
 *  resolution hop. */
const ACTOR_FEEDS = `${APPVIEW}/xrpc/app.bsky.feed.getActorFeeds`;

/** The hidden moderation tier, as a runtime value.
 *
 *  A `Record` keyed by the union rather than an array, because a Record must
 *  carry every member: a label added to mortar's `HIDDEN_LABELS`
 *  (server/crates/mortar-core/src/sources/bluesky.rs) flows through the contract
 *  fixture into `HiddenLabel` and makes this object a compile error, which is
 *  before it can become a listing bug. An array would satisfy
 *  `readonly HiddenLabel[]` while missing three of them.
 *
 *  Exported so the tests can drive one case per label off the same value rather
 *  than a retyped copy of it; a type is erased and can generate nothing. */
export const HIDDEN_LABELS: Record<HiddenLabel, true> = {
  "!hide": true,
  "!no-unauthenticated": true,
  porn: true,
  sexual: true,
  "graphic-media": true,
};

/** One feed the picker must not offer. A bare `name` denies it whoever published
 *  it; a `creator` alongside denies that one publisher's copy alone. */
interface UnlayableFeed {
  name: string;
  creator?: string;
}

/** Feeds the picker must not offer, because tapping one does not give a wall.
 *
 *  Almost all of them rank by *who is asking*: your mutuals, your mentions, the
 *  people you follow who post rarely. mason is logged out by construction, so
 *  there is no viewer for them to be about. Listing a door mason then holds shut
 *  is the same mistake the hidden-tier filter above exists to prevent, so it is
 *  fixed in the same place.
 *
 *  **Measured, not assumed, and re-measurable: `pnpm feeds:audit`.** That script
 *  (`web/scripts/audit-feeds.mjs`) pages the popular list to fifty, asks each
 *  feed for a logged-out `getFeed`, and prints which ones clear the bar. This
 *  list is its output, last run 2026-07-28: of the top fifty, **eleven** do not
 *  lay a wall.
 *
 *  **The bar is more than five posts**, and it is one bar over two failures that
 *  look different from the outside:
 *
 *  - **It does not answer.** `getFeed` with no viewer gives **502
 *    InternalServerError** for most of them and **401 AuthRequiredError** for
 *    skyfeed's. Neither is a 404, so mortar classifies both `upstream` and the
 *    reader gets "the wall wouldn't load" for a feed that was never layable.
 *  - **It answers with almost nothing.** `Teams` gives one brick, spacecowboy17's
 *    `For You` one, `Trans+Queer Shitposters` three, every time it is asked.
 *    That is a wall in name only, and a card promising one is worse than no card.
 *
 *  Five is the bar because mason lays a *wall*: a card that opens onto three
 *  bricks reads as a broken feed rather than a quiet one, whichever it is.
 *
 *  **The second kind goes stale and the first does not**, which is the honest
 *  cost of a static list. An auth-gated feed will still be auth-gated next month;
 *  a quiet feed may not still be quiet. Re-run the audit rather than trusting
 *  this list's age, and drop any entry it clears.
 *
 *  **Keyed by name rather than AT-URI**, which is the opposite of what this file
 *  does everywhere else and needs its reason on the record. The same
 *  viewer-shaped feed ships from several publishers: the popular list carries
 *  two "Mutuals" (bsky.app's and skyfeed.xyz's). A URI denylist would hide one
 *  copy, leave the rest broken in the list, and need a new entry every time
 *  somebody published another. The name is what identifies the *idea*, and some
 *  ideas cannot work logged out.
 *
 *  **The creator field is what keeps that from overreaching, and it is
 *  load-bearing.** "Discover" is skyfeed's auth-gated feed and it is also
 *  `bsky.app`'s `whats-hot`, which answers 200 and is the single most useful
 *  wall in the list. Denying the name alone would hide the best feed mason has.
 *
 *  So the split below is between a name that names the VIEWER (mutuals,
 *  mentions, what your friends liked) and a name that merely describes a KIND of
 *  feed. The first cannot work here whoever publishes it; the second is a
 *  question only a request can answer, and it is answered per publisher.
 *  Measurement beats the idea when they disagree, and it did twice: "For You"
 *  and "Only Posts" were both name-only rules hiding a publisher's copy that
 *  answers nineteen and twenty posts.
 *
 *  The residual cost is accepted rather than overlooked: a working third-party
 *  feed genuinely called "Mentions" would be hidden from browse and search. It
 *  stays reachable by paste, which is the load-bearing path.
 *
 *  Compared case-folded and trimmed, because both fields are free text.
 *
 *  Exported so the tests drive one case per entry off this value rather than a
 *  retyped copy, the way the hidden tier above is: an entry added here gets its
 *  case for free, and one removed cannot leave a stale case behind. */
export const UNLAYABLE_FEEDS: readonly UnlayableFeed[] = [
  // DENIED BY NAME, whoever published it, because the name names the VIEWER: a
  // feed of your mutuals, your mentions, what your friends liked, which of the
  // people you follow post rarely. There is no viewer here, so there is nothing
  // for these to be about, and the audit found every publisher of each that the
  // directory returned failing: "Popular With Friends" from bsky.app and urguy,
  // "Mutuals" from bsky.app, skyfeed and shreyanjain.
  { name: "popular with friends" },
  { name: "mutuals" },
  { name: "quiet posters" },
  { name: "mentions" },
  // DENIED BY PUBLISHER, because the name describes a KIND of feed that a
  // logged-out algorithm can perfectly well produce, so measurement decides and
  // the idea does not. Two of these moved here from the list above when the
  // audit first ran, and both were feeds mason was hiding for nothing:
  //
  // - skygaze.io's "For You" answers 19 posts. Two other publishers' copies give
  //   1 post and an error, so those two are named and the name is not.
  // - moonandbaboon's "Only Posts" answers 20, four times out of four. mackuba's
  //   gives 1 and dogcuddlelover's gives 2.
  //
  // skyfeed's copy is spelled "OnlyPosts", one word, which is why it has its own
  // entry: the match is exact after case-folding, so the spaced name never hid
  // it and the old name-only rule was letting the one auth-gated copy through
  // while hiding two that worked.
  { name: "for you", creator: "spacecowboy17.bsky.social" },
  { name: "for you", creator: "flicknow.xyz" },
  { name: "only posts", creator: "mackuba.eu" },
  { name: "only posts", creator: "dogcuddlelover.bsky.social" },
  { name: "onlyposts", creator: "skyfeed.xyz" },
  { name: "the 'gram", creator: "why.bsky.world" },
  { name: "discover", creator: "skyfeed.xyz" },
  { name: "latest from follows", creator: "why.bsky.world" },
  { name: "teams", creator: "retr0.id" },
  { name: "best of follows", creator: "bsky.app" },
  // 502 on every attempt, three runs apart
  { name: "media", creator: "jcsalterego.bsky.social" },
  // 200 with exactly three posts, every time it is asked. Quiet rather than
  // gated, so this is the entry most likely to be wrong later: it is the first
  // one to re-check when the audit is next run.
  { name: "trans+queer shitposters", creator: "jaz.sh" },
];

/** Whether this feed is one mason cannot lay a wall from logged out. */
function unlayable(name: string | null | undefined, creator: string | null | undefined): boolean {
  if (name === null || name === undefined) return false;
  const wanted = name.trim().toLowerCase();
  const by = creator?.trim().toLowerCase();
  return UNLAYABLE_FEEDS.some(
    (feed) => feed.name === wanted && (feed.creator === undefined || feed.creator === by),
  );
}

/** Which question the picker is answering. One at a time, because the picker
 *  shows one list of results and what the reader typed decides which question
 *  is being asked; an answer replaces whatever was showing rather than landing
 *  beside it. The value is what tells the picker which empty state to use, so
 *  "no feeds by that name" is never shown to somebody who searched nothing. */
type Question = "popular" | "search" | "creator";

/** Each question as the endpoint that answers it and the parameter that carries
 *  what the reader typed. A Record keyed by the union again, so a fourth
 *  question cannot be added without naming both halves of it. */
const QUESTIONS: Record<Question, { endpoint: string; param: string | null }> = {
  popular: { endpoint: POPULAR, param: null },
  search: { endpoint: POPULAR, param: "query" },
  creator: { endpoint: ACTOR_FEEDS, param: "actor" },
};

/** One feed generator as the picker lists it, and as `mason:feeds` stores it.
 *
 *  It is stored whole rather than as a bare URI on purpose: recents have to
 *  render when the AppView will not answer, which is exactly the state a list of
 *  URIs could not draw a single card in. */
export interface FeedListing {
  /** The AT-URI. It is what `/?feed=` takes and what dedupes the recents. */
  uri: string;
  /** The generator's display name, or its rkey when a record was published
   *  without one: an ugly name beats a nameless card. */
  name: string;
  avatar: string | null;
  /** The handle that published it. Display names are not unique (two "Discover"
   *  feeds are ordinary), so this is what tells a reader which one they found. */
  creator: string | null;
  description: string;
  likeCount: number;
}

/** The whole of a click that a feed link reads: which button took it.
 *
 *  Structural rather than `MouseEvent`, for the same reason `reader.activate`'s
 *  own activation type is structural: the unit tests build one as a plain object
 *  and the node environment they run in has no DOM at all. */
interface LinkActivation {
  button: number;
}

/** One label as the AppView reports it. Only `val` matters here; the rest of the
 *  object (src, uri, cts, ...) is discarded, exactly as mortar discards it. */
interface Label {
  val?: string;
}

/** The half of `app.bsky.feed.defs#generatorView` the picker reads. Every field
 *  is optional because it is somebody else's record arriving over the network:
 *  the lexicon calling one of them required is not a guarantee this code gets to
 *  rely on. */
interface GeneratorView {
  uri?: string;
  displayName?: string;
  description?: string;
  avatar?: string;
  likeCount?: number;
  labels?: Label[];
  creator?: { handle?: string; labels?: Label[] };
}

/** What both endpoints answer with: a page of generators and a cursor for the
 *  next one. */
interface GeneratorPage {
  feeds?: GeneratorView[];
  cursor?: string;
}

/** Whether this set of labels carries the hidden tier. */
function hidden(labels: Label[] | undefined): boolean {
  // `Object.hasOwn` rather than `in`, which walks the prototype chain and would
  // hide a feed labelled "constructor" or "toString"
  return labels?.some((l) => l.val !== undefined && Object.hasOwn(HIDDEN_LABELS, l.val)) ?? false;
}

/** One generator view as a listing, or null when the picker must not list it.
 *
 *  Three reasons not to. A view with no uri names no feed, so there is nothing to
 *  open. A view carrying the hidden tier, on the feed's own record OR on its
 *  creator's, is a feed mason would refuse to lay brick by brick once the wall
 *  was asked for, and advertising it here would be mason recommending a door it
 *  then holds shut. A view whose ranking is about a viewer mason does not have
 *  cannot be laid at all (`UNLAYABLE_FEEDS` above), which is the same mistake
 *  reached from the other end.
 *
 *  All three are applied here rather than per question, so browse, search and a
 *  creator's own list cannot disagree about what mason will show. */
function listing(view: GeneratorView): FeedListing | null {
  const uri = view.uri;
  if (!uri) return null;
  if (hidden(view.labels) || hidden(view.creator?.labels)) return null;
  // the rkey fallback is deliberately not checked: a feed published with no
  // display name is not one of these, and rkeys like `with-friends` would
  // otherwise never match anyway
  if (unlayable(view.displayName, view.creator?.handle)) return null;
  return {
    uri,
    name: view.displayName || feedRkey(uri),
    avatar: view.avatar ?? null,
    creator: view.creator?.handle ?? null,
    description: view.description ?? "",
    likeCount: view.likeCount ?? 0,
  };
}

/** The shape of the stored list, in one place: most recent first, one entry per
 *  feed, and never more than `MAX_RECENT_FEEDS` of them. Keeping the FIRST of a
 *  repeated uri is what makes reopening yesterday's feed move it to the front
 *  instead of listing it twice, because the caller puts the newest at the head.
 *
 *  Applied on the way in from storage as well as on every open: `mason:feeds` is
 *  a string a reader can edit, so its length and its contents are the picker's
 *  problem rather than a past version's promise.
 *
 *  This is what gets written back, and it deliberately says nothing about
 *  `UNLAYABLE_FEEDS`; `layable` below is the half that does. */
function ordered(feeds: FeedListing[]): FeedListing[] {
  const seen = new Set<string>();
  const kept: FeedListing[] = [];
  for (const feed of feeds) {
    if (seen.has(feed.uri)) continue;
    seen.add(feed.uri);
    kept.push(feed);
    if (kept.length === MAX_RECENT_FEEDS) break;
  }
  return kept;
}

/** The stored list minus the feeds mason cannot lay a wall from, which is the
 *  only place recents meet `UNLAYABLE_FEEDS`.
 *
 *  A feed opened before it was denied, or before mason knew it could not be
 *  laid, is already sitting in somebody's `mason:feeds`, and a recent card is
 *  the one a reader is most likely to tap.
 *
 *  It is kept apart from `ordered` so the denial really is a filter on the way
 *  out and not a delete on the way in: `ordered` is what `remember` persists,
 *  this is what the picker renders, and the two are never the same array. If an
 *  entry in the list above turns out to be wrong, or a feed starts working
 *  logged out, removing it brings the card straight back rather than asking a
 *  reader to find the feed again. Folding this back into `ordered` would put a
 *  filtered list on the write path and permanently delete the entry the first
 *  time any other feed was opened, which is exactly the recovery this promises.
 *
 *  A denied entry still costs one of the twelve slots, which is the accepted
 *  price of keeping it: the cap is how much mason remembers, not how many cards
 *  it draws. */
function layable(feeds: FeedListing[]): FeedListing[] {
  return feeds.filter((feed) => !unlayable(feed.name, feed.creator));
}

/** One stored entry, validated field by field.
 *
 *  Nothing here trusts the shape: `mason:feeds` crosses a boundary out of the
 *  reader's own hands and back, so a hand-edited entry, or one written by an
 *  older version with different fields, has to read as a card or as nothing at
 *  all rather than as a card with `undefined` in it. */
function storedListing(entry: unknown): FeedListing | null {
  if (typeof entry !== "object" || entry === null) return null;
  // narrowed to an object above, and every field is read through a typeof
  // below, so this cast asserts nothing the guard has not already established
  const record = entry as Record<string, unknown>;
  const uri = record.uri;
  if (typeof uri !== "string" || !uri) return null;
  return {
    uri,
    name: typeof record.name === "string" && record.name ? record.name : feedRkey(uri),
    avatar: typeof record.avatar === "string" ? record.avatar : null,
    creator: typeof record.creator === "string" ? record.creator : null,
    description: typeof record.description === "string" ? record.description : "",
    // a stored NaN or Infinity would render as a nonsense tally; only a real
    // count survives, and zero is hidden by the card anyway
    likeCount:
      typeof record.likeCount === "number" && Number.isFinite(record.likeCount)
        ? record.likeCount
        : 0,
  };
}

/** Whether there is a `localStorage` to reach for. `browser` is the first gate
 *  and the `typeof` is the second: the unit tests run this module in node, where
 *  the global does not exist at all, and reaching an undeclared global throws a
 *  ReferenceError rather than yielding undefined. */
function storageAvailable(): boolean {
  return browser && typeof localStorage !== "undefined";
}

/** The recents list as storage has it. */
function readRecent(): FeedListing[] {
  if (!storageAvailable()) return [];
  let raw: string | null = null;
  try {
    raw = localStorage.getItem(STORAGE_KEY);
  } catch {
    // a browser can refuse storage entirely (Safari's private mode does); no
    // recents is a smaller loss than no picker
    return [];
  }
  if (!raw) return [];
  let parsed: unknown = null;
  try {
    parsed = JSON.parse(raw);
  } catch {
    // somebody else's JSON, or half of ours after a quota failure mid-write
    return [];
  }
  if (!Array.isArray(parsed)) return [];
  const feeds: FeedListing[] = [];
  for (const entry of parsed) {
    const feed = storedListing(entry);
    if (feed) feeds.push(feed);
  }
  return ordered(feeds);
}

/** Write the recents list back. Best effort: storage can be full or refused,
 *  and a forgotten recents list must never be the reason a feed does not open. */
function store(feeds: FeedListing[]): void {
  if (!storageAvailable()) return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(feeds));
  } catch {
    // full, or refused; the list stays in memory for this session
  }
}

/** Exported for the unit tests, which build throwaway instances; the app only
 *  ever uses the `feeds` singleton below. */
export class FeedsState {
  /** The recents list exactly as `mason:feeds` holds it, denied entries and
   *  all. Private, because nothing renders this: it is the write path, and the
   *  only reason it is separate from `recent` is that the two must not be the
   *  same list (see `layable`). */
  #stored = $state<FeedListing[]>(readRecent());

  /** The feeds this reader has opened, most recent first, minus the ones mason
   *  cannot lay a wall from. The one section of the picker that owes the
   *  network nothing.
   *
   *  A getter over the stored list rather than a field of its own, so there is
   *  one list and one filter and no second copy to fall out of step with what
   *  was written. It reads `#stored`, which is `$state`, so the picker still
   *  re-renders on a `remember`. */
  get recent(): FeedListing[] {
    return layable(this.#stored);
  }

  /** Which question `results` answers. */
  question = $state<Question>("popular");

  /** What the reader typed, for the two questions that carry it, so the picker
   *  can name it back ("no feeds called ...") without holding its own copy. */
  term = $state("");

  results = $state<FeedListing[]>([]);

  /** A question is in flight: the picker shows skeletons rather than an empty
   *  state, because "nothing found" and "nothing yet" read identically and mean
   *  opposite things. */
  loading = $state(false);

  /** The AppView would not answer, so browsing is unavailable. It is a flag and
   *  not an error message because it is not fatal: recents and the paste box are
   *  the load-bearing paths and both still work, and the picker says so quietly
   *  rather than emptying itself. */
  browseUnavailable = $state(false);

  /** Where the current question continues from, or null at its end. Private:
   *  paging is `more()`'s business and nothing renders a cursor. */
  #cursor: string | null = null;

  /** Bumped on every question, so an answer to one the reader has moved on from
   *  is dropped instead of landing under the new question's heading. */
  #generation = 0;

  /** Whether the history entry the picker is sitting on is ours to pop. */
  #pushed = false;

  /** Whether the picker is up.
   *
   *  `page.state.picker` is the whole of it. Unlike the reader there is nothing
   *  held in this rune for a reload to drop, so there is no second condition to
   *  agree with and this is the one place the key is read.
   *
   *  It is a predicate here rather than a page-state test at each call site for
   *  the same reason `reader.isOpen` is: the layout dims and freezes the wall
   *  behind whichever overlay is up, and a wrapper made inert on a wider
   *  condition than the overlay renders on is a page frozen under nothing. */
  get isOpen(): boolean {
    return page.state.picker === "feeds";
  }

  /** Open the picker over whatever is behind it.
   *
   *  The pushed state carries the picker's own key and NOTHING else, and that is
   *  the whole of the mutual-exclusion rule: a push replaces `App.PageState`
   *  rather than merging into it, so this drops `brick` and the reader shuts.
   *  The reader's half has the same shape (`pushState('', { brick })` in
   *  reader.svelte.ts), so neither overlay has to remember that the other
   *  exists. Structural rather than remembered, which is why it is here and not
   *  in `FeedPicker.svelte`: nothing in this repo typechecks or runs a component
   *  body, so half a rule kept there is half a rule nothing can check. */
  openPicker() {
    // already up: a second push would leave a second entry to walk back through
    if (this.isOpen) return;
    pushState("", { picker: "feeds" });
    this.#pushed = true;
  }

  /** Shut the picker. */
  closePicker() {
    const ours = this.#pushed;
    // spent either way: whichever branch runs, that entry is dealt with once
    this.#pushed = false;
    if (ours) {
      // pop our own entry, so the picker leaves no rubble in the history stack.
      // The router clears `page.state` as the popstate lands, which is what
      // shuts the screen; nothing here has to.
      history.back();
      return;
    }
    // no entry of ours to pop, so drop the key from the current one instead.
    // Going back from here would leave mason altogether, which is the opposite
    // of closing a picker.
    replaceState("", {});
  }

  /** The resting state: what the network itself ranks. */
  async browse(): Promise<void> {
    // already resting here with something to show, so reopening the picker (or
    // a re-render) does not spend a round trip re-fetching the same list
    if (this.question === "popular" && this.results.length > 0) return;
    await this.#ask("popular", "");
  }

  /** Feeds by name. */
  async search(term: string): Promise<void> {
    const wanted = term.trim();
    // an emptied box is the resting state, not a search that found nothing:
    // telling somebody who typed nothing that there are no feeds by that name
    // is an answer to a question they did not ask
    if (!wanted) {
      await this.browse();
      return;
    }
    await this.#ask("search", wanted);
  }

  /** The feeds one person has made. `getActorFeeds` takes a bare handle, so
   *  this is one request and no resolution hop. */
  async byCreator(handle: string): Promise<void> {
    // the same normalization the handle box uses, because it is the same thing
    // being typed: "@Alice.Bsky.Social " and "alice.bsky.social" are one person
    const actor = cleanHandle(handle);
    if (!actor) {
      await this.browse();
      return;
    }
    await this.#ask("creator", actor);
  }

  /** The next page of whatever is showing. The popular list is the one the
   *  picker is specified to page, but all three questions ride endpoints that
   *  answer with a cursor, so there is nothing here worth special-casing. */
  async more(): Promise<void> {
    // no cursor is the end of the list, and a second page asked for while the
    // first is still in flight would append the same feeds twice
    if (this.loading || !this.#cursor) return;
    await this.#ask(this.question, this.term, this.#cursor);
  }

  /** Remember a feed the reader opened, most recent first.
   *
   *  Prepended to the STORED list, not to the rendered one: reading `recent`
   *  here and writing the result back is what used to delete a denied entry the
   *  first time a reader opened anything else. */
  remember(feed: FeedListing) {
    this.#stored = ordered([feed, ...this.#stored]);
    store(this.#stored);
  }

  /** Remember a feed whose link is being taken, whichever button took it.
   *
   *  Both of mason's feed links (the picker's cards and the switcher panel's
   *  recents) hang this off `onclick` AND `onauxclick`, because those two events
   *  do not overlap: a browser dispatches NO `click` for a non-primary button,
   *  so a middle click, which opens the wall in a background tab, arrives as
   *  `auxclick` alone. With the click listener on its own, a reader who
   *  middle-clicks built no recents list at all, while the same reader
   *  cmd-clicking built one, because a modified PRIMARY click does dispatch
   *  `click`. Two listeners rather than a listener each way, so the two links
   *  cannot drift apart on the rule.
   *
   *  `auxclick` also fires for the RIGHT button, which opens a context menu and
   *  no wall, so the primary and the middle buttons are the two that count. The
   *  activation neither listener can see is "open link in new tab" chosen from
   *  that menu: the browser dispatches nothing at all for it, so that feed is
   *  remembered the next time it is opened from a link rather than then. */
  rememberFromLink(event: LinkActivation, feed: FeedListing) {
    if (event.button !== 0 && event.button !== 1) return;
    this.remember(feed);
  }

  /** Ask one question and take its answer, unless the reader has asked another
   *  one since. A cursor means the answer is another page of the question
   *  already showing, so it appends instead of replacing. */
  async #ask(question: Question, term: string, cursor: string | null = null): Promise<void> {
    // no AppView to ask, and nothing rendered to show an answer in. The picker
    // is a browser surface: this guard is why importing this module during a
    // build does not open a socket.
    if (!browser) return;
    const generation = ++this.#generation;
    const appending = cursor !== null;
    this.question = question;
    this.term = term;
    this.loading = true;
    this.browseUnavailable = false;
    if (!appending) {
      // the new question's answer replaces the old one's, rather than leaving
      // yesterday's results under today's heading while this one is in flight
      this.results = [];
      this.#cursor = null;
    }

    const { endpoint, param } = QUESTIONS[question];
    const params = new URLSearchParams();
    // every value goes through URLSearchParams, so a search term or a handle
    // carrying `&` or `#` cannot rewrite the request it rides in
    if (param && term) params.set(param, term);
    params.set("limit", String(RESULTS_PER_PAGE));
    if (cursor) params.set("cursor", cursor);

    const res = await fetch(`${endpoint}?${params}`).catch(() => null);
    // the cast claims only what `GeneratorPage` claims, which is nothing: every
    // field on it and on the views inside it is optional, and each one is read
    // through a `??` or a `typeof` below. A body that is not JSON at all catches
    // into null and reads as the AppView not answering.
    const body = res?.ok ? ((await res.json().catch(() => null)) as GeneratorPage | null) : null;

    // the reader asked something else while this was in flight, so this answers
    // a question nobody is looking at any more
    if (generation !== this.#generation) return;
    this.loading = false;

    if (!body) {
      // one flag for every way browsing can fail: unreachable, a 500, a body
      // that is not JSON, an endpoint that stopped existing. None of them is
      // the picker failing, because recents and paste do not come through here.
      this.browseUnavailable = true;
      this.#cursor = null;
      return;
    }

    const listed: FeedListing[] = [];
    for (const view of body.feeds ?? []) {
      const feed = listing(view);
      // a view with no uri, or one the hidden tier covers, is simply not listed
      if (feed) listed.push(feed);
    }
    this.results = appending ? [...this.results, ...listed] : listed;
    this.#cursor = body.cursor ?? null;
  }
}

export const feeds = new FeedsState();
