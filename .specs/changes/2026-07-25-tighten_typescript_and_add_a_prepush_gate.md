# Change: Tighten TypeScript and add a pre-push gate

**Status:** Proposed · **Date:** 2026-07-25 · **Owner:** Ant Stanley · **Target:** Repo-wide

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
skips entirely, and [`06-wire-contract.md`](../06-wire-contract.md) calls that
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
| [`.specs/development-guidelines.md`](../development-guidelines.md) | Toolchain gains a hook row; TypeScript conventions gain the strict flags; Repository hygiene gains the gate; two Open questions resolve |
| [`.specs/10-build-release-deploy.md`](../10-build-release-deploy.md) | A local-gate row beside the CI gates |
| [`.specs/07-web-client.md`](../07-web-client.md) | No content change; the strict flags are noted where testing is described |

No canonical type changes.

---

## Proposed changes

### `.specs/development-guidelines.md` → Toolchain (Add)

> | `jj` pre-push hook | repo-local | Runs `just fmt-check`, `just lint` and `just test` before a push leaves the machine |

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

### `.specs/development-guidelines.md` → Repository hygiene (Add)

> - **A pre-push hook runs `just fmt-check`, `just lint` and `just test`.** It is
>   pre-push rather than pre-commit deliberately: jj's working-copy model makes
>   small, frequent commits the norm, and a per-commit gate would tax exactly the
>   habit the workflow encourages. CI re-runs the same gates plus the end-to-end
>   and wasm lanes.

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

### `.specs/10-build-release-deploy.md` → CI (Add)

> ### Local gates
>
> A `jj` pre-push hook runs `just fmt-check`, `just lint` and `just test` before
> a push leaves the machine. It is the same set CI runs first, so a red pull
> request from a formatting slip is not a thing that happens. CI remains the
> authority: it re-runs everything plus `just test-e2e` and the wasm lane, and it
> is what a merge is gated on.

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
- `just test` stays fast enough to sit in a pre-push hook. It is currently
  `cargo nextest` plus `tsc` plus `vitest`, with no network and no browser.

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

**Open questions**

- *How does a jj repo install a pre-push hook?* jj has no native hook system, and
  the Git-backend hooks do not fire for `jj git push`. The likely answer is a
  wrapper recipe (`just push`) rather than a real hook. Open until the mechanism
  is chosen; the guideline text above says "hook" and may need to say "gate".
- *Does `exactOptionalPropertyTypes` change what `contract-check.ts` proves?* The
  `Wire<T>` mapped type relaxes literals but not optionality, so it should be
  unaffected. Worth confirming while implementing rather than assuming.
