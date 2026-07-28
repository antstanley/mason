# Done Certificate · Task 08: FeedRef parser

**Task:** [08-feedref_parser.md](08-feedref_parser.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 08. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 08) is every obligation O1 to O5 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** The one request parameter besides `actor` and `cursor` that reaches an upstream
  query is parsed rather than forwarded, with every rejection path tested and no `AppState` needed
  to test it.
- **P2 · Obligations.** Done iff O1 to O5 all hold; O5 is the Reviewable item.
- **P3 · Invariants.** Must not break the `sources/` boundary rule from
  `architecture-principles.md`: the rest of the crate consumes `sources/mod.rs`, never a submodule
  directly. Must not break `mortar-core`'s wasm32 build.

## Obligations

- **O1 · All three spellings parse into the right case, and every rejection has a named test.**
  - *Claim:* `at://<did>/app.bsky.feed.generator/<rkey>` parses to the ready case, while
    `at://<handle>/app.bsky.feed.generator/<rkey>` and
    `https://bsky.app/profile/<handle|did>/feed/<rkey>` both parse to the awaiting-resolution case;
    an AT-URI naming another collection, a `javascript:` string, a scheme-relative `//bsky.app/...`,
    and a lookalike host such as `https://evil-bsky.app/...` are each rejected by their own named
    test.
  - *Evidence to collect:* run `cd server && cargo nextest run -p mortar-core feedref` and list the
    test names. Confirm four distinct rejection tests and **three** acceptance tests. Read the return
    type and confirm it has two cases, not an `Option<String>`.
  - *Checks:* trace the lookalike-host case specifically: a naive `ends_with("bsky.app")` or
    `contains("bsky.app")` accepts `evil-bsky.app`. Confirm the host comparison is exact. Then trace
    the handle-authority AT-URI: it must land in the awaiting-resolution case, not be rejected and
    not be forwarded as a finished URI, because the AT-URI mason queries with is always the DID form.
    The change spec's `FeedRef` `$def` carries all three patterns, so a parser accepting two of them
    is a divergence rather than a stricter reading.
  - *Status:* SATISFIED. Ran the command: 12 tests, 12 passed. Three acceptance tests
    (`at_did_plc_z72i7hdynmk6r22z27h6tvur_..._is_ready_to_query`,
    `at_alice_bsky_social_..._needs_a_did`, `https_bsky_app_profile_alice_bsky_social_..._needs_a_did`)
    and four rejection tests, one per class (`..._app_bsky_feed_post_is_not_a_feed_generator`,
    `javascript_alert_1_is_not_a_feed_reference`,
    `scheme_relative_..._is_not_a_feed`, `evilbsky_app_and_bsky_app_example_com_are_not_bsky_app`),
    plus four more. `feedref.rs:56` is `pub enum FeedRef { Uri(String), NeedsDid { profile, rkey } }`,
    the two cases implementation note 2 names, returned as `Option<FeedRef>`. Host check: the match is
    `raw.strip_prefix("https://bsky.app/profile/")` on the whole literal including the trailing slash
    (`feedref.rs:29,96`), not a tail comparison, so `evilbsky.app`, `bsky.app.example.com`,
    `notbsky.app`, `bsky.app@evil.example.com`, `bsky.app.` and `http://bsky.app/...` all fail on a
    byte the literal fixes. Confirmed independently in an out-of-repo probe that includes the module
    by `#[path]` and runs 47 adversarial strings plus a byte-by-byte sweep of all four variable
    positions: zero surprises, and the accepted byte sets are exactly the schema's
    (`rkey [A-Za-z0-9._~-]`, `did id [A-Za-z0-9._:%-]`, `handle [A-Za-z0-9.-]`,
    `profile [A-Za-z0-9._:%-]`). Handle-authority AT-URI lands in `NeedsDid`, never `Uri`:
    `is_did` requires a `did:plc:`/`did:web:` prefix and `is_handle` excludes `:`, so the two sets are
    disjoint.
- **O2 · A reference carrying `&` or `#` is handled explicitly.**
  - *Claim:* such a reference either fails to parse or survives intact for `urlencode` downstream,
    and the test asserts whichever it is rather than leaving it implicit.
  - *Evidence to collect:* find the test by name and read its assertion.
  - *Status:* SATISFIED. `a_reference_carrying_an_ampersand_or_a_hash_does_not_parse`
    (`feedref.rs:366`) fixes the answer at rejection and asserts it in four positions (rkey, DID
    authority, handle authority, bsky.app profile segment) plus a `#` fragment on a bsky.app link, then
    asserts the other half explicitly: the parsed `Uri` contains no `&`, `#` or `?`. The probe's byte
    sweep confirms it exhaustively: neither byte is accepted in any of the four positions, nor is
    `?`, space, `\r`, `\n`, or the `\u{1f}` the `feed_pages` cache key is built with.
- **O3 · The module is reached only through `sources/mod.rs`.**
  - *Claim:* `pub mod feedref` is declared in `sources/mod.rs` and the type is re-exported there;
    nothing outside `sources/` names `sources::feedref`.
  - *Evidence to collect:* read `server/crates/mortar-core/src/sources/mod.rs`. Run
    `grep -rn 'sources::feedref' server/crates/ --include=*.rs` and confirm every hit is inside
    `sources/`.
  - *Status:* SATISFIED. `sources/mod.rs:9` is `pub mod feedref;` in the module list and `:17` is
    `pub use feedref::FeedRef;` beside `bluesky::{AuthorYield, Follow}`, `standardsite::StdDocs` and
    `streamplace::LiveStream`. `grep -rn "sources::feedref" server/ web/` returns no hits at all, and
    the only `FeedRef` mention outside the new file is that re-export line, so the new name collides
    with nothing.
- **O4 · Meets the repo definition of done.**
  - *Claim:* negative-space tests cover every rejection path, any new bound is a named constant, and
    the gates are green.
  - *Evidence to collect:* run `just guard-wasm` and `just check`. Confirm the tests are pure
    function calls: read the test module and check for absence of `AppState`, `wiremock` and
    `#[tokio::test]`.
  - *Status:* SATISFIED. `just check` exited 0 (guard-dashes, guard-autoplay, guard-toolchain,
    fmt-check, guard-wasm, oxlint, knip, `clippy --all-targets -D warnings`, 109 cargo-nextest tests,
    `tsc --noEmit`, 21 vitest tests). `just guard-wasm` exited 0, and because that run was warm it was
    repeated from scratch in a private `CARGO_TARGET_DIR`:
    `cargo check -p mortar-core --target wasm32-unknown-unknown --all-targets` compiled the crate and
    its test targets in 37.7s, exit 0, so the bare `#[cfg(test)]` gate is sound (`pretty_assertions` is
    an unconditional dev-dependency; only `tokio` and `wiremock` are the non-wasm ones, and neither is
    used). The module contains one `cfg(` in total, `#[cfg(test)]` at `:193`, and no `AppState`,
    `wiremock`, `tokio`, `async` or `await` anywhere except the doc line that says it needs none of
    them. Every bound is a named constant: `MAX_FEED_REF_LEN_BYTES` (1024, units in the name, checked
    before any splitting and asserted in both directions by
    `a_reference_longer_than_the_cap_is_not_read`), `AT_URI_PREFIX`, `FEED_GENERATOR_COLLECTION`,
    `BSKY_FEED_URL_PREFIX`, `BSKY_FEED_SEGMENT`, `DID_METHOD_PREFIXES`.
- **O5 · Reviewable: the test names are strings somebody could paste.**
  - *Claim:* a reviewer runs `cargo nextest run -p mortar-core feedref` and reads the test names,
    each naming a real input a person would paste rather than an abstract case.
  - *Evidence to collect:* the command output.
  - *Status:* SATISFIED. Exercised: `cd server && cargo nextest run -p mortar-core feedref` prints 12
    names, and each carries the string it is about rather than a code path:
    `at_did_plc_z72i7hdynmk6r22z27h6tvur_app_bsky_feed_generator_whats_hot_is_ready_to_query`,
    `at_alice_bsky_social_app_bsky_feed_generator_whats_hot_needs_a_did`,
    `https_bsky_app_profile_alice_bsky_social_feed_whats_hot_needs_a_did`,
    `at_did_plc_aa_app_bsky_feed_post_is_not_a_feed_generator`,
    `javascript_alert_1_is_not_a_feed_reference`,
    `scheme_relative_bsky_app_profile_alice_feed_whats_hot_is_not_a_feed`,
    `evilbsky_app_and_bsky_app_example_com_are_not_bsky_app`, and five more in the same voice.

## Regression check

- No existing callers in scope: this task adds a module and two lines to `sources/mod.rs`. Trace:
  `cd server && cargo nextest run` is green and `sources/mod.rs`'s existing re-exports
  (`AuthorYield`, `Follow`, `StdDocs`, `LiveStream`) are unchanged : PRESERVED. `jj diff --stat` is
  411 added lines in the new `feedref.rs` and exactly 2 added lines in `sources/mod.rs`, no deletions
  and no modified lines; the full suite ran 109 tests, 109 passed, 0 skipped; and no code anywhere in
  `server/` or `web/` calls `parse` yet, which is task 13's hop.

## Residue

- The handle-authority spelling parses here and resolves nowhere: turning its `(profile, rkey)` pair
  into a DID is task 13's `resolve_did`, so this task ships a case with no consumer for five tasks.
  Not an obligation here, but the reason its acceptance test asserts the variant rather than a URI.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O5 are each SATISFIED against evidence collected here rather than claimed (12 feedref
tests and the full 109-test suite run, an out-of-repo 47-string adversarial probe with a byte-level
sweep that found nothing accepted outside the schema's own character sets, a from-scratch wasm32
`--all-targets` check, and `just check` at exit 0), and the regression line is PRESERVED because the
diff only adds a module and two non-colliding re-export lines.
