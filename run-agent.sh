#!/usr/bin/env bash
# Optional scheduler wrapper: ./run-agent.sh <spec-name> [task prompt]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT="${1:?Usage: run-agent.sh <spec-name> [task prompt]}"
case "$AGENT" in
    *[!a-zA-Z0-9_-]* | '') printf 'Invalid spec name: %s\n' "$AGENT" >&2; exit 2 ;;
esac
SPEC="$ROOT/specs/$AGENT.yaml"
if [ ! -f "$SPEC" ]; then
    printf 'Missing manifest: %s\n' "$SPEC" >&2
    exit 2
fi
if [ -x "$ROOT/target/release/mote.exe" ]; then
    BINARY="$ROOT/target/release/mote.exe"
elif [ -x "$ROOT/target/release/mote" ]; then
    BINARY="$ROOT/target/release/mote"
else
    printf 'MOTE binary missing; run cargo build --release\n' >&2
    exit 2
fi

case "$AGENT" in
    homelab-monitor | homelab-monitor-v2) DEFAULT_TASK='Check disk, Docker, and service health with permitted actions. Write findings to reports/homelab-latest.md; include STATUS: OK or ALERT: issue.'; REPORT="$ROOT/reports/homelab-latest.md" ;;
    git-watchdog) DEFAULT_TASK='Check this workspace Git status and recent commits. Write findings to reports/git-watchdog-latest.md; include STATUS: OK or ALERT: issue.'; REPORT="$ROOT/reports/git-watchdog-latest.md" ;;
    morning-brief) DEFAULT_TASK='Prepare a brief from accessible workspace data; report unavailable data honestly.' ;;
    weekly-recap) DEFAULT_TASK='Summarize accessible workspace activity from the past week; report unavailable data honestly.' ;;
    security-scan) DEFAULT_TASK='Inspect accessible workspace security signals; report only observed findings.' ;;
    *) DEFAULT_TASK='Describe the current workspace using the permitted capabilities.' ;;
esac
TASK="${2:-$DEFAULT_TASK}"
cd "$ROOT"
if OUTPUT=$("$BINARY" --jsonl "specs/$AGENT.yaml" "$TASK" 2>&1); then
    printf '%s\n' "$OUTPUT"
else
    CODE=$?
    printf '%s\n' "$OUTPUT" >&2
    exit "$CODE"
fi
# For report-producing workloads, inspect the fresh report written during the
# successful run rather than old observations echoed by the model.
if [ -n "${REPORT:-}" ]; then
    if [ ! -f "$REPORT" ]; then
        printf 'Successful run did not leave report: %s\n' "$REPORT" >&2
        exit 1
    fi
    if grep -qi 'ALERT:' "$REPORT"; then
        exit 1
    fi
fi
