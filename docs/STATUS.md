# Current product status

MOTE provides a Rust library/CLI, a stdio bridge, thin Python/TypeScript SDKs, and a
Rust application starter. The current source line is **0.13.1**; use the
[changelog](../CHANGELOG.md) and [release page](https://github.com/danny-dis/MOTE/releases)
for version-specific changes and published downloads.

## Available

- YAML manifests with explicit grants, workspace configuration, model chains, and limits.
- Built-in file/directory actions, restricted shell/Git modes, and optional typed decisions.
- Registered Rust handlers and Python/TypeScript host callbacks through protocol v1.
- Model fallback, empty-response retry, unfinished-response rejection, and lossless JSON actions.
- Runtime/tool/observation budgets, cancellation hooks, and completion-write accounting.
- CLI post-run JSONL and SDK terminal events.
- Source-installable SDK packages and a starter that checks callbacks and output artifacts.
- Automated Windows x64, Linux x64, and macOS Apple Silicon binary distributions,
  extracted-archive checks, checksums, and license bundles.

## Not included

- An AI model, provider account, hosted inference service, or guaranteed provider uptime.
- OS sandboxing, multi-tenant authorization, or guaranteed descendant-process termination.
- A GUI, installed scheduler, memory/database service, browser, or MCP server.
- Dynamic plug-in loading, an `AGENT.md` loader, or live runtime-event streaming.
- Published npm/PyPI packages, signed/notarized binaries, or prebuilt Intel Mac downloads.

## Verification and interpretation

Cross-platform CI exercises runtime regression tests, SDK subprocess/callback tests,
source packaging, examples, starter, formatting, linting, and dependency auditing.
Release jobs run both SDK suites against extracted release bridges. Model responses
in deterministic integration tests are scripted fixtures, not external provider calls.

Live provider tests are opt-in and environment-dependent. A completed task or green
CI does not prove the semantic correctness of arbitrary model-generated outputs or
that an external endpoint is available. Your application must validate accepted results.

The v0.13.0 downloadable example used an invalid `api_key_env` field; 0.13.1 corrects
it to `auth_env` and tests the release manifest. This does not alter the bridge protocol.
See [Security](../SECURITY.md) before deploying unattended workloads.
