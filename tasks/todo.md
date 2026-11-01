# MOTE v0.14.2 release checklist

Status: execution authorized for v0.14.2 CI/CD and the static Vercel page.
Mark gates complete only after their actual verification receipts exist.
Package-registry publishing and other live installations remain outside scope.
Context: [plan.md](plan.md).
Delivery: [v0.14.2 receipt](release-0.14.2.md). GitHub publication is complete;
Vercel production deployment is blocked on account sign-in.

## 1. Freeze the maintenance candidate

Dependencies: none. Scope: source/test selection and review receipts.

- [x] Confirm the final SDK repair review has no required findings and matches
      the exact current source hashes.
- [x] Recheck Git status, upstream ancestry, every changed path and the independent
      agent checklist; preserve unrelated work outside the release selection.
- [x] Save the reviewed maintenance changes in an explicitly authorized commit.
      Verify its paths, tree and parent: `90dc159`; the reviewed Windows Node 20
      test-launcher repair is `49e1d25`, the final release commit.

Acceptance: the candidate contains only this maintenance work, with no unresolved
verification failures or lost local work. Run GitNexus `detect_changes` before
committing and inspect its scope rather than treating a low rating as proof.

## 2. Prepare the version cohort

Dependencies: task 1. Likely files: root `Cargo.toml`, both `Cargo.lock` files,
`THIRD_PARTY_NOTICES.md`, `CHANGELOG.md`.

- [x] Confirm v0.14.2 is still unpublished and is the next chosen patch version.
- [x] Set the root package to 0.14.2, regenerate both lockfiles using Cargo, and
      structurally compare them: only the local package version may change.
- [x] Move the completed unreleased notes into a nonempty `## 0.14.2` section and
      update the current root-version notice. Preserve historical release notes.

Acceptance: package/lockfile versions agree, the dependency graph is unchanged,
and release tooling tests pass. Do not bump the starter or SDK package versions
merely to match the Rust release; bridge protocol remains v1.

## 3. Qualify the exact release candidate

Dependencies: task 2. Scope: existing checks, documentation examples and receipts.

- [x] Review current release-command examples and SDK/security guidance. The
      0.14.2 website badge is prepared in source; deployment is a separate gate.
- [x] Verify Rust debug and release all-target tests, release binaries, both examples,
      starter tests, formatting, Clippy, docs and Cargo package validation in CI.
- [x] Run all three SDK suites against the candidate bridge; verify TypeScript's
      installed-tarball declarations and Go vet/formatting.
- [x] Run all 13 release/documentation tooling tests and the dependency audit.
- [x] Verify declared minimum SDK runtimes: Python 3.10, Node 20 and Go 1.22.
      A newer toolchain's compatibility flags alone are not a runtime receipt.
- [x] After an authorized push, require complete Linux, Windows and macOS CI on
      `49e1d25`. Run `41304158443` passed all four jobs. The preceding Windows
      wildcard failure was reproduced, repaired, regression-tested and reviewed.

Core commands (use the platform-appropriate bridge path):

```sh
cargo build --locked --release --bins
cargo test --locked --all-targets
cargo test --locked --release --all-targets
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked --example basic_agent
cargo run --locked --example custom_tool
cargo test --locked --manifest-path starters/rust-agent/Cargo.toml
cargo doc --locked --no-deps
cargo package --locked
python -m pytest sdk/python/tests -q
npm --prefix sdk/typescript test
npm --prefix sdk/typescript run pack:check
go -C sdk/go test -count=1 ./...
go -C sdk/go vet ./...
python -m unittest discover -s scripts/tests -v
cargo audit
```

Acceptance: every suite passes on the exact candidate. Run deadline-sensitive
suites separately from compilation on constrained hosts; do not skip assertions,
relax production limits or call partial results green.

## 4. Publish only after authorization

Dependencies: task 3 and explicit permission to publish. Scope: the existing
[release workflow](../.github/workflows/release.yml).

- [x] Verify the published branch SHA and a new annotated v0.14.2 tag target.
- [x] Start the tag-triggered workflow and require all checks before publication.
- [x] Require extracted-archive SDK tests for Windows x64 MSVC, Linux x64 and
      macOS Apple Silicon, plus the complete asset set and SHA256SUMS.

Acceptance: a public, non-draft release exists at the intended commit, with the
expected archives and checksums. A tag push or successful build is not publication.

## 5. Verify delivery and document rollback

Dependencies: task 4. Likely file: `site/public/index.html`, if its release label
still needs updating.

- [x] Read back the public release, tag target and asset list; download and verify
      archive checksums and CLI version/help from the extracted artifact.
- [x] Prepare the website's 0.14.2 badge and release-notes link in source; verify
      desktop/mobile rendering, the Go tab and captured console errors locally.
- [ ] Deploy the verified website to the existing Vercel production project and
      read back the public page and assets. Requires Vercel account sign-in;
      the existing live page still advertises 0.13.1.
- [x] Record the release URL, commit, CI and release runs, artifact checks, and the
      retained v0.14.1 rollback target. Do not alter running installations without
      separate deployment authorization.

Acceptance: the report distinguishes implemented source, local checks,
cross-platform CI, published binaries and any live deployment.
