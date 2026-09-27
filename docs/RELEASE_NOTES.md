Download the archive matching your operating system and architecture, verify it against `SHA256SUMS`, then extract it. Each archive includes the MOTE CLI, the stdio bridge for Python/TypeScript SDKs, a read-only example manifest, quick-start instructions, and licenses. Rust is not required to run these binaries.

- **Windows x64:** ZIP; includes `mote.exe` and `mote-bridge.exe`.
- **Linux x64:** tar.gz; requires Ubuntu 22.04 or a compatible newer system with OpenSSL 3 and CA certificates.
- **macOS Apple Silicon:** tar.gz; macOS 14 or newer. Intel Mac binaries are not included.

Binaries are unsigned/not notarized. Checksums verify download integrity, not publisher identity. Models and credentials are not bundled. The SDKs remain source-installable; this workflow does not publish to npm, PyPI, or crates.io.

The release pipeline runs cross-platform CI and the dependency audit, then executes the CLI and both SDK test suites against the extracted platform archives before uploading them. MOTE is a trusted-workspace agent engine, not an operating-system sandbox.
