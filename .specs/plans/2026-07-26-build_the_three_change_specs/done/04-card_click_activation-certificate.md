# Done Certificate · Task 04: card click activation

**Task:** [04-card_click_activation.md](04-card_click_activation.md) · **Plan:** [plan.md](../plan.md)
**State:** Validated 2026-07-26

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
  - *Collected:* `grep -rn 'reader.activate' web/src/lib/components/` returns exactly four hits, and
    each is the whole of its handler body: `BrickShell.svelte:52`
    (`if (brick) reader.activate(event, brick);`), `GlazeCard.svelte:159`, `GlazeCard.svelte:214`,
    `VideoCard.svelte:159`. The `if (brick)` at `BrickShell.svelte:52` is a null guard on the
    optional prop, not a modifier test. `grep -rnE 'metaKey|ctrlKey|shiftKey|altKey|\.button\b'
    web/src/lib/components/` returns nothing; the only hits in `web/src` are
    `reader.svelte.ts:17-21,138` and `reader.test.ts`. `grep -rn 'preventDefault|stopPropagation'`
    over `components/` shows none at any of the four call sites (the hits are FeedGrid, SwitchWall,
    HandleForm, ClientPicker and `Sensitive.svelte:51-52`). `reader.svelte.ts` is not in the diff
    (`jj diff --stat` lists five files, all `.svelte`), so task 01's rule is untouched and its
    `web/src/lib/state/reader.test.ts` cases for cmd/ctrl/shift/alt/middle still run: 45 vitest
    tests pass.
  - *Checked:* `reader` resolves by step 4 (imported) in all three files that name it:
    `BrickShell.svelte:4`, `VideoCard.svelte:5`, `GlazeCard.svelte:22`, each
    `import { reader } from '$lib/state/reader.svelte'` -> the `export const reader = new
    ReaderState()` singleton at `reader.svelte.ts:198`. No local, prop or module-level `reader` in
    any of the five changed files (`$props()` destructures are accent/href/brick/label/children,
    brick/priority, brick/priority), so no shadowing. `reader.activate` resolves to
    `ReaderState.activate` at `reader.svelte.ts:132`.
  - *Status:* SATISFIED

- **O2 · A plain left click opens the reader on post and blog cards with no navigation.**
  - *Claim:* `BrickShell` receives a `brick` prop from `PostCard` and `BlogCard`, its `<a>` calls
    `reader.activate`, and an unmodified left click opens the reader and navigates nowhere.
  - *Evidence to collect:* read `BrickShell.svelte`, `PostCard.svelte:17` and `BlogCard.svelte:13`.
    Drive `just dev`, `/?actor=demo`, and left-click one card of each kind, watching the address bar.
  - *Collected:* read in full. `BrickShell.svelte:9,20` add the optional `brick?: Brick` prop,
    `:45-55` keep the same `<a>` with `{href}`, `target="_blank"`, `rel="noopener noreferrer"` and
    add only the `onclick`. `PostCard.svelte:17` and `BlogCard.svelte:13` each gained `{brick}` and
    nothing else. Driven against the static build on a preview of `/?actor=demo` (the wasm service
    worker's offline demo wall), not `just dev`, which is the same bytes CI ships: a plain left
    click on the post card's text opened `[role="dialog"]` (its `button[aria-label="Close the
    reader"]` present), `context.pages().length` stayed 1 and the tab's url was unchanged; the same
    on the blog card's body. Escape shut it again and focus landed back on the anchor
    (`activeElement` = `<a href="https://bsky.app/profile/fixture/post/1">`), which also confirms
    the single call hands `activate` a real `currentTarget`.
  - *Status:* SATISFIED

- **O3 · Video and glaze bricks have their own activation points that do not shadow their controls.**
  - *Claim:* the watch-at-source anchor in `VideoCard.svelte` and both image anchors in
    `GlazeCard.svelte` call `reader.activate`; the play button still mounts the inline player without
    opening the reader; the filmstrip arrows, ALT panel and touch reveal still do their own jobs.
  - *Evidence to collect:* read `VideoCard.svelte` around `:147` and `GlazeCard.svelte` around `:147`
    and `:195`. Confirm those four controls are siblings of the anchors rather than children, then
    exercise each of them in the browser.
  - *Checks:* if any of the four controls has become a descendant of an intercepted anchor, its click
    now reaches `activate` first. Resolve the DOM nesting by reading the markup, not by clicking.
  - *Collected:* read `VideoCard.svelte:154-165` (the watch anchor, now with the onclick) and
    `GlazeCard.svelte:151-170` (the filmstrip's per-image anchor) and `:205-217` (the single/grid
    anchor). Both glaze branches got it, including the carousel one. Driven: a plain click opened
    the reader with no second tab from the video watch link, from a glaze single/grid image and
    from a filmstrip image; dispatched at every one of the carousel's five anchors, all five
    returned `defaultPrevented === true`, so the activation is on each `{#each}` link and not only
    the first. A touch tap on a glaze picture (a `hasTouch`/`isMobile` context) opened it too.
    Then the four controls, each exercised: the play button mounted a `<video>` inside the card
    with no reader; the next arrow moved the live region from "image 1 of 5" to "image 2 of 5" with
    no reader; the ALT button opened its own
    `[role="dialog"][aria-label="image description"]` with no reader; the touch pill flipped
    `aria-expanded` false -> true and relabelled to "Hide post details" with no reader.
  - *Checked:* nesting read off the markup, not inferred from the clicks. `VideoCard.svelte:85`'s
    play button is inside the `Sensitive`/`<div class="relative">` block at `:53-131`, and the watch
    anchor is in the separate `<div class="flex flex-col gap-3 p-4">` at `:132-167`: siblings, and
    the card passes `BrickShell` no href (`:52`) so there is no outer anchor either. In
    `GlazeCard.svelte` the arrows (`:180`, `:188`), the ALT trigger (`:290`) and the touch pill
    (`:267`) all sit outside the two anchors: the arrows in the sibling controls div at `:177`,
    the pill and ALT in the overlay div at `:253`, which is a sibling of the `<div inert={showAlt}>`
    at `:140` that holds both anchors. Confirmed at runtime too: `closest('a')` is null for each.
  - *Status:* SATISFIED

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
  - *Collected:* `PostCard.svelte:17` opens `<BrickShell ... href={clientUrl(brick.url)} {brick}>`
    and wraps `<Sensitive>` at `:19`; `GlazeCard.svelte:205`'s anchor wraps `<Sensitive>` at `:218`;
    the carousel's `<Sensitive>` at `:142` wraps the strip and therefore sits OUTSIDE its anchors.
    `Sensitive.svelte` is not in this task's diff, and its handler at `:40-54` still calls
    `event.preventDefault()` (`:51`) then `event.stopPropagation()` (`:52`) before `revealed.add(id)`.
    Driven on both descendant sites (the demo wall's brick 0 carries the only `!warn` blur): on the
    post card, `closest('a')` for the show-anyway button was true, the click revealed the media (the
    reveal control gone), `[role=dialog]` stayed absent and no second tab opened; identical on the
    glaze single/grid card.
  - *Status:* SATISFIED

- **O4 · Every modified click still reaches the source in a new tab.**
  - *Claim:* cmd-click, ctrl-click, shift-click, alt-click and middle-click open the source from
    every intercepted anchor, and no anchor lost its `href`, `target="_blank"`,
    or `rel="noopener noreferrer"`, and no `clientUrl` call site lost its wrapper. `BlogCard.svelte:13` is raw `brick.url` by design and is not a regression.
  - *Evidence to collect:* run `grep -rn 'target="_blank"' web/src/lib/components/` and confirm each
    intercepted anchor still carries it with `rel="noopener noreferrer"`. Exercise all five click
    kinds on one anchor of each card type.
  - *Collected:* `grep -rn 'target="_blank"' web/src/lib/components/` returns exactly the four
    intercepted anchors (`BrickShell.svelte:47`, `GlazeCard.svelte:153` and `:207`,
    `VideoCard.svelte:156`), each still beside its `rel="noopener noreferrer"` and its `href`. The
    diff touches no attribute line: every hunk is an addition apart from the two one-line
    `{brick}` additions and a comment rewrite. All four `clientUrl(` call sites survive
    (`PostCard.svelte:17`, `GlazeCard.svelte:152` and `:206`, `VideoCard.svelte:155`);
    `BlogCard.svelte:13` is raw `brick.url` by design. `BrickShell.svelte` contains no `<button>`,
    and at runtime `closest('button')` was null for every intercepted anchor while `tagName` was
    `A`. Real clicks, watched through `context.on('page')` and `context.pages()` rather than the
    tab's url: cmd-click, shift-click and middle-click on the post anchor each opened a SECOND TAB
    at `https://bsky.app/profile/fixture/post/1` and opened no reader; cmd-click opened a second
    tab at `https://example.com/blog/3` from the blog anchor, at `https://example.com/video/13`
    from the video watch anchor (confirmed on a `framenavigated` listener), and at
    `https://bsky.app/profile/fixture/post/12` and `.../post/9` from the glaze single/grid and
    filmstrip anchors. Positive control: the same watcher reports zero new pages for every plain
    click, so the zeroes above are measured zeroes. macOS chromium delivers no `click` at all for
    ctrl+left (the OS takes it as a context menu) and maps alt+left to a download rather than a
    tab, so those two were asked of the rule directly: a `MouseEvent` dispatched at each of the
    post, blog and video anchors left `defaultPrevented` false for ctrlKey, metaKey, shiftKey,
    altKey and `button: 1`, and true only for a plain button-0 click.
  - *Status:* SATISFIED

- **O5 · Meets the repo definition of done.**
  - *Claim:* the gates are green, and the PR says plainly that the wiring itself is visible only to
    Playwright rather than citing a green typecheck.
  - *Evidence to collect:* run `just check`. Read the PR body and confirm the statement is present.
  - *Collected:* `just check` run in this workspace, exit 0: guard-dashes, guard-autoplay,
    guard-toolchain, fmt-check, guard-wasm, oxlint, knip, clippy, 133 nextest tests, both tsc
    projects, 45 vitest tests. `just test-e2e` also run, 1 passed (the service-worker smoke).
    The commit message (this run's PR body, per the gate's local rule) states it plainly: "Nothing
    in `just check` can see any of this. tsc does not parse .svelte, so zero component files enter
    the program and a green typecheck says nothing about a component body", and then records the
    chromium run that does see it, rather than citing the green gate as coverage.
  - *Status:* SATISFIED

- **O6 · Reviewable: one click of each kind, each way.**
  - *Claim:* on `/?actor=demo`, a left click on a post, blog, video and glaze brick each opens the
    reader with no navigation, and a cmd-click on each opens the source in a new tab.
  - *Evidence to collect:* the eight interactions above, performed in a browser. Note that task 06
    automates only the post-card path.
  - *Collected:* all eight exercised first-hand on `/?actor=demo` (the default layout for post,
    blog and video; `mason:layout = glaze` for the glaze wall), driving the static `web/build/`
    through a local preview in chromium. Left click: reader open, url unchanged, one page, on post,
    blog, video and glaze (both glaze branches). cmd-click: a second tab at the source on all four,
    reader shut. 29 of 29 checks in the gate's own driving script passed once the video card's
    background tab was watched on `framenavigated` instead of a first url read. Task 06 automating
    only the post path is unchanged by this.
  - *Status:* SATISFIED

## Regression check

- `BrickShell.svelte`'s `{#if href}` branch at `:31`: `VideoCard.svelte:51` and
  `GlazeCard.svelte:132` pass no href. Trace: both still render through the `{:else}` branch at
  `:40` with no anchor : PRESERVED. Read: neither card passes `href`
  (`VideoCard.svelte:52`, `GlazeCard.svelte:136`), and the `{:else}` at `:58` renders `children()`
  bare. Observed on the wall: the video `<article>` holds exactly one `<a>`, the watch link, whose
  parent is a `DIV` and not the article; a glaze single/grid `<article>` holds exactly one `<a>`,
  the image anchor; a post/blog `<article>`'s single `<a>` is its direct child. No card gained or
  lost an anchor.
- `VideoCard.svelte:47`'s collapse effect reads `player.activeId`. Trace: pressing play on a video
  card still claims under `brick.id` and mounts the inline player : PRESERVED. `VideoCard`'s
  `$effect` at `:47-49` and `player.claim(brick.id)` at `:90` are outside the diff, which adds only
  the import at `:5` and the anchor's `onclick` at `:158-160`. Driven: the play button mounted a
  `<video>` inside the card and left `[role=dialog]` absent, and the Close video control collapsed
  it again.

## Residue

- `BrickShell`'s `brick` prop is optional, so a card that forgets to pass it fails at runtime rather
  than at compile time (tsc does not read `.svelte`). Outside this DoD; note it if seen. Seen and
  benign today: the only two callers that pass `href` both pass `brick`, and the guard at
  `BrickShell.svelte:52` makes the miss a silent fall-through to the href rather than a throw. The
  hazard is a future fifth card, not this diff.

- **Two extra surfaces checked, both clean.** `LandingWall.svelte` renders the same three cards
  behind the handle form; it is `inert` + `pointer-events-none`, and a click over it on `/` opened
  no reader, changed no url and opened no tab. And `AuthorChip` carries no anchor of its own, so
  the card-wide link gained no nested `<a>` whose default this interception would now be cancelling.

- **One interim wart, for task 05 rather than here.** A video already playing keeps playing behind
  the reader's scrim, because `BrickReader` renders no body yet and so never claims
  `reader:<brick.id>`. Not a regression: before this change the same click opened a background tab
  and the card kept playing on the wall behind it. It is the shape of the wall until the reader
  grows a video body.

## Conclusion

VERDICT: DONE
CONFIDENCE: high
SUMMARY: O1 to O6 and O3b are all SATISFIED against evidence this validator collected first-hand
(`just check` exit 0 and `just test-e2e` green in the workspace, plus 29 chromium checks over the
static build on `/?actor=demo` and the glaze wall covering all four card kinds in both directions,
every carousel anchor, the four sibling controls, the one descendant, and the inert landing wall as
a control), the four interceptions are each a bare `reader.activate(event, brick)` with no modifier
logic anywhere in `web/src/lib/components/`, and both named regression traces are PRESERVED; the one
evidence limit worth naming is that macOS chromium delivers no click for ctrl+left and maps alt+left
to a download, so those two of the five modified clicks were measured by dispatching at each anchor
and reading `defaultPrevented` (false) rather than by counting a second tab.
