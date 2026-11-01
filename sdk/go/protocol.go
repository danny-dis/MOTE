package mote

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"io"
	"math"
	"unicode/utf8"
)

const maxFrame = 1_048_576

type frameReader struct {
	r *bufio.Reader
}

func newFrameReader(r io.Reader) *frameReader {
	return &frameReader{r: bufio.NewReaderSize(r, maxFrame+1)}
}

func (f *frameReader) readFrame() (map[string]any, error) {
	var buf []byte
	for {
		chunk, err := f.r.ReadSlice('\n')
		buf = append(buf, chunk...)
		if len(buf) > maxFrame {
			return nil, newProtocol("stdout frame exceeds limit", nil)
		}
		if err == nil {
			break
		}
		if errors.Is(err, bufio.ErrBufferFull) {
			continue
		}
		if errors.Is(err, io.EOF) {
			if len(buf) > 0 {
				return nil, newProtocol("unterminated stdout frame", nil)
			}
			return nil, io.EOF
		}
		return nil, err
	}
	line := buf[:len(buf)-1]
	if len(line) == 0 {
		return nil, newProtocol("blank stdout frame", nil)
	}
	if !utf8.Valid(line) {
		return nil, newProtocol("invalid UTF-8 frame", nil)
	}
	var value any
	if err := json.Unmarshal(line, &value); err != nil {
		return nil, newProtocol("malformed stdout frame", err)
	}
	obj, ok := value.(map[string]any)
	if !ok {
		return nil, newProtocol("stdout frame is not an object", nil)
	}
	return obj, nil
}

type frameWriter struct {
	w io.Writer
}

func (f *frameWriter) write(value any) error {
	var buf bytes.Buffer
	encoder := json.NewEncoder(&buf)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(value); err != nil {
		return newProtocol("frame is not JSON serializable", err)
	}
	raw := buf.Bytes()
	if len(raw) > maxFrame {
		return newProtocol("outgoing frame exceeds limit", nil)
	}
	if _, err := f.w.Write(raw); err != nil {
		return newTransport("bridge write failed", err)
	}
	return nil
}

type runFrame struct {
	Type     string     `json:"type"`
	Protocol int        `json:"protocol"`
	Manifest Manifest   `json:"manifest"`
	Task     string     `json:"task"`
	Tools    []toolSpec `json:"tools"`
}

type toolSpec struct {
	Name        string `json:"name"`
	Description string `json:"description"`
}

type toolResultOutputFrame struct {
	Type   string `json:"type"`
	ID     int    `json:"id"`
	Output string `json:"output"`
}

type toolResultErrorFrame struct {
	Type  string `json:"type"`
	ID    int    `json:"id"`
	Error string `json:"error"`
}

type toolCall struct {
	ID    int
	Name  string
	Input map[string]any
}

type runResult struct {
	State  string
	Events []map[string]any
}

func parseToolCall(msg map[string]any) (toolCall, error) {
	if !hasOnlyKeys(msg, "type", "id", "name", "input") {
		return toolCall{}, newProtocol("invalid tool call", nil)
	}
	id, ok := asID(msg["id"])
	if !ok {
		return toolCall{}, newProtocol("invalid tool call id", nil)
	}
	name, ok := msg["name"].(string)
	if !ok {
		return toolCall{}, newProtocol("invalid tool call name", nil)
	}
	input, ok := msg["input"].(map[string]any)
	if !ok {
		return toolCall{}, newProtocol("invalid tool call input", nil)
	}
	return toolCall{ID: id, Name: name, Input: input}, nil
}

func parseRunResult(msg map[string]any) (runResult, error) {
	if !hasOnlyKeys(msg, "type", "state", "events") {
		return runResult{}, newProtocol("invalid run result", nil)
	}
	state, ok := msg["state"].(string)
	if !ok || !isValidState(state) {
		return runResult{}, newProtocol("invalid run result", nil)
	}
	events, ok := parseEvents(msg["events"])
	if !ok {
		return runResult{}, newProtocol("invalid run result", nil)
	}
	return runResult{State: state, Events: events}, nil
}

func toolResultOutput(id int, output string) any {
	return toolResultOutputFrame{Type: "tool_result", ID: id, Output: output}
}

func toolResultError(id int, message string) any {
	return toolResultErrorFrame{Type: "tool_result", ID: id, Error: message}
}

func hasOnlyKeys(msg map[string]any, keys ...string) bool {
	if len(msg) != len(keys) {
		return false
	}
	for _, key := range keys {
		if _, ok := msg[key]; !ok {
			return false
		}
	}
	return true
}

func isValidState(state string) bool {
	return state == "completed" || state == "failed" || state == "cancelled"
}

func asID(value any) (int, bool) {
	number, ok := value.(float64)
	if !ok || math.Trunc(number) != number || math.Abs(number) > 1<<53 {
		return 0, false
	}
	return int(number), true
}

func parseEvents(value any) ([]map[string]any, bool) {
	list, ok := value.([]any)
	if !ok {
		return nil, false
	}
	events := make([]map[string]any, len(list))
	for i, item := range list {
		obj, ok := item.(map[string]any)
		if !ok {
			return nil, false
		}
		events[i] = obj
	}
	return events, true
}
