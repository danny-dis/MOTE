# Documentation audit facts (merged working tree)

- Package: Rust `mote` library and thin CLI, 0.12.x (`Cargo.toml`, `src/lib.rs`, `src/main.rs`). Exact release/version and merged CI status must be checked before updating counts here.
- Runtime: YAML `Manifest` (`src/config.rs`), 7 actions (`src/action.rs`), provider chain (OpenAI-compatible/DMR-X, Gemini, Anthropic in `src/model.rs`), optional typed decision adapters (`src/decision.rs`), JSONL events (`src/event.rs`), bounded iteration/run time, cancellation, per-tool limit (default 5), read/list-only transient retries (`src/runtime.rs`).
- Built-in file actions stay inside configured workspace (`src/capability.rs`). Shell and Git processes are NOT sandboxed, despite allowlists/timeouts; allowed programs can access host secrets/network. No AGENT.md loader, external policy engine, extension registry, MCP, browser, scheduler service, or guaranteed child-process isolation exists.
- Explicit successful write to `output_file` completes early. `complete:` with output_file writes a fresh report, replacing an existing one; a rejected write fails the run rather than reusing stale content. Shell/Git/write failures are not retried. Model chain retries an empty response once before falling back.
- `specs/` contains migrated manifests including historical keyless examples; parsing them does not demonstrate remote provider availability or successful scheduling. `run-agent.sh` is a wrapper, not an installed scheduler. Committed `reports/` are historical samples, not current health data.
- Local validation after the blank-response fix: 31 tests passed in both debug and release on Windows GNU with `C:/msys64/mingw64/bin/dlltool.exe` on PATH; release build, formatting, strict Clippy and `cargo audit` passed. The preceding merge commit `cad3ef3` passed GitHub CI on Windows, Linux, macOS, and dependency audit; that run predates the blank-response fix and does not validate its eventual commit.

## What to do with each document
- `README.md`: retain architecture; add accurate merged behavior, CLI quick start, local toolchain caveat, and honest untrusted-workload caveat.
- `docs/STATUS.md`: document exact implemented and missing controls and distinguish local tests from CI/real provider tests.
- `DESIGN_PRINCIPLES.md`: clarify capability authorization is not OS isolation; YAGNI is admission rule, not implemented feature.
- `ENGINEERING.md`, `SPEC.md`, `SDK_SPEC.md`: mark as design targets, not implementation; replace invalid manifest example in SPEC.
- `docs/jev-integration-spec.md`: mark adapters built, benchmark absent.
- `HOMELAB_MONITOR.md`, `SOVEREIGN_SCHEDULER.md`: demote unverified schedule/WhatsApp claims, link real manifest schema; update old runner/limits.
- `docs/LIFEOS-FABRIC-LEARNINGS.md`: leave as historical design notes unless conflicting product claims are found.
- `reports/*`: keep as historical samples, not proof of a current run.
