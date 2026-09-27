# Sovereign Scheduler — optional MOTE workloads

> **Historical proposal snapshot — not current product documentation.** This file preserves proposed workload scheduling and does not establish live health, installed schedules, or shipped features. For current MOTE documentation, see [STATUS](../STATUS.md), [BUILDING_ON_MOTE](../BUILDING_ON_MOTE.md), and [QUICKSTART](../QUICKSTART.md).

**Status:** repository manifests and an optional runner script, not installed cron jobs or alert delivery. No schedule, WhatsApp routing, or live model success is proven by this repository. Historical files under `reports/` are samples, not current health data.

The five example workloads are `homelab-monitor`, `git-watchdog`, `morning-brief`, `weekly-recap`, and `security-scan` under `specs/`. Each is constrained by its own manifest's workspace, model endpoint, and allowed capabilities. Example endpoints require a running service or provider credentials. The runner uses those manifests and passes its exit status to an external scheduler:

```bash
cargo build --release
./run-agent.sh homelab-monitor
./run-agent.sh git-watchdog "Check this workspace and write a report"
```

`git-watchdog` is scoped to the manifest workspace; the safe Git mode does **not** scan other repositories. To monitor additional repos, schedule separately scoped runs in an externally isolated environment. `security-scan` is a reduced workspace-only example with `cargo`/`git` access; it does **not** check host ports, Docker, Windows events, or npm, and has no configured report or alert exit behavior. A report already at `output_file` is not proof of this run's success: MOTE must explicitly write it or replace it with a fresh `complete:` summary. A failed write fails the run.

Possible schedules (not installed): homelab every two hours, Git watchdog every four hours, morning brief daily, weekly recap Sunday, security scan daily. Connect a scheduler and notification transport outside MOTE if these are needed. The shell/Git allowlist is not a sandbox; do not send untrusted model output to a privileged host. See [docs/STATUS.md](../STATUS.md).
