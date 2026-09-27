# MOTE binary quick-start

Download the archive for your computer from https://github.com/danny-dis/MOTE/releases/latest.
The repository is currently private: GitHub access is required to see its releases.

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
