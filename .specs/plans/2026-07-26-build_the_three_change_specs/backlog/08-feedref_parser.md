# Task 08 · FeedRef parser

**Plan:** [plan.md](../plan.md) · **Certificate:** [08-feedref_parser-certificate.md](08-feedref_parser-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `01-domain-model.md` → FeedRef, and → `04-sources-and-moderation.md` → Outbound safety; implementation note 2. Targets [`01-domain-model.md`](../../../01-domain-model.md) §Entities and [`04-sources-and-moderation.md`](../../../04-sources-and-moderation.md) §Outbound safety.
**Depends on:** none
**Produces:** the one request parameter besides `actor` and `cursor` that reaches an upstream query is parsed rather than forwarded, with every rejection path tested and no `AppState` needed to test it.
**Pointers:** `server/crates/mortar-core/src/sources/mod.rs:8` (the `pub mod` list) and `:15` (the re-export line beside `AuthorYield`/`Follow`). `architecture-principles.md` rule 1: the rest of the crate consumes `sources/mod.rs`, never a submodule directly. `sources/util.rs:8` holds `urlencode`.

## Steps

- [ ] Add `server/crates/mortar-core/src/sources/feedref.rs` with a `parse` whose return type has **two cases**: a finished AT-URI, and a `(profile segment, rkey)` pair still awaiting DID resolution. A bare `Option<String>` cannot express that distinction and would make the caller re-inspect the string the parser just parsed.
- [ ] Accept `at://<did>/app.bsky.feed.generator/<rkey>` unchanged, and `https://bsky.app/profile/<handle|did>/feed/<rkey>` as the pair.
- [ ] Reject an AT-URI naming any other collection, a `javascript:` string, a scheme-relative `//bsky.app/...`, and any host that merely ends in `bsky.app`.
- [ ] Declare `pub mod feedref` in `sources/mod.rs` and re-export the type beside `AuthorYield` and `Follow`.
- [ ] Write the tests as pure function calls: no `AppState`, no wiremock, no async.

## Definition of done

- [ ] Both accepted spellings parse into the right case, and each of the four rejection classes above has its own named negative-space test.
- [ ] A reference carrying `&` or `#` either fails to parse or survives intact for `urlencode` downstream, asserted either way rather than left implicit.
- [ ] The module is reached through `sources/mod.rs` and nothing outside `sources/` names `sources::feedref`.
- [ ] Meets the repo definition of done (negative-space tests for every rejection path, named constants for any bound, `just guard-wasm` and `just check` green).
- [ ] Reviewable: run `cd server && cargo nextest run -p mortar-core feedref` and read the test names; each one is a string somebody could paste.

## Open questions

- The change spec's schema pattern requires `at://did:(plc|web):...`, so `at://<handle>/app.bsky.feed.generator/<rkey>` (a legal AT-URI spelling people do paste) is a `bad_request`. If that is deliberate, say so beside the pattern in task 19's schema fold; if not, widen it here.
