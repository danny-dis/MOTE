# MOTE homelab monitor example

**Status:** `specs/homelab-monitor.yaml` is an example workload, not evidence of an installed two-hour schedule or live alert delivery. Its configured DMR-X endpoint must be running. It grants `shell`, `read_file`, and `write_file`; the shell allowlist contains `powershell` and `docker`. Those programs retain the invoking user's permissions — this is **not** an OS sandbox.

```bash
cargo build --release
./run-agent.sh homelab-monitor
```

The runner asks MOTE to inspect disk, Docker, and service health and write `reports/homelab-latest.md` under the configured workspace. It returns a nonzero exit on MOTE failure or if the newly written report contains `ALERT:`; an external scheduler may route that exit status. The runner does not install cron jobs, send WhatsApp messages, or guarantee that a model successfully wrote a current report. Existing `reports/` files are historical samples. For real monitoring, verify the endpoint, report timestamp/content, and scheduler/notification integration separately. See [docs/STATUS.md](docs/STATUS.md).
