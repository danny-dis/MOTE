package mote_test

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
	"time"

	mote "github.com/danny-dis/MOTE/sdk/go"
)

// fixtureBin is the absolute path to the compiled protocol-v1 fixture bridge.
// It is populated by TestMain before any test runs.
var fixtureBin string

func TestMain(m *testing.M) {
	dir, err := os.Getwd()
	if err != nil {
		fmt.Fprintln(os.Stderr, "cannot determine working directory:", err)
		os.Exit(1)
	}

	tmp, err := os.MkdirTemp("", "mote-bridge-fixture-*")
	if err != nil {
		fmt.Fprintln(os.Stderr, "cannot create temp dir:", err)
		os.Exit(1)
	}

	suffix := ""
	if runtime.GOOS == "windows" {
		suffix = ".exe"
	}
	bin := filepath.Join(tmp, "bridge"+suffix)

	// TestMain runs before the test timeout starts, so bound fixture setup too.
	buildCtx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	build := exec.CommandContext(buildCtx, "go", "build", "-o", bin, "./testdata/bridge")
	build.Dir = dir
	build.Stdout = os.Stderr
	build.Stderr = os.Stderr
	buildErr := build.Run()
	cancel()
	if buildErr != nil {
		fmt.Fprintln(os.Stderr, "failed to build bridge fixture:", buildErr)
		_ = os.RemoveAll(tmp)
		os.Exit(1)
	}

	fixtureBin = bin
	code := m.Run()
	_ = os.RemoveAll(tmp)
	os.Exit(code)
}

// fixtureManifest builds a manifest with a >=1 model (required by a real
// bridge; the fixture ignores it) and the supplied capabilities.
func fixtureManifest(capabilities ...string) mote.Manifest {
	return mote.Manifest{
		Name:         "test",
		Workspace:    ".",
		Capabilities: capabilities,
		Models: []mote.ModelConfig{
			{Provider: "fixture", Model: "fixture", Endpoint: "http://unused.invalid"},
		},
	}
}

func toolNames(tools map[string]mote.Tool) []string {
	names := make([]string, 0, len(tools))
	for name := range tools {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

// runManifest runs the fixture with an explicit manifest, inheriting the
// host environment (the SDK merges os.Environ with the provided entries and
// lets the provided entries win by key).
func runManifest(t *testing.T, mode string, manifest mote.Manifest, tools map[string]mote.Tool, timeout time.Duration) (*mote.RunResult, error) {
	t.Helper()
	client := &mote.Client{
		Binary:  fixtureBin,
		Timeout: timeout,
		Env:     []string{"TEST_BRIDGE_MODE=" + mode},
	}
	return client.Run(context.Background(), manifest, "go", tools)
}

// runMode is the common helper: it grants every registered tool.
func runMode(t *testing.T, mode string, tools map[string]mote.Tool, timeout time.Duration) (*mote.RunResult, error) {
	t.Helper()
	return runManifest(t, mode, fixtureManifest(toolNames(tools)...), tools, timeout)
}

// upperTool returns a single "x" tool that uppercases input["value"].
func upperTool() map[string]mote.Tool {
	return map[string]mote.Tool{
		"x": {
			Description: "x",
			Handler: func(_ context.Context, input map[string]any) (string, error) {
				value, _ := input["value"].(string)
				return strings.ToUpper(value), nil
			},
		},
	}
}
