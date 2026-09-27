# Security and deployment scope

MOTE is suitable for **trusted workspaces and trusted operators** when deployed with a deliberately restricted manifest and an external isolation boundary. It is **not** a sandbox for untrusted model output, code, repositories, or users. Passing CI does not change that boundary.

## What MOTE enforces

- No tool is granted unless its capability is listed in the manifest. The **host** must control which manifest is accepted; a caller-supplied manifest is not a trusted permission decision.
- Built-in file actions reject absolute paths, parent traversal, and symlink targets that resolve outside the workspace at validation time.
- `max_output_bytes` (1–1,048,576; default 65,536) caps shell output and rejects oversized file and directory observations before they enter the next model prompt or event stream.
- Per-tool calls, iterations, and elapsed run time are bounded. Failed writes cannot claim a fresh report.
- The JSONL action event omits `write_file` content, but observations and other action arguments are still recorded.

## What MOTE does **not** enforce

- Shell/Git programs run with the invoking account's OS permissions. An allowlisted interpreter (PowerShell, Python, Node, etc.) can run code and read files or use the network outside the workspace. `unsafe_shell: false` is not an OS sandbox.
- Workspace path checking is not atomic with opening the file. A concurrently writable workspace can change a symlink between the check and the open operation.
- Executable names are resolved using the process environment's search path. An untrusted `PATH` directory could substitute a program.
- Command timeouts and cancellation stop the direct process best-effort; descendants or blocked operations can outlive the run.
- Model requests include task prompts and tool observations. Logs can include file contents, command output, and other action arguments. The configured provider may be remote.
- There is no multi-tenant access control, tool plug-in sandbox, installed scheduler, or guarantee of live provider uptime. Host-registered tool handlers run with the host process's permissions and can block past a runtime deadline; registration alone is not authorization.

## Deployment checklist

1. Run each workload in a dedicated low-privilege OS account or an externally isolated container/VM, with only the files and network destinations it needs. Do not put secrets or unrelated repositories in its workspace or environment.
2. Grant only the necessary capabilities. Avoid shell and Git entirely for file-only agents. If shell is essential, prefer narrowly scoped programs and an operator-controlled search path; do not allow interpreters on an untrusted host without OS isolation.
3. Pin the model endpoint deliberately and review where observations are sent. Protect and rotate provider credentials independently of MOTE; do not put them in manifests. Register custom Rust handlers only from trusted host code; YAML cannot load handlers by itself.
4. Limit who can edit the manifest, workspace, executable search path, and runtime environment. Do not let an adversary mutate the workspace concurrently.
5. Restrict access to JSONL logs and reports; treat historical reports under `docs/archive/` as samples, never as live health evidence. Arrange external monitoring and backups.
6. Exercise the agent against the actual model endpoint and deployment environment before enabling unattended runs. Failures, rate limits, and network timeouts must be monitored externally.

## Reporting a vulnerability

Do not post exploit details, credentials, or sensitive logs in a public issue.
Contact the repository maintainer through an existing private channel. If you do
not have one, open an issue requesting a private security contact **without**
disclosing the vulnerability. GitHub private vulnerability reporting may not be
enabled or available; this policy does not assume it is.

Include the affected release/commit, operating system, minimal reproduction,
security impact, and proposed mitigation. Remove real keys and private user data.
There is no paid support contract, bug bounty, or guaranteed response deadline.

## Updates and release trust

Use the latest published patch release and review its changelog. MOTE is pre-1.0;
older releases do not have a promised long-term-support or backport schedule.
A fix is not available until its release or commit is published.

Release archives are unsigned and macOS binaries are not notarized. Compare the
archive hash against `SHA256SUMS` from the same trusted GitHub release. A checksum
checks integrity, not publisher identity; repository/account compromise can affect
both files. Do not disable system-wide security protections to run a download.

SDK timeouts terminate the bridge, not necessarily its descendant processes or
application callbacks. Isolate or bound host code separately. Auth credentials are
read from the variable named by `auth_env`; never commit its value to YAML.
