package mote_test

import (
	"context"
	"errors"
	"path/filepath"
	"strings"
	"testing"
	"time"

	mote "github.com/danny-dis/MOTE/sdk/go"
)

func TestCooperativeHandlerHonorsDeadline(t *testing.T) {
	var called bool
	var deadline time.Time
	tools := map[string]mote.Tool{
		"x": {
			Description: "x",
			Handler: func(ctx context.Context, _ map[string]any) (string, error) {
				called = true
				var ok bool
				deadline, ok = ctx.Deadline()
				if !ok || ctx.Err() != nil {
					t.Fatalf("handler must enter with a live deadline: deadline=%v err=%v", deadline, ctx.Err())
				}
				<-ctx.Done()
				if !errors.Is(ctx.Err(), context.DeadlineExceeded) {
					t.Fatalf("handler context error = %v, want DeadlineExceeded", ctx.Err())
				}
				return "", ctx.Err()
			},
		},
	}

	// Allow fixture startup separately from the prompt deadline-delivery check.
	result, err := runMode(t, "success", tools, 5*time.Second)
	if err == nil {
		t.Fatalf("expected timeout, got result %+v", result)
	}
	if !errors.Is(err, mote.ErrTimeout) {
		t.Fatalf("error = %v, want ErrTimeout", err)
	}
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("error = %v, want DeadlineExceeded cause", err)
	}
	if !called {
		t.Fatal("handler was never invoked")
	}
	if elapsed := time.Since(deadline); elapsed > 3*time.Second {
		t.Fatalf("deadline not enforced promptly: %v", elapsed)
	}
}

func TestRegistrationWithoutGrantNeverInvokesCallback(t *testing.T) {
	var called bool
	tools := map[string]mote.Tool{
		"x": {
			Description: "x",
			Handler: func(_ context.Context, _ map[string]any) (string, error) {
				called = true
				return "BAD", nil
			},
		},
	}

	result, err := runManifest(t, "denied", fixtureManifest(), tools, 5*time.Second)
	if err != nil {
		t.Fatalf("run failed: %v", err)
	}
	if called {
		t.Fatal("callback invoked without a grant")
	}
	if result.Success() {
		t.Fatalf("expected failure result, got %+v", result)
	}
}

func TestTrailingFrameAfterTerminalRejected(t *testing.T) {
	for _, mode := range []string{"trailing", "truncated"} {
		t.Run(mode, func(t *testing.T) {
			result, err := runMode(t, mode, upperTool(), 5*time.Second)
			if err == nil {
				t.Fatalf("expected error, got result %+v", result)
			}
			if !errors.Is(err, mote.ErrProtocol) {
				t.Fatalf("error = %v, want ErrProtocol", err)
			}
		})
	}
}

func TestMalformedFailsPromptly(t *testing.T) {
	var started time.Time
	tools := map[string]mote.Tool{"x": {
		Description: "Confirm startup before measuring protocol rejection",
		Handler: func(context.Context, map[string]any) (string, error) {
			started = time.Now()
			return "OK", nil
		},
	}}
	result, err := runMode(t, "malformed_after_callback", tools, 5*time.Second)
	if started.IsZero() {
		t.Fatal("callback was not reached before measuring protocol rejection")
	}
	if err == nil {
		t.Fatalf("expected error, got result %+v", result)
	}
	if !errors.Is(err, mote.ErrProtocol) {
		t.Fatalf("error = %v, want ErrProtocol", err)
	}
	if elapsed := time.Since(started); elapsed > 2*time.Second {
		t.Fatalf("malformed frame did not fail promptly: %v", elapsed)
	}
}

func TestOutgoingFramesAreBounded(t *testing.T) {
	t.Run("task too large", func(t *testing.T) {
		client := &mote.Client{
			Binary:  fixtureBin,
			Timeout: 5 * time.Second,
			Env:     []string{"TEST_BRIDGE_MODE=missingresult"},
		}
		result, err := client.Run(context.Background(), fixtureManifest("x"), strings.Repeat("x", 1_048_576), upperTool())
		if err == nil {
			t.Fatalf("expected error, got result %+v", result)
		}
		if !errors.Is(err, mote.ErrProtocol) {
			t.Fatalf("error = %v, want ErrProtocol", err)
		}
	})

	t.Run("handler output too large", func(t *testing.T) {
		tools := map[string]mote.Tool{
			"x": {
				Description: "x",
				Handler: func(_ context.Context, _ map[string]any) (string, error) {
					return strings.Repeat("x", 1_048_576), nil
				},
			},
		}
		result, err := runMode(t, "success", tools, 5*time.Second)
		if err == nil {
			t.Fatalf("expected error, got result %+v", result)
		}
		if !errors.Is(err, mote.ErrProtocol) {
			t.Fatalf("error = %v, want ErrProtocol", err)
		}
	})
}

func TestNegativeTimeoutRejected(t *testing.T) {
	// Note: the Go implementation treats Timeout == 0 as "use the default",
	// so only strictly negative durations are setup errors.
	for _, timeout := range []time.Duration{-1, -time.Second} {
		client := &mote.Client{
			Binary:  fixtureBin,
			Timeout: timeout,
			Env:     []string{"TEST_BRIDGE_MODE=success"},
		}
		result, err := client.Run(context.Background(), fixtureManifest("x"), "go", upperTool())
		if !errors.Is(err, mote.ErrSetup) {
			t.Fatalf("timeout %v: error = %v (result %+v), want ErrSetup", timeout, err, result)
		}
	}
}

func TestStderrFloodDrained(t *testing.T) {
	result, err := runMode(t, "stderrflood", upperTool(), 5*time.Second)
	if err != nil {
		t.Fatalf("run failed: %v", err)
	}
	if !result.Success() {
		t.Fatalf("expected success, got %+v", result)
	}
}

func TestMissingExecutableFailsTransport(t *testing.T) {
	client := &mote.Client{
		Binary:  filepath.Join(t.TempDir(), "mote-bridge-does-not-exist-5985"),
		Timeout: 2 * time.Second,
	}
	result, err := client.Run(context.Background(), fixtureManifest(), "go", nil)
	if err == nil {
		t.Fatalf("expected error, got result %+v", result)
	}
	if !errors.Is(err, mote.ErrTransport) {
		t.Fatalf("error = %v, want ErrTransport", err)
	}
}

func TestNoReadChildCannotBlockStartup(t *testing.T) {
	started := time.Now()
	result, err := runMode(t, "noread", upperTool(), 300*time.Millisecond)
	if err == nil {
		t.Fatalf("expected timeout, got result %+v", result)
	}
	if !errors.Is(err, mote.ErrTimeout) {
		t.Fatalf("error = %v, want ErrTimeout", err)
	}
	if elapsed := time.Since(started); elapsed > 3*time.Second {
		t.Fatalf("startup blocked too long: %v", elapsed)
	}
}

func TestExitMismatchIsNotSuccess(t *testing.T) {
	result, err := runMode(t, "mismatch", upperTool(), 5*time.Second)
	if err == nil {
		t.Fatalf("expected error, got result %+v", result)
	}
	if !errors.Is(err, mote.ErrProtocol) {
		t.Fatalf("error = %v, want ErrProtocol", err)
	}
}
