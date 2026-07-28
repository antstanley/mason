# Task 21 · snapshot carries the refresh flag

**Plan:** [plan.md](../plan.md) · **Certificate:** [21-snapshot_carries_refresh-certificate.md](21-snapshot_carries_refresh-certificate.md)

**Implements:** [`changes/merged/2026-07-26-refresh_the_wall.md`](../../../changes/merged/2026-07-26-refresh_the_wall.md) §Proposed changes → `01-domain-model.md` → Entities → Snapshot, and → `02-feed-engine.md` → Refresh (the idempotence paragraph); implementation notes 2 and 3. Targets [`01-domain-model.md`](../../../01-domain-model.md) §Entities → Snapshot.
**Depends on:** 13, 20
**Produces:** the flag reaches a fill that was spawned detached, and provably never reaches an extension wave.
**Pointers:** `snapshot.rs:88` (the `Snapshot` struct: `id`, `seed`, `viewer`, `mode` are `pub` fields, and `Inner` at `:235` is the mutex-guarded half the `01` state table actually lists), `:105` (`Snapshot::new`), `:346` (`ensure_snapshot`), `:375` (`get_or_build`), `:684` (`test_snapshot`). `fill.rs:56` and `:89` (the two `fill::fill` call sites), `:151` and `:153` (the two `fill::extend` call sites), `:219` (`fan_out_authors`). `feed.rs:86` and `:95` (the two call sites task 13 has already rewritten around).

## Steps

- [ ] Add `pub refresh: bool` to `Snapshot` beside `id`, `seed`, `viewer` and `mode`, set in `Snapshot::new` from a new argument. It is immutable for the snapshot's life and is **not** a field of `Inner`, so it takes no lock to read.
- [ ] Give `ensure_snapshot` and `get_or_build` a `refresh: bool` argument and forward it into the `get_or_insert_with` closure. Both `feed.rs` call sites pass a literal `false` in this task.
- [ ] Give `fan_out_authors` an **explicit** `refresh: bool` parameter rather than having it read `snapshot.refresh`. `fill::extend` passes the same `Arc<Snapshot>`, so a function that read the field could not tell a wave from a fill.
- [ ] Pass `snapshot.refresh` from `fill::fill` at both call sites, and a literal `false` from `fill::extend` at both, with a comment saying why: a wave asks authors this wall has never asked, so there is nothing cached to bypass, and honouring the flag per wave would multiply a refresh's cost by the length of the scroll.
- [ ] Add the test that no signature can enforce: build a snapshot with `refresh: true`, run a wave against a wiremock author feed whose entry is already warm, and assert zero further `getAuthorFeed` calls for that author. Put it in a module gated `#[cfg(all(test, not(target_arch = "wasm32")))]`, matching `feed.rs:214`, `snapshot.rs:618` and `cohort.rs:123`. **`fill.rs` has no test module at all today** (`grep -n 'cfg(test)' src/algo/fill.rs` is empty), so this is the same first-module-in-the-file situation task 20 was warned about: `wiremock` and `tokio` are `cfg(not(target_arch = "wasm32"))` dev-dependencies at `Cargo.toml:41` and `just guard-wasm` compiles `--all-targets`, so a bare `#[cfg(test)]` here breaks the wasm32 build without breaking a single test.

## Definition of done

- [ ] `Snapshot.refresh` is a `pub` immutable field, not guarded state inside `Inner`, and `test_snapshot` and every in-file `Snapshot::new` construction compile with `refresh = false`.
- [ ] `fan_out_authors` takes the flag explicitly; `grep -n 'snapshot.refresh' src/algo/fill.rs` shows it only in `fill::fill`, never in `fill::extend` or `fan_out_authors`.
- [ ] The wave test above is present and green; it is the only thing standing between a refreshed snapshot and re-reading the cohort on every scroll wave.
- [ ] clippy is clean at `-D warnings`: `fan_out_authors` now has six parameters, still under the default `too_many_arguments` threshold of seven.
- [ ] The new test module in `fill.rs` is gated `#[cfg(all(test, not(target_arch = "wasm32")))]`, and `just guard-wasm` is run as a first-class check rather than folded into `just check`, because it is the only gate that can see a wiremock module.
- [ ] Meets the repo definition of done (`cargo nextest run`, `cargo clippy --workspace --all-targets -- -D warnings`, `just guard-wasm` and `just check` all green).
- [ ] Reviewable: read `fill.rs` and confirm the four call sites read `snapshot.refresh, snapshot.refresh, false, false` in that order, then run the wave test.
