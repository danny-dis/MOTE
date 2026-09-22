# MOTE 0.12.0 status

MOTE is a reusable Rust library with a thin CLI. Capabilities default to empty, built-in file actions check workspace paths, and process execution has an executable allowlist, output cap and timeout. Cancellation and iteration controls and structured JSONL events are available. Model credentials are loaded from named environment variables.

These controls do not provide OS isolation. Allowed executables (especially interpreters and Git) can access files, network and secrets with the invoking user's permissions; subprocess descendants and blocked network calls may outlive cancellation. Run untrusted work in a separately isolated environment.

HTTP model adapters use the in-process Rust client and report non-success or malformed provider responses as errors. DMR-X can be used through an OpenAI-compatible local endpoint; see `specs/dmr-x-local.yaml`.
