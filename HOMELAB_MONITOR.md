# MOTE Homelab Monitor Agent

A scheduled health-check agent that runs on top of MOTE to monitor your homelab infrastructure.

## What it does (every 2 hours)

1. **Disk space** — checks all drives, alerts below 15% free
2. **Docker** — lists running/stopped containers, alerts on unexpected stops
3. **Services** — checks critical services (Postiz, OpenClaw, etc.)
4. **Network** — pings gateway, checks internet, DNS resolution
5. **Git repos** — scans for uncommitted changes, unpushed commits
6. **Report** — writes a timestamped status report to `~/homelab-reports/`
7. **Alert** — if anything is wrong, outputs a summary for the cron job to forward

## Agent Spec

```yaml
name: homelab-monitor
capabilities: [shell, read_file, write_file]
max_iterations: 12
model: "gemini-3.5-flash"
provider: "google"
```

## Task Prompt

```
You are a homelab monitoring agent. Run these checks and report:

1. Disk space: Get free space on all drives. Alert if any drive is below 15% free.
2. Docker: List all containers (running and stopped). Note any that are unexpectedly stopped.
3. Services: Check if these processes are running: docker, node, python, nginx.
4. Network: Ping 8.8.8.8 and 1.1.1.1. Check if google.com resolves.
5. Git repos: Check C:\Users\pc\postiz-app and C:\Users\pc\chimera for uncommitted changes.
6. Write a report to C:\Users\pc\homelab-reports\{timestamp}.md with all findings.
7. If any check fails, end with "ALERT: [issue]". If all good, end with "STATUS: OK".

Be thorough but concise. Use shell commands to gather real data.
```

## Schedule

Run every 2 hours via Hermes cron. On failure, send alert to WhatsApp.
