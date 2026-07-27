# 03 - Grout and the Mixer

**Status:** Draft · **Date:** 2026-07-27 · **Owner:** Ant Stanley

Two pure modules decide what the wall looks like. `algo/score.rs` computes
**grout**, a brick's rank within its own kind. `algo/mix.rs` lays bricks one at a
time, choosing the next *kind* by need and then the best brick *within* that
kind. Neither reads a clock or performs IO: `now` is always a parameter, so tests
are exact. The engine that calls them is described in
[02-feed-engine.md](02-feed-engine.md).

---

## Responsibilities

1. Rank bricks against others of the same kind (grout).
2. Decide, for each slot on the wall, which kind the wall currently wants.
3. Keep one author from dominating a screen.
4. Give a live stream the top of the wall.
5. Be deterministic given `(pool, wall, seed, now)`, so that laying 20 then 20
   equals laying 40. Cursor pagination depends on this.

The mixer does **not** own: admission (that is the snapshot's), freshness
enforcement at the door (also the snapshot's), or any notion of the reader.
Neither module runs for a feed wall. A feed generator publishes an order and
mason lays it in that order; re-ranking somebody else's algorithm by grout would
produce a wall that is neither theirs nor mason's.

---

## The grout score

```
grout(brick, now) = decay × boost

decay = 0.5 ^ (age_hours / half_life_hours(brick))
boost = 1 + ln(1 + engagement(brick))
```

Recency dominates and engagement modulates. A ten-half-life age gap is a 2^10
decay gap, and a viral post at 100k likes has a boost of about 12.5, so no amount
of engagement closes a large recency gap. Reposts weigh double a like.

### Per-kind constants

| Kind | Half-life | Hard age window | Engagement signal |
|---|---|---|---|
| Post | 12 h | 72 h | `likes + 2 × reposts` |
| Blog | 3 d | 14 d | none (0) |
| Video, Bluesky | 12 h | 72 h | `likes` |
| Video, archived stream | 14 d | 90 d | `likes` (always 0 upstream) |
| Video, live | (exempt) | (exempt) | `viewerCount` |

Half-lives are deliberately shorter than the medium's shelf life, so the freshest
brick of a kind clearly outranks yesterday's.

The **age window is a hard gate, not a preference**. Decay alone leaves a
week-old post technically eligible, and on a quiet follow graph it will surface.
A Bluesky video ages like the post it is, and this is not incidental: when videos
carried the archived-stream window, stale clips filled the gap left by expired
text posts and the wall ended up 42% video.

A live stream is exempt from the window entirely. "Live" is a fact about the
present, not a claim about a timestamp: a streamer broadcasting on the same
record since March is still broadcasting. Its engagement is its current audience,
and it is the only brick whose signal is being generated while you look at it.

### Untrusted timestamps

`created_at` is author-supplied JSON, so `age_seconds` defends against both
directions:

- More than `MAX_FUTURE_SKEW_SECS = 600` in the future is treated exactly like an
  unparseable date. Clamping a future date to age zero pinned it to the top of
  every wall it touched.
- Within the skew allowance, age clamps to zero: honest clock skew still counts
  as brand new.
- Unparseable or too-far-future dates make `is_fresh` false, so the brick is
  never admitted. Merely scoring it low was not enough; it lingered on every wall
  forever.

### The widened window

`within_age(brick, now, max_age_hours)` is the caller-supplied variant. The glaze
wall uses it at 30 days: it is built from `posts_with_media`, which reaches back
weeks, and the 72-hour post window would throw most of it away. A wider window is
still a window; 40 days out is gone either way.

---

## The mixer

### Kinds and target ratio

```rust
const TARGET: [f64; KINDS] = [0.68, 0.15, 0.09, 0.05, 0.03];
//                            post  blog  bsky  vod   live
pub const KINDS: usize = 5;
```

`kind_index` maps a brick to its slot. The five slots exist because the kinds
have incomparable signals: an archived stream has no engagement at all, so
ranking it against a liked Bluesky video buries it at the bottom of every wall.

### Choosing the next brick

```
lay_next(pool, wall, seed, now):

  recent_authors = last 8 authors on the wall            (AUTHOR_WINDOW = 8)
  counts         = per-kind population of the wall
  need[k]        = TARGET[k] / (counts[k]/laid + 0.05)   scale-free
  scores[i]      = grout(pool[i], now) × jitter(seed, pool[i].id)

  if no live brick is on the wall yet and the pool has one:
      lay it, skipping the kind lottery                  ← the deadline exemption

  leader(kind) = argmax scores[i] among pool[i] of that kind,
                 excluding authors inside the diversity window
  pick         = argmax over kinds of  need[kind] × jitter(seed, "<position>:<kind>")

  if no kind has an eligible leader (every candidate is inside the window):
      fall back to the author holding the FEWEST bricks on the wall,
      breaking ties by score
```

Four properties are worth naming:

- **Need is scale-free.** It is target share over actual share, so it composes
  with grout by multiplication rather than needing a tuned additive weight. The
  `+ 0.05` keeps a kind with zero laid bricks from producing an infinite need.
- **Jitter is deterministic.** `jitter(seed, id)` hashes the id with the seed into
  `[0.85, 1.15]`. The wall feels different across seeds and is identical within
  one. The kind lottery uses a *positional* jitter key (`"<position>:<kind>"`), so
  two adjacent slots do not resolve the same way.
- **Scores are computed once per `lay_next`.** `grout` parses a date string, and
  scoring inside the comparators re-parsed each brick's date many times per call;
  the preview loop re-lays a pool of hundreds every 350 ms. `now` and `seed` are
  fixed across a call, so each score is a constant, computed once and indexed by
  pool position. Ranking and determinism are unchanged.
- **The diversity window is hard while it can be honoured.** When it genuinely
  cannot (a wall built from one author's feed), the fallback picks the author with
  the fewest bricks laid, not the highest score. Scoring again would just re-pick
  the dominant author, which is how a first page ended up belonging to one person.

### Live jumps the queue

A live stream is the only brick on the wall with a deadline. The first one the
pool can offer skips the kind lottery and opens the wall. Any others are spaced
out by the need factor, which is rare: it means two people you follow are
streaming at once.

### Laying

`lay(pool, wall, count, seed, now)` calls `lay_next` up to `count` times, pushing
each result onto the wall and stopping when the pool runs dry. Laying is the
moment the pool's composition becomes the wall's composition, which is why the
first page defers it until the slow sources have had a chance to contribute (see
[02](02-feed-engine.md), the mix deadline).

---

## Determinism and pagination

```
lay(pool, wall, 20, seed, now) ; lay(pool, wall, 20, seed, now)
  ≡ lay(pool, wall, 40, seed, now)
```

`lay_next` reads only the pool, the wall so far, the seed, and `now`. Nothing
carries over between calls. This is what makes an opaque `{seed, offset}` cursor
sufficient: a snapshot evicted mid-scroll rebuilds from the same seed and the
still-warm per-author caches, and the wall it rebuilds matches closely enough
that continuity holds. Determinism of the jitter is exact; continuity of the pool
is best-effort, because the caches behind it expire on their own schedules.

---

## Implementation layout

```
server/crates/mortar-core/src/algo/
  score.rs   half_life_hours · max_age_hours · is_fresh · within_age
             is_live · created_at · author_key · grout
  mix.rs     kind_index · jitter · need · lay_next · lay
```

Both are `#[cfg(test)]`-tested in place with hand-built bricks and a fixed `now`.
The tests pin the behaviours that were regressions: decay halving at the
half-life, engagement never beating a big recency gap, reposts weighing double,
a future-dated brick sinking instead of pinning, and an unparseable date being
stale rather than immortal.

---

## Assumptions and open questions

**Assumptions**

- Like and repost counts from the AppView are non-negative and may be absent
  (they default to zero). They are summed with saturating arithmetic, because a
  hostile pair near `u64::MAX` would overflow a plain sum.
- Archived streams carry no like count upstream, so their engagement is always
  zero and they rank by recency alone within their kind.
- A follow graph large enough to fill the pool exists for most viewers; the
  fallback path (one author's bricks only) is the degenerate case.

**Decisions**

- *Two-step selection.* **Kind first by need, then brick by grout.** One global
  ranking compares engagement-boosted posts against blogs with no engagement
  signal at all, and posts win every slot.
- *Multiplicative need.* **`target / (actual + 0.05)`.** Scale-free, so it
  composes with grout without a tuned weight, and the epsilon bounds the
  cold-start case.
- *Hard age windows.* **72 h posts, 14 d blogs, 90 d archived streams.** Decay
  alone leaves stale content eligible on a quiet graph, and mason is for what the
  people you follow are making, present tense.
- *A Bluesky video ages like a post.* **72 h, 12 h half-life.** With the stream
  window it filled the gap left by expired text posts and the wall reached 42%
  video.
- *Live exempt from the window.* **Its timestamp is not a claim about the
  present.** A livestream record is created once and reused for months.
- *Future-dated bricks sink.* **Beyond 600 seconds of skew, treated as
  unparseable.** Clamping to age zero pinned author-controlled timestamps to the
  top of every wall.
- *Diversity fallback picks the quietest author.* **Fewest bricks laid, ties by
  score.** Re-scoring re-picks the dominant author, which is the bug it exists to
  fix.
- *Scores precomputed per call.* **One pass, indexed by pool position.** Identical
  ranking, and it takes the repeated RFC3339 parse out of the comparators that the
  350 ms preview loop hammers.

**Open questions**

- *Target ratio.* 68/15/9/5/3 is a judgement, not a measurement. Open: is there a
  signal (dwell, click-through) mason could ever collect that would justify
  moving it, given it collects nothing today?
- *Blog ranking.* Blogs have no engagement input, so within their kind they are
  ordered by recency and jitter alone. Open: is there a defensible signal, or is
  this correct for the medium?
- *`AUTHOR_WINDOW = 8` versus page size 24.* Three of one author's bricks can
  appear on a single page. Open only if it reads as repetition in practice.
