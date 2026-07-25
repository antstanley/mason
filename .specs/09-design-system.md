# 09 - Design System

**Status:** Draft · **Date:** 2026-07-25 · **Owner:** Ant Stanley

mason's visual system is a Tailwind v4 `@theme` block in `web/src/app.css` plus a
short base layer. There is no component library and no token package: the theme
block is the token set, and Tailwind generates the utilities from it. This page
defines the tokens, the schemes, the motion policy, and the accessibility
conformance target. The components that consume them are in
[08-wall-and-bricks.md](08-wall-and-bricks.md).

---

## Responsibilities

1. Define one token set, consumed as Tailwind utilities, for both colour schemes.
2. Keep the chrome quiet enough that bricks carry the colour.
3. Give every brick kind a recognisable accent, with text-safe variants where the
   display accent fails contrast.
4. Define one focus language and one motion policy, both honouring
   `prefers-reduced-motion`.

---

## Naming

The material metaphor is the naming system, and it is deliberate: **brick** is a
content card, **mortar** is the feed engine, **grout** is the ranking score,
**kiln** is the dark scheme's fired-clay tone, **plaster** and **chalk** are the
light surfaces.

Card is the web name for a brick. The Rust engine models content as `Brick`; the
Svelte renderers are `*Card.svelte`. The two vocabularies are deliberate, not
drift: a brick is the model, a card is the rendered brick.

Voice is lowercase, brick-punning and brief. No em dashes anywhere, in UI copy,
code comments or commits; `just guard-dashes` enforces it across tracked source
and docs.

---

## Colour tokens

All colours are `oklch`, so lightness is perceptual and the contrast reasoning
below holds.

### Surfaces

| Token | Value | Use |
|---|---|---|
| `--color-plaster` | `oklch(0.955 0.004 260)` | Light page background: mortar white |
| `--color-plaster-deep` | `oklch(0.9 0.006 260)` | Recessed light surfaces |
| `--color-chalk` | `oklch(0.985 0.002 260)` | Light card face |
| `--color-kiln` | `oklch(0.25 0.03 50)` | Dark card face: fired clay |
| `--color-kiln-deep` | `oklch(0.2 0.028 50)` | Dark page background |
| `--color-ink` | `oklch(0.28 0.02 40)` | Body text: warm ink on cool stone |

**The field between bricks is mortar: cool stone, chroma near zero.** Warmth
lives in the ink, the accents and the content, never in the body background. The
cream background was on mason's own anti-reference list.

### Brick accents

Each kind gets a display accent and, where the display accent fails contrast as
text, a deeper ink variant.

| Kind | Display | Text-safe |
|---|---|---|
| Post | `--color-brick-post` `oklch(0.66 0.15 237)` sky | `--color-brick-post-ink` `oklch(0.48 0.14 237)` |
| Blog | `--color-brick-blog` `oklch(0.7 0.19 45)` tangerine | `--color-brick-blog-ink` `oklch(0.47 0.16 45)` |
| Video | `--color-brick-video` `oklch(0.58 0.22 295)` violet | `--color-brick-video-ink` `oklch(0.45 0.2 295)`, `--color-brick-video-bright` `oklch(0.74 0.17 295)` for dark |

The display accents are for borders, fills and tints. The `-ink` variants exist
because the display values fall short of AA on light tints and surfaces, which
was measured in the July 2026 critique rather than assumed.

### Live

| Token | Value | Use |
|---|---|---|
| `--color-live` | `oklch(0.55 0.22 25)` | The LIVE badge fill (white text, 5.4:1) and ink on chalk (5.2:1) |
| `--color-live-bright` | `oklch(0.75 0.16 25)` | Ink on kiln in the dark scheme |

This is the one hot signal on an otherwise unhurried wall, and it is not an
urgency mechanic: a livestream really is happening while you look at it, and it
really will be gone tomorrow. The obvious brighter red failed both contrast
checks, which is why the token is this deep.

### Pops

| Token | Value | Use |
|---|---|---|
| `--color-pop-pink` | `oklch(0.68 0.24 350)` | Focus ring in dark, input focus border |
| `--color-pop-pink-deep` | `oklch(0.52 0.22 350)` | Primary CTA fill (white text), focus ring in light |
| `--color-pop-pink-quiet` | `oklch(0.51 0.16 350)` | The persistent header switcher chip (white text, 6.3:1) |
| `--color-pop-lime` | `oklch(0.85 0.21 128)` | Blog tag chips |
| `--color-pop-sun` | `oklch(0.85 0.17 90)` | Avatar fallback |

`pop-pink-quiet` is `pop-pink-deep` with the chroma dialled down, so the
persistent switcher sits quieter than the one-shot CTA.

### Shape and elevation

| Token | Value |
|---|---|
| `--radius-card` | `1.25rem` |
| `--shadow-brick` | `0 2px 0 0 oklch(0 0 0 / 0.12)` |
| `--shadow-brick-lift` | `0 10px 24px -8px oklch(0 0 0 / 0.28)` |

`shadow-brick` is a hard 2px offset with no blur: a brick sits on the wall, it
does not float above it. `shadow-brick-lift` is the hover state.

### Type

| Token | Stack |
|---|---|
| `--font-display` | Bricolage Grotesque, Avenir Next, system-ui, sans-serif |
| `--font-sans` | system-ui, -apple-system, sans-serif |

Bricolage Grotesque is loaded from Google Fonts at weights 400, 700 and 800 with
`display=swap`, preconnected in the layout head. Body text is the system stack:
no webfont blocks the wall.

---

## Colour schemes

There is no theme toggle. The scheme follows `prefers-color-scheme`, applied
through media queries in `app.css`, and the shell declares both `theme-color`
values so the browser chrome matches.

| | Light | Dark (kiln) |
|---|---|---|
| Page | `plaster` | `kiln-deep` |
| Card | `chalk` | `kiln` |
| Text | `ink` | `chalk` |
| Focus ring | `pop-pink-deep` | `pop-pink` |
| Placeholder | `ink` at 70% | `chalk` at 70% |

Components carry `dark:` variants at the call site; there is no separate dark
token set beyond the `-bright` accent variants that exist for the cases where the
light-mode ink variant is too dark on kiln.

---

## Focus

One focus language, everywhere:

```css
outline: 3px solid var(--color-pop-pink-deep);
outline-offset: 2px;
transition-property: none;
```

It lives in `@layer base` under `:where(…)`, and both details are load-bearing. A
component still needs to override the offset with a utility, because cards clip
their own children and their ring has to be drawn inside (`focus-visible:
outline-offset-[-3px]`); an unlayered rule would beat every Tailwind utility no
matter its specificity. `:where()` keeps the selector at zero specificity for the
same reason. The colour is scheme-aware to clear 3:1 on both the mortar field and
the dark kiln, and transitions are disabled so a focus ring never animates in.

---

## Motion

| Motion | Curve | Under reduced motion |
|---|---|---|
| Brick entrance (`--animate-brick-in`) | `0.42s cubic-bezier(0.16, 1, 0.3, 1)`, from `translateY(-14px) rotate(-0.5deg) scale(0.98)` | Crossfade, `0.25s linear` |
| Card hover lift and 0.6 degree tilt | 200ms, gated `motion-safe:` at the call site | Not applied |
| Button press and hover scale | Gated `motion-safe:` | Not applied |
| Layout picker thumb | 300ms `cubic-bezier(0.16, 1, 0.3, 1)` | `transition-none` |
| Skeleton pulse | Tailwind's `animate-pulse` | Static at 0.7 opacity |
| Glaze filmstrip | `scroll-behavior: smooth` | `scroll-auto` |
| The warming reflow | Continuous, until frozen | Frozen before it moves at all |

A brick is **tapped down and stays down**: fast, decelerating, no rebound. The
overshoot curve read as bounce, which is why the entrance is ease-out-quint
rather than a spring.

`prefers-reduced-motion` is handled two ways, and both are in use: global
overrides in `app.css` for the animations the theme owns, and `motion-safe:`
variants at every call site for transforms a component adds. The reduced-motion
reader is also the one reader who never sees the warming reflow, because
`FeedGrid` freezes it immediately rather than letting the wall rearrange itself.

---

## Accessibility

**Conformance target: WCAG 2.2 AA.**

| Criterion | How it is met |
|---|---|
| Contrast (1.4.3) | ≥4.5:1 body, ≥3:1 large text and UI. The `-ink` accent variants exist for exactly this |
| Keyboard (2.1.1) | Full navigability including the endless scroll; the pump is driven by scroll position, not by a mouse |
| Focus visible (2.4.7) | One 3px ring, scheme-aware, never animated |
| Target size (2.5.8) | `min-h-11` (44px) on every control |
| Motion (2.3.3) | A `prefers-reduced-motion` alternative for every animation |
| Status messages (4.1.3) | One polite live region narrates the wall's async state |
| Headings (2.4.6) | One `h1` at a time; the wall's is `sr-only` and steps aside for an error heading |
| Bypass blocks (2.4.1) | A skip link to `#wall` |

Videos never autoplay. That is an accessibility stance as much as a design one,
and it is enforced in CI rather than left to review (see
[08](08-wall-and-bricks.md)).

### The declared exception

**Captions (1.2.2, and 1.2.4 for live streams) are not met for video.** Neither
Bluesky video nor Streamplace ships caption data, so bricks play without a
caption track. The model carries `CaptionTrack` end to end and the player renders
tracks the moment mortar supplies them; until an upstream does, the wall cannot
claim 1.2.2 or 1.2.4 for video. This is stated rather than quietly omitted.

---

## Design principles

From `PRODUCT.md`, restated here because they are what the token choices answer
to:

1. **Bricks are the stars.** Content carries the colour; chrome recedes. Any UI
   element competing with a brick loses.
2. **Serendipity over relevance.** The mixer's exploration picks get equal visual
   dignity, never sponsored-slot styling.
3. **Materials, not metaphors.** Brick, mortar and kiln show up as texture,
   weight and motion physics, not as illustrations or mascots.
4. **Unhurried by design.** No mechanics that manufacture urgency. No badges, no
   unread counters, no countdowns.
5. **A wall is a gift.** A wall viewed through someone else's handle should feel
   like a made thing worth passing on.

The anti-references are as load-bearing as the principles: Pinterest's
infinite-mall density and engagement-bait overlays; corporate SaaS cream
backgrounds and gradient CTAs; doomscroll urgency mechanics.

---

## Assets

| Asset | Source |
|---|---|
| `favicon.svg`, `favicon-32.png`, `apple-touch-icon.png`, `icon-192.png`, `icon-512.png` | Generated by `pnpm icons` (`web/scripts/render-icons.mjs`) |
| `og.png` (1200×630) | Generated by `pnpm og` from `web/scripts/og-template.html` |
| `site.webmanifest` | `display: standalone`, kiln background and theme colour |

The OG image is the wordmark on a dark masonry wall of colour-coded bricks, and
its `og:image:alt` says so, because a link preview is often the first thing
anyone sees of mason.

---

## Assumptions and open questions

**Assumptions**

- `oklch()` is supported. It is used for every colour token with no fallback.
- `color-mix(in oklch, …)` is supported; it is used for placeholder colour.
- Google Fonts is reachable. If it is not, the display stack falls back to Avenir
  Next and then system-ui, and nothing blocks.

**Decisions**

- *oklch for every token.* **Perceptual lightness.** Contrast reasoning between
  an accent and its `-ink` variant is only tractable if lightness means the same
  thing across hues.
- *Cool, near-achromatic field.* **`plaster`, chroma 0.004.** Warmth in the
  background is the corporate-cream anti-reference; the wall's colour has to come
  from the bricks.
- *Separate `-ink` accent variants.* **Display colours are not text colours.** The
  sky, tangerine and violet accents fail AA as text on light surfaces, and hiding
  that behind "close enough" is how contrast bugs ship.
- *No theme toggle.* **`prefers-color-scheme` only.** A toggle is state to
  persist, sync across tabs, and render before paint; the system preference is
  already the reader's answer.
- *Hard 2px shadow.* **A brick sits, it does not float.** A blurred elevation
  shadow reads as a floating card, which is the material the design rejects.
- *Ease-out-quint entrance.* **Tapped down, no rebound.** The overshoot curve read
  as bounce, which is playful in the wrong register.
- *Two reduced-motion mechanisms.* **Global overrides plus `motion-safe:` at call
  sites.** The theme owns its own animations; a component's transform hover is the
  component's to gate.
- *The caption gap is declared, not hidden.* **Stated in `PRODUCT.md` and here.**
  Claiming AA while shipping uncaptioned video would be false.

**Open questions**

- *Contrast verification is manual.* The `-ink` variants came out of a measured
  critique, but nothing in CI checks contrast, so a new token or a new pairing can
  regress silently. Open: is an automated contrast check worth adding to the lint
  lane?
- *Bricolage Grotesque from Google Fonts.* A third-party font host on a
  privacy-forward app is a tension. Self-hosting would remove the connection at
  the cost of a build step and bundle size. Open.
- *Dark-mode accent coverage.* Only video has a `-bright` variant. Post and blog
  use their display accents on kiln, which passes, but the set is asymmetric.
  Open: does it want completing for consistency?
