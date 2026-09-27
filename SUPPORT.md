# Support and troubleshooting

For usage questions or reproducible bugs, use [GitHub Issues](https://github.com/danny-dis/MOTE/issues).
Repository access is required while the project is private. There is no promised
support SLA. Never attach API keys, sensitive workspace contents, or unredacted logs.
For vulnerabilities, follow [Security reporting](SECURITY.md#reporting-a-vulnerability).

## Before reporting a problem

1. Record `mote --version`, your OS/architecture, and the source tag or commit.
2. Verify that CLI, bridge, and source-installed SDKs are from compatible versions.
3. Try the read-only [quick-start](docs/QUICKSTART.md) in a small test workspace.
4. Include a redacted manifest, exact command, exit status, and expected versus actual result.
5. Distinguish the runtime's result from whether the generated content is correct.

## Common problems

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
Use the ESM import and package name documented in the [TypeScript SDK](sdk/typescript/README.md).

**Output exists but the run failed, or a completed result is wrong.** Side effects may
happen before failure. Inspect events and independently check output; do not blindly
retry writes or external callbacks. A completed state is not semantic validation.

**No live progress events.** Current CLI JSONL and SDK runtime events are emitted after
the run. Host-tool callbacks are interactive; runtime event streaming is not implemented.

**OS download warning.** Releases are unsigned/not notarized. Verify origin and checksum;
do not disable system-wide protections. Build from source if your policy requires it.

**Linux shared-library error.** Use the matching architecture and an Ubuntu 22.04-compatible
system with OpenSSL 3 and CA certificates, or build on your target system.
