# Automated binary releases

The `Release` workflow runs automatically when a `v*` tag is pushed. It accepts
stable `vMAJOR.MINOR.PATCH` tags matching the root Cargo package version and only
commits reachable from `production-ready`. It does not release every commit or
invent version numbers.

## Publish a version

1. Update `Cargo.toml` and both lockfiles (`Cargo.lock` and
   `starters/rust-agent/Cargo.lock`) when changing the crate version. Update docs
   and `docs/RELEASE_NOTES.md` if needed. Commit and push to `production-ready`.
2. Ensure the working tree is clean and up to date.
3. Create and push an annotated version tag matching the new `Cargo.toml` version.
   The following example assumes an unreleased `0.13.2`; substitute your new version:

   ```sh
   git tag -a v0.13.2 -m "MOTE v0.13.2"
   git push origin v0.13.2
   ```

From there **no manual build or asset upload is needed**. GitHub Actions:

- Validates the tag/version and branch ancestry.
- Runs the reusable CI workflow, including all three OSes and dependency audit.
- Builds Windows x64 (MSVC with static C runtime), Linux x64 (Ubuntu 22.04), and
  macOS Apple Silicon (macOS 14) archives.
- Extracts each archive, runs CLI version/help checks, and runs both SDK suites
  against its actual packaged bridge.
- Requires every platform, validates archive digests, and assembles `SHA256SUMS`.
- Uploads all assets to a draft, verifies names and sizes, then publishes it as
  the latest stable release.

Only the publish job receives `contents: write`; build/test jobs are read-only.
The workflow uses GitHub's built-in token, not a stored personal access token.
Concurrent attempts for the same tag are serialized. Failed builds cannot publish.

## Recovery

For transient runner/network errors, rerun failed jobs on the same workflow run.
An incomplete draft can be resumed: validated assets are replaced before publishing.
A published release is never overwritten by this workflow. For code changes,
fix on `production-ready`, increment the version, and create a new tag rather than
moving an existing release tag. Do not delete archives to hide a failed release.

The workflow deliberately does not publish to package registries, sign/notarize
binaries, or change repository visibility. Private repository releases require
repository access. Checksums detect corruption; they are not code signatures.

## Documentation before tagging

- Update `CHANGELOG.md`, this release's `RELEASE_NOTES.md`, and versioned status text.
- Check README, binary quick-start, configuration, SDK READMEs, and security limits.
- Update root and starter `Cargo.lock` for the new root crate version without
  refreshing unrelated dependency versions; verify `git diff` before committing.
- Run `python -m unittest discover -s scripts/tests -v` (includes relative-link checks)
  and `cargo test --locked --test release_manifest` (the exact downloadable YAML).
- Review archived samples and Git history before any separately authorized public launch.
- Keep repository visibility and registry publication separate from version-tag releases.

## Local checks

```sh
python -m unittest discover -s scripts/tests -v
cargo build --locked --release --bins
python scripts/licenses.py --target YOUR_TARGET --output target/release
python scripts/release.py --binary-dir target/release --output /path/outside/repo --target YOUR_TARGET --tag v0.13.1
```

Use the target matching the binaries you actually built. In particular, do not
label MinGW/GNU Windows binaries as MSVC binaries. The CI platform matrix builds
with explicit target triples. The packager only copies an explicit file allowlist;
workspaces, credentials, build caches, and source-tree debris are not included.
