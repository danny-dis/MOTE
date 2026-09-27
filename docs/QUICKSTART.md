# MOTE binary quick-start

Download the archive for your computer from https://github.com/danny-dis/MOTE/releases/latest.
Source and release downloads are public; no GitHub account is required to download.

- Windows x64: `x86_64-pc-windows-msvc.zip` (Windows 10/11).
- Linux x64: `x86_64-unknown-linux-gnu.tar.gz` (Ubuntu 22.04 or newer compatible system, OpenSSL 3 and CA certificates required).
- macOS Apple Silicon: `aarch64-apple-darwin.tar.gz` (macOS 14 or newer).
- Intel Macs and other architectures: build from source for now.

Verify the archive against `SHA256SUMS` before extracting:

```powershell
# Windows PowerShell: compare this digest with the matching SHA256SUMS line.
Get-FileHash .\mote-v*-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

```sh
# Linux (run from the download directory; missing other platforms are ignored).
sha256sum --check --ignore-missing SHA256SUMS
# macOS: compare with the matching SHA256SUMS line.
shasum -a 256 mote-v*-aarch64-apple-darwin.tar.gz
```

Archives contain `mote` and `mote-bridge` (`.exe` on Windows), this guide,
`agent.yaml`, an empty `workspace`, and licenses. No Rust toolchain is needed.
The binaries are not code-signed/notarized; your OS may show a warning. Checksums
check integrity, not publisher identity. Do not disable system-wide security controls.

## Command line

Open a terminal in the extracted directory:

```powershell
.\mote.exe --version
.\mote.exe --help
```

```sh
./mote --version
./mote --help
```

Edit `agent.yaml` with your provider's OpenAI-compatible **chat completions**
endpoint and model name. The example defaults to a local server, not a bundled model.
If authentication is required, set the environment variable named in `auth_env`
(`MOTE_API_KEY`) before running. For a keyless local server, remove `auth_env`
from the manifest. Never put the key into YAML or commit it.
Put only the files you want the agent to read in `workspace`.

```powershell
.\mote.exe --jsonl agent.yaml "List the workspace files and summarize them"
```

```sh
./mote --jsonl agent.yaml "List the workspace files and summarize them"
```

`--jsonl` displays the tool observations and lifecycle events after the run ends;
it is not a live stream. Without it, the CLI prints only the final state and event
count, not a chat-style answer. Treat event output as potentially sensitive.

The example grants only `list_dir` and `read_file`; it cannot write or run shell
commands. Permission checks are not an OS sandbox. Use trusted workspaces and
isolate untrusted workloads yourself. Provider availability and output correctness
are separate from successful installation.

## Python and TypeScript

Install the SDK from the same release's source tag. Packages are not yet published
to PyPI/npm. See https://github.com/danny-dis/MOTE/blob/production-ready/docs/BUILDING_ON_MOTE.md.
Pass the extracted bridge's absolute path to your client:

```python
from mote_sdk import Client
client = Client(binary=r"C:\path\to\mote-bridge.exe")
```

```javascript
import { Client } from "@mote-agent/sdk";
const client = new Client({ binary: "/absolute/path/to/mote-bridge" });
```

`mote-bridge` speaks JSON over stdin/stdout; it is not an interactive console or
HTTP server. Use the SDK rather than double-clicking it. Application callbacks run
with the host user's privileges; bridge timeouts do not terminate every descendant.

## Troubleshooting

**Unknown field `api_key_env`.** Use `auth_env: MOTE_API_KEY`. The v0.13.0 release
example had this typo; upgrade to v0.13.1 or fix that field. Unknown manifest fields
are deliberately rejected, not silently ignored.

**Missing auth environment variable.** Set the variable named by `auth_env` in the
same terminal or service that launches MOTE. It names a variable, not a literal key.
For a keyless local model, remove `auth_env` entirely.

**Connection refused / all models failed / HTTP 429 or 503.** Verify the endpoint,
model name, credentials, quota, and provider availability. MOTE does not start a model
server. OpenAI-compatible endpoints must include the chat-completions route, not just
`/v1`. Increasing retries does not fix authentication or quota errors.

**Capability rejected.** A trusted manifest must explicitly grant the action. A custom
tool also needs a registered host handler. Do not grant shell access just to hide an error.

**Bridge appears to hang when double-clicked.** It is not a GUI. SDKs communicate with
`mote-bridge` over stdin/stdout. Launch `mote` from a terminal for CLI use.

**SDK cannot find the executable.** Pass the bridge's absolute path; on Windows it is
`mote-bridge.exe`. SDK installation does not install or download the bridge.

**Import fails in Node.** Build the source package before installing it into your app.
Use the ESM import and package name documented in the [TypeScript SDK](https://github.com/danny-dis/MOTE/blob/production-ready/sdk/typescript/README.md).

**Output exists but the run failed, or a completed result is wrong.** Side effects may
happen before failure. Inspect events and independently check output; do not blindly
retry writes or external callbacks. A completed state is not semantic validation.

**No live progress events.** Current CLI JSONL and SDK runtime events are emitted after
the run. Host-tool callbacks are interactive; runtime event streaming is not implemented.

**OS download warning.** Releases are unsigned/not notarized. Verify origin and checksum;
do not disable system-wide protections. Build from source if your policy requires it.

**Linux shared-library error.** Use the matching architecture and an Ubuntu 22.04-compatible
system with OpenSSL 3 and CA certificates, or build on your target system.

Still stuck? Check that the CLI, bridge, and SDK source versions match, reproduce
in a small non-sensitive workspace, and follow the
[bug-report guidance](https://github.com/danny-dis/MOTE/blob/production-ready/CONTRIBUTING.md#support-and-bug-reports).
