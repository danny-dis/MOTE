# Rust application starter

This is a small application embedding MOTE, not a mock-model demo. Its CLI loads
an actual HTTP model chain from YAML, registers a Rust `uppercase` handler, grants
it explicitly, writes an output file, and independently checks both tool execution
and the exact artifact. It refuses to report success merely because a model said
it finished.

From the repository root:

```bash
# First edit the endpoint/model in starters/rust-agent/agent.yaml.
# For authenticated endpoints, set auth_env to an environment-variable NAME.
cargo run --locked --manifest-path starters/rust-agent/Cargo.toml -- starters/rust-agent/agent.yaml
```

The default endpoint is local DMR-X; a reachable provider is still required.
Success prints `Verified report.txt: MOTE WORKS`. The file is written under
`starters/rust-agent/workspace/`; workspace paths are relative to the YAML file.
No API key is bundled. Runtime observations are sent to the model endpoint.

## Make it your application

1. Copy this directory into your project and update its `mote-agent` dependency.
   The in-repo path points at `../..`. Outside this checkout, use a pinned Git
   revision of `https://github.com/danny-dis/MOTE` (package `mote-agent`, Rust import
   `mote`). Do not accidentally depend on the unrelated crates.io `mote` package.
2. Replace `uppercase` with your host-owned operation; validate its input and
   bound its blocking work. Describe its input contract to the model.
3. Grant only required names in YAML. Registration alone grants no permission.
4. Replace the task and `run` acceptance criteria with your product's checks.
5. Supply your own UI, persistence, scheduling, approvals and workload isolation.

## Deterministic tests

```bash
cargo test --locked --manifest-path starters/rust-agent/Cargo.toml
cargo fmt --check --manifest-path starters/rust-agent/Cargo.toml
cargo clippy --locked --manifest-path starters/rust-agent/Cargo.toml --all-targets -- -D warnings
```

These use scripted `Model` implementations and no API keys. They test accepted
output, rejected output, and a model attempting to skip the required callback.
They do not prove provider availability. Neither the runtime nor custom handlers
are an OS sandbox; use isolated workers for untrusted workloads.
