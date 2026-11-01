package mote

import (
	"context"
	"errors"
	"io"
	"os"
	"os/exec"
	"sort"
	"strings"
	"sync"

	"time"
)

type RunResult struct {
	State      string
	Events     []map[string]any
	ReturnCode int
}

func (r *RunResult) Success() bool {
	return r != nil && r.State == "completed" && r.ReturnCode == 0
}

type Client struct {
	Binary  string
	Args    []string
	Timeout time.Duration
	Cwd     string
	Env     []string
}

// Run executes one bridge process. Handlers execute synchronously and must
// cooperate with their context: cancelling the child cannot stop host code.
func (c *Client) Run(ctx context.Context, manifest Manifest, task string, tools map[string]Tool) (*RunResult, error) {
	if ctx == nil {
		ctx = context.Background()
	}
	timeout := c.Timeout
	if timeout == 0 {
		timeout = 150 * time.Second
	}
	if timeout <= 0 {
		return nil, newSetup("timeout must be a positive, finite duration", nil)
	}
	if tools == nil {
		tools = map[string]Tool{}
	}
	for _, tool := range tools {
		if tool.Description == "" || tool.Handler == nil {
			return nil, newSetup("invalid tool registration", nil)
		}
	}

	grants := make(map[string]struct{}, len(manifest.Capabilities))
	for _, capability := range manifest.Capabilities {
		grants[capability] = struct{}{}
	}

	binary := c.Binary
	if binary == "" {
		binary = "mote-bridge"
	}

	runCtx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()
	if runCtx.Err() != nil {
		return nil, timeoutError(runCtx)
	}

	stdinR, stdinW, err := os.Pipe()
	if err != nil {
		return nil, newTransport("could not start mote bridge", err)
	}
	stdoutR, stdoutW, err := os.Pipe()
	if err != nil {
		_ = stdinR.Close()
		_ = stdinW.Close()
		return nil, newTransport("could not start mote bridge", err)
	}
	stderrR, stderrW, err := os.Pipe()
	if err != nil {
		_ = stdinR.Close()
		_ = stdinW.Close()
		_ = stdoutR.Close()
		_ = stdoutW.Close()
		return nil, newTransport("could not start mote bridge", err)
	}

	cmd := exec.CommandContext(runCtx, binary, c.Args...)
	cmd.Dir = c.Cwd
	cmd.Env = buildEnv(c.Env)
	cmd.Stdin = stdinR
	cmd.Stdout = stdoutW
	cmd.Stderr = stderrW

	if err := cmd.Start(); err != nil {
		_ = stdinR.Close()
		_ = stdinW.Close()
		_ = stdoutR.Close()
		_ = stdoutW.Close()
		_ = stderrR.Close()
		_ = stderrW.Close()
		if runCtx.Err() != nil {
			return nil, timeoutError(runCtx)
		}
		return nil, newTransport("could not start mote bridge", err)
	}
	_ = stdinR.Close()
	_ = stdoutW.Close()
	_ = stderrW.Close()
	// Release blocked writes even if a descendant inherits the read handle
	// and outlives the direct bridge child.
	stopWrite := context.AfterFunc(runCtx, func() { _ = stdinW.Close() })
	defer stopWrite()

	stop := make(chan struct{})
	var stopOnce sync.Once
	shutdown := func() { stopOnce.Do(func() { close(stop) }) }

	// Bounded channel: a child that floods stdout while a host callback runs
	// applies backpressure to the pipe instead of exhausting memory.
	frames := make(chan frameItem, 4)
	go readFrames(stdoutR, frames, stop)
	go drainStderr(stderrR)

	waitCh := make(chan error, 1)
	go func() { waitCh <- cmd.Wait() }()

	reaped := false
	defer func() {
		cancel()
		shutdown()
		_ = stdinW.Close()
		_ = stdoutR.Close()
		_ = stderrR.Close()
		if !reaped {
			select {
			case <-waitCh:
			case <-time.After(2 * time.Second):
				if cmd.Process != nil {
					_ = cmd.Process.Kill()
				}
				select {
				case <-waitCh:
				case <-time.After(2 * time.Second):
				}
			}
		}
	}()

	writer := &frameWriter{w: stdinW}
	write := func(value any) error {
		if runCtx.Err() != nil {
			return timeoutError(runCtx)
		}
		err := writer.write(value)
		if runCtx.Err() != nil {
			return timeoutError(runCtx)
		}
		return err
	}
	if err := write(newRunFrame(manifest, task, tools)); err != nil {
		return nil, err
	}

	expectedID := 1
	var terminal *runResult
	for terminal == nil {
		item, err := nextItem(runCtx, frames)
		if err != nil {
			return nil, err
		}
		if item.err != nil {
			return nil, item.err
		}
		if item.eof {
			return nil, newProtocol("bridge closed before run result", nil)
		}
		messageType, _ := item.msg["type"].(string)
		switch messageType {
		case "tool_call":
			call, err := parseToolCall(item.msg)
			if err != nil {
				return nil, err
			}
			if call.ID != expectedID {
				return nil, newProtocol("invalid tool call id", nil)
			}
			expectedID++
			tool, registered := tools[call.Name]
			_, granted := grants[call.Name]
			if !registered || !granted {
				if err := write(toolResultError(call.ID, "tool unavailable")); err != nil {
					return nil, err
				}
				continue
			}
			output, callErr := invokeTool(runCtx, tool, call.Input)
			if callErr != nil {
				if err := write(toolResultError(call.ID, "tool failed")); err != nil {
					return nil, err
				}
				continue
			}
			if err := write(toolResultOutput(call.ID, output)); err != nil {
				return nil, err
			}
		case "run_result":
			result, err := parseRunResult(item.msg)
			if err != nil {
				return nil, err
			}
			terminal = &result
		case "error":
			return nil, newProtocol("bridge protocol error", nil)
		default:
			return nil, newProtocol("unknown bridge message", nil)
		}
	}

	_ = stdinW.Close()

	trailing, err := nextItem(runCtx, frames)
	if err != nil {
		return nil, err
	}
	if trailing.err != nil {
		return nil, trailing.err
	}
	if !trailing.eof {
		return nil, newProtocol("stdout after terminal result", nil)
	}

	var waitErr error
	select {
	case waitErr = <-waitCh:
		reaped = true
	case <-runCtx.Done():
		return nil, timeoutError(runCtx)
	}

	if runCtx.Err() != nil {
		return nil, timeoutError(runCtx)
	}

	code := exitCode(waitErr)
	if terminal.State == "completed" {
		if code != 0 {
			return nil, newProtocol("state and exit status disagree", nil)
		}
	} else if code != 1 {
		return nil, newProtocol("invalid failed exit status", nil)
	}

	return &RunResult{State: terminal.State, Events: terminal.Events, ReturnCode: code}, nil
}

type frameItem struct {
	msg map[string]any
	err error
	eof bool
}

func nextItem(runCtx context.Context, frames <-chan frameItem) (frameItem, error) {
	if runCtx.Err() != nil {
		return frameItem{}, timeoutError(runCtx)
	}
	select {
	case item := <-frames:
		if runCtx.Err() != nil {
			return frameItem{}, timeoutError(runCtx)
		}
		return item, nil
	case <-runCtx.Done():
		return frameItem{}, timeoutError(runCtx)
	}
}

func readFrames(out io.Reader, frames chan<- frameItem, stop <-chan struct{}) {
	reader := newFrameReader(out)
	for {
		msg, err := reader.readFrame()
		if err != nil {
			if errors.Is(err, io.EOF) {
				select {
				case frames <- frameItem{eof: true}:
				case <-stop:
				}
				return
			}
			var moteErr *MoteError
			if !errors.As(err, &moteErr) {
				err = newTransport("stdout read failed", err)
			}
			select {
			case frames <- frameItem{err: err}:
			case <-stop:
			}
			return
		}
		select {
		case frames <- frameItem{msg: msg}:
		case <-stop:
			return
		}
	}
}

func drainStderr(r io.Reader) {
	_, _ = io.Copy(io.Discard, r)
}

func invokeTool(ctx context.Context, tool Tool, input map[string]any) (output string, err error) {
	defer func() {
		if recover() != nil {
			output = ""
			err = errors.New("tool handler failed")
		}
	}()
	return tool.Handler(ctx, input)
}

func newRunFrame(manifest Manifest, task string, tools map[string]Tool) runFrame {
	names := make([]string, 0, len(tools))
	for name := range tools {
		names = append(names, name)
	}
	sort.Strings(names)
	specs := make([]toolSpec, 0, len(names))
	for _, name := range names {
		specs = append(specs, toolSpec{Name: name, Description: tools[name].Description})
	}
	return runFrame{Type: "run", Protocol: 1, Manifest: manifest, Task: task, Tools: specs}
}

func buildEnv(extra []string) []string {
	base := os.Environ()
	if extra == nil {
		return base
	}
	values := make(map[string]string, len(base)+len(extra))
	order := make([]string, 0, len(base)+len(extra))
	for _, entry := range base {
		key, _, ok := strings.Cut(entry, "=")
		if !ok {
			continue
		}
		if _, exists := values[key]; !exists {
			order = append(order, key)
		}
		values[key] = entry
	}
	for _, entry := range extra {
		key, _, ok := strings.Cut(entry, "=")
		if !ok {
			continue
		}
		if _, exists := values[key]; !exists {
			order = append(order, key)
		}
		values[key] = entry
	}
	merged := make([]string, 0, len(order))
	for _, key := range order {
		merged = append(merged, values[key])
	}
	return merged
}

func exitCode(err error) int {
	if err == nil {
		return 0
	}
	var exitErr *exec.ExitError
	if errors.As(err, &exitErr) {
		return exitErr.ExitCode()
	}
	return -1
}

func timeoutError(ctx context.Context) *MoteError {
	var cause error
	if ctx != nil {
		cause = ctx.Err()
	}
	return newTimeout("bridge timed out", cause)
}
