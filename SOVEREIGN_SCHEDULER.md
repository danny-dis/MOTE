# Sovereign Scheduler — MOTE Agent Layer

A design for scheduled agents running on top of MOTE v0.7.

## Architecture

```
┌─────────────────────────────────────────────┐
│                 Hermes Agent                 │
│           (Cron Job Scheduler)               │
│                                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │ 08:00   │  │ 14:00   │  │ 20:00   │    │
│  │ Morning │  │ Afternoon│ │ Evening │    │
│  │ Brief   │  │ Check   │ │  Recap  │    │
│  └────┬────┘  └────┬────┘  └────┬────┘    │
│       │            │            │           │
│       ▼            ▼            ▼           │
│  ┌─────────────────────────────────────┐    │
│  │          MOTE Runtime               │    │
│  │  ./mote specs/<agent>.yaml "task"   │    │
│  └─────────────────────────────────────┘    │
│       │                                     │
│       ▼                                     │
│  ┌─────────────────────────────────────┐    │
│  │      Output → WhatsApp Alert        │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

## Agent Roster

### 1. `homelab-monitor` — Every 2 hours
- Disk space, Docker, services, network
- Writes report to `~/homelab-reports/`
- Alerts only on issues

### 2. `git-watchdog` — Every 4 hours
- Scans repos for uncommitted changes, unpushed branches
- Reminds if you forgot to commit

### 3. `morning-brief` — Daily 08:00
- Weather, tasks, calendar summary
- Quick system health snapshot

### 4. `weekly-recap` — Sundays at 18:00
- Last 7 days of commits across all repos
- Disk usage trend
- Docker uptime stats

### 5. `security-scan` — Daily 02:00 (quiet hours)
- Check for exposed ports, outdated packages, failed SSH attempts
- Review Docker image ages

## Implementation

Each agent is:
1. A **spec file** (`specs/<name>.yaml`) — capabilities + model config
2. A **task prompt** — what to do each run
3. A **cron entry** — when to run it
4. An **output handler** — where to send results

## Example: git-watchdog

```yaml
# specs/git-watchdog.yaml
name: git-watchdog
capabilities: [shell, read_file, list_dir]
max_iterations: 15
model: "gemini-3.5-flash"
provider: "google"
auth: "<key>"
```

Task prompt:
```
Scan these repos for issues:
- C:\Users\pc\postiz-app
- C:\Users\pc\chimera
- C:\Users\pc\dmr-X

For each repo:
1. Check git status for uncommitted changes
2. Check for unpushed commits
3. List branches and their last commit date
4. Alert if a branch is stale (>7 days no activity)

Write findings to C:\Users\pc\homelab-reports\git-watchdog-{date}.md
End with STATUS: OK or ALERT: [issue]
```

Cron: `0 */4 * * *` (every 4 hours)

## Current Limitations

1. **Model reliability** — MOTE requires the model to perfectly follow the action format. Gemini 3.5 flash struggles with this. A more capable model (GPT-4o, Claude) or a tighter system prompt would help.
2. **No memory between runs** — each invocation is stateless. A memory layer (NOESIS integration?) would enable trend tracking.
3. **No conditional logic** — MOTE can't branch based on results. It's linear: think → act → observe.
4. **Single model per run** — no multi-model fallback if one fails.

## Next Steps

1. Test with a more capable model (GPT-4o-mini or Claude)
2. Add a wrapper script that parses MOTE output and routes alerts
3. Set up the cron jobs in Hermes
4. Build a dashboard that reads the report files
