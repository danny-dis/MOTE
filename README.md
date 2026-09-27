# MOTE

**A small execution engine for AI agents. Build the application around it.**

[![CI](https://github.com/danny-dis/MOTE/actions/workflows/ci.yml/badge.svg?branch=production-ready)](https://github.com/danny-dis/MOTE/actions/workflows/ci.yml)
[![Release](https://github.com/danny-dis/MOTE/actions/workflows/release.yml/badge.svg)](https://github.com/danny-dis/MOTE/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

MOTE — **Minimal Orchestration & Task Execution** — is a Rust library and command-line
program. It asks a model for the next action, checks the configured capabilities,
executes a tool, and feeds the result back until the task finishes or a limit is reached.
Your application owns the interface, users, approvals, storage, scheduling, and output validation.

[Website](https://mote-runtime.vercel.app) ·
[Download](https://github.com/danny-dis/MOTE/releases/latest) ·
[Quick start](docs/QUICKSTART.md) · [Documentation](#documentation) · [Changelog](CHANGELOG.md)

> Source and downloads are public. Bring your own model and credentials.
> Capability checks are **not an OS sandbox**: use trusted workspaces and operators.

## Download and run

Get the archive for your computer from [Releases](https://github.com/danny-dis/MOTE/releases/latest):

- **Windows x64:** ZIP containing `mote.exe` and `mote-bridge.exe`.
- **Linux x64:** tar.gz; Ubuntu 22.04-compatible system with OpenSSL 3 and CA certificates.
- **macOS Apple Silicon:** tar.gz; macOS 14 or newer. Other architectures require a source build.

No Rust installation is needed for the binaries. Verify the checksum and extract the
archive. Follow the [quick start](docs/QUICKSTART.md) to configure `agent.yaml`, select
a reachable model endpoint, and set the environment variable named by `auth_env`.
For a keyless local model, remove `auth_env`. Put a non-sensitive file in `workspace`, then run:

```sh
./mote --jsonl agent.yaml "List the workspace files and summarize them"
```

On Windows PowerShell, replace `./mote` with `.\mote.exe`.
The example grants only directory listing and file reads. JSONL events arrive after
the run; without `--jsonl`, the CLI prints final state and event count, not a chat answer.
Binaries are unsigned/not notarized; checksums verify integrity, not publisher identity.

## What is included

- **Rust library and CLI:** YAML configuration, capability grants, model fallback,
  run/tool/observation limits, cancellation hooks, and post-run events.
- **Built-in tools:** file reads/writes, directory listing, restricted shell/Git modes,
  and optional typed decisions. Access is opt-in.
- **Application-owned tools:** Rust handlers or Python/TypeScript callbacks through
  the separate `mote-bridge` process. Registration does not grant permission.
- **Thin SDKs and a Rust starter:** no third-party SDK runtime dependencies.
- **Tested distributions:** cross-platform CI, extracted-archive integration tests,
  SHA-256 checksums, and bundled dependency license notices.

The Rust package is `mote-agent`, imported as `mote`; the unrelated `mote` package
on crates.io is **not this project**. Python/TypeScript SDKs install from source,
not npm/PyPI. Use source and SDKs matching your installed release.

## What MOTE does not provide

A bundled model, hosted inference, GUI, database/memory service, scheduler, browser,
MCP server, multi-user access control, dynamically loaded plugins, or OS sandbox.
`AGENT.md` does not configure the runtime; the CLI reads YAML.

Tool callbacks are interactive, but runtime events are buffered until the run ends.
Timeouts do not guarantee termination of all descendant processes or blocking host callbacks.
A completed run or green CI does not prove model-output correctness or provider availability.
Deterministic tests use scripted model responses; validate real outputs in your deployment.

## Documentation

- **Use:** [Quick start and troubleshooting](docs/QUICKSTART.md) ·
  [Configuration reference](docs/CONFIGURATION.md).
- **Build:** [Choose an integration](docs/BUILDING_ON_MOTE.md) ·
  [Python SDK](sdk/python/README.md) · [TypeScript SDK](sdk/typescript/README.md) ·
  [Rust starter](starters/rust-agent/README.md).
- **Understand:** [Architecture and design principles](docs/ARCHITECTURE.md) ·
  [Bridge protocol](docs/BRIDGE_PROTOCOL.md) · [Security](SECURITY.md).
- **Maintain:** [Contributing and support](CONTRIBUTING.md) ·
  [Release procedure](docs/RELEASING.md) · [Changelog](CHANGELOG.md).

Source docs follow `production-ready`. For an installed release, use the matching Git
tag and bundled quick-start. SDK package and bridge protocol versions are independent
of the Rust crate version. Maintainers choose version tags; ordinary commits do not
publish binary releases.

## Community and license

[Report a bug or ask a question](https://github.com/danny-dis/MOTE/issues/new/choose) ·
[Security reporting](SECURITY.md#reporting-a-vulnerability)

MOTE is [MIT licensed](LICENSE). Binary archives include dependency license texts and
Rust standard-library notices; see [third-party notices](THIRD_PARTY_NOTICES.md).
