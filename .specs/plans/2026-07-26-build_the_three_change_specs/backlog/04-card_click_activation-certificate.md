# Done Certificate · Task 04: card click activation

**Task:** [04-card_click_activation.md](04-card_click_activation.md) · **Plan:** [plan.md](../plan.md)
**State:** Authored 2026-07-26, unverified

> Verification protocol for Task 04. A validating agent discharges it: collect each obligation's
> evidence, run its checks, set the Status, then derive the Conclusion by the rubric.

## Definition

DONE(Task 04) is every obligation O1 to O6 below holding, O3b included, each backed by the
evidence it names.

## Premises

- **P1 · Goal.** A plain unmodified left click opens the reader on all four card kinds, and every
  modified click still reaches the source in a new tab.
- **P2 · Obligations.** Done iff O1 to O3, O3b and O4 to O6 all hold; O6 is the Reviewable item.
- **P3 · Invariants.** Must not break: `BrickShell`'s no-href branch (`:40`), which two cards rely
  on; `VideoCard`'s play button and its `player.claim(brick.id)` at `:89`; `GlazeCard`'s filmstrip
  arrows (`:170`, `:178`), ALT panel (`:250`) and touch reveal pill (`:273`); `Sensitive`'s
  show-anyway button, which unlike those four is a **descendant** of an intercepted anchor on two
  cards and stays a reveal only because task 02 stopped its propagation; or the `clientUrl` rewrite
  on every outbound anchor.

## Obligations

- **O1 · The modifier rule exists in exactly one place.**
  - *Claim:* every interception is a single call to `reader.activate(event, brick)`, so the
    modifier-key logic is not duplicated per card.
  - *Evidence to collect:* run `grep -rn 'reader.activate' web/src/lib/components/` and confirm each
    hit is a bare call with no surrounding modifier test. Run
    `grep -rnE 'metaKey|ctrlKey|shiftKey|altKey' web/src/lib/components/` and expect no hits.
  - *Checks:* resolve `reader` in each card to the singleton from `$lib/state/reader.svelte`, not a
    prop or a local.
  - *Status:* unverified

- **O2 · A plain left click opens the reader on post and blog cards with no navigation.**
  - *Claim:* `BrickShell` receives a `brick` prop from `PostCard` and `BlogCard`, its `<a>` calls
    `reader.activate`, and an unmodified left click opens the reader and navigates nowhere.
  - *Evidence to collect:* read `BrickShell.svelte`, `PostCard.svelte:17` and `BlogCard.svelte:13`.
    Drive `just dev`, `/?actor=demo`, and left-click one card of each kind, watching the address bar.
  - *Status:* unverified

- **O3 · Video and glaze bricks have their own activation points that do not shadow their controls.**
  - *Claim:* the watch-at-source anchor in `VideoCard.svelte` and both image anchors in
    `GlazeCard.svelte` call `reader.activate`; the play button still mounts the inline player without
    opening the reader; the filmstrip arrows, ALT panel and touch reveal still do their own jobs.
  - *Evidence to collect:* read `VideoCard.svelte` around `:147` and `GlazeCard.svelte` around `:147`
    and `:195`. Confirm those four controls are siblings of the anchors rather than children, then
    exercise each of them in the browser.
  - *Checks:* if any of the four controls has become a descendant of an intercepted anchor, its click
    now reaches `activate` first. Resolve the DOM nesting by reading the markup, not by clicking.
  - *Status:* unverified

- **O3b · The one control that IS a descendant still stops.**
  - *Claim:* `Sensitive`'s show-anyway button sits inside an intercepted anchor on two of the four
    cards (`PostCard.svelte:17` wrapping `:19`, and `GlazeCard.svelte:195` wrapping `:201`), and it
    still carries the `event.stopPropagation()` task 02 added, so a reveal stays a reveal.
  - *Evidence to collect:* read all three sites, plus `GlazeCard.svelte:138`, where the carousel
    branch puts `Sensitive` outside the anchors instead. Read `Sensitive.svelte`'s handler and
    confirm the stop survives this task's diff.
  - *Checks:* this is the one place where "siblings, not children" is false, so the sibling reasoning
    in O3 does not cover it. Nothing in `just check` sees either file; task 06's dedicated case
    (click "show anyway", expect the media revealed and `[role=dialog]` absent) is the lane, and it
    is separate from the "still revealed when the reader opens on it" case precisely because that one
    passes under the broken behaviour too.
  - *Status:* unverified

- **O4 · Every modified click still reaches the source in a new tab.**
  - *Claim:* cmd-click, ctrl-click, shift-click, alt-click and middle-click open the source from
    every intercepted anchor, and no anchor lost its `href`, `target="_blank"`,
    or `rel="noopener noreferrer"`, and no `clientUrl` call site lost its wrapper. `BlogCard.svelte:13` is raw `brick.url` by design and is not a regression.
  - *Evidence to collect:* run `grep -rn 'target="_blank"' web/src/lib/components/` and confirm each
    intercepted anchor still carries it with `rel="noopener noreferrer"`. Exercise all five click
    kinds on one anchor of each card type.
  - *Status:* unverified

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green, and the PR says plainly that the wiring itself is visible only to
    Playwright rather than citing a green typecheck.
  - *Evidence to collect:* run `just check`. Read the PR body and confirm the statement is present.
  - *Status:* unverified

- **O6 · Reviewable: one click of each kind, each way.**
  - *Claim:* on `/?actor=demo`, a left click on a post, blog, video and glaze brick each opens the
    reader with no navigation, and a cmd-click on each opens the source in a new tab.
  - *Evidence to collect:* the eight interactions above, performed in a browser. Note that task 06
    automates only the post-card path.
  - *Status:* unverified

## Regression check

- `BrickShell.svelte`'s `{#if href}` branch at `:31`: `VideoCard.svelte:51` and
  `GlazeCard.svelte:132` pass no href. Trace: both still render through the `{:else}` branch at
  `:40` with no anchor : (PRESERVED / REGRESSION)
- `VideoCard.svelte:47`'s collapse effect reads `player.activeId`. Trace: pressing play on a video
  card still claims under `brick.id` and mounts the inline player : (PRESERVED / REGRESSION)

## Residue

- `BrickShell`'s `brick` prop is optional, so a card that forgets to pass it fails at runtime rather
  than at compile time (tsc does not read `.svelte`). Outside this DoD; note it if seen.

## Conclusion

VERDICT: (DONE | PARTIAL | NOT_DONE)
CONFIDENCE: (high | medium | low)
SUMMARY:
