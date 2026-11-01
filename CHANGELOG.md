# Changelog

User-facing changes are listed by release. Git tags identify source and binary
releases. SDK package versions and the bridge protocol have separate versions.

## Unreleased

## 0.14.2

- Exercise the declared SDK minimums in CI: Python 3.10, Node 20 and Go 1.22.
  Keep release tooling on Python 3.11, and extracted-release SDK checks on
  Python 3.11 and Node 22. Add a regression for this runtime separation.
- Consolidate action capability names, file-read/directory-list retry handling,
  and provider authentication lookup without changing public APIs or permissions.
- Remove unused Go stderr retention and the Python SDK's single-reader collection;
  keep bounded transport handling and existing process cleanup.
- Add regressions for capability grants, read/list budgets and errors,
  authentication values, and the exact repeated-action/iteration-limit failures.
- Make TypeScript fixtures independent of the working directory, flush terminal
  frames before closing, and check protocol errors and callback redaction explicitly.
  Separate ordinary fixture startup time from the existing short deadline checks.
- Make the Go cooperative-handler test verify a live deadline and its propagated
  cause after callback entry, and bound fixture compilation before tests start.
  Measure malformed-frame rejection after callback readiness, and require exact
  protocol errors in TypeScript lifecycle guards. Keep the separate short
  startup-timeout regressions unchanged.

## 0.14.1

- Add the official Go 1.22+ SDK (`sdk/go`) for bridge protocol v1, alongside
  Python and TypeScript. The standard-library-only SDK supports host tool
  callbacks, explicit capability grants, typed errors and per-run child processes.
- Define the supported Go SDK contract and release acceptance checks in
  [Go SDK specification](docs/GO_SDK_SPEC.md).
- Preserve TIMEOUT errors and context causes when deadlines or cancellation
  interrupt pipe writes; document synchronous callback cancellation limits.
- Exercise Go callbacks, redaction and real-bridge observations in tests, and
  include the Go suite in extracted-release integration checks.

## 0.14.0

- Strip symbols from release binaries. Use `CARGO_PROFILE_RELEASE_STRIP=none` to keep
  symbols in a diagnostic build.
- Discard unused SDK stderr output through the OS rather than retaining buffers;
  Python also avoids a stderr reader thread and queue.
- Removed experimental HTTP connection pooling: the locked ureq 2.12.1 library
  clears socket timeouts on reuse, allowing later requests to stall. Fresh
  per-request clients preserve connect/read/write timeouts. Added a regression
  that delays the second same-timeout request's response headers.

- Fixed Python SDK test startup assumptions and closed the local test server's socket.
  Timeout tests still enforce short deadlines and now check child-process termination;
  trailing-output and callback-error checks have stronger assertions.
- Isolated pending-callback test timing from child-process startup in the TypeScript
  SDK suite (test-only release fix; runtime deadlines are unchanged).
- HTTP test servers now consume complete requests before responding and closing,
  avoiding platform-specific socket resets in the timeout/size-limit fixtures.
- Rewrote documentation and website copy in plain language, preserving commands,
  API contracts, and security limits.
- Removed the unused input file and disk-check script, and the duplicate demo manifest.
- Removed a redundant report-preservation test covered by the stricter single-iteration
  output-write test, and a Rust indentation override that repeated the default.
- Consolidated documentation around the README, quick-start, architecture, and changelog.
- Removed obsolete proposals, audit snapshots, and sample reports from the current tree;
  prior versions remain in Git history.
- Replaced private-access instructions with public source and download links.
- Release notes are generated from the tagged version's changelog section.

## 0.13.1

### Fixed

- Corrected the downloadable `agent.yaml` authentication field from `api_key_env`
  to `auth_env`. The manifest parser rejected version 0.13.0's sample before it
  could call a model. Users of that release can apply the same one-line fix.
- Added a regression test that parses the actual release example and checks its
  read-only grants and credential-variable name.

### Documentation

- Put downloads first in the README and added a documentation index, configuration
  reference, architecture guide, contributor guide, troubleshooting, and issue
  and pull-request templates.
- Updated language guides and quick-start instructions to match the actual APIs.
- Separated historical proposals and sample reports from current product docs.

## 0.13.0

### Added

- First downloadable Windows x64, Linux x64, and macOS Apple Silicon releases.
- A tag-triggered workflow that builds and tests releases, verifies archives,
  bundles licenses, and publishes checksums.
- Separate `mote-bridge` executable with versioned stdio callbacks.
- Python and TypeScript SDKs that install from source, plus a Rust application starter.

### Reliability

- JSON actions preserve file content. The runtime rejects explicitly unfinished
  provider responses, counts completion writes against the normal write budget,
  and gives models clearer execution context.
- Regression, SDK integration, packaging, and cross-platform CI checks.

### Known limits

- The bundled example's authentication-field error is fixed in 0.13.1 (see above).
- Binaries are unsigned/not notarized; SDKs are not published to package registries.
- No OS sandbox, installed scheduler, live event streaming, or bundled model.
