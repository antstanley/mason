# Done Certificate · Task 21: snapshot carries the refresh flag

**Task:** [21-snapshot_carries_refresh.md](21-snapshot_carries_refresh.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Status:* SATISFIED. `pub refresh: bool` is `snapshot.rs:103`, between `mode` (`:95`) and
    `created` (`:107`), and so above the `inner: Mutex<Inner>` boundary at `:108`; `Inner`
    (`:244` to `:270`) has no such field. `Snapshot::new` (`:113`) takes it fifth and stores it
    at `:126`. The two in-file constructions are `ensure_snapshot`'s `get_or_insert_with`
    closure (`:374`, forwards its argument) and the new `for_test` (`:650`); `test_snapshot`
    (`:723`) now delegates to `for_test(id, "did:plc:viewer", seed, mode, false)`.
    `cargo nextest run -p mortar-core` → 148 tests run, 148 passed.
    Check: the reads at `fill.rs:67` and `:104` are bare field reads through `Arc<Snapshot>`
    (no `fn refresh` exists on `Snapshot`, so there is no method/field ambiguity), and take no
    lock, so a fill never contends with the mixer to learn the flag.

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
  - *Status:* SATISFIED. `fan_out_authors` (`fill.rs:250`) carries `refresh: bool` at `:255`,
    between `deep_media` and `keep`, and its body reads only that parameter (`:266`, `:268`);
    the snapshot it also holds is never asked for the flag. The four call sites in order:
    `:62-70` (glaze fill) `snapshot.refresh`, `:103-106` (wall posts fill) `snapshot.refresh`,
    `:175` (glaze wave) literal `false`, `:178` (wall posts wave) literal `false`, under the
    comment at `:167-172` saying why. `grep -n 'snapshot.refresh' fill.rs` returns three lines,
    not the two the authored evidence predicted: `:28` is `fill`'s own doc comment, `:67` and
    `:104` the call sites. All three are `fill::fill`'s, none in `extend` or `fan_out_authors`,
    so the DoD item ("only in `fill::fill`, never in `fill::extend` or `fan_out_authors`")
    holds; the drift is in the certificate's count, not the code.
    Check: `extend` (`:145`) does hand `fan_out_authors` the same `Arc<Snapshot>` `fill` (`:33`)
    does, confirmed by reading both, which is exactly why the parameter and O3 exist.

- **O3 · A refreshed snapshot's waves still read the cache.**
  - *Claim:* a snapshot built with `refresh: true`, running a wave against a wiremock author feed
    whose entry is already warm, issues zero further `getAuthorFeed` calls for that author.
  - *Evidence to collect:* run the named test. Confirm it seeds the `author_feed` cache, constructs
    the snapshot with `refresh: true`, calls `fill::extend`, and asserts the mock hit count for that
    author is unchanged.
  - *Checks:* this is the one rule of the task that no signature enforces. If the test asserts only
    that the wave completed, it does not discharge the obligation.
  - *Status:* SATISFIED. `algo::fill::refresh_tests::a_wave_of_a_refreshed_wall_never_re_reads_an_author_feed`
    (`fill.rs:437-471`) warms the author's entry with one ordinary `author_feed_cached` read
    (baseline 1), mounts a newer answer so a wave that DID refresh would succeed and be caught
    rather than merely miss a mock, builds `for_test("s", VIEWER, 1, Mode::Wall, true)`, runs
    `extend`, and asserts three things: the wave really did ask about the author
    (`fanned().contains(AUTHOR)`, so zero reads is not vacuous), the getAuthorFeed count is
    still 1, and the cached entry is still the older rkey. Ran it: PASS, and PASS on 15
    consecutive runs.
    Check: the test does far more than assert the wave completed, and I broke it to prove it.
    Flipping both of `extend`'s literal `false`s to `snapshot.refresh` fails this test and only
    this test (`left: 2, right: 1`); flipping `fill`'s two `snapshot.refresh` to `false` fails
    only `a_refreshed_walls_fill_re_reads_the_author_feed` (`left: 1, right: 2`). The workspace
    was restored byte-identically after each mutation (sha1 5cca23f4…, `jj diff --stat`
    unchanged).

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
  - *Status:* SATISFIED. `fill.rs:302` reads `#[cfg(all(test, not(target_arch = "wasm32")))]`
    above `mod refresh_tests`, with a comment at `:294-301` saying why. `just guard-wasm` run on
    its own: green (`cargo check -p mortar-core -p mortar-wasm --target wasm32-unknown-unknown
    --all-targets`).
    Check: I confirmed the gate is load-bearing rather than decorative. Weakened to a bare
    `#[cfg(test)]`, `just guard-wasm` fails (E0432/E0433, "could not compile mortar-core (lib
    test)", recipe exit 101) while `cargo nextest run` stays 148/148 green: exactly the failure
    mode the obligation names. Restored, and guard-wasm re-run green.

- **O4 · clippy is clean at the new arity.**
  - *Claim:* `cargo clippy --workspace --all-targets -- -D warnings` passes;
    `fan_out_authors` now has six parameters, under the default `too_many_arguments` threshold of
    seven.
  - *Evidence to collect:* run the command. Count `fan_out_authors`'s parameters.
  - *Status:* SATISFIED. `cargo clippy --workspace --all-targets -- -D warnings` → clean, no
    warnings emitted. `fan_out_authors` (`fill.rs:250-257`) takes six: `state`, `snapshot`,
    `cohort`, `deep_media`, `refresh`, `keep`, one under the default threshold, and
    `grep -rn 'too_many_arguments\|#\[allow' src/algo/` is empty, so the arity is genuinely
    under the lint rather than silenced.

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green.
  - *Evidence to collect:* run `cd server && cargo nextest run`, `cargo clippy --workspace
    --all-targets -- -D warnings`, `just guard-wasm` and `just check`.
  - *Status:* SATISFIED. Run in the workspace by this gate, not taken from the report:
    `cargo nextest run` → 148 tests run, 148 passed, 0 skipped (146 before this task, plus the
    two new ones); `cargo clippy --workspace --all-targets -- -D warnings` → clean;
    `just guard-wasm` → green; `just check` → exit 0 (guard-dashes, guard-autoplay,
    guard-toolchain, fmt-check, guard-wasm, wasm, lint = oxlint + knip + clippy, and test = 148
    rust + tsc both projects + 45 vitest). No changeset was added and none is owed: both
    `feed.rs` sites pass a literal `false`, so nothing user-visible changes in this task.

- **O6 · Reviewable: the four call sites read `true, true, false, false` in intent.**
  - *Claim:* a reviewer reads `fill.rs` and sees `fill::fill` forwarding the snapshot's flag twice
    and `fill::extend` passing `false` twice, then runs the wave test.
  - *Evidence to collect:* the read plus the test run.
  - *Status:* SATISFIED. Exercised: read `fill.rs` top to bottom and the four call sites do read
    `snapshot.refresh` (`:67`), `snapshot.refresh` (`:104`), `false` (`:175`), `false` (`:178`)
    in that order, the two fills first and the two waves after. Then ran the wave test
    (`cargo nextest run -E 'test(algo::fill)'`) → 2 passed, 146 skipped.

## Regression check

- `feed.rs:86 ensure_snapshot` and `feed.rs:95 get_or_build` both gained an argument. Trace: both
  pass `false` here, so the graph wall's five in-crate `handle_feed` tests pass unchanged :
  **PRESERVED**. Now `feed.rs:184` (preview) and `:194` (freeze/normal), each a literal `false`
  with a comment saying no front parses `?refresh=` yet. Traced: `false` → `Snapshot { refresh:
  false }` → `fill` passes `false` at both fan-outs → `author_feed_cached(.., false)` consults
  the five-minute entry exactly as before. `handle_feed`'s signature is untouched, so no front
  crate changed and nothing parses a query parameter: the whole diff is three engine files.
- `algo/snapshot.rs:684 test_snapshot` is used by the snapshot and mixer tests. Trace: those tests
  pass with `refresh = false` : **PRESERVED**. Now `:723`, delegating to
  `for_test(id, "did:plc:viewer", seed, mode, false)`, which is `Snapshot::new` plus the one new
  field. All 148 tests pass, the snapshot and mixer modules among them.
- Task 13's feed wall does not build a snapshot. Trace: the feed-wall wiremock test is unaffected :
  **PRESERVED**. `handle_feed` returns into `feed_wall` at `feed.rs:132`, ahead of every snapshot
  call; `feed::tests::a_feed_cursor_on_the_graph_wall_lays_a_fresh_wall` and the four
  `sources::fetch::feed_page_tests` all pass.
- Task 20's `refresh_fallback` and the two readers : **PRESERVED**. `sources/fetch.rs` is not in
  the diff at all (`jj diff --stat` is three files), and its five `refresh_tests` pass.
- The wasm32 build : **PRESERVED**. `just guard-wasm` green, and green again after the bare-gate
  counterfactual was reverted.

## Residue

- `fresh_seed` is `xxh3(did, unix_millis)` and `ensure_snapshot` runs its `make` closure only on a
  genuine insert, so two cursorless requests for the same DID in the same millisecond share a
  snapshot and the second one's flag is discarded. Recorded in the plan's open questions as prose
  for task 27, not defended in code here.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1, O2, O3, O3b, O4, O5 and O6 are all SATISFIED on evidence this gate collected itself,
the load-bearing one (O3) doubly so because both mutations were applied and each failed its own
test and only its own, and every regression line traces PRESERVED, the diff being three engine
files whose two `feed.rs` call sites still pass a literal `false`.

Two notes that change no status. First, a small drift: the authored evidence for O2 predicts
"exactly two hits" from `grep -n 'snapshot.refresh' fill.rs` and there are three, the extra being
`fill`'s own doc comment at `:28`. The DoD item is the contract and it says "only in `fill::fill`,
never in `fill::extend` or `fan_out_authors`", which holds. Second, the Residue paragraph below
remains accurate and unaddressed by design: `ensure_snapshot` consults `refresh` only on the
insert, so two cursorless requests for the same DID in the same millisecond share a snapshot and
the second flag is discarded. That is the same property that makes a refresh idempotent per wall,
which is what `02-feed-engine.md` asks for; it is prose for task 27, not a defect here.
