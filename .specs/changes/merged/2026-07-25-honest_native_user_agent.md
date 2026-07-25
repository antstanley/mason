# Change: Give the native mortar an honest user agent

**Status:** Merged · **Date:** 2026-07-25 · **Merged:** 2026-07-25 · **Owner:** Ant Stanley · **Target:** Repo-wide

`mortar-server` identifies itself to every upstream as
`mason-mortar/0.1 (atproto discovery wall; https://github.com)`. The workspace is
at 0.7.0, the version is a hard-coded literal that `scripts/sync-version.mjs`
cannot reach, and the contact URL is a placeholder pointing at GitHub's home
page. This change derives the version from the crate and gives the string a real
contact URL.

---

## Motivation

A user agent is the one thing an unauthenticated client tells an operator about
itself. mason reads public AppView endpoints, `plc.directory` and a hundred
individual PDSes without credentials, so if any of those operators wants to rate
limit, debug or contact whoever is generating traffic, the user agent is all they
have. A stale version and a placeholder URL make it useless for exactly that
purpose, which is the only purpose it has.

The version drift is also a small correctness failure in a repo that treats
single-versioning as a rule.
[`10-build-release-deploy.md`](../../10-build-release-deploy.md) states that root
`package.json` propagates the version everywhere it appears; this string is the
one place that claim is untrue, and it is untrue silently, because a literal in a
builder chain looks nothing like a version declaration.

Note the scope: this affects **server mode only**. The wasm build sends the
browser's own user agent and cannot override it, which is by design.

---

## Affected spec pages

| Canonical page | Nature of change |
|---|---|
| [`.specs/04-sources-and-moderation.md`](../../04-sources-and-moderation.md) | HTTP policy gains a user-agent row; the Open question resolves |
| [`.specs/10-build-release-deploy.md`](../../10-build-release-deploy.md) | Versioning: the propagation list gains the crate-derived case |

No canonical type changes.

---

## Proposed changes

### `.specs/04-sources-and-moderation.md` → HTTP policy (Add)

> | User agent (native only) | `mason/<crate version> (+<contact URL>)`, derived from `env!("CARGO_PKG_VERSION")` |
>
> The browser build sends the browser's own user agent and cannot override it,
> which is correct: in local mode the request genuinely is the reader's browser
> making it. The native string carries a contact URL because mason reads public
> endpoints unauthenticated, and the user agent is the only channel an upstream
> operator has to reach whoever is generating the traffic.

### `.specs/04-sources-and-moderation.md` → Assumptions and open questions → Open questions (Remove)

> Remove the bullet beginning *The native user agent is stale.* It is answered by
> this change.

### `.specs/04-sources-and-moderation.md` → Assumptions and open questions → Decisions (Add)

> - *User agent derived, not written.* **`env!("CARGO_PKG_VERSION")`.** A literal
>   version string is a version declaration nothing propagates to, and it drifted
>   by six minor releases before anyone noticed.

### `.specs/10-build-release-deploy.md` → Versioning (Modify)

> Add to the propagation list, after the `Cargo.lock` bullet:
>
> - The native user agent, which reads `env!("CARGO_PKG_VERSION")` at compile
>   time and therefore needs no propagation step of its own.

---

## Implementation notes

One line of code and one decision that is not mine to make.

```
1. server/crates/mortar-core/src/http.rs:66
     Replace the literal with:
       concat!("mason/", env!("CARGO_PKG_VERSION"), " (+", MASON_CONTACT, ")")
     where MASON_CONTACT is a const in http.rs. `env!` is compile-time, so this
     stays a &'static str and the builder chain is unchanged.

     CORRECTED ON MERGE, do not copy the line above: `concat!` accepts literals
     only, and a `const` is a path expression that never expands to one, so that
     shape does not compile. `env!` works because it is a built-in macro that
     expands to a literal before `concat!` sees it. What shipped makes the whole
     string one named const instead, so the URL still appears exactly once. The
     const is also `#[cfg(not(target_arch = "wasm32"))]`, because its only use
     site is the native arm of `Http::new` and an ungated const is dead code in
     the wasm lane.

2. Pick the contact URL. The candidates are the repo
   (https://github.com/antstanley/mason) or the deployed site
   (https://mason.iamstan.dev). The repo is the better target: it is where an
   operator can open an issue, and it does not change if the site moves.

3. cargo test -p mortar-core && just lint
```

There is no test to add: the string is not behaviour mason can assert anything
about beyond its own construction, and pinning it in a test would only duplicate
the literal. The compile-time derivation is what makes it correct.

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

- `env!("CARGO_PKG_VERSION")` in `mortar-core` resolves to the workspace version,
  since every crate inherits `version.workspace = true`.
- No upstream allowlists or rate limits mason by its current user-agent string,
  so changing it cannot break an existing arrangement.

**Decisions**

- *Compile-time derivation over a sync-script target.* **`env!`.** Adding another
  regex to `sync-version.mjs` would fix the symptom and leave the class of bug in
  place; the compiler already knows the version.
- *Keep the wasm build's user agent alone.* **The browser's own.** In local mode
  the request is the reader's browser making it, and a spoofed user agent would
  misrepresent that to the operator on the other end. `fetch` cannot override it
  from a service worker anyway.

**Open questions**

- *Which contact URL?* The repo or the deployed site. The implementation notes
  recommend the repo; the owner's call.
  **Resolved on merge:** the repo, `https://github.com/antstanley/mason`. It is
  where an operator can open an issue, and it does not change if the site moves.
- *Does the string want to name the build mode?* An operator seeing
  `mason/0.7.0` cannot tell it apart from a browser-mode reader, but browser-mode
  readers do not send it at all, so every request carrying this string is a native
  mortar. Probably not worth the extra token.
