# Done Certificate · Task 09: cursor two shapes

**Task:** [09-cursor_two_shapes.md](09-cursor_two_shapes.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 09. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 09) is every obligation O1 to O5 below holding, each backed by the evidence it names.

## Premises

- **P1 · Goal.** A feed cursor round-trips, and every cursor mason has ever issued still decodes to
  the graph shape.
- **P2 · Obligations.** Done iff O1 to O5 all hold; O5 is the Reviewable item.
- **P3 · Invariants.** Must not break mid-scroll readers across the deploy: a cursor issued before
  this change carries `{seed, offset}` and possibly a stray `snapshot` key, and must still land the
  reader at their offset rather than on a fresh wall.

## Obligations

- **O1 · All three pre-existing cursor tests still pass, and their data is unchanged.**
  - *Claim:* the file holds **three** tests, not two: `roundtrip` (`cursor.rs:32`),
    `a_cursor_carrying_the_removed_snapshot_key_still_decodes_to_its_seed_and_offset` (`:46`) and
    `garbage_is_none` (`:59`). All three pass. `garbage_is_none` is unchanged in full. The other two
    keep their inputs and their expected `(seed, offset)` of `(42, 96)`, and their **only** edit is
    respelling `Cursor { .. }` as `Cursor::Wall { .. }`.
  - *Evidence to collect:* run `cd server && cargo nextest run -p mortar-core cursor`. Diff
    `algo/cursor.rs`'s test module against the previous revision. Confirm `garbage_is_none` has a
    zero-line diff, and that the diffs on the other two are the constructor spelling and nothing
    else: the base64 literal in the legacy test and both `42` / `96` values must be untouched.
  - *Checks:* do **not** read a zero-line diff on `roundtrip` or the legacy test as success. Both
    write struct expressions (`:33` and `:51`), which cannot compile against an enum, so an unedited
    body means either the change was not made or `Cursor` was contorted (a struct wrapping an enum,
    a type alias, a `Default`) to keep them compiling. Separately: an untagged enum decodes
    structurally, so `{"seed":42}` must still be `None`, which it is only because `Wall` requires
    both fields and `Feed` requires `feed`. Confirm neither variant gained a `#[serde(default)]`
    that would make a partial object match.
  - *Status:* unverified

- **O2 · Both shapes round-trip and the demo re-encode still produces a graph cursor.**
  - *Claim:* a `Feed` cursor round-trips through encode and decode; a `Wall` cursor still does; the
    demo preview re-encode at `feed.rs:64` still produces a `Wall` cursor.
  - *Evidence to collect:* run the named round-trip tests. Read `feed.rs` around `:57` and `:64` and
    confirm the demo branch reads an offset from the `Wall` arm and re-encodes a `Wall`.
  - *Checks:* `Feed` is declared first in the enum, which is load-bearing for the untagged decode.
    Confirm the declaration order and the comment saying so.
  - *Status:* unverified

- **O3 · The wrong shape on the wrong path degrades rather than panics.**
  - *Claim:* a `Feed` cursor arriving on the graph path yields a fresh wall, not a panic and not a
    500; on the demo path it yields offset 0.
  - *Evidence to collect:* find and run the two negative-space tests by name. Read `feed.rs:76` and
    `feed.rs:57` and confirm both match on the arm rather than unwrapping a field.
  - *Status:* unverified

- **O4 · `Cursor` is named only where it should be.**
  - *Claim:* no module outside `algo/cursor.rs` and `feed.rs` names `Cursor`.
  - *Evidence to collect:* run `grep -rn 'Cursor' server/crates/mortar-core/src --include=*.rs` and
    confirm every hit is in one of the two files (or is an unrelated identifier such as a `cursor`
    field on a response type).
  - *Status:* unverified

- **O5 · Meets the repo definition of done, and reviewable: the same bytes still decode to the same wall.**
  - *Claim:* negative-space tests cover the wrong-shape cases, the gates are green, and the proof of
    backward compatibility is that the legacy test still feeds
    `{"snapshot":"wall-0123456789abcdef","seed":42,"offset":96}` and still expects seed 42 and
    offset 96.
  - *Evidence to collect:* run `just guard-wasm` and `just check`. Then read the diff of
    `algo/cursor.rs` and confirm the changes are the enum, the signatures, `::Wall` on two
    constructor spellings, and new tests. Nothing else.
  - *Checks:* the obligation is on the data, not on the diff being empty. A reviewer who requires an
    untouched test module here is asking for something the type change makes impossible; a reviewer
    who accepts a changed base64 literal or a changed expected offset has lost the guarantee that a
    reader mid-scroll survives the deploy.
  - *Status:* unverified

## Regression check

- `feed.rs:51` decodes an incoming cursor. Trace: a base64url of
  `{"snapshot":"wall-0123456789abcdef","seed":42,"offset":96}` still yields seed 42 and offset 96 :
  (PRESERVED / REGRESSION)
- `feed.rs:190 demo_page` and `feed.rs:102` both encode cursors. Trace: both still produce a `Wall`
  shape, so a reader mid-scroll on a graph wall is unaffected : (PRESERVED / REGRESSION)

## Residue

- A future field added to either shape could make one start matching the other. Not an obligation
  here; the plan records the mitigation (keep `feed` required, keep `Feed` first, add a round-trip
  test for both shapes with any new field).

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
