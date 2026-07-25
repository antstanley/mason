# Change: Tighten TypeScript and add a pre-push gate

**Status:** Merged · **Date:** 2026-07-25 · **Merged:** 2026-07-25 · **Owner:** Ant Stanley · **Target:** Repo-wide

Two gaps between mason's stated discipline and its enforcement, closed together
because they are the same gap seen twice. The TypeScript side runs `strict` but
not `noUncheckedIndexedAccess` or `exactOptionalPropertyTypes`, so array indexing
in the layout code is unchecked and the wire's optional-versus-nullable
distinction is only half enforced. And every gate mason declares runs in CI only:
nothing checks anything before a push, so the first sign of a formatting or lint
failure is a red pull request.

---

## Motivation

mason's Tiger Style commitment is "assume any invariant you did not check can be
violated". The layout code violates it routinely and invisibly: `Masonry` and
`Bento` index arrays (`entries[0]`, `kids[1]`, `opts[i]`, `images[0]`) and treat
the result as present. Most of those are genuinely safe, but `tsc` cannot tell
which, and today it is not being asked to.

`exactOptionalPropertyTypes` matters for a different reason. The wire contract
makes a real distinction between a field mortar serialises as `null` and one it
skips entirely, and [`06-wire-contract.md`](../../06-wire-contract.md) calls that
distinction contract rather than accident. Without the flag, `{blur: undefined}`
and `{}` are interchangeable to the compiler, which is exactly the confusion the
contract exists to prevent.

The pre-push gate is the cheaper half. `just fmt-check`, `just lint` and the fast
tests take seconds; the round trip through CI to learn that `oxfmt` disagrees
takes minutes. Every gate exists, none of them runs automatically.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/development-guidelines.md`](../../development-guidelines.md) | Toolchain gains a hook row; TypeScript conventions gain the strict flags; Repository hygiene gains the gate; two Open questions resolve |
| [`.specs/10-build-release-deploy.md`](../../10-build-release-deploy.md) | A local-gate row beside the CI gates |
| [`.specs/07-web-client.md`](../../07-web-client.md) | No content change; the strict flags are noted where testing is described |

No canonical type changes.

CORRECTED ON MERGE: the Toolchain row is a `just push` row, not a hook row, for
the reason worked out under the first Open question below. `07-web-client.md` was
left untouched, not even a note: the flags changed no behaviour that page
describes, and a page edited to say nothing is a page whose date now lies about
when it was last true. The two pages that actually moved are
`development-guidelines.md` and `10-build-release-deploy.md`.

---

## Proposed changes

### `.specs/development-guidelines.md` → Toolchain (Add)

> | `jj` pre-push hook | repo-local | Runs `just fmt-check`, `just lint` and `just test` before a push leaves the machine |

CORRECTED ON MERGE, do not copy the row above: there is no hook, and there cannot
be one. What shipped is

> | `just push` | repo-local | The local gate: fmt-check, lint, the guards and test, then `jj git push` |

and the gate it names is `just check`, which runs five recipes rather than three.
`guard-autoplay` and `guard-dashes` are in it because the Definition of done on
that same page already requires both to pass locally, and leaving them out would
have meant the local gate and the definition of done disagreed about what done
means. They cost two greps.

### `.specs/development-guidelines.md` → TypeScript and Svelte conventions → Code style (Modify)

> - **`strict` is on, with `noUncheckedIndexedAccess` and
>   `exactOptionalPropertyTypes`.** Indexing an array yields `T | undefined` and
>   must be narrowed; an optional property cannot be satisfied by an explicit
>   `undefined`. The second flag is what makes the wire's optional-versus-nullable
>   distinction (see `06-wire-contract.md`) enforceable rather than conventional.
>   The typecheck is `tsc --noEmit`, not `svelte-check`: `svelte-check` crashes on
>   TypeScript 7 (its programmatic API stabilises in TS 7.1, around October 2026).
>   Do not swap it back yet.

On merge, that `06-wire-contract.md` reference becomes a relative link, since
`development-guidelines.md` sits beside it. It is left bare here because a link
that resolves from `.specs/changes/` would not resolve from the page it lands on.

The bullet also gained a sentence the block above does not have: narrow with a
guard or a `?.`, never a non-null assertion. That was a Decision in this document
rather than proposed canonical text, and all seven indexing sites honoured it, so
it belongs on the page that describes the branch.

### `.specs/development-guidelines.md` → Repository hygiene (Add)

> - **A pre-push hook runs `just fmt-check`, `just lint` and `just test`.** It is
>   pre-push rather than pre-commit deliberately: jj's working-copy model makes
>   small, frequent commits the norm, and a per-commit gate would tax exactly the
>   habit the workflow encourages. CI re-runs the same gates plus the end-to-end
>   and wasm lanes.

CORRECTED ON MERGE, again for the noun. The shipped bullet is

> - **`just push` is the local gate.** It runs `just check` (fmt-check, lint, the
>   two guards, test) and then `jj git push`. It is a wrapper recipe, not a hook:
>   jj has no hook system, and `jj git push` does not run the colocated repo's
>   `.git/hooks/pre-push`, so typing `jj git push` directly skips the gate. CI is
>   still the authority and re-runs the same set plus the e2e and wasm lanes.

The pre-push-not-pre-commit reasoning survives intact and moved to a Decision on
the same page. The jujutsu bullet that said pull requests go out with
`jj git push` was updated to say `just push` in the same pass: leaving it would
have documented the one path that skips the gate.

### `.specs/development-guidelines.md` → Assumptions and open questions → Open questions (Remove)

> Remove the bullets beginning *No pre-push hook exists* and
> *`tsconfig.json` sets `strict` but not `noUncheckedIndexedAccess`*. Both are
> answered by this change.

### `.specs/development-guidelines.md` → Assumptions and open questions → Decisions (Add)

> - *Pre-push, never pre-commit.* **One gate, at the push.** jj makes small
>   commits cheap and frequent; gating each one would make the cheapest operation
>   in the workflow the slowest.
> - *`noUncheckedIndexedAccess` and `exactOptionalPropertyTypes` on.* **The layout
>   code indexes arrays freely and the wire distinguishes absent from null.**
>   Both flags turn a convention the code already tries to follow into something
>   the compiler enforces.

Both landed as written. A third joined them, replacing the Open question this
change was supposed to answer with a hook and answered with a recipe instead:

> - *A wrapper recipe, not a hook.* **`just push`.** jj has no hooks and does not
>   fire git's, so the only honest local gate is one you invoke; CI stays the
>   authority.

### `.specs/10-build-release-deploy.md` → CI (Add)

> ### Local gates
>
> A `jj` pre-push hook runs `just fmt-check`, `just lint` and `just test` before
> a push leaves the machine. It is the same set CI runs first, so a red pull
> request from a formatting slip is not a thing that happens. CI remains the
> authority: it re-runs everything plus `just test-e2e` and the wasm lane, and it
> is what a merge is gated on.

CORRECTED ON MERGE on both the mechanism and the placement. The section describes
`just check` and `just push` and says wrapper recipe rather than hook throughout,
including why nothing can intercept a push here. It landed as the last subsection
of **Commands**, beside Test lanes and Guards, rather than inside **CI**: the CI
section opens by counting four workflows and then names them one per subsection,
so a fifth subsection that is not a workflow would have contradicted its own
first line. Commands is also where the two new recipes get their table row, so
the gate is described next to the recipes it composes.

---

## Implementation notes

Do the two halves as separate commits: the hook is inert, the compiler flags will
produce a real error list.

```
1. Pre-push hook.
     jj has no built-in hook mechanism, so wire it where the repo pushes from.
     Add scripts/pre-push.sh running: just fmt-check && just lint && just test
     Document invoking it in AGENTS.md, and add a `just check` recipe that runs
     the same three so the hook and the manual path cannot drift.

     CORRECTED ON MERGE, do not follow step 1: there is no scripts/pre-push.sh
     and there is no hook to install one into. What shipped is two recipes in
     the justfile and nothing else:

       check: fmt-check lint guard-autoplay guard-dashes test

       push *ARGS: check
           jj git push {{ARGS}}

     `check` is dependencies-only, so `just` runs the five before an empty body.
     The last sentence of step 1 has its reasoning backwards: a script and a
     recipe holding the same command list ARE the drift, because two artefacts
     can disagree. One recipe, called by the push wrapper as a dependency,
     cannot disagree with itself, so there is exactly one list of gates.

2. exactOptionalPropertyTypes in web/tsconfig.json:12 (beside "strict": true).
     Expect errors where an optional prop is passed an explicit undefined.
     Likely sites: the `priority` and `blur` props threaded through the card
     components, and the `?:` fields on VideoBrick and PostBrick in types.ts.

3. noUncheckedIndexedAccess in the same block. This is the larger list. Known
     indexing sites to expect errors at:
       web/src/lib/components/Masonry.svelte     entries[0], colHeights[i]
       web/src/lib/components/Bento.svelte       entries[0], images[0]
       web/src/lib/components/cards/GlazeCard.svelte  kids[0], kids[1], images[i]
       web/src/lib/components/ClientPicker.svelte     opts[i], CLIENTS[0]
       web/src/lib/components/cards/SkeletonCard.svelte  heights[i], tints[i]
       web/src/lib/components/LayoutPicker.svelte     querySelectorAll()[i]
     Narrow with an explicit guard or a `?.`; do not silence with a non-null
     assertion. A `!` here would reintroduce exactly the unchecked assumption the
     flag exists to surface, and Tiger Style treats an unjustified cast as a bug.

4. cd web && pnpm check:ci   until clean, then just lint && just test.
```

CORRECTED AFTER MERGE, and this is the most important note in this document.
Everything below reads as a compiler measurement and is not one. `tsc` does not
parse `.svelte`: zero component files enter the program, and `pnpm check:ci`
exits 0 on a component containing `const bad: string = 42`. Every site named
below lives in a `.svelte` file, so no compiler ever reported any of them.

What is true: the site list below is an accurate reading of the code, the
narrowings applied for it are real defensive improvements, and both flags are
genuinely live **for `.ts` files**, which is where the wire types and the
contract guard live. What is false is the framing, that a compiler flagged these
and now enforces them. It did not and does not. The canonical statement of the
coverage boundary is in `development-guidelines.md`, under the TypeScript
conventions; read that rather than the counts below.

The original text follows, unedited, because the sequence of a plausible claim
propagating through a spec into three agents' work is the useful part of this
history:

`exactOptionalPropertyTypes` produced **one** error, not the several step 2
expected: `Sensitive.svelte`'s `blur` prop, which every caller forwards an
optional brick field into. Its type became `blur?: Blur | undefined`. Neither
`types.ts` nor the `?:` fields on `VideoBrick` and `PostBrick` needed anything;
the wire types keep the bare `blur?: Blur` on purpose, and the widening is a
component-boundary concern rather than a wire one.

`noUncheckedIndexedAccess` produced **seven**, across six files. Of the six files
step 3 named, two were right (`Masonry.svelte`, for both `entries[0]` and the
`colHeights` indexing, and `Bento.svelte` for `entries[0]` but not `images[0]`)
and four produced no errors at all: `GlazeCard.svelte`, `ClientPicker.svelte`,
`SkeletonCard.svelte` and `LayoutPicker.svelte`. Three files the list did not
predict did: `FeedGrid.svelte`, `SkeletonGrid.svelte` and `VideoPlayer.svelte`,
all three the same `entries[0]` on an observer callback. That one pattern is six
of the seven errors; the layout-code framing in the Motivation was pointing at
the wrong thing, and the real finding is that every `ResizeObserver` and
`IntersectionObserver` callback in the app indexed its entries unchecked.

The no-non-null-assertion decision held at every site. Six became an early return
or a `?.`; the seventh, `colHeights[col]` in `Masonry.svelte`, is unreachable
today and ships as `if (y === undefined) continue`, with a comment saying so, on
the reasoning that skipping a brick until the next pass beats placing it at a
guessed offset.

Land the flags in one change rather than incrementally. A half-enabled strictness
setting is a setting nobody trusts.

---

## Merge plan

1. Apply each `Proposed changes` block to its canonical page; bump each page's
   `**Date:**` to the merge date.
2. No schema change.
3. Flip this file's `**Status:**` to `Merged`, add `**Merged:** YYYY-MM-DD`, and
   move it to `.specs/changes/merged/`.
4. Update `.specs/README.md`: remove it from the pending list.

---

## Assumptions and open questions

**Assumptions**

- The error list from the two flags is tens of sites, not hundreds. The web
  source is about 4400 lines and the indexing is concentrated in the layout
  components.
  **Measured on merge:** eight sites across six files, so even the optimistic
  half of this assumption was pessimistic. The second clause is wrong: the
  indexing that failed is concentrated in observer callbacks, which are spread
  across the component tree, not in the layout components.
- `just test` stays fast enough to sit in a pre-push hook. It is currently
  `cargo nextest` plus `tsc` plus `vitest`, with no network and no browser.
  Held, and the gate that shipped adds `guard-autoplay` and `guard-dashes` to it,
  which are two greps.

**Decisions**

- *Both flags together, not one at a time.* **One change, one error list.** A
  partially-strict `tsconfig` invites the next reader to assume the flags are on
  when they are not.
- *No non-null assertions to clear the errors.* **Narrow, or restructure.** A `!`
  converts a compiler-visible assumption back into an invisible one, which is the
  opposite of what the flag buys.
- *The hook does not run the slow lanes.* **`test-e2e` and `test-wasm` stay in
  CI.** Both need a browser; a hook that takes a minute gets bypassed, and a
  bypassed hook is worse than none.
  Held as written, minus the noun: `just check` excludes both lanes for exactly
  this reason, and the justfile comment says so. Note that a bypassed *gate* is
  not worse than none here, which is what makes an unenforceable one tolerable:
  going around `just push` leaves you where the repo already was, with CI
  re-running the same set and `main` branch-protected behind it.

**Open questions**

- *How does a jj repo install a pre-push hook?* jj has no native hook system, and
  the Git-backend hooks do not fire for `jj git push`. The likely answer is a
  wrapper recipe (`just push`) rather than a real hook. Open until the mechanism
  is chosen; the guideline text above says "hook" and may need to say "gate".
  **Resolved on merge: it cannot, and the guideline text does now say gate.**
  Three mechanisms were checked rather than assumed. (1) jj 0.43.0 has no hook
  feature: `jj util config-schema` contains no hook key and no `[hooks]` table,
  and `jj --help` lists no hook command. (2) This repo is colocated
  (`.jj/repo/store/type` is `git`, `git.colocate` is true) and `.git/hooks`
  exists, but a git hook does not help. In a throwaway colocated repo, an
  executable `.git/hooks/pre-push` that appended to a log file did **not** run
  for `jj git push`, and did run for a raw `git push` to the same remote
  immediately afterwards. Git-backend pre-push hooks fire only for the one
  command this repo forbids. (3) A jj alias
  (`aliases.push = ["util", "exec", "--", "just", "push"]`) was rejected: jj
  aliases are single-word and cannot shadow the built-in `git push` path, so it
  intercepts nothing; it lives in the local, untracked `.jj/repo/config.toml`, so
  it needs a per-clone install step that can be skipped silently; and
  `jj util exec` warns that it may be removed or replaced. So nothing can
  intercept a push, and the honest framing is a gate you invoke.
- *Does `exactOptionalPropertyTypes` change what `contract-check.ts` proves?* The
  `Wire<T>` mapped type relaxes literals but not optionality, so it should be
  unaffected. Worth confirming while implementing rather than assuming.
  **Confirmed on merge:** unaffected. `Wire<T>` is homomorphic
  (`{ [K in keyof T]: Wire<T[K]> }`), so it preserves the `?` modifier it maps
  over, and both `contract-check.ts` and `types.ts` typechecked unchanged under
  the flag. The one error the flag produced was at a component boundary, not on
  the wire.
