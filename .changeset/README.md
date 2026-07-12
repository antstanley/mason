# Changesets

mason ships as one thing, so it carries one version: the root `package.json` is
the source of truth, and `pnpm version` propagates it to `web/package.json`, the
Rust workspace and its lockfile. Nothing is published to npm; a release is a tag
and a GitHub release.

Add a changeset with any user-visible change:

```sh
pnpm changeset          # describe the change, pick major/minor/patch
```

Commit the generated file. On merge to `main`, CI keeps a "chore: version mason"
PR open collecting the pending changesets. Merging that PR bumps the version,
writes CHANGELOG.md, tags, and cuts the release.

Infrastructure-only changes (CI, deploy config) usually need no changeset.
