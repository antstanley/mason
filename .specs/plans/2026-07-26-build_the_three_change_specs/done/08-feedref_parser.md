# Task 08 · FeedRef parser

**Plan:** [plan.md](../plan.md) · **Certificate:** [08-feedref_parser-certificate.md](08-feedref_parser-certificate.md)

**Implements:** [`changes/merged/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/merged/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `01-domain-model.md` → FeedRef, and → `04-sources-and-moderation.md` → Outbound safety; implementation note 2. Targets [`01-domain-model.md`](../../../01-domain-model.md) §Entities and [`04-sources-and-moderation.md`](../../../04-sources-and-moderation.md) §Outbound safety.
**Depends on:** none
**Produces:** the one request parameter besides `actor` and `cursor` that reaches an upstream query is parsed rather than forwarded, with every rejection path tested and no `AppState` needed to test it.
**Pointers:** `server/crates/mortar-core/src/sources/mod.rs:8` (the `pub mod` list) and `:15` (the re-export line beside `AuthorYield`/`Follow`). `architecture-principles.md` rule 1: the rest of the crate consumes `sources/mod.rs`, never a submodule directly. `sources/util.rs:8` holds `urlencode`.

## Steps

- [ ] Add `server/crates/mortar-core/src/sources/feedref.rs` with a `parse` whose return type has **two cases**: a finished AT-URI, and a `(profile segment, rkey)` pair still awaiting DID resolution. A bare `Option<String>` cannot express that distinction and would make the caller re-inspect the string the parser just parsed.
- [ ] Accept **three** spellings into those two cases: `at://<did>/app.bsky.feed.generator/<rkey>` as the finished AT-URI; `at://<handle>/app.bsky.feed.generator/<rkey>` as the pair, because a handle-authority AT-URI is a legal spelling people do paste and mason always queries with the DID form; and `https://bsky.app/profile/<handle|did>/feed/<rkey>` as the pair. The change spec says so at its implementation note 2 and pins all three patterns in the `FeedRef` `$def`.
- [ ] Reject an AT-URI naming any other collection, a `javascript:` string, a scheme-relative `//bsky.app/...`, and any host that merely ends in `bsky.app`.
- [ ] Declare `pub mod feedref` in `sources/mod.rs` and re-export the type beside `AuthorYield` and `Follow`.
- [ ] Write the tests as pure function calls: no `AppState`, no wiremock, no async.

## Definition of done

- [ ] All three accepted spellings parse into the right case (the DID-authority AT-URI to the finished URI, the handle-authority AT-URI and the `bsky.app` URL to the pair), and each of the four rejection classes above has its own named negative-space test.
- [ ] A reference carrying `&` or `#` either fails to parse or survives intact for `urlencode` downstream, asserted either way rather than left implicit.
- [ ] The module is reached through `sources/mod.rs` and nothing outside `sources/` names `sources::feedref`.
- [ ] Meets the repo definition of done (negative-space tests for every rejection path, named constants for any bound, `just guard-wasm` and `just check` green).
- [ ] Reviewable: run `cd server && cargo nextest run -p mortar-core feedref` and read the test names; each one is a string somebody could paste.
