# Done Certificate · Task 10: feed_not_found error code

**Task:** [10-feed_not_found_error_code.md](10-feed_not_found_error_code.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

> Verification protocol for Task 10. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. This is wire
> regeneration 1 of 3; the fixture diff is the load-bearing artifact.

## Definition

DONE(Task 10) is every obligation O1 to O6 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** A fifth error code walks the whole forcing chain from `error.rs` to `types.ts` in
  one commit, leaving the repo green.
- **P2 · Obligations.** Done iff O1 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break the four existing error codes' pinned wire strings, the
  service worker's `satisfies MortarErrorCode` uses, or `feed.svelte.ts:197 #fail`'s classification.

## Obligations

- **O1 · The Rust half of the forcing chain is complete.**
  - *Claim:* `AppError::FeedNotFound(String)` maps to `(404, "feed_not_found")`; `variants()` is
    `[AppError; 5]`; both `wasm_envelope_is_pinned_per_variant` and
    `server_envelope_is_pinned_per_variant` carry the new literal strings; `ALL_CODES` is
    `[&str; 5]`; `code_key` has a matching arm; `errors()` carries an instance; and the key-set
    assert in `contract()` passes.
  - *Evidence to collect:* read `error.rs` and `tests/contract.rs` at the named sites. Run
    `cd server && cargo nextest run -p mortar-core`, expect green.
  - *Checks:* `code_key` asserts `code == error.status_and_code().1`. Confirm the new arm indexes
    `ALL_CODES[4]` and that the assertion still fires for it, so the fixture key cannot diverge from
    the wire code.
  - *Collected:* `error.rs:9` to `:14` declares `FeedNotFound(String)` with a why-comment and
    `#[error("feed not found: {0}")]`; `:47` is the `status_and_code` arm `(404, "feed_not_found")`.
    The three hand-written lists were folded into one table, `pinned_wire() -> [(AppError,
    &'static str, &'static str); 5]` at `:93` to `:121`, tuple order (variant, server body, wasm
    throw); the FeedNotFound row at `:106` to `:109` carries both literals. `variants() -> [AppError;
    5]` survives at `:126` as `pinned_wire().map(|(error, _, _)| error)`, so there is one list, not
    two. `wasm_envelope_is_pinned_per_variant` destructures `(error, _, wire)` and
    `server_envelope_is_pinned_per_variant` destructures `(error, wire, _)`; `zip` is gone, so the
    truncation hole the task warned about cannot recur. `contract.rs:227` is `ALL_CODES: [&str; 5]`
    with `"feed_not_found"` at index 2, `:241` adds the `errors()` instance
    (`at://did:plc:nobody/app.bsky.feed.generator/gone`, the same canonical instance as `error.rs`),
    and `:257` is the `code_key` arm. Ran `cd server && cargo nextest run`: 109 tests run, 109
    passed, exit 0, including `error::tests::wasm_envelope_is_pinned_per_variant`,
    `error::tests::server_envelope_is_pinned_per_variant`, `error::tests::envelope_round_trips` and
    `mortar-core::contract wire_contract_matches_the_committed_fixture`.
  - *Check result:* the new arm indexes `ALL_CODES[2]`, not `[4]`: the code was inserted mid-array
    to match the enum's declaration order and the change spec's own `MortarErrorCode` enum, and
    `Upstream` was renumbered to `[4]`. The property the check exists for is preserved rather than
    weakened, and both halves of it were exercised rather than argued. The array still must grow (a
    surviving `[&str; 4]` makes the renumbered `ALL_CODES[4]` a constant out-of-bounds index), and
    the assert still fires for the new variant: repointing the arm to `ALL_CODES[3]` in a scratch
    mutation failed `wire_contract_matches_the_committed_fixture` with "the fixture key must equal
    the wire code", and the file was restored (sha256 `843af68b…` before and after). Both pinned
    strings are genuinely pinned, not merely present: mutating the FeedNotFound wasm string's status
    to 403 failed only `wasm_envelope_is_pinned_per_variant`, and mutating its server string failed
    only `server_envelope_is_pinned_per_variant`, which also rules out a tuple-order swap. `error.rs`
    restored (sha256 `d87aafcd…` before and after).
  - *Status:* SATISFIED

- **O2 · The fixture diff contains only the new error entry.**
  - *Claim:* the committed `contract.json` diff adds `errors.feed_not_found.server` and
    `errors.feed_not_found.wasm` and nothing else.
  - *Evidence to collect:* read the diff of
    `server/crates/mortar-core/tests/fixtures/contract.json` line by line. `UPDATE_FIXTURE=1`
    rewrites the file wholesale, so a regeneration on a stale tree would silently drop keys; confirm
    the pre-existing `bricks`, `pages`, `query` and `vocab` objects are byte-identical.
  - *Collected:* `jj diff --git` on the fixture is one hunk, eleven added lines and zero deleted:
    lines 186 to 196, the `feed_not_found` object with its `server` and `wasm` members, sorted
    between `bad_request` and `login_required` (serde_json's `Map` is a `BTreeMap`, so key order is
    sorted and the assert sorts `ALL_CODES` before comparing). 393 lines before, 404 after. Checked
    byte-for-byte rather than by eye: deleting those eleven lines from the committed file reproduces
    the parent revision's file exactly. Parsed both revisions and compared: top-level key sets equal
    (`bricks`, `errors`, `pages`, `query`, `vocab`); `bricks`, `pages`, `query` and `vocab` each
    identical under a canonical dump; all four pre-existing error entries identical; nothing dropped;
    `feed_not_found` the only addition. The regeneration therefore ran on this workspace's tip, so
    tasks 14 and 23 inherit a complete fixture.
  - *Status:* SATISFIED

- **O3 · The TypeScript half follows and the existing guard proves it.**
  - *Claim:* `MortarErrorCode` in `web/src/lib/types.ts` includes `"feed_not_found"`, and
    `contract-check.ts`'s existing `ErrorCodesMatch` passes with **no edit** to that file.
  - *Evidence to collect:* read `types.ts` and confirm the union has five members. Confirm
    `contract-check.ts` is unchanged in the diff. Run `cd web && pnpm check:ci`, expect clean.
  - *Checks:* `ErrorCodesMatch` is `Equal<keyof typeof contract.errors, MortarErrorCode>`, which is
    bidirectional. Confirm removing the new member makes it fail, so the pass is not vacuous.
  - *Collected:* `types.ts:137` to `:142` is a five-member union carrying `"feed_not_found"` third,
    matching the change spec's `MortarErrorCode` `$def` ordering. `jj diff --stat` lists four files
    and `web/src/lib/contract-check.ts` is not among them, so `ErrorCodesMatch` at `:78` is the
    unedited guard it was. `cd web && pnpm check:ci` exits 0.
  - *Check result:* not vacuous. Reverting `MortarErrorCode` to its four-member form (fixture and
    Rust left as committed) failed `pnpm check:ci` with
    `src/lib/contract-check.ts(78,38): error TS2344: Type 'false' does not satisfy the constraint
    'true'`, exit 1. `types.ts` restored (sha256 `c4ee29e9…` before and after).
  - *Status:* SATISFIED

- **O4 · The unconstructed variant is not a lint failure.**
  - *Claim:* no production code constructs `FeedNotFound` yet, and clippy at `-D warnings` does not
    flag it, because `AppError` is public.
  - *Evidence to collect:* run `cd server && cargo clippy --workspace --all-targets -- -D warnings`,
    expect clean.
  - *Collected:* `FeedNotFound` appears at exactly five sites, all of them the declaration or a test
    lane: `error.rs:14`, `:47`, `:106`, `contract.rs:241`, `:257`. No production path constructs it,
    which task 12 does. `cargo clippy --workspace --all-targets -- -D warnings` ran inside
    `just check` and emitted no diagnostics, exit 0.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the wire changed, so `contract.json`, `types.ts` and the spec set agree, and both
    `cargo test` and `tsc` pass in the same commit.
  - *Evidence to collect:* run `just check`. Note the spec-set half lands in task 19; record here
    that `06-wire-contract.md` still describes four codes and that the divergence is scheduled.
  - *Collected:* `just check` exits 0: guard-dashes, guard-autoplay, guard-toolchain, fmt-check,
    guard-wasm, lint (oxlint reports the four pre-existing warnings in `FeedGrid.svelte` and
    `service-worker.ts`, both untouched by this diff; knip clean; clippy clean) and test (cargo
    nextest 109/109, `pnpm check:ci`, vitest 39/39). `model.rs` is untouched, correctly: no response
    shape changed. The regenerated `contract.json` and `types.ts` agree, proved in both directions
    under O3 and O6. Spec set: `.specs/06-wire-contract.md:82` to `:87` still lists four rows and
    `:104` still says `MortarErrorCode` "names only mortar's own four". That is this obligation's
    named residue rather than a miss: plan.md:34 has the canonical pages describe the current branch
    until each spec's merge task runs, plan.md:484 makes it an explicit decision, and plan.md:170
    assigns `06-wire-contract.md` to task 19. `canonical-types.schema.json`'s `MortarErrorCode`
    `$def` is deferred to the same task for the same reason.
  - *Status:* SATISFIED

- **O6 · Reviewable: neither half passes without the other.**
  - *Claim:* `cd server && cargo nextest run` and `cd web && pnpm check:ci` are both green from a
    single checkout of this commit.
  - *Evidence to collect:* run both from one checkout. Then, as a one-off probe, revert the
    `types.ts` line and confirm `pnpm check:ci` goes red; restore it.
  - *Collected:* from this one checkout (`@` at `c313b388`, working copy untouched by the gate),
    `cd server && cargo nextest run` is 109 passed / 0 failed, exit 0, and `cd web && pnpm check:ci`
    exits 0. The interlock was exercised in both directions, not one. TypeScript half removed: the
    four-member `MortarErrorCode` probe under O3 turns `pnpm check:ci` red at `contract-check.ts:78`.
    Rust half removed: restoring the parent revision's `contract.json` while keeping the Rust change
    fails `mortar-core::contract wire_contract_matches_the_committed_fixture`, with the test's own
    "regenerate the fixture" instruction. Both files restored and verified by sha256
    (`c4ee29e9…` for `types.ts`, `cf0d5168…` for `contract.json`); `jj diff --stat` after the probes
    is the same 4 files, 75 insertions, 23 deletions it was before them.
  - *Status:* SATISFIED

## Regression check

- `web/src/service-worker.ts:253` uses `"bad_request" satisfies MortarErrorCode`. Trace: still
  typechecks after the union grew : PRESERVED. Widening a union cannot invalidate a literal
  `satisfies` against it, and `pnpm check:ci` (which does typecheck `.ts`, including
  `service-worker.ts`) exits 0.
- `web/src/lib/state/feed.svelte.ts:200` and `:204` use `satisfies MortarErrorCode`. Trace: both
  still typecheck, and `#fail`'s else branch still catches the new code as `feed-unavailable` until
  task 16 gives it its own arm : PRESERVED. `#fail` at `:197` is an if / else-if / else chain over
  two literals with no exhaustiveness check, so a fifth code cannot break it and falls to
  `this.error = "feed-unavailable"` at `:211`. The file is a `.ts` module, so tsc does parse it
  (`rg` skips it only because it holds one deliberate NUL byte as a cache-key separator at `:1684`,
  which is pre-existing and unchanged).
- Engine-side consumers, traced for the same reason: `mortar-server/src/routes/feed.rs:30` to `:34`
  reads `status_and_code()` and `envelope()` generically, and `mortar-wasm/src/lib.rs:107` reads
  `envelope_with_status()` generically, so neither needed an arm; the only exhaustive matches over
  `AppError` are `status_and_code` and the test-lane `code_key`, and both gained one : PRESERVED.
  The four existing codes' wire bytes are byte-identical in the fixture, checked under O2.

## Residue

- `06-wire-contract.md` now describes four error codes while the fixture carries five. That
  divergence is deliberate and closes in task 19. Record it rather than fixing it here.
  Confirmed present and scheduled: `:82` to `:87` and `:104`, owned by task 19 per plan.md:170.
- `pinned_wire()` is still a hand-written list of variants, so nothing forces a future sixth variant
  to appear in it; what the fold bought is that a variant present there cannot be present without
  both of its wire strings. The chain that forces coverage remains `contract.rs`'s exhaustive
  `code_key`, its constant index into `ALL_CODES`, and the key-set assert, exactly as the task says.
- `BadRequest`'s Display reword is correctly absent: it belongs to task 13, and including it would
  have put a second unrelated wire change into this regeneration.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 are all SATISFIED against evidence collected here rather than reported, including
four scratch mutations that show the new variant's two pinned strings, the fixture key / wire code
assert and the TypeScript union guard all fire; the fixture diff is eleven added lines and nothing
else; and both named regression callers plus the two engine-side consumers are PRESERVED.
