# Change: Drop the dead `snapshot` field from the cursor

**Status:** Merged · **Date:** 2026-07-25 · **Merged:** 2026-07-25 · **Owner:** Ant Stanley · **Target:** Repo-wide

`Cursor` carries three fields and mortar reads two of them. `snapshot` is written
on every cursor mason has ever issued and is never consulted: `handle_feed`
recomputes the snapshot id from the resolved DID, the seed and the mode. This
change removes the field, shrinking every cursor by about thirty bytes and
removing a piece of internal state from a value the client can see and edit.

---

## Motivation

A cursor is an opaque, attacker-writable token that rides in a URL. Every field
in it is either load-bearing or a liability, and `snapshot` is currently neither
useful nor inert: it exposes the mode tag and a hash of the viewer's DID to
anyone who base64-decodes a shared pagination link, for no benefit.

It is also a maintenance hazard. A reader of `algo/cursor.rs` reasonably assumes
the field is consulted somewhere, and a future change that starts trusting it
would be reintroducing a lookup key the engine deliberately derives instead. The
current code is correct; the field is what makes it look otherwise.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/01-domain-model.md`](../../01-domain-model.md) | The `Cursor` entity loses a field |
| [`.specs/06-wire-contract.md`](../../06-wire-contract.md) | Resolve the open question; describe the two-field payload |
| [`.specs/02-feed-engine.md`](../../02-feed-engine.md) | The request-flow diagram's cursor encoding |
| [`.specs/canonical-types.schema.json`](../../canonical-types.schema.json) | `CursorPayload` drops `snapshot` |

---

## Proposed changes

### `.specs/01-domain-model.md` → Entities → Cursor (Modify)

> ### Cursor
>
> The `CursorPayload` is `{seed: u64, offset: usize}`, JSON-serialised and
> base64url (no padding) encoded into the opaque `Cursor` string the client sees.
> It is attacker-writable, so every consumer treats it defensively: a garbage or
> tampered cursor decodes to `None` and falls back to a fresh wall, and the
> offset is added with `checked_add` / `saturating_add`.
>
> `seed` is the load-bearing field. It drives the cohort shuffle and the mixer's
> jitter, so a snapshot evicted mid-scroll rebuilds into a closely-matching wall
> from the same seed and the still-warm per-author caches. The snapshot id itself
> is not carried: `handle_feed` derives it from the resolved DID, the seed and
> the mode, which are all it has ever used.

### `.specs/06-wire-contract.md` → Assumptions and open questions → Open questions (Remove)

> Remove the bullet beginning *`Cursor.snapshot` is written and never read.* The
> question is answered by this change.

### `.specs/06-wire-contract.md` → The endpoint → `cursor` parameter (Modify)

> | `cursor` | no | opaque base64url of `{seed, offset}` | Where to continue; absent starts a fresh wall |

### `.specs/02-feed-engine.md` → Entry point → Request flow (Modify)

> Replace the two `encode{snapshot, seed, offset}` occurrences in the flow diagram
> with `encode{seed, offset}`.

CORRECTED ON MERGE: there was only ever one such occurrence, on the committed
line of the flow diagram. The preview branch writes its cursor as
`cursor(offset 0)` and names no fields, and the decode line already read
`Some{seed, offset}`, which was wrong before this change and is right after it.

---

## Type changes

```json
{
  "$comment": "Fragment for 2026-07-25-drop_snapshot_from_cursor. Replaces CursorPayload in .specs/canonical-types.schema.json on merge. The opaque Cursor string def is unchanged.",
  "$defs": {
    "CursorPayload": {
      "description": "What a decoded cursor carries. Engine-internal: the client never inspects it.",
      "type": "object",
      "required": ["seed", "offset"],
      "additionalProperties": false,
      "properties": {
        "seed": {
          "description": "The seed for the cohort shuffle and the mixer's jitter. Carrying it is what lets an evicted snapshot rebuild deterministically mid-scroll.",
          "type": "integer",
          "minimum": 0
        },
        "offset": {
          "description": "The next item offset within the snapshot's wall.",
          "type": "integer",
          "minimum": 0
        }
      }
    }
  }
}
```

---

## Implementation notes

The change is small and entirely inside the engine. No TypeScript changes are
needed: the client treats the cursor as an opaque string and `types.ts` types it
as one.

```
1. server/crates/mortar-core/src/algo/cursor.rs:8
     Remove the `snapshot` field from `Cursor`. Update the roundtrip test at :35.
     Keep the garbage-is-None test unchanged; it must still pass.

2. server/crates/mortar-core/src/feed.rs:64, :94, :111, :211
     Four Cursor constructions lose their `snapshot:` line. Two of them
     (:64, :211) currently pass the literal "fixture" for the demo wall, which
     disappears with the field. At :94 and :111 the `snap.id.clone()` argument
     goes away, which is the only reason those sites hold the snapshot at all.

3. Regenerate the wire fixture:
     UPDATE_FIXTURE=1 cargo test -p mortar-core --test contract
     The fixture's page cursors are the literal "opaque-cursor-token", so it is
     unlikely to change; regenerate anyway to confirm.

4. just test && just lint
```

### Compatibility

Old cursors decode as unknown-field JSON. `serde_json::from_slice` into a struct
without `deny_unknown_fields` ignores extra fields, so a cursor issued before
this change still decodes correctly afterwards, and a cursor issued after it
still decodes on an old build (the field would be missing and deserialisation
would fail, falling back to a fresh wall). Neither direction can produce a 500;
both are covered by the existing garbage-decodes-to-None behaviour.

Verify this rather than assume it: add a test that decodes a cursor JSON carrying
a stray `snapshot` key and asserts it yields the seed and offset.

---

## Merge plan

1. Apply each `Proposed changes` block to its canonical page; bump each page's
   `**Date:**` to the merge date.
2. Replace `CursorPayload` in `.specs/canonical-types.schema.json` with the
   fragment above.
3. Flip this file's `**Status:**` to `Merged`, add `**Merged:** YYYY-MM-DD`, and
   move it to `.specs/changes/merged/`.
4. Update `.specs/README.md`: remove it from the pending list.

---

## Assumptions and open questions

**Assumptions**

- No consumer outside this repo decodes a mason cursor. There is no published API
  and the field has never been documented as meaningful.
- `serde` ignores unknown fields on `Cursor` (no `deny_unknown_fields`), so old
  cursors keep working. The implementation notes call for a test that proves it.

**Decisions**

- *Remove rather than start validating.* **Drop the field.** Validating a cursor
  against a recomputed snapshot id would add a failure mode (a legitimate cursor
  rejected after a mode or seed derivation change) to guard against a threat that
  does not exist: a forged snapshot id cannot make the engine serve another
  viewer's wall, because the DID comes from `actor`, not from the cursor.
- *No wire version bump.* **The cursor is opaque and neither direction is worse
  than a fresh wall.** A version field would be new state to carry in order to
  protect a token whose failure mode is already benign.
  **Measured on merge**, and it is better than this decision assumed. Only one
  direction degrades at all. An old cursor on a new build does not degrade: serde
  ignores the stray key and returns `Ok`, so a reader mid-scroll across the deploy
  keeps their exact seed and offset. A new cursor on an old build fails with
  ``missing field `snapshot` ``, which `.ok()?` turns into `None`, and the reader
  gets a fresh wall from offset zero. Neither path can produce a 500.

**Open questions**

- *Does the demo wall still need a cursor payload at all?* With `snapshot` gone,
  the demo page's cursor is `{seed: 0, offset}`. The seed is unused for fixtures.
  Open: leave it for uniformity, or is a separate demo cursor shape worth the
  branch?
  **Resolved on merge:** leave it. Both demo sites keep `{seed: 0, offset}`, so
  `demo` pages through exactly the same encode/decode path as a real actor and
  the demo wall keeps testing the code the real wall runs. A demo-only cursor
  shape would be a second branch in `handle_feed` to save eight bytes on a
  fixture page.
