# Build an application on MOTE

MOTE supplies the execution loop. Your application owns authentication, UI,
persistence, scheduling, business tools, approvals, and output validation.
An HTTP worker service is not required.

## Choose an integration

- **[Python SDK](../sdk/python/README.md):** Python 3.10+, standard-library runtime;
  install from source and register Python functions as tools.
- **[TypeScript / Node SDK](../sdk/typescript/README.md):** Node 20+, no runtime npm
  dependencies; build/install the source package and register JavaScript callbacks.
- **[Rust starter](../starters/rust-agent/README.md):** embed the library directly,
  register a Rust handler, and validate its output file. No bridge is needed.
- **[CLI](QUICKSTART.md):** run YAML-configured tasks without embedding a library.

Each language guide is the canonical source for installation, runnable examples,
API details, errors, and tests. SDKs are not published to npm/PyPI and do not install
or download the bridge. The Rust package is `mote-agent`, imported as `mote`; use a
local path or pinned Git revision for this source, not the unrelated `mote` crate.

## Get the bridge

Python/TypeScript users can [download](https://github.com/danny-dis/MOTE/releases/latest)
the CLI/bridge archive without installing Rust. Follow the [quick start](QUICKSTART.md),
verify the checksum, and pass the absolute path to `mote-bridge` (`mote-bridge.exe`
on Windows). Downloads are public but unsigned/not notarized.

Alternatively, build both executables with stable Rust and its native build tools:

```sh
cargo build --locked --release --bins
```

The binaries are under `target/release`. SDKs use one bridge subprocess per run,
without a command shell, and support executable paths containing spaces. Both SDKs
can use built-in capabilities without custom callbacks. Select a reachable model
and configure credentials by environment-variable name using `auth_env`; see
[Configuration](CONFIGURATION.md).

## Application responsibilities

- **Grants:** register a tool handler and explicitly grant its name in the manifest.
  Models cannot invent executable tools or grant permissions.
- **Validation:** callbacks take JSON objects and return strings. Validate their
  fields yourself; descriptions are not JSON Schema enforcement.
- **Side effects:** independently validate results before accepting output or
  publishing changes. Failed or timed-out runs may already have written files or
  invoked callbacks; do not blindly retry them.
- **Isolation:** callbacks retain host-user privileges. Neither the SDK nor bridge
  is a sandbox. Bound or isolate blocking callbacks and untrusted workloads yourself.
- **Events:** callbacks are interactive; runtime event lists arrive after completion.
  Protect logs and observations that may contain workspace data.
- **Versions:** match SDK source to the runtime release. SDK package versions and
  the [bridge protocol](BRIDGE_PROTOCOL.md) are independent of the crate version.

See [Security](../SECURITY.md) for deployment boundaries and
[Contributing](../CONTRIBUTING.md#sdk-and-starter-checks) for verification commands.
CI executes real subprocesses and callbacks against scripted local model fixtures;
live endpoint availability and model-output correctness need separate checks.
