# MOTE Rust application starter

This starter embeds MOTE directly in a Rust application. It is not a mock-model
demo: the CLI loads a real HTTP model chain from `agent.yaml`, registers a Rust
`uppercase` handler, grants it explicitly, writes `report.txt`, and independently
validates both tool use and the exact artifact.

The starter is source-based and uses the repository's path dependency
`mote-agent` (imported as `mote`); it is not published to crates.io. Rust and a
stable Rust toolchain are required. It does not use `mote-bridge` or either
language SDK.

## Run it

Run these commands from the repository root:

```bash
# First edit starters/rust-agent/agent.yaml for a reachable model endpoint and model.
cargo run --locked --manifest-path starters/rust-agent/Cargo.toml -- starters/rust-agent/agent.yaml
```

The default YAML points at a local DMR-X-style endpoint and model `auto`; no
provider or gateway is bundled. For an authenticated endpoint, set
`auth_env` to the name of an environment variable containing the credential;
never put the credential in YAML. The endpoint must be reachable and return a
supported completion. See [CONFIGURATION](../../docs/CONFIGURATION.md) for
manifest and model fields, [BUILDING_ON_MOTE](../../docs/BUILDING_ON_MOTE.md)
for integration guidance, and [QUICKSTART](../../docs/QUICKSTART.md) for
release/runtime context.

On success the application prints `Verified report.txt: MOTE WORKS`. The file
is written under `starters/rust-agent/workspace/`, because the workspace path
is resolved relative to the YAML file. The application refuses success unless
the `uppercase` tool was actually called and the artifact is exactly
`MOTE WORKS`; a model claiming completion is not enough.

## Make it your application

1. Copy this starter into your project and update its `mote-agent` path
   dependency. For a project outside this checkout, use a pinned Git revision
   of `https://github.com/danny-dis/MOTE` and package `mote-agent`; do not
   accidentally select an unrelated crates.io package named `mote`.
2. Replace `uppercase` with a host-owned operation. Validate input and bound
   blocking work; its description guides the model but is not schema validation.
3. Grant only the required names in `capabilities`. Registration alone is not
   permission.
4. Replace the task and `run` acceptance checks with product-specific checks.
5. Supply your own authentication, UI, persistence, scheduling, approvals and
   workload isolation.

Host tools and the runtime execute with the application's privileges. MOTE does
not provide an OS sandbox, and this starter makes no sandbox promise. Isolate
untrusted workloads yourself. Model observations and runtime events may contain
workspace data and are sent to the configured model; handle them as sensitive.
The Rust API returns a final event list rather than live progress events.

## Deterministic checks

Run from the repository root:

```bash
cargo test --locked --manifest-path starters/rust-agent/Cargo.toml
cargo fmt --check --manifest-path starters/rust-agent/Cargo.toml
cargo clippy --locked --manifest-path starters/rust-agent/Cargo.toml --all-targets -- -D warnings
```

These tests use scripted `Model` implementations and no API keys. They cover
accepted output, rejected output, and a model attempting to skip the required
callback. They do not prove provider availability or live inference.
