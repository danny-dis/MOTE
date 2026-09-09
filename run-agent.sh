#!/usr/bin/env bash
# Sovereign Scheduler — MOTE Agent Runner
# Usage: ./run-agent.sh <agent-name> ["task prompt"]
# Example: ./run-agent.sh homelab-monitor

set -euo pipefail

AGENT_NAME="${1:?Usage: run-agent.sh <agent-name> [task-prompt]}"
TASK_PROMPT="${2:-}"
MOTE_DIR="C:/Users/pc/agents/MOTE"
SPEC_FILE="$MOTE_DIR/specs/$AGENT_NAME.yaml"
BINARY="$MOTE_DIR/target/release/mote.exe"
REPORT_DIR="C:/Users/pc/agents/MOTE/reports"

if [ ! -f "$SPEC_FILE" ]; then
    echo "ERROR: Spec file not found: $SPEC_FILE"
    exit 1
fi

if [ ! -f "$BINARY" ]; then
    echo "ERROR: MOTE binary not found. Run: cd $MOTE_DIR && cargo build --release"
    exit 1
fi

# Default task prompts per agent
if [ -z "$TASK_PROMPT" ]; then
    case "$AGENT_NAME" in
        homelab-monitor)
            TASK_PROMPT="You are a homelab monitoring agent. Run these checks and write a report:\n1. Disk space: df -h C: | tail -1. Alert if below 15% free.\n2. Docker: docker ps -a --format 'table {{.Names}}\t{{.Status}}'. Note stopped containers.\n3. Services: tasklist | grep -i 'docker\|node\|python\|nginx'.\n4. Network: ping -n 1 8.8.8.8. Check exit code.\n5. Write findings to C:\\Users\\pc\\agents\\MOTE\\reports\\homelab-latest.md\nEnd with STATUS: OK or ALERT: [issue]"
            ;;
        git-watchdog)
            TASK_PROMPT="Scan these repos for uncommitted changes and unpushed commits:\n- C:\\Users\\pc\\postiz-app\n- C:\\Users\\pc\\chimera\n- C:\\Users\\pc\\dmr-X\n\nFor each: git status, git log --oneline -3, git branch -vv.\nAlert if uncommitted changes exist or branches are ahead of remote.\nWrite findings to C:\\Users\\pc\\agents\\MOTE\\reports\\git-watchdog-latest.md\nEnd with STATUS: OK or ALERT: [issue]"
            ;;
        morning-brief)
            TASK_PROMPT="Generate a morning brief. Include:\n1. Current date and time\n2. Weather from curl wttr.in/Nairobi?format=3\n3. System uptime\n4. Pending tasks from C:\\Users\\pc\\agents\\MOTE\\reports\\ (any ALERT files?)\nWrite to C:\\Users\\pc\\agents\\MOTE\\reports\\morning-brief-latest.md\nEnd with STATUS: OK"
            ;;
        weekly-recap)
            TASK_PROMPT="Generate a weekly recap:\n1. Disk usage trend: df -h C:\n2. Last 7 days of commits across repos (postiz-app, chimera, dmr-X)\n3. Docker uptime: docker ps --format 'table {{.Names}}\t{{.RunningFor}}'\n4. Report files from this week in C:\\Users\\pc\\agents\\MOTE\\reports\\\nWrite to C:\\Users\\pc\\agents\\MOTE\\reports\\weekly-recap-latest.md\nEnd with STATUS: OK"
            ;;
        security-scan)
            TASK_PROMPT="Run a security scan:\n1. Listening ports: netstat -an | grep LISTEN\n2. Docker images older than 30 days: docker images --format 'table {{.Repository}}\t{{.Tag}}\t{{.CreatedSince}}'\n3. Check for failed SSH attempts in Windows Event Log (wevtutil qe Security /q:\"*[System[EventID=4625]]\" /c:5)\n4. Outdated npm packages in chimera: cd C:\\Users\\pc\\chimera && npm outdated 2>&1 | head -20\nWrite findings to C:\\Users\\pc\\agents\\MOTE\\reports\\security-scan-latest.md\nEnd with STATUS: OK or ALERT: [issue]"
            ;;
        *)
            echo "ERROR: Unknown agent: $AGENT_NAME"
            exit 1
            ;;
    esac
fi

echo "=== Sovereign Scheduler ==="
echo "Agent: $AGENT_NAME"
echo "Time: $(date)"
echo "========================="

# Run MOTE
cd "$MOTE_DIR"
OUTPUT=$(./target/release/mote.exe "specs/$AGENT_NAME.yaml" "$TASK_PROMPT" 2>&1) || true

echo "$OUTPUT"

# Check for alerts
ALERTS=$(echo "$OUTPUT" | grep -i "ALERT:" || true)
STATUS=$(echo "$OUTPUT" | grep -E "Final state:" | head -1)
ITERATIONS=$(echo "$OUTPUT" | grep -E "Iterations used:" | head -1)

# Summary for cron delivery
echo ""
echo "========================="
echo "SUMMARY: $AGENT_NAME"
echo "$STATUS"
echo "$ITERATIONS"
if [ -n "$ALERTS" ]; then
    echo "⚠️ ALERTS FOUND:"
    echo "$ALERTS"
fi
echo "========================="

# Exit with error code if alerts found (for cron notification)
if [ -n "$ALERTS" ]; then
    exit 1
fi

exit 0
