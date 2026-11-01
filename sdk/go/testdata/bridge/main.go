// Command bridge is a deterministic, stdlib-only fixture that speaks MOTE
// protocol v1. It is driven entirely by the TEST_BRIDGE_MODE environment
// variable and is used by the Go SDK test suite. It never writes logs to
// stdout: stdout carries protocol frames only.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"time"
)

const (
	oversizedBytes = 1_048_577
	stderrFlood    = 200_000
)

func main() {
	mode := os.Getenv("TEST_BRIDGE_MODE")
	if mode == "" {
		mode = "success"
	}
	in := bufio.NewReaderSize(os.Stdin, 1<<20)

	switch mode {
	case "noread":
		// Never touch stdin. Used to prove a child that does not read cannot
		// hold startup open forever once the deadline elapses.
		time.Sleep(4 * time.Second)

	case "malformed":
		readLine(in)
		writeRaw("{bad}\n")

	case "malformed_after_callback":
		readLine(in)
		writeLine(map[string]any{"type": "tool_call", "id": 1, "name": "x", "input": map[string]any{}})
		readLine(in)
		writeRaw("{bad}\n")
		// Stay alive so prompt rejection cannot be mistaken for child exit.
		time.Sleep(30 * time.Second)

	case "oversized":
		readLine(in)
		writeRaw(strings.Repeat("x", oversizedBytes) + "\n")

	case "invalidutf8":
		readLine(in)
		_, _ = os.Stdout.Write([]byte{0xff, 0xfe, 0x0a})

	case "truncated":
		readLine(in)
		writeRaw(`{"type":"run_result"`)

	case "blank":
		readLine(in)
		writeRaw("\n")

	case "wrongid":
		readLine(in)
		writeLine(map[string]any{"type": "tool_call", "id": 2, "name": "x", "input": map[string]any{}})

	case "duplicate":
		readLine(in)
		completed := encode(map[string]any{"type": "run_result", "state": "completed", "events": []any{}})
		writeRaw(completed)
		writeRaw(completed)

	case "missingresult":
		readLine(in)
		// Exit without ever producing a terminal frame.

	case "mismatch":
		readLine(in)
		writeLine(map[string]any{"type": "run_result", "state": "completed", "events": []any{}})
		os.Exit(1)

	case "trailing":
		readLine(in)
		writeLine(map[string]any{"type": "run_result", "state": "completed", "events": []any{}})
		// One extra byte/frame after the valid terminal result.
		writeRaw("{}")

	case "stderrflood":
		readLine(in)
		_, _ = os.Stderr.WriteString(strings.Repeat("x", stderrFlood))
		writeLine(map[string]any{"type": "run_result", "state": "completed", "events": []any{}})

	case "hang":
		readLine(in)
		time.Sleep(30 * time.Second)

	case "denied":
		readLine(in)
		writeLine(map[string]any{"type": "tool_call", "id": 1, "name": "x", "input": map[string]any{}})
		// Wait for the host's reply (an error, since the tool is not granted),
		// then report failure regardless of what it says.
		readLine(in)
		writeLine(map[string]any{"type": "run_result", "state": "failed", "events": []any{}})
		os.Exit(1)

	case "success":
		readLine(in)
		writeLine(map[string]any{"type": "tool_call", "id": 1, "name": "x", "input": map[string]any{"value": "hello"}})
		var reply map[string]any
		if err := json.Unmarshal([]byte(readLine(in)), &reply); err != nil {
			os.Exit(3)
		}
		writeLine(map[string]any{
			"type":   "run_result",
			"state":  "completed",
			"events": []any{map[string]any{"type": "hello", "reply": reply}},
		})

	default:
		fmt.Fprintln(os.Stderr, "unknown TEST_BRIDGE_MODE:", mode)
		os.Exit(2)
	}
}

func readLine(in *bufio.Reader) string {
	line, _ := in.ReadString('\n')
	return line
}

func encode(value any) string {
	raw, err := json.Marshal(value)
	if err != nil {
		fmt.Fprintln(os.Stderr, "encode:", err)
		os.Exit(3)
	}
	return string(raw) + "\n"
}

func writeRaw(raw string) {
	_, _ = os.Stdout.WriteString(raw)
}

func writeLine(value any) {
	writeRaw(encode(value))
}
