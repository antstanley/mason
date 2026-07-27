import { cleanHandle } from "$lib/state/handle.svelte";

/** What the picker's one input turns out to be asking for.
 *
 *  The picker offers a single field, because a reader with a feed link in their
 *  clipboard, a reader looking for "science", and a reader who knows whose feeds
 *  they want are all doing the same thing: naming a feed. What they typed is
 *  what decides which question mason asks, and this module is that decision,
 *  kept out of `FeedPicker.svelte` because nothing in this repo typechecks or
 *  runs a component body and this is the one part of the picker with a wrong
 *  answer.
 *
 *  On the `feed` case mason is a client of its own engine: the value is handed
 *  to mortar as `?feed=`, and
 *  `server/crates/mortar-core/src/sources/feedref.rs` is the authority on what
 *  parses. The check below exists only so an unparseable paste can be said in
 *  place, at the input, instead of laying a wall that immediately fails. It is
 *  therefore deliberately NO STRICTER than mortar's parser: a value mason would
 *  lay must never be refused here, and anything that slips through lands on the
 *  wall's own error panel, which is a worse message but never a lost feed. */
type Asked =
  /** Nothing typed: the resting state, which is what the network ranks. */
  | { kind: "browse" }
  /** A feed reference, ready to hand to `/?feed=`. */
  | { kind: "feed"; ref: string }
  /** Typed as a reference and is not one, so nothing navigates. */
  | { kind: "unparseable" }
  /** Handle-shaped, so it means "the feeds this person made". */
  | { kind: "creator"; handle: string }
  /** Anything else: feeds by name. */
  | { kind: "search"; term: string };

/** The AT-URI scheme. */
const AT_URI_PREFIX = "at://";

/** The one collection a feed reference may name. An `at://` URI naming any
 *  other (a post, a list, a profile record) points at something mortar cannot
 *  page, so the picker says so rather than laying a wall that cannot exist. */
const FEED_GENERATOR_COLLECTION = "app.bsky.feed.generator";

/** The exact origin and path prefix of a bsky.app feed link, matched as one
 *  literal for the same reason mortar matches it as one: a lookalike host is
 *  then structurally unable to pass. `https://evilbsky.app/profile/...` fails on
 *  the byte before `bsky.app`, and `https://bsky.app.example.com/profile/...`
 *  fails on the `.` where this literal requires a `/`. */
const BSKY_FEED_URL_PREFIX = "https://bsky.app/profile/";

/** The path segment that tells a bsky.app feed link from a post or a list one. */
const BSKY_FEED_SEGMENT = "feed";

/** The DID methods an atproto identity is spelled with. */
const DID_METHOD_PREFIXES = ["did:plc:", "did:web:"];

/** The longest reference this reads, in characters. mortar's own cap
 *  (`MAX_FEED_REF_LEN_BYTES`) is 1024 bytes; a character can be more than one
 *  byte, so this bound is the more permissive of the two, which is the right
 *  direction: a paste this accepts and mortar rejects costs an error panel,
 *  while one this rejected and mortar would have laid costs a feed. */
const MAX_FEED_REF_LEN_CHARS = 1024;

/** A record key, the schema's `[A-Za-z0-9._~-]`. */
const RKEY = /^[A-Za-z0-9._~-]+$/;

/** A DID's method-specific id, and a bsky.app profile segment, which is the
 *  union of the handle and DID sets because the link form carries either. `&`,
 *  `#` and `?` are all outside it. */
const DID_ID = /^[A-Za-z0-9._:%-]+$/;

/** A handle authority. The set excludes `:`, which is what keeps it disjoint
 *  from the DID form, so no authority can be read as both. */
const HANDLE_AUTHORITY = /^[A-Za-z0-9.-]+$/;

/** Any URI scheme, `javascript:` and `data:` included. Something carrying one is
 *  a value somebody pasted rather than a phrase somebody typed, so it is
 *  answered as a reference (and refused as one) instead of being run as a
 *  search for a string with a colon in it. */
const HAS_SCHEME = /^[A-Za-z][A-Za-z0-9+.-]*:/;

/** A domain name, which is what an atproto handle is: dotted labels, each
 *  starting and ending alphanumeric, with an alphabetic last label. It is what
 *  separates "alice.bsky.social" (whose feeds mason can list) from "science"
 *  (which is a search term), and it is deliberately shape only: whether the
 *  handle resolves is the AppView's answer, not this module's. */
const HANDLE_SHAPE =
  /^[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?(\.[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?)*\.[A-Za-z]{2,}$/;

/** The longest handle atproto allows. It is checked before the shape above
 *  rather than left to it, because that pattern nests a quantifier inside a
 *  repeated group and a long non-matching string is the input that makes such a
 *  pattern expensive. The bound is what keeps it linear. */
const MAX_HANDLE_LEN_CHARS = 253;

/** Split a path into exactly three segments. Exactly, not at least: a fourth
 *  segment, or the empty one a trailing slash leaves, means the string is not
 *  one of the accepted shapes, and a remainder would ride into the `?feed=`
 *  value mortar then queries with. */
function threeSegments(path: string): [string, string, string] | null {
  const parts = path.split("/");
  const [first, second, third] = parts;
  if (parts.length !== 3 || !first || !second || !third) return null;
  return [first, second, third];
}

/** `did:plc:` or `did:web:` followed by a non-empty method-specific id. */
function isDid(authority: string): boolean {
  return DID_METHOD_PREFIXES.some((prefix) => {
    if (!authority.startsWith(prefix)) return false;
    return DID_ID.test(authority.slice(prefix.length));
  });
}

/** `<authority>/<collection>/<rkey>`, the part of an AT-URI after `at://`. */
function readAtUri(path: string): boolean {
  const segments = threeSegments(path);
  if (!segments) return false;
  const [authority, collection, rkey] = segments;
  if (collection !== FEED_GENERATOR_COLLECTION || !RKEY.test(rkey)) return false;
  // a handle authority is a legal spelling people do paste; mortar resolves it
  // to a DID before it queries anything, so both forms are references mason lays
  return isDid(authority) || HANDLE_AUTHORITY.test(authority);
}

/** `<profile>/feed/<rkey>`, the part of a bsky.app feed link after the origin. */
function readBskyFeedUrl(path: string): boolean {
  const segments = threeSegments(path);
  if (!segments) return false;
  const [profile, segment, rkey] = segments;
  return segment === BSKY_FEED_SEGMENT && DID_ID.test(profile) && RKEY.test(rkey);
}

/** The reference a value names, or null when it names none.
 *
 *  What comes back is the value itself rather than a rebuilt URI: mortar parses
 *  the parameter again on the other side and rebuilds the canonical spelling
 *  there, so rebuilding it here as well would be a second author of one string.
 */
function feedRef(value: string): string | null {
  if (value.length > MAX_FEED_REF_LEN_CHARS) return null;
  if (value.startsWith(AT_URI_PREFIX)) {
    return readAtUri(value.slice(AT_URI_PREFIX.length)) ? value : null;
  }
  if (value.startsWith(BSKY_FEED_URL_PREFIX)) {
    return readBskyFeedUrl(value.slice(BSKY_FEED_URL_PREFIX.length)) ? value : null;
  }
  // the two prefixes above are the whole accepted surface. A `javascript:`
  // string, a scheme-relative `//bsky.app/...` and a bare host all stop here.
  return null;
}

/** Whether a value was pasted as a reference rather than typed as a phrase.
 *
 *  Answered before it is parsed, and separately from it, because the two
 *  questions have different consequences: something that reads as a reference
 *  and does not parse is an error the reader has to see, while a phrase that
 *  happens not to parse is just a search. */
function referenceShaped(value: string): boolean {
  // whitespace rules out a clipboard reference and rules in a phrase, and it is
  // checked first so "the year 2026: a retrospective" stays a search
  if (/\s/.test(value)) return false;
  // a scheme, the bare host somebody copied out of an address bar, and the
  // scheme-relative form a copied link sometimes arrives as. All three name a
  // location rather than a phrase, so all three are answered as references,
  // which for the last two means refused: mortar accepts neither.
  return HAS_SCHEME.test(value) || value.startsWith("bsky.app/") || value.startsWith("//");
}

/** Which question one input is asking. */
export function askedFor(raw: string): Asked {
  const value = raw.trim();
  if (!value) return { kind: "browse" };
  if (referenceShaped(value)) {
    const ref = feedRef(value);
    return ref ? { kind: "feed", ref } : { kind: "unparseable" };
  }
  // the handle box's own normalization, not a second spelling of it, so
  // "@Alice.Bsky.Social" and "alice.bsky.social" are one person here exactly as
  // they are there
  const handle = cleanHandle(value);
  if (handle.length <= MAX_HANDLE_LEN_CHARS && HANDLE_SHAPE.test(handle)) {
    return { kind: "creator", handle };
  }
  return { kind: "search", term: value };
}
