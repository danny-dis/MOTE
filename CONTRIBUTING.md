# Contributing to MOTE

Thanks for contributing. Keep changes focused: MOTE is an execution engine, not a
full agent platform. Read [Architecture and design principles](docs/ARCHITECTURE.md)
and [Security](SECURITY.md) before proposing core features.

## Scope and discussion

Use an issue to describe the concrete task a feature enables, why application code
cannot provide it, and its dependency/security cost. For bugs, provide a minimal
reproduction with a redacted manifest. Do not post credentials or private workspace
contents. Security issues follow the private reporting process in `SECURITY.md`.

## Support and bug reports

Use [GitHub Issues](https://github.com/danny-dis/MOTE/issues) for usage questions or
reproducible bugs. There is no promised support SLA. Check the
[quick-start troubleshooting](docs/QUICKSTART.md#troubleshooting) and existing issues first.

Include the CLI/bridge/SDK versions, OS and architecture, source tag or commit,
redacted manifest, exact command, exit status, and expected versus actual result.
Reproduce in a small non-sensitive workspace and distinguish runtime completion
from whether the generated result is correct. Never attach credentials, private
workspace contents, or unredacted logs; report vulnerabilities via [Security](SECURITY.md#reporting-a-vulnerability).

## Development setup

Clone the repository and work from its root. The integration branch is
`production-ready`, not `main`. Source and downloads are public.

- Stable Rust and the native build tools required by your Rust target.
- Windows: MSVC Rust plus Visual Studio C++ build tools, or a complete compatible
  GNU/MinGW toolchain. Do not mix GNU and MSVC binaries/toolchains.
- Linux: a C toolchain, `pkg-config`, and OpenSSL development headers.
- macOS: Xcode command-line tools.
- Python 3.11 for repository release tooling and CI parity; the Python SDK's supported
  runtime is documented in its package metadata.
- Node 22 for CI parity with the TypeScript SDK.

```sh
cargo build --locked --release --bins
cargo test --locked --all-targets
cargo test --locked --release --all-targets
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo run --locked --example basic_agent
cargo run --locked --example custom_tool
cargo doc --locked --no-deps
python -m unittest discover -s scripts/tests -v
```

## SDK and starter checks

Build the release bridge first. Set `MOTE_BRIDGE` to its absolute path so SDK
end-to-end tests run rather than skip. POSIX shell:

```sh
export MOTE_BRIDGE="$PWD/target/release/mote-bridge"
```

Windows PowerShell:

```powershell
$env:MOTE_BRIDGE = (Resolve-Path .\target\release\mote-bridge.exe).Path
```

Then, from the repository root:

```sh
python -m pip install "./sdk/python[test]"
python -m pytest sdk/python/tests -q
npm --prefix sdk/typescript ci
npm --prefix sdk/typescript test
npm --prefix sdk/typescript run pack:check
cargo test --locked --manifest-path starters/rust-agent/Cargo.toml
cargo fmt --check --manifest-path starters/rust-agent/Cargo.toml
cargo clippy --locked --manifest-path starters/rust-agent/Cargo.toml --all-targets -- -D warnings
```

The SDK integration fixtures use a local scripted HTTP model server. They execute
real subprocesses and callbacks, but are not live-provider reliability tests.
For optional real-model testing, read `python scripts/live_smoke.py --help`, configure
your own endpoint, and independently inspect outputs. Do not put credentials in PRs.

CI also audits dependencies and checks the distributable Cargo package. To reproduce:

```sh
cargo install cargo-audit --locked
cargo audit
cargo package --locked
```

Package verification expects a clean tree. Use `--allow-dirty` only for local
pre-commit packaging checks, never as evidence of a published release.

## Pull requests

- Use a focused branch and target `production-ready`.
- Add a regression test before fixing a behavioral defect.
- Keep public API examples, configuration docs, and changelog entries consistent.
- Explain trust-boundary changes and side effects. Avoid speculative abstractions.
- Run the relevant checks and state exactly what passed, failed, or was not run.
- Never report a model-generated artifact as correct from exit status alone.
- Preserve lockfiles, existing work, historical evidence, and user data.

## Releases and documentation

Maintainers choose a new version and push its tag; CI builds and publishes binary
archives automatically. Follow [Releasing](docs/RELEASING.md). Do not move published
tags or replace released assets to hide mistakes. Keep current docs focused on
shipped behavior; obsolete proposals and reports can be recovered from Git history.
Use the root README as the documentation index and the changelog as the single
source for version-specific release notes.
