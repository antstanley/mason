# Task 09 · cursor two shapes

**Plan:** [plan.md](../plan.md) · **Certificate:** [09-cursor_two_shapes-certificate.md](09-cursor_two_shapes-certificate.md)

**Implements:** [`changes/2026-07-26-lay_a_bluesky_feed.md`](../../../changes/2026-07-26-lay_a_bluesky_feed.md) §Proposed changes → `01-domain-model.md` → Cursor, and the `CursorPayload` fragment in §Type changes; implementation note 6. Targets [`01-domain-model.md`](../../../01-domain-model.md) §Entities → Cursor.
**Depends on:** none
**Produces:** a feed cursor round-trips, and every cursor mason has ever issued still decodes to the graph shape.
**Pointers:** `server/crates/mortar-core/src/algo/cursor.rs:8` (the struct), `:15` (`encode`), `:21` (`decode`). **Three** existing tests, not two: `:32` (`roundtrip`, whose body writes the struct expression `Cursor { seed: 42, offset: 96 }` at `:33`), `:46` (the legacy stray-`snapshot`-key test, whose body writes `Some(Cursor { seed: 42, offset: 96 })` at `:51`), `:59` (`garbage_is_none`, including the `{"seed":42}` case, which names no constructor at all). Three consumers, not one: `feed.rs:51` (decode), `feed.rs:57` and `:64` (the **demo** branch reads `decoded.map(|c| c.offset)` and re-encodes a `Cursor { seed: 0, offset }`), `feed.rs:76`, `:90`, `:103` (the graph branch), `feed.rs:202` (`demo_page`).

## Steps

- [ ] Turn `Cursor` into `#[serde(untagged)] enum { Feed { feed: String }, Wall { seed: u64, offset: usize } }`, with `Feed` declared **first** and a comment saying the order is load-bearing.
- [ ] Update `encode` and `decode` signatures and every one of the three consumers, including the demo branch at `feed.rs:57` and `:64`, which the change spec's note 6 names as the third call site and asks for a test of.
- [ ] Make a `Feed` cursor arriving on the graph path yield a fresh wall rather than a panic or a 500, and on the demo path yield offset 0.
- [ ] Add round-trip tests for both shapes, and a test that a feed cursor handed to the demo wall lays from offset 0.

## Definition of done

- [ ] All three existing cursor tests still pass, and what is **invariant** is stated precisely, because two of them cannot compile unedited: `roundtrip` at `:32` and `a_cursor_carrying_the_removed_snapshot_key_still_decodes_to_its_seed_and_offset` at `:46` each write a `Cursor { .. }` struct expression, which does not exist once `Cursor` is an enum. The invariant is the **data**, not the diff: the same base64 input in the legacy test still decodes to seed 42 and offset 96, `roundtrip` still round-trips seed 42 and offset 96, and the only permitted edit to either body is respelling the constructor as `Cursor::Wall { .. }`. `garbage_is_none` at `:59` names no constructor and must be unchanged **in full**, including its `{"seed":42}` case; that one is the structural guard on the untagged decode and an edit to it is a red flag rather than a rebase.
- [ ] A feed cursor round-trips, a graph cursor still round-trips, and the demo preview re-encode still produces a `Wall` cursor.
- [ ] No module outside `algo/cursor.rs` and `feed.rs` names `Cursor`, proven by grep.
- [ ] Meets the repo definition of done (negative-space tests for the wrong-shape-on-the-wrong-path cases, `just guard-wasm` and `just check` green).
- [ ] Reviewable: run `cd server && cargo nextest run -p mortar-core cursor`, then read the diff of the test module. The proof of backward compatibility is that the legacy test still feeds the byte string `{"snapshot":"wall-0123456789abcdef","seed":42,"offset":96}` and still expects seed 42 and offset 96, with `garbage_is_none` untouched. It is **not** an empty diff: two bodies necessarily gain `::Wall`, and a diff that shows otherwise means the type was contorted to keep a struct expression compiling.

## Open questions

- An untagged enum decodes structurally, so a field added to either shape later could make one start matching the other. Keeping `feed` required and `Feed` first is the guard; any future cursor field lands with a round-trip test for both shapes.
