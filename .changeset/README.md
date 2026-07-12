# Changesets

Add one with any user-visible change:

```sh
pnpm changeset      # describe the change, pick major/minor/patch
```

Commit the generated file. On merge to `main`, CI collects the pending changesets
into a "chore: version mason" PR; merging that bumps the version everywhere
(root, `web/`, the Rust workspace and its lockfile), writes `CHANGELOG.md`, tags,
and cuts the GitHub release.

**Releasing is not deploying**: the live site only changes when the deploy
workflow is dispatched.

Infrastructure-only changes usually need no changeset. Full flow, and what
major/minor/patch mean for mason: see the Releases section of the root README.
