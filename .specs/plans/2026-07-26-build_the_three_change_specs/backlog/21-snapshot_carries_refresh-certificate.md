# Done Certificate · Task 21: snapshot carries the refresh flag

**Task:** [21-snapshot_carries_refresh.md](21-snapshot_carries_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 21. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric. O3 is the
> obligation no signature can enforce; weight it accordingly.

## Definition

DONE(Task 21) is every obligation O1 to O6 below holding, O3b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** The flag reaches a fill that was spawned detached, and provably never reaches an
  extension wave.
- **P2 · Obligations.** Done iff O1 to O3, O3b and O4 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not change behaviour at all in this task: both `feed.rs` call sites pass
  `false`, so the graph wall, the demo wall and the feed wall all behave exactly as before.

## Obligations

- **O1 · The flag is an immutable field on `Snapshot`, not guarded state.**
  - *Claim:* `pub refresh: bool` sits beside `id`, `seed`, `viewer` and `mode` on `Snapshot`, set in
    `Snapshot::new` from a new argument, and is not a field of `Inner`; `test_snapshot` and every
    in-file construction compile with `refresh = false`.
  - *Evidence to collect:* read `algo/snapshot.rs` around `:88`, `:105`, `:235` and `:684`. Confirm
    `refresh` is above the `Inner` boundary. Run `cd server && cargo nextest run -p mortar-core`.
  - *Checks:* resolve where `refresh` is read in `fill.rs`. Reading it must not take the `Inner`
    mutex; if it did, a fill would contend with the mixer on every author.
  - *Status:* unverified

- **O2 · `fan_out_authors` takes the flag explicitly and the waves pass `false`.**
  - *Claim:* `fan_out_authors` has a `refresh: bool` parameter and does not read `snapshot.refresh`;
    `fill::fill` passes `snapshot.refresh` at both call sites and `fill::extend` passes a literal
    `false` at both, with a comment saying why.
  - *Evidence to collect:* run `grep -n 'snapshot.refresh' server/crates/mortar-core/src/algo/fill.rs`
    and confirm exactly two hits, both inside `fill::fill`. Read the four call sites in order and
    confirm they read `snapshot.refresh, snapshot.refresh, false, false`.
  - *Checks:* `fill::extend` passes the same `Arc<Snapshot>` to `fan_out_authors` as `fill::fill`
    does, so a function reading the field could not tell a wave from a fill. This is the exact defect
    the change spec's own parenthetical would have introduced.
  - *Status:* unverified

- **O3 · A refreshed snapshot's waves still read the cache.**
  - *Claim:* a snapshot built with `refresh: true`, running a wave against a wiremock author feed
    whose entry is already warm, issues zero further `getAuthorFeed` calls for that author.
  - *Evidence to collect:* run the named test. Confirm it seeds the `author_feed` cache, constructs
    the snapshot with `refresh: true`, calls `fill::extend`, and asserts the mock hit count for that
    author is unchanged.
  - *Checks:* this is the one rule of the task that no signature enforces. If the test asserts only
    that the wave completed, it does not discharge the obligation.
  - *Status:* unverified

- **O3b · The new test module is gated for wasm32.**
  - *Claim:* the wave test lives in a module gated `#[cfg(all(test, not(target_arch = "wasm32")))]`.
  - *Evidence to collect:* read the module attribute in `src/algo/fill.rs`. Run `just guard-wasm` and
    expect green.
  - *Checks:* `fill.rs` had **no** test module before this task, so there is nothing in the file to
    copy the gating from; `feed.rs:214`, `snapshot.rs:618` and `cohort.rs:123` are the correct
    pattern. `wiremock` and `tokio` are `cfg(not(target_arch = "wasm32"))` dev-dependencies
    (`Cargo.toml:41`) and `guard-wasm` compiles `--all-targets`, so a bare `#[cfg(test)]` here breaks
    the wasm32 build while `cargo nextest`, `lint` and `tsc` all stay green. `just guard-wasm` is the
    only gate that can see it, which is why it is run on its own rather than trusted inside
    `just check`.
  - *Status:* unverified

- **O4 · clippy is clean at the new arity.**
  - *Claim:* `cargo clippy --workspace --all-targets -- -D warnings` passes;
    `fan_out_authors` now has six parameters, under the default `too_many_arguments` threshold of
    seven.
  - *Evidence to collect:* run the command. Count `fan_out_authors`'s parameters.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd server && cargo nextest run`, `cargo clippy --workspace
    --all-targets -- -D warnings`, `just guard-wasm` and `just check`.
  - *Status:* unverified

- **O6 · Reviewable: the four call sites read `true, true, false, false` in intent.**
  - *Claim:* a reviewer reads `fill.rs` and sees `fill::fill` forwarding the snapshot's flag twice
    and `fill::extend` passing `false` twice, then runs the wave test.
  - *Evidence to collect:* the read plus the test run.
  - *Status:* unverified

## Regression check

- `feed.rs:86 ensure_snapshot` and `feed.rs:95 get_or_build` both gained an argument. Trace: both
  pass `false` here, so the graph wall's five in-crate `handle_feed` tests pass unchanged :
  (PRESERVED / REGRESSION)
- `algo/snapshot.rs:684 test_snapshot` is used by the snapshot and mixer tests. Trace: those tests
  pass with `refresh = false` : (PRESERVED / REGRESSION)
- Task 13's feed wall does not build a snapshot. Trace: the feed-wall wiremock test is unaffected :
  (PRESERVED / REGRESSION)

## Residue

- `fresh_seed` is `xxh3(did, unix_millis)` and `ensure_snapshot` runs its `make` closure only on a
  genuine insert, so two cursorless requests for the same DID in the same millisecond share a
  snapshot and the second one's flag is discarded. Recorded in the plan's open questions as prose
  for task 27, not defended in code here.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
