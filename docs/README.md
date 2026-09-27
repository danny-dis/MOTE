# Documentation

Start with the path matching your task. These guides describe the shipped runtime;
old proposals are separated into the archive.

## Use MOTE

- [Quick start](QUICKSTART.md): download, configure a model, run a task.
- [Configuration reference](CONFIGURATION.md): manifest fields, capabilities, authentication, CLI output.
- [Current status](STATUS.md): supported features and explicit limitations.
- [Troubleshooting and support](../SUPPORT.md): common failures and useful bug reports.
- [Security](../SECURITY.md): deployment boundaries and vulnerability reporting.

## Build an application

- [Application guide](BUILDING_ON_MOTE.md): choose Rust, Python, or TypeScript.
- [Python SDK](../sdk/python/README.md).
- [TypeScript SDK](../sdk/typescript/README.md).
- [Rust starter](../starters/rust-agent/README.md).
- [Architecture](ARCHITECTURE.md): what each layer does and what your application owns.
- [Bridge protocol v1](BRIDGE_PROTOCOL.md): interoperating over stdin/stdout.

## Contribute and maintain

- [Contributing](../CONTRIBUTING.md): setup, tests, review, and scope.
- [Design principles](../DESIGN_PRINCIPLES.md): keep the engine small.
- [Changelog](../CHANGELOG.md): versioned changes.
- [Release procedure](RELEASING.md): tested, automatic binary publishing.
- [License inventory](../THIRD_PARTY_NOTICES.md): source inventory versus bundled notices.
- [Historical archive](archive/README.md): preserved proposals, research, and sample reports.

## Version scope

The source documentation follows `production-ready`. For an installed release, use
its bundled quick-start and the source tree at the matching Git tag. SDK package
versions and the bridge protocol version are independent of the Rust crate version.
No documentation page promises provider uptime, universal model compatibility, or
correctness of model-generated output.
