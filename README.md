# MOTE

**A small execution engine for AI agents. Build the application around it.**

[![CI](https://github.com/danny-dis/MOTE/actions/workflows/ci.yml/badge.svg?branch=production-ready)](https://github.com/danny-dis/MOTE/actions/workflows/ci.yml)
[![Release](https://github.com/danny-dis/MOTE/actions/workflows/release.yml/badge.svg)](https://github.com/danny-dis/MOTE/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

MOTE — **Minimal Orchestration & Task Execution** — is a Rust library and command-line
program. It asks a model for the next action, checks the configured capabilities,
executes a tool, and feeds the result back until the task finishes or a limit is reached.

Use it to build a focused document worker, repository assistant, or application-specific
agent without adopting a full agent platform. Your application owns the interface,
users, approvals, storage, scheduling, and acceptance checks.

[Download](https://github.com/danny-dis/MOTE/releases/latest) ·
[Quick start](docs/QUICKSTART.md) · [Documentation](docs/README.md) ·
[Build an application](docs/BUILDING_ON_MOTE.md) · [Changelog](CHANGELOG.md)

> MOTE is for trusted workspaces and trusted operators. Capability checks are **not an
> OS sandbox**. Connect your own model/provider; no model, account, or API key is bundled.
> The repository is currently private, so source and downloads require access.

## Choose your starting point

| You want to… | Start here |
|---|---|
| Run a task without installing Rust | [Download and run the CLI](docs/QUICKSTART.md) |
| Use Python functions as agent tools | [Python SDK](sdk/python/README.md) |
| Use JavaScript/TypeScript functions as tools | [TypeScript SDK](sdk/typescript/README.md) |
| Embed the engine directly in Rust | [Rust application starter](starters/rust-agent/README.md) |
| Configure models, permissions, and limits | [Configuration reference](docs/CONFIGURATION.md) |
| Contribute or build from source | [Contributing](CONTRIBUTING.md) |

## Download and run

Get the archive for your computer from [Releases](https://github.com/danny-dis/MOTE/releases/latest):

- **Windows x64:** ZIP containing `mote.exe` and `mote-bridge.exe`.
- **Linux x64:** tar.gz; Ubuntu 22.04 or compatible newer system with OpenSSL 3 and CA certificates.
- **macOS Apple Silicon:** tar.gz; macOS 14 or newer. Intel Macs currently require a source build.

Verify its checksum, extract it, and open a terminal in the extracted directory.
On Windows PowerShell:

```powershell
.\mote.exe --version
.\mote.exe --help
```

On Linux/macOS:

```sh
./mote --version
./mote --help
```

Edit the included `agent.yaml` to select your model endpoint and model name. Set the
credential environment variable named by `auth_env`, or remove that field for a
keyless local server. Put a small, non-sensitive file in `workspace`, then run:

```sh
./mote agent.yaml "List the workspace files and summarize them"
```

On Windows use `.\mote.exe` in place of `./mote`. The example grants only directory
listing and file reads. Follow the [full quick start](docs/QUICKSTART.md) for setup,
authentication, and troubleshooting. Binaries are currently unsigned/not notarized.

## What is included

- **Rust runtime and CLI:** YAML configuration, explicit capability grants, model
  fallback, run/tool limits, cancellation hooks, and post-run JSONL events.
- **Built-in tools:** file reads/writes, directory listing, restricted shell/Git modes,
  and optional typed decisions. Tool access is opt-in.
- **Custom tools:** application-owned Rust handlers, or Python/TypeScript callbacks
  through the separate `mote-bridge` process.
- **Thin SDKs:** source-installable packages with no third-party runtime dependencies.
- **Verified distributions:** automated cross-platform checks, extracted-archive
  integration tests, SHA-256 checksums, and upstream license notices.

The Rust package is `mote-agent`; its library import is `mote`. The unrelated `mote`
package on crates.io is **not this project**. These SDKs are not published to npm/PyPI.

## What MOTE deliberately does not provide

A chat UI, model hosting, database, scheduler, browser, MCP server, multi-user access
control, or sandbox. An `AGENT.md` file does not configure this runtime; the CLI reads
YAML. Tool callbacks are interactive, but runtime events arrive after the run, not as
live progress. A completed run is not proof its generated answer or file is correct.

See [current status](docs/STATUS.md), [architecture](docs/ARCHITECTURE.md), and
[security boundaries](SECURITY.md) before deploying unattended agents.

## Development and releases

From a source checkout with stable Rust and the platform's Rust build prerequisites:

```sh
cargo build --locked --release --bins
cargo test --locked --all-targets
cargo run --locked --example custom_tool
```

The examples use deterministic models and do not need a provider account.
See [Contributing](CONTRIBUTING.md) for the full verification commands.

Pushing a stable version tag matching `Cargo.toml` triggers the
[automatic release workflow](docs/RELEASING.md). It tests before publishing; maintainers
still choose and tag each version. There is no automatic release on every commit.

## Community and license

[Report a bug](https://github.com/danny-dis/MOTE/issues/new/choose) ·
[Support](SUPPORT.md) · [Security reporting](SECURITY.md#reporting-a-vulnerability) ·
[Design principles](DESIGN_PRINCIPLES.md)

MOTE is [MIT licensed](LICENSE). Binary archives include dependency license texts and
Rust standard-library notices; see [third-party notices](THIRD_PARTY_NOTICES.md).
Historical proposals and sample reports are preserved in [the archive](docs/archive/README.md),
not presented as shipped features.
