# Changesets

Add one with any user-visible change:

```sh
pnpm changeset      # describe the change, pick major/minor/patch
```

Commit the generated file. On merge to `main`, CI collects the pending changesets
into a "chore: version mason" PR; merging that bumps the version everywhere
(root, `web/`, the Rust workspace and its lockfile), writes `CHANGELOG.md`, tags,
and cuts the GitHub release.

**A release is a ship**: cutting one deploys it, so the tag, the notes and the
live site always describe the same code. Merging an ordinary PR to `main` does
not deploy; it only updates the pending version PR.

Infrastructure-only changes usually need no changeset. Full flow, and what
major/minor/patch mean for mason: see the Releases section of the root README.
