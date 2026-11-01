# Automated binary releases

Pushing a `v*` tag starts the `Release` workflow. It accepts stable
`vMAJOR.MINOR.PATCH` tags that match the root Cargo package version and point to
commits reachable from `production-ready`. Maintainers choose the version and tag;
ordinary commits do not trigger releases.

## Publish a version

1. Update `Cargo.toml` and both lockfiles (`Cargo.lock` and
   `starters/rust-agent/Cargo.lock`) for the new crate version. Update the docs
   and add a non-empty `## MAJOR.MINOR.PATCH` section to `CHANGELOG.md`. The workflow
   uses that section as the GitHub release notes and blocks publication if it is
   missing or empty. Commit and push to `production-ready`.
2. Check that the working tree is clean and up to date.
3. Create and push an annotated tag matching the new `Cargo.toml` version.
   These commands illustrate `0.14.2`; always use a new, unpublished version:

   ```sh
   git tag -a v0.14.2 -m "MOTE v0.14.2"
   git push origin v0.14.2
   ```

GitHub Actions handles the build and asset upload:

- Validates the tag, package version, and branch ancestry.
- Runs reusable CI on all three OSes, including the dependency audit.
- Builds archives for Windows x64 (MSVC with static C runtime), Linux x64
  (Ubuntu 22.04), and macOS Apple Silicon (macOS 14).
- Extracts each archive, checks CLI version/help output, and runs the Python,
  TypeScript and Go SDK suites
  against the packaged bridge.
- Requires all platforms, validates archive digests, and assembles `SHA256SUMS`.
- Uploads the assets to a draft, checks names and sizes, and publishes the latest
  stable release.

Only the publish job has `contents: write`; build and test jobs are read-only.
The workflow uses GitHub's built-in token instead of a stored personal access
token. Attempts for the same tag run serially. Failed builds cannot publish.

## Recovery

Rerun failed jobs in the same workflow run after transient runner or network errors.
An incomplete draft can resume, with validated assets replaced before publication.
The workflow refuses to overwrite a published release. For code changes, fix
`production-ready`, increment the version, and create a new tag. Do not move a
released tag or delete archives to hide a failed release.

The workflow does not publish to package registries, sign or notarize binaries,
or change repository visibility. Source and releases are public. Checksums detect
corruption; they are not code signatures.

## Documentation before tagging

- Update the release section in `CHANGELOG.md` and version-specific examples.
- Review README, binary quick-start, configuration, SDK READMEs, and security limits.
- Update root and starter `Cargo.lock` for the new root crate version. Leave
  unrelated dependency versions unchanged and review `git diff` before committing.
- Run `python -m unittest discover -s scripts/tests -v` for release tooling and
  relative links, and `cargo test --locked --test release_manifest` for the exact
  downloadable YAML.
- Check new samples, docs, and commit contents for secrets and personal data before pushing.
- Treat repository visibility and registry publication as separate decisions from version-tag releases.

## Local checks

Release builds strip symbols. For a diagnostic build that keeps them, run:

```sh
CARGO_PROFILE_RELEASE_STRIP=none cargo build --locked --release --bins
```

This changes only that build; the default release artifacts stay stripped.

```sh
python -m unittest discover -s scripts/tests -v
cargo build --locked --release --bins
python scripts/licenses.py --target YOUR_TARGET --output target/release
python scripts/release.py --binary-dir target/release --output /path/outside/repo --target YOUR_TARGET --tag v0.14.2
```

Use the target that matches the binaries you built. Do not label MinGW/GNU Windows
binaries as MSVC binaries. The CI matrix builds with explicit target triples.
Packaging copies only allowlisted files, excluding workspaces, credentials, build
caches, and source-tree debris.
