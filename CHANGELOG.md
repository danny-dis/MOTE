# Changelog

This file records user-facing changes. Git tags identify source and binary releases.
SDK package versions and the bridge protocol are versioned separately.

## Unreleased

- Consolidated documentation around the README, quick-start, architecture, and changelog.
- Removed obsolete proposals, audit snapshots, and sample reports from the current tree;
  prior versions remain in Git history.
- Public source and download links replace private-access instructions.
- Release notes are generated from the tagged version's changelog section.

## 0.13.1

### Fixed

- Corrected the downloadable `agent.yaml` authentication field from `api_key_env`
  to `auth_env`. Version 0.13.0's sample was rejected by the manifest parser before
  model execution; users of that release can make the same one-line correction.
- Added a regression test that parses the actual release example and checks its
  read-only grants and credential-variable name.

### Documentation

- Release-first README, documentation index, configuration reference, architecture,
  contributor guide, troubleshooting, and issue/pull-request templates.
- Updated language guides and quick-start instructions to match the actual APIs.
- Clearly separated historical proposals and sample reports from current product docs.

## 0.13.0

### Added

- First downloadable Windows x64, Linux x64, and macOS Apple Silicon releases.
- Tag-triggered build, test, archive verification, license bundling, and checksum publication.
- Separate `mote-bridge` executable with versioned stdio callbacks.
- Source-installable Python and TypeScript SDKs and a Rust application starter.

### Reliability

- Lossless JSON actions, rejection of explicitly unfinished provider responses,
  completion writes counted against the normal write budget, and clearer model context.
- Regression, SDK integration, packaging, and cross-platform CI checks.

### Known limits

- The bundled example's authentication-field error is fixed in 0.13.1 (see above).
- Binaries are unsigned/not notarized; SDKs are not published to package registries.
- No OS sandbox, installed scheduler, live event streaming, or bundled model.
