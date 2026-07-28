//! The `?feed=` request parameter, parsed into a feed generator reference.
//!
//! A feed reference is the one request parameter besides `actor` and `cursor`
//! that reaches an upstream query, so it is parsed rather than forwarded:
//! nothing that fails the checks below is ever interpolated into a `getFeed`
//! call or into the cache key derived from it. That is the third class of
//! untrusted string in `04-sources-and-moderation.md`, Outbound safety, beside
//! the URLs that reach an `<a href>` and the values that reach a query string.
//!
//! It is deliberately pure string work. Resolving a handle to a DID is the
//! caller's, which is what keeps this module testable with no `AppState`, no
//! cache and no network, and what keeps the resolution hop in the one place
//! that already owns the `did` cache.

/// The AT-URI scheme, and the prefix a `Uri` is rebuilt with.
const AT_URI_PREFIX: &str = "at://";

/// The one collection a feed reference may name. An `at://` URI naming any
/// other (a post, a list, a profile record) points at something `getFeed`
/// cannot page, so it is rejected here rather than upstream.
const FEED_GENERATOR_COLLECTION: &str = "app.bsky.feed.generator";

/// The exact origin and path prefix of a bsky.app feed link. Matching the whole
/// prefix as one literal, rather than parsing a host out and comparing its
/// tail, is what makes a lookalike host structurally unable to match:
/// `https://evilbsky.app/profile/...` fails on the byte before `bsky.app`, and
/// `https://bsky.app.example.com/profile/...` fails on the `.` where this
/// literal requires a `/`.
const BSKY_FEED_URL_PREFIX: &str = "https://bsky.app/profile/";

/// The path segment that sits between the profile and the rkey in a bsky.app
/// feed link, and the thing that distinguishes it from a post or a lists link.
const BSKY_FEED_SEGMENT: &str = "feed";

/// The DID methods an atproto identity is spelled with. An authority using any
/// other method is not an identity mason can resolve, so it is neither a DID
/// nor (because of the colons) a legal handle, and the reference is rejected.
const DID_METHOD_PREFIXES: [&str; 2] = ["did:plc:", "did:web:"];

/// The longest reference this parser will look at, in bytes. atproto bounds
/// every part one is built from (a handle is at most 253 bytes, an rkey at most
/// 512, the collection is 23), so a legal spelling lands comfortably under this
/// cap and anything past it is not a reference somebody has in their clipboard:
/// it is a string aimed at the cache key this value ends up as part of.
const MAX_FEED_REF_LEN_BYTES: usize = 1024;

/// A validated pointer to a Bluesky feed generator record.
///
/// Two cases rather than one string, because the caller has to know whether a
/// DID is still owed. Collapsing them into an `Option<String>` would make every
/// caller re-inspect the string this module just parsed, and a second reading
/// of the same string is a second place for the two readings to disagree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedRef {
    /// The DID-authority AT-URI mason queries `getFeed` with, ready as it
    /// stands.
    Uri(String),
    /// A reference whose authority is still a handle: an `at://` URI spelled
    /// with one, or a bsky.app link's profile segment (which may be a handle or
    /// a DID). The caller resolves `profile` to a DID and rebuilds the AT-URI
    /// from it and `rkey`.
    NeedsDid { profile: String, rkey: String },
}

/// Build the DID-authority AT-URI a `NeedsDid` reference becomes once its
/// profile segment has been resolved.
///
/// It lives beside the parser rather than at the call site so the one
/// collection literal mason queries with is spelled in exactly one module: a
/// second spelling is a second thing to keep in step with the check that vets
/// it, and the two would disagree the day either moves.
pub fn uri_for(did: &str, rkey: &str) -> String {
    format!("{AT_URI_PREFIX}{did}/{FEED_GENERATOR_COLLECTION}/{rkey}")
}

/// Read a `feed` query value. `None` is a `bad_request`: unlike `mode` and
/// `intent`, an unparseable feed reference has no default to fall back to,
/// because it names no wall at all.
///
/// Three spellings are accepted, two of which are what people actually have in
/// their clipboard:
///
/// | Given | Returned |
/// |---|---|
/// | `at://<did>/app.bsky.feed.generator/<rkey>` | `Uri` |
/// | `at://<handle>/app.bsky.feed.generator/<rkey>` | `NeedsDid` |
/// | `https://bsky.app/profile/<handle\|did>/feed/<rkey>` | `NeedsDid` |
pub fn parse(raw: &str) -> Option<FeedRef> {
    if raw.len() > MAX_FEED_REF_LEN_BYTES {
        return None;
    }
    if let Some(path) = raw.strip_prefix(AT_URI_PREFIX) {
        return parse_at_uri(path);
    }
    if let Some(path) = raw.strip_prefix(BSKY_FEED_URL_PREFIX) {
        return parse_bsky_feed_url(path);
    }
    // The two prefixes above are the whole accepted surface, and there is no
    // fallback behind them. A `javascript:` string, a scheme-relative
    // `//bsky.app/...` and a bare host all land here and stop.
    None
}

/// `<authority>/<collection>/<rkey>`, the part of an AT-URI after `at://`.
fn parse_at_uri(path: &str) -> Option<FeedRef> {
    let (authority, collection, rkey) = three_segments(path)?;
    if collection != FEED_GENERATOR_COLLECTION || !is_rkey(rkey) {
        return None;
    }
    if is_did(authority) {
        // Rebuilt from the vetted parts rather than echoed back, so a `Uri` is
        // by construction exactly the canonical spelling, and a future
        // loosening of the input side cannot leak an unvetted byte into the
        // upstream query.
        Some(FeedRef::Uri(format!(
            "{AT_URI_PREFIX}{authority}/{FEED_GENERATOR_COLLECTION}/{rkey}"
        )))
    } else if is_handle(authority) {
        // A handle-authority AT-URI is a legal spelling and one people do
        // paste. mason always queries with the DID form, so this one owes a
        // resolution hop exactly as a bsky.app link does.
        Some(FeedRef::NeedsDid {
            profile: authority.to_string(),
            rkey: rkey.to_string(),
        })
    } else {
        None
    }
}

/// `<profile>/feed/<rkey>`, the part of a bsky.app feed link after the origin.
fn parse_bsky_feed_url(path: &str) -> Option<FeedRef> {
    let (profile, segment, rkey) = three_segments(path)?;
    if segment != BSKY_FEED_SEGMENT || !is_profile_segment(profile) || !is_rkey(rkey) {
        return None;
    }
    // The profile segment is a handle or a DID and the caller cannot tell which
    // without looking, so both take the same resolution path. A DID resolves to
    // itself out of the `did` cache, which is cheaper than a second parse here
    // that would have to agree with this one forever.
    Some(FeedRef::NeedsDid {
        profile: profile.to_string(),
        rkey: rkey.to_string(),
    })
}

/// Split a path into exactly three segments. Exactly, not at least: a fourth
/// segment, or the empty one a trailing slash produces, means the string is not
/// one of the three accepted shapes. Accepting a remainder would let a path
/// suffix or a query string ride into the upstream request.
fn three_segments(path: &str) -> Option<(&str, &str, &str)> {
    let mut parts = path.split('/');
    let first = parts.next()?;
    let second = parts.next()?;
    let third = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((first, second, third))
}

/// `did:plc:` or `did:web:` followed by a non-empty method-specific id, whose
/// character set is the schema's `[A-Za-z0-9._:%-]`.
fn is_did(authority: &str) -> bool {
    DID_METHOD_PREFIXES.iter().any(|prefix| {
        authority
            .strip_prefix(prefix)
            .is_some_and(|id| !id.is_empty() && id.bytes().all(is_did_byte))
    })
}

/// A handle authority, the schema's `[A-Za-z0-9.-]`. The set excludes `:`,
/// which is what keeps it disjoint from the DID form: no string can be read as
/// both, so the two cases can never be ambiguous.
fn is_handle(authority: &str) -> bool {
    !authority.is_empty()
        && authority
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
}

/// A bsky.app profile segment, the schema's `[A-Za-z0-9._:%-]`. It is the union
/// of the handle and DID sets because the link form carries either.
fn is_profile_segment(segment: &str) -> bool {
    !segment.is_empty() && segment.bytes().all(is_did_byte)
}

/// A record key, the schema's `[A-Za-z0-9._~-]`.
fn is_rkey(rkey: &str) -> bool {
    !rkey.is_empty()
        && rkey
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'~' | b'-'))
}

/// The character set shared by a DID's method-specific id and a bsky.app
/// profile segment. `&`, `#` and `?` are all outside it, which is what stops a
/// reference from rewriting the query it is interpolated into even before
/// `urlencode` sees it.
fn is_did_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'%' | b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// The form mason ultimately queries with, and the only one that needs no
    /// resolution hop. Both DID methods are accepted; nothing else is.
    #[test]
    fn at_did_plc_z72i7hdynmk6r22z27h6tvur_app_bsky_feed_generator_whats_hot_is_ready_to_query() {
        let raw = "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot";
        assert_eq!(parse(raw), Some(FeedRef::Uri(raw.to_string())));

        let web = "at://did:web:feeds.example.com/app.bsky.feed.generator/3k2a.b_c~d";
        assert_eq!(parse(web), Some(FeedRef::Uri(web.to_string())));
    }

    /// A handle-authority AT-URI is a legal spelling people paste, and mason
    /// queries with the DID form only, so it comes back as the pair rather than
    /// as a finished URI the caller would then have to re-inspect.
    #[test]
    fn at_alice_bsky_social_app_bsky_feed_generator_whats_hot_needs_a_did() {
        assert_eq!(
            parse("at://alice.bsky.social/app.bsky.feed.generator/whats-hot"),
            Some(FeedRef::NeedsDid {
                profile: "alice.bsky.social".to_string(),
                rkey: "whats-hot".to_string(),
            })
        );
    }

    /// The link in the address bar when somebody is looking at a feed, which is
    /// the spelling most readers will have. Its profile segment carries a
    /// handle or a DID and both take the same resolution path.
    #[test]
    fn https_bsky_app_profile_alice_bsky_social_feed_whats_hot_needs_a_did() {
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot"),
            Some(FeedRef::NeedsDid {
                profile: "alice.bsky.social".to_string(),
                rkey: "whats-hot".to_string(),
            })
        );
        assert_eq!(
            parse("https://bsky.app/profile/did:plc:z72i7hdynmk6r22z27h6tvur/feed/whats-hot"),
            Some(FeedRef::NeedsDid {
                profile: "did:plc:z72i7hdynmk6r22z27h6tvur".to_string(),
                rkey: "whats-hot".to_string(),
            })
        );
    }

    /// The two spellings meet here: a reference that owed a resolution hop
    /// becomes, once its profile segment is a DID, byte for byte the URI the
    /// DID spelling of the same feed parses to. If they ever diverged, one
    /// feed would occupy two `feed_pages` cache keys and the reader would pay
    /// two upstream reads for one page.
    #[test]
    fn a_resolved_reference_rebuilds_the_did_spelling_exactly() {
        let did = "did:plc:z72i7hdynmk6r22z27h6tvur";
        let Some(FeedRef::NeedsDid { rkey, .. }) =
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot")
        else {
            panic!("a bsky.app link parses to the pair");
        };
        let rebuilt = uri_for(did, &rkey);
        assert_eq!(
            parse(&rebuilt),
            Some(FeedRef::Uri(rebuilt.clone())),
            "what the caller rebuilds must itself parse as a finished URI"
        );
        assert_eq!(
            rebuilt,
            "at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot"
        );
    }

    /// The collection is the whole check on an AT-URI. Pointing `getFeed` at a
    /// post or a list would fail upstream in a way the reader could not read,
    /// and a collection that merely starts with the right one is a different
    /// collection.
    #[test]
    fn at_did_plc_aa_app_bsky_feed_post_is_not_a_feed_generator() {
        assert_eq!(parse("at://did:plc:aa/app.bsky.feed.post/3k2a"), None);
        assert_eq!(parse("at://did:plc:aa/app.bsky.graph.list/3k2a"), None);
        assert_eq!(parse("at://did:plc:aa/app.bsky.actor.profile/self"), None);
        assert_eq!(parse("at://did:plc:aa/app.bsky.feed.generatorx/3k2a"), None);
        assert_eq!(
            parse("at://alice.bsky.social/app.bsky.feed.post/3k2a"),
            None
        );
    }

    /// The reference reaches an upstream query rather than an `<a href>`, so
    /// `is_http_url` never sees it and this parser is the only thing standing
    /// between a `javascript:` string and the rest of the engine.
    #[test]
    fn javascript_alert_1_is_not_a_feed_reference() {
        assert_eq!(parse("javascript:alert(1)"), None);
        assert_eq!(
            parse("javascript:https://bsky.app/profile/alice/feed/x"),
            None
        );
        assert_eq!(parse("data:text/html,<script>"), None);
        assert_eq!(parse(""), None);
    }

    /// A scheme-relative URL inherits whatever scheme it is resolved against,
    /// so it names no origin on its own. The prefix match requires the literal
    /// `https://`, and a bare host is the same omission one byte shorter.
    #[test]
    fn scheme_relative_bsky_app_profile_alice_feed_whats_hot_is_not_a_feed() {
        assert_eq!(parse("//bsky.app/profile/alice/feed/whats-hot"), None);
        assert_eq!(parse("bsky.app/profile/alice/feed/whats-hot"), None);
        assert_eq!(parse("/profile/alice/feed/whats-hot"), None);
    }

    /// The rule is the exact origin, not a suffix: a host that merely ends in
    /// `bsky.app`, or one that continues past it, is somebody else's host. A
    /// tail comparison would accept both, which is why the check is a whole
    /// prefix including the trailing slash.
    #[test]
    fn evilbsky_app_and_bsky_app_example_com_are_not_bsky_app() {
        assert_eq!(parse("https://evilbsky.app/profile/alice/feed/x"), None);
        assert_eq!(
            parse("https://bsky.app.example.com/profile/alice/feed/x"),
            None
        );
        assert_eq!(parse("https://notbsky.app/profile/alice/feed/x"), None);
        assert_eq!(
            parse("https://bsky.app@evil.example.com/profile/a/feed/x"),
            None
        );
    }

    /// A bsky.app link only ever leaves mason as a `(profile, rkey)` pair, so
    /// it has to be exactly three segments. A query string or a deeper path
    /// would otherwise ride along into the AT-URI the caller rebuilds.
    #[test]
    fn https_bsky_app_profile_alice_feed_whats_hot_with_anything_after_it_is_not_a_feed() {
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot?utm_source=x"),
            None
        );
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot/"),
            None
        );
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot/extra"),
            None
        );
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/lists/x"),
            None
        );
        assert_eq!(parse("https://bsky.app/profile/alice.bsky.social"), None);
        assert_eq!(parse("https://bsky.app/profile//feed/whats-hot"), None);
    }

    /// The same three-segment rule on the AT-URI side, plus the empty rkey a
    /// trailing slash leaves behind. An empty rkey would build an AT-URI that
    /// resolves to no record at all.
    #[test]
    fn at_did_plc_aa_app_bsky_feed_generator_with_no_rkey_is_not_a_feed() {
        assert_eq!(parse("at://did:plc:aa/app.bsky.feed.generator/"), None);
        assert_eq!(parse("at://did:plc:aa/app.bsky.feed.generator"), None);
        assert_eq!(parse("at://did:plc:aa/app.bsky.feed.generator/a/b"), None);
        assert_eq!(parse("at:///app.bsky.feed.generator/3k2a"), None);
        assert_eq!(parse("at://"), None);
    }

    /// An authority is a DID mason can resolve or a handle, and nothing else.
    /// An unfamiliar method carries colons, so it fails the handle set too and
    /// cannot slip through as one.
    #[test]
    fn at_did_evil_aa_app_bsky_feed_generator_3k2a_is_not_an_authority_mason_resolves() {
        assert_eq!(parse("at://did:evil:aa/app.bsky.feed.generator/3k2a"), None);
        assert_eq!(parse("at://did:plc:/app.bsky.feed.generator/3k2a"), None);
        assert_eq!(
            parse("at://alice bsky social/app.bsky.feed.generator/3k2a"),
            None
        );
    }

    /// The parameter is interpolated into the `getFeed` query and into the
    /// cache key derived from it, so a reference carrying `&` or `#` must not
    /// survive to reach either. It cannot: neither byte is in any of the three
    /// character sets, so such a reference never parses at all, and what does
    /// parse is built only from those sets.
    #[test]
    fn a_reference_carrying_an_ampersand_or_a_hash_does_not_parse() {
        assert_eq!(
            parse("at://did:plc:aa/app.bsky.feed.generator/x&limit=1"),
            None
        );
        assert_eq!(
            parse("at://did:plc:aa&x/app.bsky.feed.generator/3k2a"),
            None
        );
        assert_eq!(
            parse("at://alice.bsky.social&x/app.bsky.feed.generator/3k2a"),
            None
        );
        assert_eq!(
            parse("https://bsky.app/profile/alice&x/feed/whats-hot"),
            None
        );
        assert_eq!(
            parse("https://bsky.app/profile/alice.bsky.social/feed/whats-hot#frag"),
            None
        );

        let Some(FeedRef::Uri(uri)) =
            parse("at://did:plc:z72i7hdynmk6r22z27h6tvur/app.bsky.feed.generator/whats-hot")
        else {
            panic!("a did-authority at-uri parses to a finished uri");
        };
        assert!(!uri.contains('&'));
        assert!(!uri.contains('#'));
        assert!(!uri.contains('?'));
    }

    /// atproto bounds every part a reference is built from, so a string past
    /// the cap is not a reference: it is length aimed at the cache key. The
    /// bound is checked before any splitting so an oversized value costs one
    /// comparison.
    #[test]
    fn a_reference_longer_than_the_cap_is_not_read() {
        let rkey = "a".repeat(MAX_FEED_REF_LEN_BYTES);
        let over = format!("at://did:plc:aa/app.bsky.feed.generator/{rkey}");
        assert!(over.len() > MAX_FEED_REF_LEN_BYTES);
        assert_eq!(parse(&over), None);

        // A reference at the longest shape atproto itself allows stays under
        // the cap, so the bound rejects nothing real.
        let long_but_legal = format!(
            "at://{}/app.bsky.feed.generator/{}",
            "a".repeat(253),
            "b".repeat(512)
        );
        assert!(long_but_legal.len() <= MAX_FEED_REF_LEN_BYTES);
        assert!(parse(&long_but_legal).is_some());
    }
}
