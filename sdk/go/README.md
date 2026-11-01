# MOTE Go SDK

The MOTE Go SDK lets a Go application host tools and run MOTE through the
versioned stdio `mote-bridge` executable. It is source-installable and is not
published to a module proxy as a released package. The SDK uses only the Go
standard library at runtime.

See the [official v0.14.1 Go SDK specification](../../docs/GO_SDK_SPEC.md)
for the supported contract and release acceptance checks.

Requirements: Go 1.22 or newer (`go 1.22` in `sdk/go/go.mod`). A Rust toolchain
is not required when you use a downloaded bridge executable. The bridge is
available in the [GitHub Releases](https://github.com/danny-dis/MOTE/releases)
archives. Source and downloads are public; no GitHub account is needed. Release
binaries are unsigned/not notarized. Verify the archive checksum, expect an OS
warning where applicable, and do not treat a checksum as publisher identity.
See the repository [QUICKSTART](../../docs/QUICKSTART.md) for platform assets
and extraction instructions.

## Quick start

Run the following from the repository root after extracting a release archive
or building the bridge. On Windows, append `.exe` to the executable path.

```bash
# Source checkout option; requires stable Rust.
cargo build --locked --release --bin mote-bridge
```

Import the SDK from a local checkout by adding a `replace` directive to your
`go.mod`, pointing at the absolute path of `sdk/go`:

```
require github.com/danny-dis/MOTE/sdk/go v0.0.0

replace github.com/danny-dis/MOTE/sdk/go => /absolute/path/to/MOTE/sdk/go
```

The checked-in example runs directly from the source checkout. Set the environment
variables described below, then run it from `sdk/go`:

```bash
go run ./example
```

Point `MOTE_BRIDGE` at the downloaded or built `mote-bridge`, set
`MOTE_MODEL_ENDPOINT` to a reachable OpenAI-compatible chat-completions
endpoint, and set `MOTE_MODEL`. If the endpoint requires authentication, set
`MODEL_API_KEY`; the manifest must refer to the variable by name with
`auth_env: "MODEL_API_KEY"`. No provider, gateway, model, or API key is
bundled, and the SDK does not install or start a service.

The checked-in example defaults to a local DMR-X-style endpoint and model
`auto`; change the environment for your provider. Its success checks include
the callback and the observed `HELLO` result, but running it requires a live,
reachable model endpoint.

For configuration fields, defaults, model providers, workspace boundaries and
timeouts, see [CONFIGURATION](../../docs/CONFIGURATION.md). For the complete
integration context, see [BUILDING_ON_MOTE](../../docs/BUILDING_ON_MOTE.md) and
[QUICKSTART](../../docs/QUICKSTART.md).

## API

```go
package main

import (
	"context"
	"log"
	"strings"
	"time"

	"github.com/danny-dis/MOTE/sdk/go"
)

func main() {
	ctx, cancel := context.WithTimeout(context.Background(), 150*time.Second)
	defer cancel()

	result, err := (&mote.Client{Binary: "/absolute/path/to/mote-bridge"}).Run(
		ctx,
		mote.Manifest{
			Name:         "demo",
			Capabilities: []string{"uppercase"},
			Models: []mote.ModelConfig{{
				Provider: "openai-compatible",
				Model:    "your-model",
				Endpoint: "https://your-provider.example/v1/chat/completions",
				AuthEnv:  "MODEL_API_KEY",
			}},
		},
		`Call uppercase with {"text":"hello"}, then complete.`,
		map[string]mote.Tool{
			"uppercase": {
				Description: "Input: text string. Returns uppercase text.",
				Handler: func(ctx context.Context, input map[string]any) (string, error) {
					return strings.ToUpper(input["text"].(string)), nil
				},
			},
		},
	)
	if err != nil {
		log.Fatal(err)
	}
	if !result.Success() {
		log.Fatal("Agent run failed")
	}
	log.Println(result.Events)
}
```

`Client{Binary, Args, Timeout, Cwd, Env}` starts one trusted executable per
`Run`. `Binary` defaults to `mote-bridge` when empty. `Args` is for an
application-owned launcher; it is never passed through a shell. `Timeout` is a
positive duration, defaulting to 150 seconds. `Env` overrides inherited
environment variables and `Cwd` controls the bridge process working directory.

`Run(ctx, manifest, task, tools)` returns `*RunResult` with `State`, `Events`
and `ReturnCode`. `Success()` requires state `completed` and exit code `0`;
failed or cancelled runtime work returns a false result. Setup, protocol,
transport and deadline failures return a `*MoteError`; inspect its `Code`
(`SETUP`, `PROTOCOL`, `TRANSPORT` or `TIMEOUT`) or match the exported sentinels
`ErrSetup`, `ErrProtocol`, `ErrTransport` and `ErrTimeout` with `errors.Is`.

`Tool{Description, Handler}` receives a JSON object and must return a string.
Validate domain-specific fields in the handler. Handler panics and returned
errors become a generic tool failure; exception text and sensitive inputs are
not sent to the model. Registering a tool is not permission: its name must also
be in `Manifest.Capabilities`. Runs and callbacks are not automatically
retried, and a failed or timed-out run may already have side effects.

The SDK limits each bridge stdout frame to 1 MiB including its newline and
continuously drains and discards stderr without retaining it. `Events` and model
observations arrive after the run; there is no live-event or progress-stream API. Treat them as
potentially sensitive. The SDK watchdog kills and reaps the direct bridge child
on a deadline, but Go callbacks execute synchronously in the host process and
cannot be forcibly interrupted. A handler that ignores its context keeps `Run`
blocked even after the bridge is stopped. Bound or isolate blocking callbacks
and descendants yourself.
Neither the SDK nor the bridge is an OS sandbox; host code runs with the
application's privileges.

## Tests

From `sdk/go`:

```bash
go test ./...
gofmt -l .
go vet ./...
```

Set `MOTE_BRIDGE` to a bridge executable before running the tests. The E2E
coverage uses a real subprocess and callback with a local scripted HTTP model;
it is deterministic and does not test live inference. No module proxy
publication is implied by this source package.
