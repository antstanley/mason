# Done Certificate · Task 08: FeedRef parser

**Task:** [08-feedref_parser.md](08-feedref_parser.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

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
  - *Status:* unverified

- **O2 · A reference carrying `&` or `#` is handled explicitly.**
  - *Claim:* such a reference either fails to parse or survives intact for `urlencode` downstream,
    and the test asserts whichever it is rather than leaving it implicit.
  - *Evidence to collect:* find the test by name and read its assertion.
  - *Status:* unverified

- **O3 · The module is reached only through `sources/mod.rs`.**
  - *Claim:* `pub mod feedref` is declared in `sources/mod.rs` and the type is re-exported there;
    nothing outside `sources/` names `sources::feedref`.
  - *Evidence to collect:* read `server/crates/mortar-core/src/sources/mod.rs`. Run
    `grep -rn 'sources::feedref' server/crates/ --include=*.rs` and confirm every hit is inside
    `sources/`.
  - *Status:* unverified

- **O4 · Meets the repo definition of done.**
  - *Claim:* negative-space tests cover every rejection path, any new bound is a named constant, and
    the gates are green.
  - *Evidence to collect:* run `just guard-wasm` and `just check`. Confirm the tests are pure
    function calls: read the test module and check for absence of `AppState`, `wiremock` and
    `#[tokio::test]`.
  - *Status:* unverified

- **O5 · Reviewable: the test names are strings somebody could paste.**
  - *Claim:* a reviewer runs `cargo nextest run -p mortar-core feedref` and reads the test names,
    each naming a real input a person would paste rather than an abstract case.
  - *Evidence to collect:* the command output.
  - *Status:* unverified

## Regression check

- No existing callers in scope: this task adds a module and two lines to `sources/mod.rs`. Trace:
  `cd server && cargo nextest run` is green and `sources/mod.rs`'s existing re-exports
  (`AuthorYield`, `Follow`, `StdDocs`, `LiveStream`) are unchanged : (PRESERVED / REGRESSION)

## Residue

- The handle-authority spelling parses here and resolves nowhere: turning its `(profile, rkey)` pair
  into a DID is task 13's `resolve_did`, so this task ships a case with no consumer for five tasks.
  Not an obligation here, but the reason its acceptance test asserts the variant rather than a URI.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
